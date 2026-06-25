use crate::models::{CommandError, ProcessInfo, ProcessMetrics, ProcessRow, ProcessSnapshot};
use std::collections::HashMap;
use std::sync::Mutex;
use sysinfo::{ProcessesToUpdate, System};

pub trait ProcessProvider: Send + Sync + 'static {
    fn snapshot(&self) -> Result<ProcessSnapshot, CommandError>;
}

pub struct SysinfoProcessProvider {
    system: Mutex<System>,
    gpu_usage: Mutex<GpuUsageCollector>,
}

impl SysinfoProcessProvider {
    pub fn new() -> Self {
        Self {
            system: Mutex::new(System::new_all()),
            gpu_usage: Mutex::new(GpuUsageCollector::new()),
        }
    }
}

impl ProcessProvider for SysinfoProcessProvider {
    fn snapshot(&self) -> Result<ProcessSnapshot, CommandError> {
        let mut system = self.system.lock().map_err(|_| {
            CommandError::process_snapshot_failed("process provider state is unavailable")
        })?;

        system.refresh_processes(ProcessesToUpdate::All, true);
        system.refresh_cpu_usage();
        let cpu_count = system.cpus().len().max(1) as f32;
        let total_cpu_percent = system.global_cpu_usage();
        let gpu_usage = self
            .gpu_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();

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
                        cpu_percent: (process.cpu_usage() / cpu_count).clamp(0.0, 100.0),
                        gpu_percent: gpu_usage.by_pid.get(&process.pid().as_u32()).copied().unwrap_or_default(),
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
            total_cpu_percent,
            total_gpu_percent: gpu_usage.total_percent,
            processes,
        })
    }
}

#[derive(Default)]
struct GpuUsageSnapshot {
    by_pid: HashMap<u32, f32>,
    total_percent: f32,
}

#[cfg(windows)]
struct GpuUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
}

#[cfg(windows)]
unsafe impl Send for GpuUsageCollector {}

#[cfg(windows)]
impl GpuUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW};

        let mut query = std::ptr::null_mut();
        let mut counter = std::ptr::null_mut();
        let path = Self::wide("\\GPU Engine(*)\\Utilization Percentage");
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let added = opened && unsafe { PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut counter) } == 0;

        if added {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
        }

        Self { query, counter, ready: false }
    }

    fn sample(&mut self) -> GpuUsageSnapshot {
        use windows_sys::Win32::System::Performance::{PdhCollectQueryData, PdhGetFormattedCounterArrayW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_MORE_DATA};

        if self.query.is_null() || self.counter.is_null() {
            return GpuUsageSnapshot::default();
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return GpuUsageSnapshot::default();
        }

        if !self.ready {
            self.ready = true;
            return GpuUsageSnapshot::default();
        }

        let mut buffer_size = 0;
        let mut item_count = 0;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || buffer_size == 0 || item_count == 0 {
            return GpuUsageSnapshot::default();
        }

        let mut buffer = vec![0u8; buffer_size as usize];
        let items = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                items,
            )
        };

        if status != 0 {
            return GpuUsageSnapshot::default();
        }

        let mut by_pid = HashMap::new();
        let mut by_engine = HashMap::<String, f32>::new();
        let items = unsafe { std::slice::from_raw_parts(items, item_count as usize) };

        for item in items {
            if item.FmtValue.CStatus != 0 {
                continue;
            }

            let name = Self::string_from_wide(item.szName);
            let value = unsafe { item.FmtValue.Anonymous.doubleValue }.max(0.0) as f32;
            if value <= 0.0 {
                continue;
            }

            if let Some(pid) = Self::pid_from_instance(&name) {
                *by_pid.entry(pid).or_insert(0.0) += value;
            }

            let engine = Self::engine_from_instance(&name);
            *by_engine.entry(engine).or_insert(0.0) += value;
        }

        for value in by_pid.values_mut() {
            *value = value.clamp(0.0, 100.0);
        }

        GpuUsageSnapshot {
            by_pid,
            total_percent: by_engine.values().copied().fold(0.0, f32::max).clamp(0.0, 100.0),
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn string_from_wide(value: windows_sys::core::PWSTR) -> String {
        if value.is_null() {
            return String::new();
        }

        let mut length = 0;
        unsafe {
            while *value.add(length) != 0 {
                length += 1;
            }

            String::from_utf16_lossy(std::slice::from_raw_parts(value, length))
        }
    }

    fn pid_from_instance(value: &str) -> Option<u32> {
        let start = value.find("pid_")? + 4;
        let digits = value[start..]
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>();
        digits.parse().ok()
    }

    fn engine_from_instance(value: &str) -> String {
        value
            .split("engtype_")
            .nth(1)
            .and_then(|engine| engine.split('_').next())
            .unwrap_or("unknown")
            .to_string()
    }
}

#[cfg(windows)]
impl Drop for GpuUsageCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[cfg(not(windows))]
struct GpuUsageCollector;

#[cfg(not(windows))]
impl GpuUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> GpuUsageSnapshot {
        GpuUsageSnapshot::default()
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
        let bytes = unsafe {
            std::slice::from_raw_parts(
                bits as *const u8,
                icon_size as usize * icon_size as usize * 4,
            )
        };
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

        encoded.ok().map(|_| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(png)
            )
        })
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
