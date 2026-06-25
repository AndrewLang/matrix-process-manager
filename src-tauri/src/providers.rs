use crate::models::{CommandError, ProcessInfo, ProcessMetrics, ProcessRow, ProcessSnapshot};
use std::sync::Mutex;
use sysinfo::{ProcessesToUpdate, System};

pub trait ProcessProvider: Send + Sync + 'static {
    fn snapshot(&self) -> Result<ProcessSnapshot, CommandError>;
}

pub struct SysinfoProcessProvider {
    system: Mutex<System>,
}

impl SysinfoProcessProvider {
    pub fn new() -> Self {
        Self {
            system: Mutex::new(System::new_all()),
        }
    }
}

impl ProcessProvider for SysinfoProcessProvider {
    fn snapshot(&self) -> Result<ProcessSnapshot, CommandError> {
        let mut system = self.system.lock().map_err(|_| {
            CommandError::process_snapshot_failed("process provider state is unavailable")
        })?;

        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut processes = system
            .processes()
            .values()
            .map(|process| {
                let disk = process.disk_usage();
                let path = process
                    .exe()
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_default();
                ProcessRow {
                    info: ProcessInfo {
                        pid: process.pid().as_u32(),
                        name: process.name().to_string_lossy().into_owned(),
                        publisher: String::new(),
                        status: format!("{:?}", process.status()),
                        user: process
                            .user_id()
                            .map(|user_id| user_id.to_string())
                            .unwrap_or_default(),
                        icon_data_url: process_icon_data_url(&path),
                        path,
                    },
                    metrics: ProcessMetrics {
                        cpu_percent: process.cpu_usage(),
                        memory_bytes: process.memory(),
                        disk_read_bytes: disk.read_bytes,
                        disk_written_bytes: disk.written_bytes,
                    },
                }
            })
            .collect::<Vec<_>>();

        processes.sort_by(|left, right| {
            right
                .metrics
                .cpu_percent
                .partial_cmp(&left.metrics.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ProcessSnapshot {
            total_processes: processes.len(),
            processes,
        })
    }
}

#[cfg(windows)]
fn process_icon_data_url(path: &str) -> Option<String> {
    use base64::Engine;
    use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
    use std::ffi::OsStr;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows_sys::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL};

    if path.is_empty() {
        return None;
    }

    let wide_path = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut file_info = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut file_info,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if result == 0 || file_info.hIcon.is_null() {
        return None;
    }

    let icon_size = 32;
    let memory_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if memory_dc.is_null() {
        unsafe {
            let _ = DestroyIcon(file_info.hIcon);
        }
        return None;
    }

    let bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: icon_size,
            biHeight: -icon_size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe {
        CreateDIBSection(
            std::ptr::null_mut(),
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        )
    };

    if bitmap.is_null() || bits.is_null() {
        unsafe {
            let _ = DeleteDC(memory_dc);
            let _ = DestroyIcon(file_info.hIcon);
        }
        return None;
    }

    let previous_object = unsafe { SelectObject(memory_dc, bitmap) };
    let drawn = unsafe {
        DrawIconEx(
            memory_dc,
            0,
            0,
            file_info.hIcon,
            icon_size,
            icon_size,
            0,
            std::ptr::null_mut(),
            DI_NORMAL,
        )
    };

    let data_url = if drawn != 0 {
        let bytes = unsafe { std::slice::from_raw_parts(bits as *const u8, icon_size as usize * icon_size as usize * 4) };
        let rgba = bytes
            .chunks_exact(4)
            .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
            .collect::<Vec<_>>();
        let mut png = Vec::new();
        let encoded = PngEncoder::new(&mut png).write_image(
            &rgba,
            icon_size as u32,
            icon_size as u32,
            ColorType::Rgba8.into(),
        );

        encoded
            .ok()
            .map(|_| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
    } else {
        None
    };

    unsafe {
        let _ = SelectObject(memory_dc, previous_object);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(memory_dc);
        let _ = DestroyIcon(file_info.hIcon);
    }

    data_url
}

#[cfg(not(windows))]
fn process_icon_data_url(_path: &str) -> Option<String> {
    None
}
