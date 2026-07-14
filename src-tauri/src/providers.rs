#[cfg(windows)]
use crate::models::GpuEngineUsage;
use crate::models::{
    CommandError, CpuInfo, DiskDriveUsage, GpuAdapterUsage, MemoryInfo, NetworkAdapterUsage,
    ProcessInfo, ProcessMetrics, ProcessRow, ProcessSnapshot, WindowsInfo,
};
#[cfg(windows)]
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use sysinfo::{ProcessesToUpdate, System};

pub trait ProcessProvider: Send + Sync + 'static {
    fn snapshot(&self) -> Result<ProcessSnapshot, CommandError>;
}

pub struct SysinfoProcessProvider {
    system: Mutex<System>,
    cpu_usage: Mutex<CpuUsageCollector>,
    gpu_usage: Mutex<GpuUsageCollector>,
    disk_usage: Mutex<DiskUsageCollector>,
    network_usage: Mutex<NetworkUsageCollector>,
    memory_usage: Mutex<MemoryUsageCollector>,
    memory_info: Mutex<MemoryInfoCollector>,
    cpu_cache_info: CpuCacheInfo,
    cpu_virtualization: Option<String>,
    windows_info: WindowsInfo,
}

impl SysinfoProcessProvider {
    pub fn new() -> Self {
        Self {
            system: Mutex::new(System::new_all()),
            cpu_usage: Mutex::new(CpuUsageCollector::new()),
            gpu_usage: Mutex::new(GpuUsageCollector::new()),
            disk_usage: Mutex::new(DiskUsageCollector::new()),
            network_usage: Mutex::new(NetworkUsageCollector::new()),
            memory_usage: Mutex::new(MemoryUsageCollector::new()),
            memory_info: Mutex::new(MemoryInfoCollector::new()),
            cpu_cache_info: CpuCacheInfo::read(),
            cpu_virtualization: cpu_virtualization_status(),
            windows_info: WindowsInfoReader::read(),
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
        system.refresh_memory();
        let cpu_count = system.cpus().len().max(1) as f32;
        let total_cpu_percent = self
            .cpu_usage
            .lock()
            .ok()
            .and_then(|mut collector| collector.sample())
            .unwrap_or_else(|| system.global_cpu_usage());
        let cpu_info = CpuInfo {
            model: system
                .cpus()
                .first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_default(),
            current_speed_mhz: average_cpu_frequency(system.cpus()),
            base_speed_mhz: system
                .cpus()
                .first()
                .map(|cpu| cpu.frequency())
                .unwrap_or_default(),
            sockets: 1,
            cores: System::physical_core_count().unwrap_or(system.cpus().len()),
            logical_processors: system.cpus().len(),
            uptime_seconds: System::uptime(),
            total_threads: total_thread_count(&system),
            total_handles: total_handle_count(&system),
            virtualization: self.cpu_virtualization.clone(),
            l1_cache_bytes: self.cpu_cache_info.l1_bytes,
            l2_cache_bytes: self.cpu_cache_info.l2_bytes,
            l3_cache_bytes: self.cpu_cache_info.l3_bytes,
        };
        let gpu_usage = self
            .gpu_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let disk_usage = self
            .disk_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let network_usage = self
            .network_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let private_memory = self
            .memory_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let memory_info = self
            .memory_info
            .lock()
            .map(|mut collector| collector.sample(system.total_memory(), system.used_memory()))
            .unwrap_or_else(|_| {
                MemoryInfoCollector::fallback(system.total_memory(), system.used_memory())
            });
        let visible_window_pids = visible_window_process_ids();

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
                        has_visible_window: visible_window_pids.contains(&process.pid().as_u32()),
                        icon_data_url: file_icon_data_url(&path),
                        path,
                    },
                    metrics: ProcessMetrics {
                        cpu_percent: (process.cpu_usage() / cpu_count).clamp(0.0, 100.0),
                        gpu_percent: gpu_usage
                            .by_pid
                            .get(&process.pid().as_u32())
                            .copied()
                            .unwrap_or_default(),
                        memory_bytes: private_memory
                            .get(&process.pid().as_u32())
                            .copied()
                            .unwrap_or_else(|| process.memory()),
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
            total_disk_percent: disk_usage.total_percent,
            total_network_percent: network_usage.total_percent,
            used_memory_bytes: system.used_memory(),
            total_memory_bytes: system.total_memory(),
            cpu_info,
            memory_info,
            gpu_adapters: gpu_usage.adapters,
            disk_drives: disk_usage.drives,
            network_adapters: network_usage.adapters,
            windows_info: self.windows_info.clone(),
            processes,
        })
    }
}

fn average_cpu_frequency(cpus: &[sysinfo::Cpu]) -> u64 {
    if cpus.is_empty() {
        return 0;
    }

    cpus.iter().map(|cpu| cpu.frequency()).sum::<u64>() / cpus.len() as u64
}

struct WindowsInfoReader;

impl WindowsInfoReader {
    fn read() -> WindowsInfo {
        windows_info().unwrap_or_default()
    }
}

#[cfg(windows)]
fn windows_info() -> Option<WindowsInfo> {
    use std::collections::HashMap;
    use std::os::windows::process::CommandExt;

    let script = "$cs=Get-CimInstance Win32_ComputerSystem;$os=Get-CimInstance Win32_OperatingSystem;$cv=Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion';$crypto=Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography';$installed=try{[Management.ManagementDateTimeConverter]::ToDateTime($os.InstallDate).ToString('M/d/yyyy')}catch{''};$build=if($cv.UBR -ne $null){\"$($cv.CurrentBuild).$($cv.UBR)\"}else{$cv.CurrentBuild};$experience=@($cv.ExperiencePack,$cv.WindowsFeatureExperiencePack,$cv.'Windows Feature Experience Pack')|Where-Object{$_}|Select-Object -First 1;\"deviceName|$env:COMPUTERNAME\";\"manufacturer|$($cs.Manufacturer)\";\"model|$($cs.Model)\";\"systemType|$($os.OSArchitecture)\";\"deviceId|$($crypto.MachineGuid)\";\"productId|$($cv.ProductId)\";\"osEdition|$($cv.ProductName)\";\"osVersion|$($cv.DisplayVersion)\";\"installedOn|$installed\";\"osBuild|$build\";\"experience|$experience\"";
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x08000000)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let values = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('|'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .filter(|(_, value)| !value.is_empty())
        .collect::<HashMap<_, _>>();

    Some(WindowsInfo {
        device_name: values.get("deviceName").cloned(),
        manufacturer: values.get("manufacturer").cloned(),
        model: values.get("model").cloned(),
        system_type: values.get("systemType").cloned(),
        device_id: values.get("deviceId").cloned(),
        product_id: values.get("productId").cloned(),
        os_edition: values.get("osEdition").cloned(),
        os_version: values.get("osVersion").cloned(),
        installed_on: values.get("installedOn").cloned(),
        os_build: values.get("osBuild").cloned(),
        experience: values.get("experience").cloned(),
    })
}

#[cfg(target_os = "macos")]
fn windows_info() -> Option<WindowsInfo> {
    let hardware = std::process::Command::new("/usr/sbin/system_profiler")
        .args(["SPHardwareDataType", "-json", "-detailLevel", "mini"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());
    let version = std::process::Command::new("/usr/bin/sw_vers")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());

    macos_info_from_outputs(hardware.as_deref(), version.as_deref(), System::host_name())
}

#[cfg(target_os = "macos")]
fn macos_info_from_outputs(
    hardware_output: Option<&str>,
    version_output: Option<&str>,
    device_name: Option<String>,
) -> Option<WindowsInfo> {
    let hardware =
        hardware_output.and_then(|output| serde_json::from_str::<serde_json::Value>(output).ok());
    let overview = hardware
        .as_ref()
        .and_then(|value| value.get("SPHardwareDataType"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_object);
    let hardware_value = |key: &str| {
        overview
            .and_then(|values| values.get(key))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    };
    let version_values = version_output
        .into_iter()
        .flat_map(str::lines)
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim(), value.trim().to_string()))
        .collect::<HashMap<_, _>>();

    if overview.is_none() && version_values.is_empty() && device_name.is_none() {
        return None;
    }

    let machine_name = hardware_value("machine_name");
    let machine_model = hardware_value("machine_model");
    let model = match (machine_name, machine_model) {
        (Some(name), Some(identifier)) => Some(format!("{name} ({identifier})")),
        (name, identifier) => name.or(identifier),
    };

    Some(WindowsInfo {
        device_name,
        manufacturer: Some("Apple Inc.".to_string()),
        model,
        system_type: hardware_value("chip_type")
            .or_else(|| Some(std::env::consts::ARCH.to_string())),
        device_id: None,
        product_id: hardware_value("model_number"),
        os_edition: version_values.get("ProductName").cloned(),
        os_version: version_values.get("ProductVersion").cloned(),
        installed_on: None,
        os_build: version_values.get("BuildVersion").cloned(),
        experience: hardware_value("boot_rom_version"),
    })
}

#[cfg(not(any(windows, target_os = "macos")))]
fn windows_info() -> Option<WindowsInfo> {
    None
}

#[derive(Default)]
struct CpuCacheInfo {
    l1_bytes: Option<u64>,
    l2_bytes: Option<u64>,
    l3_bytes: Option<u64>,
}

impl CpuCacheInfo {
    fn read() -> Self {
        platform_cpu_cache_info().unwrap_or_default()
    }
}

#[cfg(windows)]
fn platform_cpu_cache_info() -> Option<CpuCacheInfo> {
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationCache, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let mut byte_count = 0u32;
    unsafe {
        GetLogicalProcessorInformationEx(RelationCache, std::ptr::null_mut(), &mut byte_count);
    }

    if byte_count == 0 {
        return None;
    }

    let mut buffer = vec![0u8; byte_count as usize];
    let info = buffer.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
    if unsafe { GetLogicalProcessorInformationEx(RelationCache, info, &mut byte_count) } == 0 {
        return None;
    }

    let mut offset = 0usize;
    let mut cache_info = CpuCacheInfo::default();

    while offset < byte_count as usize {
        let item = unsafe {
            std::ptr::read_unaligned(
                buffer.as_ptr().add(offset) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX
            )
        };
        let cache = unsafe { item.Anonymous.Cache };
        match cache.Level {
            1 => {
                cache_info.l1_bytes =
                    Some(cache_info.l1_bytes.unwrap_or_default() + cache.CacheSize as u64)
            }
            2 => {
                cache_info.l2_bytes =
                    Some(cache_info.l2_bytes.unwrap_or_default() + cache.CacheSize as u64)
            }
            3 => {
                cache_info.l3_bytes =
                    Some(cache_info.l3_bytes.unwrap_or_default() + cache.CacheSize as u64)
            }
            _ => {}
        }

        if item.Size == 0 {
            break;
        }
        offset += item.Size as usize;
    }

    Some(cache_info)
}

#[cfg(target_os = "macos")]
fn platform_cpu_cache_info() -> Option<CpuCacheInfo> {
    let output = std::process::Command::new("/usr/sbin/sysctl")
        .arg("-a")
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| macos_cpu_cache_info_from_sysctl(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(target_os = "macos")]
fn macos_cpu_cache_info_from_sysctl(output: &str) -> CpuCacheInfo {
    let values = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter_map(|(key, value)| Some((key.trim(), value.trim().parse::<u64>().ok()?)))
        .collect::<HashMap<_, _>>();
    let mut l1_bytes = 0u64;
    let mut l2_bytes = 0u64;

    for level in 0..8 {
        let prefix = format!("hw.perflevel{level}");
        let core_count = values
            .get(format!("{prefix}.physicalcpu").as_str())
            .copied()
            .unwrap_or_default();
        if core_count == 0 {
            continue;
        }

        let l1_data = values
            .get(format!("{prefix}.l1dcachesize").as_str())
            .copied()
            .unwrap_or_default();
        let l1_instruction = values
            .get(format!("{prefix}.l1icachesize").as_str())
            .copied()
            .unwrap_or_default();
        l1_bytes = l1_bytes.saturating_add(
            l1_data
                .saturating_add(l1_instruction)
                .saturating_mul(core_count),
        );
        l2_bytes = l2_bytes.saturating_add(
            values
                .get(format!("{prefix}.l2cachesize").as_str())
                .copied()
                .unwrap_or_default(),
        );
    }

    if l1_bytes == 0 {
        let core_count = values.get("hw.physicalcpu").copied().unwrap_or(1);
        l1_bytes = values
            .get("hw.l1dcachesize")
            .copied()
            .unwrap_or_default()
            .saturating_add(values.get("hw.l1icachesize").copied().unwrap_or_default())
            .saturating_mul(core_count);
    }
    if l2_bytes == 0 {
        l2_bytes = values.get("hw.l2cachesize").copied().unwrap_or_default();
    }

    CpuCacheInfo {
        l1_bytes: (l1_bytes > 0).then_some(l1_bytes),
        l2_bytes: (l2_bytes > 0).then_some(l2_bytes),
        l3_bytes: values
            .get("hw.l3cachesize")
            .copied()
            .filter(|size| *size > 0),
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn platform_cpu_cache_info() -> Option<CpuCacheInfo> {
    None
}

#[cfg(windows)]
fn cpu_virtualization_status() -> Option<String> {
    use windows_sys::Win32::System::Threading::{
        IsProcessorFeaturePresent, PF_VIRT_FIRMWARE_ENABLED,
    };

    Some(
        if unsafe { IsProcessorFeaturePresent(PF_VIRT_FIRMWARE_ENABLED) } != 0 {
            "Enabled"
        } else {
            "Disabled"
        }
        .to_string(),
    )
}

#[cfg(target_os = "macos")]
fn cpu_virtualization_status() -> Option<String> {
    let output = std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "kern.hv_support"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    match String::from_utf8_lossy(&output.stdout).trim() {
        "1" => Some("Supported".to_string()),
        "0" => Some("Unavailable".to_string()),
        _ => None,
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn cpu_virtualization_status() -> Option<String> {
    None
}

fn total_thread_count(system: &System) -> usize {
    windows_thread_count().unwrap_or_else(|| sysinfo_thread_count(system))
}

fn sysinfo_thread_count(system: &System) -> usize {
    system
        .processes()
        .values()
        .filter_map(|process| process.tasks().map(|tasks| tasks.len()))
        .sum()
}

#[cfg(windows)]
fn windows_thread_count() -> Option<usize> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        let mut count = 0usize;

        if Thread32First(snapshot, &mut entry) != 0 {
            loop {
                count += 1;

                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        Some(count)
    }
}

#[cfg(not(windows))]
fn windows_thread_count() -> Option<usize> {
    None
}

#[cfg(windows)]
fn total_handle_count(system: &System) -> Option<usize> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetProcessHandleCount, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    let mut total = 0usize;
    let mut sampled = false;

    for pid in system.processes().keys().map(|pid| pid.as_u32()) {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                continue;
            }

            let mut count = 0u32;
            if GetProcessHandleCount(handle, &mut count) != 0 {
                total += count as usize;
                sampled = true;
            }

            CloseHandle(handle);
        }
    }

    sampled.then_some(total)
}

#[cfg(not(windows))]
fn total_handle_count(_system: &System) -> Option<usize> {
    None
}

#[cfg(windows)]
fn visible_window_process_ids() -> HashSet<u32> {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
    };

    unsafe extern "system" fn collect_window_pid(hwnd: HWND, lparam: LPARAM) -> i32 {
        if unsafe { IsWindowVisible(hwnd) } == 0 || unsafe { GetWindowTextLengthW(hwnd) } == 0 {
            return 1;
        }

        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut pid);
        }

        if pid != 0 {
            let pids = unsafe { &mut *(lparam as *mut HashSet<u32>) };
            pids.insert(pid);
        }

        1
    }

    let mut pids = HashSet::new();
    unsafe {
        EnumWindows(
            Some(collect_window_pid),
            &mut pids as *mut HashSet<u32> as LPARAM,
        );
    }

    pids
}

#[cfg(not(windows))]
fn visible_window_process_ids() -> HashSet<u32> {
    HashSet::new()
}

#[cfg(windows)]
struct CpuUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
}

#[cfg(windows)]
unsafe impl Send for CpuUsageCollector {}

#[cfg(windows)]
impl CpuUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut counter = std::ptr::null_mut();
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;

        if opened {
            for path in [
                "\\Processor Information(_Total)\\% Processor Utility",
                "\\Processor(_Total)\\% Processor Time",
            ] {
                let wide_path = GpuUsageCollector::wide(path);
                if unsafe { PdhAddEnglishCounterW(query, wide_path.as_ptr(), 0, &mut counter) } == 0
                {
                    break;
                }
            }
        }

        if !counter.is_null() {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
        }

        Self {
            query,
            counter,
            ready: false,
        }
    }

    fn sample(&mut self) -> Option<f32> {
        use windows_sys::Win32::System::Performance::{
            PdhCollectQueryData, PdhGetFormattedCounterValue, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
        };

        if self.query.is_null() || self.counter.is_null() {
            return None;
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return None;
        }

        if !self.ready {
            self.ready = true;
            return None;
        }

        let mut value = PDH_FMT_COUNTERVALUE::default();
        let status = unsafe {
            PdhGetFormattedCounterValue(
                self.counter,
                PDH_FMT_DOUBLE,
                std::ptr::null_mut(),
                &mut value,
            )
        };

        if status != 0 || value.CStatus != 0 {
            return None;
        }

        Some(unsafe { value.Anonymous.doubleValue }.clamp(0.0, 100.0) as f32)
    }
}

#[cfg(windows)]
impl Drop for CpuUsageCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[cfg(not(windows))]
struct CpuUsageCollector;

#[cfg(not(windows))]
impl CpuUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> Option<f32> {
        None
    }
}

#[derive(Default)]
struct GpuUsageSnapshot {
    by_pid: HashMap<u32, f32>,
    total_percent: f32,
    adapters: Vec<GpuAdapterUsage>,
}

#[cfg(windows)]
#[derive(Default)]
struct GpuAdapterAccumulator {
    engines: HashMap<String, f32>,
    instance_count: usize,
}

#[cfg(windows)]
struct GpuUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
    models: Vec<String>,
}

#[cfg(windows)]
unsafe impl Send for GpuUsageCollector {}

#[cfg(windows)]
impl GpuUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut counter = std::ptr::null_mut();
        let path = Self::wide("\\GPU Engine(*)\\Utilization Percentage");
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let added =
            opened && unsafe { PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut counter) } == 0;

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

        Self {
            query,
            counter,
            ready: false,
            models: GpuAdapterModels::read(),
        }
    }

    fn sample(&mut self) -> GpuUsageSnapshot {
        use windows_sys::Win32::System::Performance::{
            PdhCollectQueryData, PdhGetFormattedCounterArrayW, PDH_FMT_COUNTERVALUE_ITEM_W,
            PDH_FMT_DOUBLE, PDH_MORE_DATA,
        };

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
        let mut by_adapter = BTreeMap::<String, GpuAdapterAccumulator>::new();
        let items = unsafe { std::slice::from_raw_parts(items, item_count as usize) };

        for item in items {
            if item.FmtValue.CStatus != 0 {
                continue;
            }

            let name = Self::string_from_wide(item.szName);
            let value = unsafe { item.FmtValue.Anonymous.doubleValue }.max(0.0) as f32;

            if value > 0.0 {
                if let Some(pid) = Self::pid_from_instance(&name) {
                    *by_pid.entry(pid).or_insert(0.0) += value;
                }
            }

            let adapter_key = Self::adapter_key_from_instance(&name);
            let engine = Self::engine_from_instance(&name);
            let adapter = by_adapter.entry(adapter_key).or_default();
            adapter.instance_count += 1;
            *adapter.engines.entry(engine).or_insert(0.0) += value;
        }

        for value in by_pid.values_mut() {
            *value = value.clamp(0.0, 100.0);
        }

        let mut adapter_groups = by_adapter.into_iter().collect::<Vec<_>>();
        adapter_groups.sort_by(|left, right| {
            right
                .1
                .instance_count
                .cmp(&left.1.instance_count)
                .then_with(|| left.0.cmp(&right.0))
        });
        if !self.models.is_empty() {
            adapter_groups.truncate(self.models.len());
        }

        let adapters = adapter_groups
            .into_iter()
            .enumerate()
            .map(|(adapter_index, (_adapter_key, adapter))| {
                let mut engines = adapter
                    .engines
                    .into_iter()
                    .map(|(name, utilization_percent)| GpuEngineUsage {
                        name,
                        utilization_percent: utilization_percent.clamp(0.0, 100.0),
                    })
                    .collect::<Vec<_>>();
                engines.sort_by(|left, right| left.name.cmp(&right.name));
                let utilization_percent = engines
                    .iter()
                    .map(|engine| engine.utilization_percent)
                    .fold(0.0, f32::max)
                    .clamp(0.0, 100.0);

                GpuAdapterUsage {
                    name: self
                        .models
                        .get(adapter_index)
                        .map(|model| format!("GPU {} - {}", adapter_index, model))
                        .unwrap_or_else(|| format!("GPU {}", adapter_index)),
                    adapter_index,
                    utilization_percent,
                    engines,
                }
            })
            .collect::<Vec<_>>();

        GpuUsageSnapshot {
            by_pid,
            total_percent: adapters
                .iter()
                .map(|adapter| adapter.utilization_percent)
                .fold(0.0, f32::max)
                .clamp(0.0, 100.0),
            adapters,
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

    fn adapter_key_from_instance(value: &str) -> String {
        if let Some(start) = value.find("luid_") {
            if let Some(end) = value[start..].find("_phys_") {
                return value[start..start + end].to_string();
            }
        }

        value
            .split("phys_")
            .nth(1)
            .and_then(|adapter| adapter.split('_').next())
            .map(|adapter| format!("phys_{adapter}"))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[cfg(windows)]
struct GpuAdapterModels;

#[cfg(windows)]
impl GpuAdapterModels {
    fn read() -> Vec<String> {
        use std::os::windows::process::CommandExt;

        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name }",
            ])
            .creation_flags(0x08000000)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
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

#[derive(Default)]
struct DiskUsageSnapshot {
    total_percent: f32,
    drives: Vec<DiskDriveUsage>,
}

#[cfg(windows)]
#[derive(Clone, Default)]
struct DiskDriveDetails {
    disk_index: usize,
    name: String,
    labels: Vec<String>,
    capacity_bytes: Option<u64>,
    formatted_bytes: Option<u64>,
    system_disk: bool,
    page_file: bool,
    disk_type: Option<String>,
}

#[cfg(windows)]
struct DiskDriveDetailsReader;

#[cfg(windows)]
impl DiskDriveDetailsReader {
    fn read() -> Vec<DiskDriveDetails> {
        use std::os::windows::process::CommandExt;

        let script = "$system=$env:SystemDrive;$pages=@(Get-CimInstance Win32_PageFileUsage|ForEach-Object{$_.Name.Substring(0,2)});Get-CimInstance Win32_DiskDrive|Sort-Object Index|ForEach-Object{$d=$_;$parts=@(Get-CimAssociatedInstance -InputObject $d -Association Win32_DiskDriveToDiskPartition);$logical=@();foreach($p in $parts){$logical+=@(Get-CimAssociatedInstance -InputObject $p -Association Win32_LogicalDiskToPartition)};$formatted=(($logical|Measure-Object -Property Size -Sum).Sum);$labels=($logical|Sort-Object DeviceID|ForEach-Object{$_.DeviceID}) -join ',';$isSystem=@($logical|Where-Object{$_.DeviceID -eq $system}).Count -gt 0;$isPage=@($logical|Where-Object{$pages -contains $_.DeviceID}).Count -gt 0;\"DISK|$($d.Index)|$($d.Caption)|$($d.Size)|$($d.MediaType)|$($d.InterfaceType)|$formatted|$isSystem|$isPage|$labels\"}";
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(0x08000000)
            .output();

        let Ok(output) = output else {
            return Vec::new();
        };

        if !output.status.success() {
            return Vec::new();
        }

        Self::parse(&String::from_utf8_lossy(&output.stdout))
    }

    fn parse(output: &str) -> Vec<DiskDriveDetails> {
        output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts = line.split('|').collect::<Vec<_>>();
                let ["DISK", index, name, capacity, media_type, interface_type, formatted, system_disk, page_file, labels] = parts.as_slice() else {
                    return None;
                };

                Some(DiskDriveDetails {
                    disk_index: index.parse().ok()?,
                    name: if name.trim().is_empty() {
                        format!("Disk {index}")
                    } else {
                        name.trim().to_string()
                    },
                    labels: labels
                        .split(',')
                        .map(str::trim)
                        .filter(|label| !label.is_empty())
                        .map(ToOwned::to_owned)
                        .collect(),
                    capacity_bytes: capacity.parse().ok().filter(|value| *value > 0),
                    formatted_bytes: formatted.parse().ok().filter(|value| *value > 0),
                    system_disk: system_disk.eq_ignore_ascii_case("true"),
                    page_file: page_file.eq_ignore_ascii_case("true"),
                    disk_type: Self::disk_type(media_type, interface_type),
                })
            })
            .collect()
    }

    fn disk_type(media_type: &str, interface_type: &str) -> Option<String> {
        let media = media_type.trim();
        let interface = interface_type.trim();
        let kind = if media.to_ascii_lowercase().contains("ssd") {
            "SSD"
        } else if media.to_ascii_lowercase().contains("hdd")
            || media.to_ascii_lowercase().contains("fixed")
        {
            "HDD"
        } else if media.is_empty() {
            "Disk"
        } else {
            media
        };

        if interface.is_empty() {
            Some(kind.to_string())
        } else {
            Some(format!("{kind} ({interface})"))
        }
    }
}

#[cfg(windows)]
struct DiskUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    active_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    response_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    read_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    write_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
    details: Vec<DiskDriveDetails>,
}

#[cfg(windows)]
unsafe impl Send for DiskUsageCollector {}

#[cfg(windows)]
impl DiskUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut active_counter = std::ptr::null_mut();
        let mut response_counter = std::ptr::null_mut();
        let mut read_counter = std::ptr::null_mut();
        let mut write_counter = std::ptr::null_mut();
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let active_path = GpuUsageCollector::wide("\\PhysicalDisk(*)\\% Disk Time");
        let response_path = GpuUsageCollector::wide("\\PhysicalDisk(*)\\Avg. Disk sec/Transfer");
        let read_path = GpuUsageCollector::wide("\\PhysicalDisk(*)\\Disk Read Bytes/sec");
        let write_path = GpuUsageCollector::wide("\\PhysicalDisk(*)\\Disk Write Bytes/sec");
        let added = opened
            && unsafe {
                PdhAddEnglishCounterW(query, active_path.as_ptr(), 0, &mut active_counter)
            } == 0
            && unsafe {
                PdhAddEnglishCounterW(query, response_path.as_ptr(), 0, &mut response_counter)
            } == 0
            && unsafe { PdhAddEnglishCounterW(query, read_path.as_ptr(), 0, &mut read_counter) }
                == 0
            && unsafe { PdhAddEnglishCounterW(query, write_path.as_ptr(), 0, &mut write_counter) }
                == 0;

        if added {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
            active_counter = std::ptr::null_mut();
            response_counter = std::ptr::null_mut();
            read_counter = std::ptr::null_mut();
            write_counter = std::ptr::null_mut();
        }

        Self {
            query,
            active_counter,
            response_counter,
            read_counter,
            write_counter,
            ready: false,
            details: DiskDriveDetailsReader::read(),
        }
    }

    fn sample(&mut self) -> DiskUsageSnapshot {
        use windows_sys::Win32::System::Performance::PdhCollectQueryData;

        if self.query.is_null()
            || self.active_counter.is_null()
            || self.response_counter.is_null()
            || self.read_counter.is_null()
            || self.write_counter.is_null()
        {
            return DiskUsageSnapshot::default();
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return DiskUsageSnapshot::default();
        }

        if !self.ready {
            self.ready = true;
            return DiskUsageSnapshot::default();
        }

        let active = self.double_counter_array(self.active_counter);
        let response = self.double_counter_array(self.response_counter);
        let read = self.double_counter_array(self.read_counter);
        let write = self.double_counter_array(self.write_counter);
        let mut drives = active
            .into_iter()
            .filter_map(|(instance, active_time_percent)| {
                let disk_index = Self::disk_index(&instance)?;
                let details = self
                    .details
                    .iter()
                    .find(|details| details.disk_index == disk_index);
                let name = details
                    .map(|details| details.name.clone())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| format!("Disk {disk_index}"));

                Some(DiskDriveUsage {
                    name,
                    labels: details
                        .map(|details| details.labels.clone())
                        .unwrap_or_default(),
                    disk_index,
                    active_time_percent: active_time_percent.clamp(0.0, 100.0) as f32,
                    average_response_time_ms: response
                        .get(&instance)
                        .copied()
                        .unwrap_or_default()
                        .max(0.0) as f32
                        * 1000.0,
                    read_bytes_per_sec: read.get(&instance).copied().unwrap_or_default().max(0.0)
                        as u64,
                    write_bytes_per_sec: write.get(&instance).copied().unwrap_or_default().max(0.0)
                        as u64,
                    capacity_bytes: details.and_then(|details| details.capacity_bytes),
                    formatted_bytes: details.and_then(|details| details.formatted_bytes),
                    system_disk: details.map(|details| details.system_disk),
                    page_file: details.map(|details| details.page_file),
                    disk_type: details.and_then(|details| details.disk_type.clone()),
                })
            })
            .collect::<Vec<_>>();

        drives.sort_by_key(|drive| drive.disk_index);

        DiskUsageSnapshot {
            total_percent: drives
                .iter()
                .map(|drive| drive.active_time_percent)
                .fold(0.0, f32::max)
                .clamp(0.0, 100.0),
            drives,
        }
    }

    fn double_counter_array(
        &self,
        counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ) -> HashMap<String, f64> {
        use windows_sys::Win32::System::Performance::{
            PdhGetFormattedCounterArrayW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
            PDH_MORE_DATA,
        };

        let mut buffer_size = 0;
        let mut item_count = 0;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || buffer_size == 0 || item_count == 0 {
            return HashMap::new();
        }

        let mut buffer = vec![0u8; buffer_size as usize];
        let items = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                items,
            )
        };

        if status != 0 {
            return HashMap::new();
        }

        unsafe { std::slice::from_raw_parts(items, item_count as usize) }
            .iter()
            .filter_map(|item| {
                if item.FmtValue.CStatus != 0 {
                    return None;
                }

                let instance = GpuUsageCollector::string_from_wide(item.szName);
                if instance == "_Total" {
                    return None;
                }

                Some((instance, unsafe { item.FmtValue.Anonymous.doubleValue }))
            })
            .collect()
    }

    fn disk_index(instance: &str) -> Option<usize> {
        instance.split_whitespace().next()?.parse().ok()
    }
}

#[cfg(windows)]
impl Drop for DiskUsageCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[derive(Default)]
struct NetworkUsageSnapshot {
    total_percent: f32,
    adapters: Vec<NetworkAdapterUsage>,
}

#[cfg(windows)]
#[derive(Clone, Default)]
struct NetworkAdapterDetails {
    adapter_index: Option<usize>,
    name: String,
    connection_name: Option<String>,
    mac_address: Option<String>,
    adapter_type: Option<String>,
    link_speed_bits_per_sec: Option<u64>,
    ipv4_addresses: Vec<String>,
    ipv6_addresses: Vec<String>,
}

#[cfg(windows)]
struct NetworkUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    receive_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    send_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
    details: Vec<NetworkAdapterDetails>,
}

#[cfg(windows)]
unsafe impl Send for NetworkUsageCollector {}

#[cfg(windows)]
impl NetworkUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut receive_counter = std::ptr::null_mut();
        let mut send_counter = std::ptr::null_mut();
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let receive_path = GpuUsageCollector::wide("\\Network Interface(*)\\Bytes Received/sec");
        let send_path = GpuUsageCollector::wide("\\Network Interface(*)\\Bytes Sent/sec");
        let added = opened
            && unsafe {
                PdhAddEnglishCounterW(query, receive_path.as_ptr(), 0, &mut receive_counter)
            } == 0
            && unsafe { PdhAddEnglishCounterW(query, send_path.as_ptr(), 0, &mut send_counter) }
                == 0;

        if added {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
            receive_counter = std::ptr::null_mut();
            send_counter = std::ptr::null_mut();
        }

        Self {
            query,
            receive_counter,
            send_counter,
            ready: false,
            details: NetworkAdapterDetailsReader::read(),
        }
    }

    fn sample(&mut self) -> NetworkUsageSnapshot {
        use windows_sys::Win32::System::Performance::PdhCollectQueryData;

        if self.query.is_null() || self.receive_counter.is_null() || self.send_counter.is_null() {
            return NetworkUsageSnapshot::default();
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return NetworkUsageSnapshot::default();
        }

        if !self.ready {
            self.ready = true;
            return NetworkUsageSnapshot::default();
        }

        let receive = self.double_counter_array(self.receive_counter);
        let send = self.double_counter_array(self.send_counter);
        let mut receive = receive.into_iter().collect::<Vec<_>>();
        receive.sort_by(|left, right| left.0.cmp(&right.0));
        let mut adapters = receive
            .into_iter()
            .enumerate()
            .map(|(adapter_index, (instance, receive_bytes_per_sec))| {
                let details = self
                    .details
                    .iter()
                    .find(|details| Self::matches(&instance, details));
                let adapter_index = details
                    .and_then(|details| details.adapter_index)
                    .unwrap_or(adapter_index);
                let send_bytes_per_sec =
                    send.get(&instance).copied().unwrap_or_default().max(0.0) as u64;
                let receive_bytes_per_sec = receive_bytes_per_sec.max(0.0) as u64;
                let link_speed_bits_per_sec =
                    details.and_then(|details| details.link_speed_bits_per_sec);
                let utilization_percent = link_speed_bits_per_sec
                    .filter(|speed| *speed > 0)
                    .map(|speed| {
                        ((receive_bytes_per_sec + send_bytes_per_sec) as f64 * 8.0 / speed as f64
                            * 100.0)
                            .clamp(0.0, 100.0) as f32
                    })
                    .unwrap_or_default();

                NetworkAdapterUsage {
                    name: details
                        .map(|details| details.name.clone())
                        .filter(|name| !name.is_empty())
                        .unwrap_or(instance),
                    adapter_index,
                    utilization_percent,
                    receive_bytes_per_sec,
                    send_bytes_per_sec,
                    link_speed_bits_per_sec,
                    connection_name: details.and_then(|details| details.connection_name.clone()),
                    mac_address: details.and_then(|details| details.mac_address.clone()),
                    adapter_type: details.and_then(|details| details.adapter_type.clone()),
                    ipv4_addresses: details
                        .map(|details| details.ipv4_addresses.clone())
                        .unwrap_or_default(),
                    ipv6_addresses: details
                        .map(|details| details.ipv6_addresses.clone())
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        adapters.sort_by(|left, right| left.name.cmp(&right.name));

        NetworkUsageSnapshot {
            total_percent: adapters
                .iter()
                .map(|adapter| adapter.utilization_percent)
                .fold(0.0, f32::max)
                .clamp(0.0, 100.0),
            adapters,
        }
    }

    fn double_counter_array(
        &self,
        counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ) -> HashMap<String, f64> {
        use windows_sys::Win32::System::Performance::{
            PdhGetFormattedCounterArrayW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
            PDH_MORE_DATA,
        };

        let mut buffer_size = 0;
        let mut item_count = 0;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || buffer_size == 0 || item_count == 0 {
            return HashMap::new();
        }

        let mut buffer = vec![0u8; buffer_size as usize];
        let items = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                items,
            )
        };

        if status != 0 {
            return HashMap::new();
        }

        unsafe { std::slice::from_raw_parts(items, item_count as usize) }
            .iter()
            .filter_map(|item| {
                if item.FmtValue.CStatus != 0 {
                    return None;
                }

                let instance = GpuUsageCollector::string_from_wide(item.szName);
                (!instance.is_empty())
                    .then_some((instance, unsafe { item.FmtValue.Anonymous.doubleValue }))
            })
            .collect()
    }

    fn matches(instance: &str, details: &NetworkAdapterDetails) -> bool {
        let instance = Self::normalized(instance);
        let name = Self::normalized(&details.name);
        let connection = details
            .connection_name
            .as_deref()
            .map(Self::normalized)
            .unwrap_or_default();
        !name.is_empty() && (instance.contains(&name) || name.contains(&instance))
            || !connection.is_empty()
                && (instance.contains(&connection) || connection.contains(&instance))
    }

    fn normalized(value: &str) -> String {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect()
    }
}

#[cfg(windows)]
impl Drop for NetworkUsageCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[cfg(windows)]
struct NetworkAdapterDetailsReader;

#[cfg(windows)]
impl NetworkAdapterDetailsReader {
    fn read() -> Vec<NetworkAdapterDetails> {
        use std::os::windows::process::CommandExt;

        let script = "$configs=@{};Get-CimInstance Win32_NetworkAdapterConfiguration | Where-Object { $_.IPEnabled -eq $true } | ForEach-Object { $configs[[string]$_.InterfaceIndex]=$_.IPAddress };Get-CimInstance Win32_NetworkAdapter | Where-Object { $_.PhysicalAdapter -eq $true -and $_.NetEnabled -eq $true } | ForEach-Object { $ips=@($configs[[string]$_.InterfaceIndex]);$ipv4=($ips|Where-Object{$_ -match '^\\d+\\.\\d+\\.\\d+\\.\\d+$'}) -join ',';$ipv6=($ips|Where-Object{$_ -match ':'}) -join ',';\"NIC|$($_.InterfaceIndex)|$($_.Name)|$($_.NetConnectionID)|$($_.MACAddress)|$($_.AdapterType)|$($_.Speed)|$ipv4|$ipv6\" }";
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(0x08000000)
            .output();

        let Ok(output) = output else {
            return Vec::new();
        };

        if !output.status.success() {
            return Vec::new();
        }

        Self::parse(&String::from_utf8_lossy(&output.stdout))
    }

    fn parse(output: &str) -> Vec<NetworkAdapterDetails> {
        output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts = line.split('|').collect::<Vec<_>>();
                let ["NIC", adapter_index, name, connection_name, mac_address, adapter_type, speed, ipv4, ipv6] = parts.as_slice() else {
                    return None;
                };

                Some(NetworkAdapterDetails {
                    adapter_index: adapter_index.parse().ok(),
                    name: name.trim().to_string(),
                    connection_name: (!connection_name.trim().is_empty()).then(|| connection_name.trim().to_string()),
                    mac_address: (!mac_address.trim().is_empty()).then(|| mac_address.trim().to_string()),
                    adapter_type: (!adapter_type.trim().is_empty()).then(|| adapter_type.trim().to_string()),
                    link_speed_bits_per_sec: speed.parse().ok().filter(|value| *value > 0),
                    ipv4_addresses: Self::addresses(ipv4),
                    ipv6_addresses: Self::addresses(ipv6),
                })
            })
            .collect()
    }

    fn addresses(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(str::trim)
            .filter(|address| !address.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[cfg(windows)]
struct MemoryUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    pid_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    private_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
}

#[cfg(windows)]
unsafe impl Send for MemoryUsageCollector {}

#[cfg(windows)]
impl MemoryUsageCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut pid_counter = std::ptr::null_mut();
        let mut private_counter = std::ptr::null_mut();
        let pid_path = GpuUsageCollector::wide("\\Process(*)\\ID Process");
        let private_path = GpuUsageCollector::wide("\\Process(*)\\Working Set - Private");
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let pid_added = opened
            && unsafe { PdhAddEnglishCounterW(query, pid_path.as_ptr(), 0, &mut pid_counter) } == 0;
        let private_added = pid_added
            && unsafe {
                PdhAddEnglishCounterW(query, private_path.as_ptr(), 0, &mut private_counter)
            } == 0;

        if private_added {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
            pid_counter = std::ptr::null_mut();
            private_counter = std::ptr::null_mut();
        }

        Self {
            query,
            pid_counter,
            private_counter,
        }
    }

    fn sample(&mut self) -> HashMap<u32, u64> {
        use windows_sys::Win32::System::Performance::PdhCollectQueryData;

        if self.query.is_null() || self.pid_counter.is_null() || self.private_counter.is_null() {
            return HashMap::new();
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return HashMap::new();
        }

        let pids = self.large_counter_array(self.pid_counter);
        let private_bytes = self.large_counter_array(self.private_counter);
        let mut by_pid = HashMap::new();

        for (instance, pid) in pids {
            if pid <= 0 {
                continue;
            }

            if let Some(bytes) = private_bytes
                .get(&instance)
                .copied()
                .filter(|bytes| *bytes >= 0)
            {
                by_pid.insert(pid as u32, bytes as u64);
            }
        }

        by_pid
    }

    fn large_counter_array(
        &self,
        counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ) -> HashMap<String, i64> {
        use windows_sys::Win32::System::Performance::{
            PdhGetFormattedCounterArrayW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_LARGE, PDH_MORE_DATA,
        };

        let mut buffer_size = 0;
        let mut item_count = 0;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_LARGE,
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || buffer_size == 0 || item_count == 0 {
            return HashMap::new();
        }

        let mut buffer = vec![0u8; buffer_size as usize];
        let items = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_LARGE,
                &mut buffer_size,
                &mut item_count,
                items,
            )
        };

        if status != 0 {
            return HashMap::new();
        }

        unsafe { std::slice::from_raw_parts(items, item_count as usize) }
            .iter()
            .filter_map(|item| {
                if item.FmtValue.CStatus != 0 {
                    return None;
                }

                Some((GpuUsageCollector::string_from_wide(item.szName), unsafe {
                    item.FmtValue.Anonymous.largeValue
                }))
            })
            .collect()
    }
}

#[cfg(windows)]
impl Drop for MemoryUsageCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[derive(Default)]
struct MemoryHardwareInfo {
    installed_bytes: Option<u64>,
    speed_mhz: Option<u64>,
    slots_used: Option<usize>,
    slots_total: Option<usize>,
    form_factor: Option<String>,
}

#[cfg(windows)]
struct MemoryInfoCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    compressed_counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    hardware: MemoryHardwareInfo,
}

#[cfg(windows)]
unsafe impl Send for MemoryInfoCollector {}

#[cfg(windows)]
impl MemoryInfoCollector {
    fn new() -> Self {
        use windows_sys::Win32::System::Performance::{
            PdhAddEnglishCounterW, PdhCollectQueryData, PdhOpenQueryW,
        };

        let mut query = std::ptr::null_mut();
        let mut compressed_counter = std::ptr::null_mut();
        let path = GpuUsageCollector::wide("\\Memory\\Compressed Page Count");
        let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0;
        let added = opened
            && unsafe { PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut compressed_counter) }
                == 0;

        if added {
            unsafe {
                PdhCollectQueryData(query);
            }
        } else if !query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(query);
            }
            query = std::ptr::null_mut();
            compressed_counter = std::ptr::null_mut();
        }

        Self {
            query,
            compressed_counter,
            hardware: MemoryHardwareReader::read(),
        }
    }

    fn sample(&mut self, total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        let performance = WindowsMemoryPerformance::read();
        let total_physical = performance
            .as_ref()
            .map(|info| info.total_physical_bytes)
            .filter(|bytes| *bytes > 0)
            .unwrap_or(total_memory_bytes);
        let available_bytes = performance
            .as_ref()
            .map(|info| info.available_bytes)
            .unwrap_or_else(|| total_physical.saturating_sub(used_memory_bytes));
        let compressed_bytes =
            self.compressed_bytes(performance.as_ref().map(|info| info.page_size));

        MemoryInfo {
            installed_bytes: self.hardware.installed_bytes,
            in_use_bytes: total_physical.saturating_sub(available_bytes),
            compressed_bytes,
            available_bytes,
            committed_bytes: performance
                .as_ref()
                .map(|info| info.committed_bytes)
                .unwrap_or(used_memory_bytes),
            commit_limit_bytes: performance
                .as_ref()
                .map(|info| info.commit_limit_bytes)
                .unwrap_or(total_memory_bytes),
            cached_bytes: performance
                .as_ref()
                .map(|info| info.cached_bytes)
                .unwrap_or_default(),
            paged_pool_bytes: performance
                .as_ref()
                .map(|info| info.paged_pool_bytes)
                .unwrap_or_default(),
            non_paged_pool_bytes: performance
                .as_ref()
                .map(|info| info.non_paged_pool_bytes)
                .unwrap_or_default(),
            speed_mhz: self.hardware.speed_mhz,
            slots_used: self.hardware.slots_used,
            slots_total: self.hardware.slots_total,
            form_factor: self.hardware.form_factor.clone(),
            hardware_reserved_bytes: self
                .hardware
                .installed_bytes
                .and_then(|installed| installed.checked_sub(total_physical)),
        }
    }

    fn fallback(total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        MemoryInfo {
            installed_bytes: Some(total_memory_bytes),
            in_use_bytes: used_memory_bytes,
            compressed_bytes: None,
            available_bytes: total_memory_bytes.saturating_sub(used_memory_bytes),
            committed_bytes: used_memory_bytes,
            commit_limit_bytes: total_memory_bytes,
            cached_bytes: 0,
            paged_pool_bytes: 0,
            non_paged_pool_bytes: 0,
            speed_mhz: None,
            slots_used: None,
            slots_total: None,
            form_factor: None,
            hardware_reserved_bytes: None,
        }
    }

    fn compressed_bytes(&self, page_size: Option<u64>) -> Option<u64> {
        use windows_sys::Win32::System::Performance::{
            PdhCollectQueryData, PdhGetFormattedCounterValue, PDH_FMT_COUNTERVALUE, PDH_FMT_LARGE,
        };

        if self.query.is_null() || self.compressed_counter.is_null() {
            return None;
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return None;
        }

        let mut value = PDH_FMT_COUNTERVALUE::default();
        let status = unsafe {
            PdhGetFormattedCounterValue(
                self.compressed_counter,
                PDH_FMT_LARGE,
                std::ptr::null_mut(),
                &mut value,
            )
        };

        if status != 0 || value.CStatus != 0 {
            return None;
        }

        let pages = unsafe { value.Anonymous.largeValue };
        (pages >= 0).then_some(pages as u64 * page_size.unwrap_or(4096))
    }
}

#[cfg(windows)]
impl Drop for MemoryInfoCollector {
    fn drop(&mut self) {
        if !self.query.is_null() {
            unsafe {
                windows_sys::Win32::System::Performance::PdhCloseQuery(self.query);
            }
        }
    }
}

#[cfg(windows)]
struct WindowsMemoryPerformance {
    total_physical_bytes: u64,
    available_bytes: u64,
    committed_bytes: u64,
    commit_limit_bytes: u64,
    cached_bytes: u64,
    paged_pool_bytes: u64,
    non_paged_pool_bytes: u64,
    page_size: u64,
}

#[cfg(windows)]
impl WindowsMemoryPerformance {
    fn read() -> Option<Self> {
        use windows_sys::Win32::System::ProcessStatus::{
            GetPerformanceInfo, PERFORMANCE_INFORMATION,
        };

        let mut info = PERFORMANCE_INFORMATION {
            cb: std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32,
            ..Default::default()
        };

        if unsafe { GetPerformanceInfo(&mut info, info.cb) } == 0 {
            return None;
        }

        let page_size = info.PageSize as u64;
        Some(Self {
            total_physical_bytes: info.PhysicalTotal as u64 * page_size,
            available_bytes: info.PhysicalAvailable as u64 * page_size,
            committed_bytes: info.CommitTotal as u64 * page_size,
            commit_limit_bytes: info.CommitLimit as u64 * page_size,
            cached_bytes: info.SystemCache as u64 * page_size,
            paged_pool_bytes: info.KernelPaged as u64 * page_size,
            non_paged_pool_bytes: info.KernelNonpaged as u64 * page_size,
            page_size,
        })
    }
}

#[cfg(windows)]
struct MemoryHardwareReader;

#[cfg(windows)]
impl MemoryHardwareReader {
    fn read() -> MemoryHardwareInfo {
        use std::os::windows::process::CommandExt;

        let script = "$m=@(Get-CimInstance Win32_PhysicalMemory);$a=@(Get-CimInstance Win32_PhysicalMemoryArray);$m|ForEach-Object{\"MODULE|$($_.Capacity)|$($_.ConfiguredClockSpeed)|$($_.Speed)|$($_.FormFactor)\"};\"SLOTS|$((($a|Measure-Object -Property MemoryDevices -Sum).Sum))\"";
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(0x08000000)
            .output();

        let Ok(output) = output else {
            return MemoryHardwareInfo::default();
        };

        if !output.status.success() {
            return MemoryHardwareInfo::default();
        }

        Self::parse(&String::from_utf8_lossy(&output.stdout))
    }

    fn parse(output: &str) -> MemoryHardwareInfo {
        let mut installed_bytes = 0u64;
        let mut speed_values = Vec::new();
        let mut form_factor = None;
        let mut slots_used = 0usize;
        let mut slots_total = None;

        for line in output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            let parts = line.split('|').collect::<Vec<_>>();
            match parts.as_slice() {
                ["MODULE", capacity, configured_speed, speed, factor] => {
                    installed_bytes += capacity.parse::<u64>().unwrap_or_default();
                    let module_speed = configured_speed
                        .parse::<u64>()
                        .ok()
                        .filter(|value| *value > 0)
                        .or_else(|| speed.parse::<u64>().ok().filter(|value| *value > 0));
                    if let Some(module_speed) = module_speed {
                        speed_values.push(module_speed);
                    }
                    form_factor = form_factor.or_else(|| Self::form_factor(factor));
                    slots_used += 1;
                }
                ["SLOTS", slots] => {
                    slots_total = slots.parse::<usize>().ok().filter(|slots| *slots > 0);
                }
                _ => {}
            }
        }

        MemoryHardwareInfo {
            installed_bytes: (installed_bytes > 0).then_some(installed_bytes),
            speed_mhz: (!speed_values.is_empty())
                .then(|| speed_values.iter().sum::<u64>() / speed_values.len() as u64),
            slots_used: (slots_used > 0).then_some(slots_used),
            slots_total,
            form_factor,
        }
    }

    fn form_factor(value: &str) -> Option<String> {
        Some(
            match value.parse::<u16>().ok()? {
                8 => "DIMM",
                12 => "SODIMM",
                13 => "SRIMM",
                14 => "RIMM",
                15 => "FB-DIMM",
                _ => return None,
            }
            .to_string(),
        )
    }
}

#[cfg(not(windows))]
struct MemoryUsageCollector;

#[cfg(not(windows))]
impl MemoryUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> HashMap<u32, u64> {
        HashMap::new()
    }
}

#[cfg(target_os = "macos")]
struct MemoryInfoCollector {
    hardware: MemoryHardwareInfo,
}

#[cfg(target_os = "macos")]
impl MemoryInfoCollector {
    fn new() -> Self {
        Self {
            hardware: macos_memory_hardware_info(),
        }
    }

    fn sample(&mut self, total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        let installed_bytes = self.hardware.installed_bytes.or(Some(total_memory_bytes));
        MemoryInfo {
            installed_bytes,
            in_use_bytes: used_memory_bytes,
            compressed_bytes: macos_compressed_memory_bytes(),
            available_bytes: total_memory_bytes.saturating_sub(used_memory_bytes),
            committed_bytes: used_memory_bytes,
            commit_limit_bytes: total_memory_bytes,
            cached_bytes: 0,
            paged_pool_bytes: 0,
            non_paged_pool_bytes: 0,
            speed_mhz: self.hardware.speed_mhz,
            slots_used: self.hardware.slots_used,
            slots_total: self.hardware.slots_total,
            form_factor: self.hardware.form_factor.clone(),
            hardware_reserved_bytes: installed_bytes
                .and_then(|installed| installed.checked_sub(total_memory_bytes)),
        }
    }

    fn fallback(total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        MemoryInfo {
            installed_bytes: Some(total_memory_bytes),
            in_use_bytes: used_memory_bytes,
            compressed_bytes: None,
            available_bytes: total_memory_bytes.saturating_sub(used_memory_bytes),
            committed_bytes: used_memory_bytes,
            commit_limit_bytes: total_memory_bytes,
            cached_bytes: 0,
            paged_pool_bytes: 0,
            non_paged_pool_bytes: 0,
            speed_mhz: None,
            slots_used: None,
            slots_total: None,
            form_factor: None,
            hardware_reserved_bytes: None,
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_memory_hardware_info() -> MemoryHardwareInfo {
    let Some(profile) = macos_system_profiler(&["SPMemoryDataType", "SPHardwareDataType"]) else {
        return MemoryHardwareInfo::default();
    };
    let memory = profile
        .get("SPMemoryDataType")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_object);
    let hardware = profile
        .get("SPHardwareDataType")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(serde_json::Value::as_object);
    let capacity = memory
        .and_then(|values| values.get("SPMemoryDataType"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            hardware
                .and_then(|values| values.get("physical_memory"))
                .and_then(serde_json::Value::as_str)
        })
        .and_then(macos_capacity_bytes);
    let memory_type = memory
        .and_then(|values| values.get("dimm_type"))
        .and_then(serde_json::Value::as_str);

    MemoryHardwareInfo {
        installed_bytes: capacity,
        speed_mhz: None,
        slots_used: None,
        slots_total: None,
        form_factor: Some(
            memory_type
                .map(|kind| format!("{kind} unified memory"))
                .unwrap_or_else(|| "Unified memory".to_string()),
        ),
    }
}

#[cfg(target_os = "macos")]
fn macos_compressed_memory_bytes() -> Option<u64> {
    let output = std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "vm.compressor_bytes_used"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().parse().ok())
        .flatten()
}

#[cfg(target_os = "macos")]
fn macos_capacity_bytes(value: &str) -> Option<u64> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<f64>().ok()?;
    let multiplier = match parts.next()?.to_ascii_uppercase().as_str() {
        "TB" => 1024_u64.pow(4),
        "GB" => 1024_u64.pow(3),
        "MB" => 1024_u64.pow(2),
        "KB" => 1024,
        _ => return None,
    };
    Some((amount * multiplier as f64) as u64)
}

#[cfg(not(any(windows, target_os = "macos")))]
struct MemoryInfoCollector;

#[cfg(not(any(windows, target_os = "macos")))]
impl MemoryInfoCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self, total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        Self::fallback(total_memory_bytes, used_memory_bytes)
    }

    fn fallback(total_memory_bytes: u64, used_memory_bytes: u64) -> MemoryInfo {
        MemoryInfo {
            installed_bytes: Some(total_memory_bytes),
            in_use_bytes: used_memory_bytes,
            compressed_bytes: None,
            available_bytes: total_memory_bytes.saturating_sub(used_memory_bytes),
            committed_bytes: used_memory_bytes,
            commit_limit_bytes: total_memory_bytes,
            cached_bytes: 0,
            paged_pool_bytes: 0,
            non_paged_pool_bytes: 0,
            speed_mhz: None,
            slots_used: None,
            slots_total: None,
            form_factor: None,
            hardware_reserved_bytes: None,
        }
    }
}

#[cfg(target_os = "macos")]
struct DiskUsageCollector {
    disks: sysinfo::Disks,
    last_sample: std::time::Instant,
}

#[cfg(target_os = "macos")]
impl DiskUsageCollector {
    fn new() -> Self {
        Self {
            disks: sysinfo::Disks::new_with_refreshed_list(),
            last_sample: std::time::Instant::now(),
        }
    }

    fn sample(&mut self) -> DiskUsageSnapshot {
        self.disks.refresh(true);
        let elapsed_seconds = self.last_sample.elapsed().as_secs_f64().max(0.001);
        self.last_sample = std::time::Instant::now();
        let mut drives = self
            .disks
            .list()
            .iter()
            .filter(|disk| macos_disk_is_visible(disk))
            .enumerate()
            .map(|(disk_index, disk)| {
                let usage = disk.usage();
                let read_bytes_per_sec = (usage.read_bytes as f64 / elapsed_seconds) as u64;
                let write_bytes_per_sec = (usage.written_bytes as f64 / elapsed_seconds) as u64;
                let throughput = read_bytes_per_sec.saturating_add(write_bytes_per_sec);
                let active_time_percent = (throughput as f64 / (500.0 * 1024.0 * 1024.0) * 100.0)
                    .clamp(0.0, 100.0) as f32;
                let mount_point = disk.mount_point().to_string_lossy().into_owned();
                let name = disk.name().to_string_lossy().into_owned();
                let file_system = disk.file_system().to_string_lossy();

                DiskDriveUsage {
                    name: if name.is_empty() {
                        mount_point.clone()
                    } else {
                        name
                    },
                    labels: vec![mount_point.clone()],
                    disk_index,
                    active_time_percent,
                    average_response_time_ms: 0.0,
                    read_bytes_per_sec,
                    write_bytes_per_sec,
                    capacity_bytes: (disk.total_space() > 0).then(|| disk.total_space()),
                    formatted_bytes: Some(
                        disk.total_space().saturating_sub(disk.available_space()),
                    ),
                    system_disk: Some(mount_point == "/"),
                    page_file: None,
                    disk_type: Some(format!("{:?} ({file_system})", disk.kind())),
                }
            })
            .collect::<Vec<_>>();
        drives.sort_by(|left, right| {
            right
                .system_disk
                .cmp(&left.system_disk)
                .then_with(|| left.name.cmp(&right.name))
        });
        for (disk_index, drive) in drives.iter_mut().enumerate() {
            drive.disk_index = disk_index;
        }

        DiskUsageSnapshot {
            total_percent: drives
                .iter()
                .map(|drive| drive.active_time_percent)
                .fold(0.0, f32::max),
            drives,
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_disk_is_visible(disk: &sysinfo::Disk) -> bool {
    let mount_point = disk.mount_point().to_string_lossy();
    disk.total_space() > 0
        && (mount_point == "/" || mount_point.starts_with("/Volumes/"))
        && !mount_point.starts_with("/System/Volumes/")
}

#[cfg(not(any(windows, target_os = "macos")))]
struct DiskUsageCollector;

#[cfg(not(any(windows, target_os = "macos")))]
impl DiskUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> DiskUsageSnapshot {
        DiskUsageSnapshot::default()
    }
}

#[cfg(target_os = "macos")]
struct NetworkUsageCollector {
    networks: sysinfo::Networks,
    last_sample: std::time::Instant,
}

#[cfg(target_os = "macos")]
impl NetworkUsageCollector {
    fn new() -> Self {
        Self {
            networks: sysinfo::Networks::new_with_refreshed_list(),
            last_sample: std::time::Instant::now(),
        }
    }

    fn sample(&mut self) -> NetworkUsageSnapshot {
        self.networks.refresh(true);
        let elapsed_seconds = self.last_sample.elapsed().as_secs_f64().max(0.001);
        self.last_sample = std::time::Instant::now();

        let mut interfaces = self.networks.list().iter().collect::<Vec<_>>();
        interfaces.sort_by(|left, right| left.0.cmp(right.0));
        let adapters = interfaces
            .into_iter()
            .filter(|(name, _)| macos_network_is_visible(name))
            .enumerate()
            .map(|(adapter_index, (name, data))| {
                let mut ipv4_addresses = Vec::new();
                let mut ipv6_addresses = Vec::new();
                for network in data.ip_networks() {
                    match network.addr {
                        std::net::IpAddr::V4(address) => ipv4_addresses.push(address.to_string()),
                        std::net::IpAddr::V6(address) => ipv6_addresses.push(address.to_string()),
                    }
                }
                let mac_address = data.mac_address().to_string();

                NetworkAdapterUsage {
                    name: name.clone(),
                    adapter_index,
                    utilization_percent: 0.0,
                    receive_bytes_per_sec: (data.received() as f64 / elapsed_seconds) as u64,
                    send_bytes_per_sec: (data.transmitted() as f64 / elapsed_seconds) as u64,
                    link_speed_bits_per_sec: None,
                    connection_name: Some(name.clone()),
                    mac_address: (mac_address != "00:00:00:00:00:00").then_some(mac_address),
                    adapter_type: Some(macos_network_type(name).to_string()),
                    ipv4_addresses,
                    ipv6_addresses,
                }
            })
            .collect::<Vec<_>>();

        NetworkUsageSnapshot {
            total_percent: 0.0,
            adapters,
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_network_is_visible(name: &str) -> bool {
    name != "lo0"
        && !name.starts_with("awdl")
        && !name.starts_with("llw")
        && !name.starts_with("utun")
        && !name.starts_with("gif")
        && !name.starts_with("stf")
}

#[cfg(target_os = "macos")]
fn macos_network_type(name: &str) -> &'static str {
    if name.starts_with("en") {
        "Ethernet or Wi-Fi"
    } else if name.starts_with("bridge") {
        "Network bridge"
    } else {
        "Network interface"
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
struct NetworkUsageCollector;

#[cfg(not(any(windows, target_os = "macos")))]
impl NetworkUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> NetworkUsageSnapshot {
        NetworkUsageSnapshot::default()
    }
}

#[cfg(target_os = "macos")]
struct GpuUsageCollector {
    adapters: Vec<GpuAdapterUsage>,
}

#[cfg(target_os = "macos")]
impl GpuUsageCollector {
    fn new() -> Self {
        Self {
            adapters: macos_gpu_adapters(),
        }
    }

    fn sample(&mut self) -> GpuUsageSnapshot {
        GpuUsageSnapshot {
            adapters: self.adapters.clone(),
            ..GpuUsageSnapshot::default()
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_gpu_adapters() -> Vec<GpuAdapterUsage> {
    let Some(profile) = macos_system_profiler(&["SPDisplaysDataType"]) else {
        return Vec::new();
    };
    profile
        .get("SPDisplaysDataType")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(adapter_index, value)| {
            let values = value.as_object()?;
            let model = values
                .get("sppci_model")
                .or_else(|| values.get("_name"))
                .and_then(serde_json::Value::as_str)?;
            let cores = values
                .get("sppci_cores")
                .and_then(serde_json::Value::as_str)
                .filter(|cores| !cores.is_empty());
            Some(GpuAdapterUsage {
                name: cores
                    .map(|cores| format!("{model} ({cores} cores)"))
                    .unwrap_or_else(|| model.to_string()),
                adapter_index,
                utilization_percent: 0.0,
                engines: Vec::new(),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn macos_system_profiler(data_types: &[&str]) -> Option<serde_json::Value> {
    let output = std::process::Command::new("/usr/sbin/system_profiler")
        .args(data_types)
        .args(["-json", "-detailLevel", "mini"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| serde_json::from_slice(&output.stdout).ok())
        .flatten()
}

#[cfg(not(any(windows, target_os = "macos")))]
struct GpuUsageCollector;

#[cfg(not(any(windows, target_os = "macos")))]
impl GpuUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> GpuUsageSnapshot {
        GpuUsageSnapshot::default()
    }
}

#[cfg(windows)]
pub(crate) fn file_icon_data_url(path: &str) -> Option<String> {
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
pub(crate) fn file_icon_data_url(_path: &str) -> Option<String> {
    None
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::{
        cpu_virtualization_status, macos_capacity_bytes, macos_cpu_cache_info_from_sysctl,
        macos_info_from_outputs, platform_cpu_cache_info, windows_info, DiskUsageCollector,
        GpuUsageCollector, MemoryInfoCollector, NetworkUsageCollector,
    };

    #[test]
    fn aggregates_heterogeneous_mac_cpu_caches() {
        let output = r#"
hw.perflevel0.physicalcpu: 4
hw.perflevel0.l1dcachesize: 131072
hw.perflevel0.l1icachesize: 196608
hw.perflevel0.l2cachesize: 16777216
hw.perflevel1.physicalcpu: 6
hw.perflevel1.l1dcachesize: 65536
hw.perflevel1.l1icachesize: 131072
hw.perflevel1.l2cachesize: 6291456
"#;

        let info = macos_cpu_cache_info_from_sysctl(output);

        assert_eq!(info.l1_bytes, Some(2_490_368));
        assert_eq!(info.l2_bytes, Some(23_068_672));
        assert_eq!(info.l3_bytes, None);
    }

    #[test]
    fn reads_current_mac_cpu_metadata() {
        let info = platform_cpu_cache_info().expect("macOS CPU cache metadata");

        assert!(info.l1_bytes.is_some_and(|bytes| bytes > 0));
        assert!(info.l2_bytes.is_some_and(|bytes| bytes > 0));
        assert_eq!(cpu_virtualization_status().as_deref(), Some("Supported"));
    }

    #[test]
    fn parses_mac_memory_capacity() {
        assert_eq!(macos_capacity_bytes("16 GB"), Some(16 * 1024_u64.pow(3)));
        assert_eq!(
            macos_capacity_bytes("1.5 TB"),
            Some((1.5 * 1024_u64.pow(4) as f64) as u64)
        );
        assert_eq!(macos_capacity_bytes("unknown"), None);
    }

    #[test]
    fn reads_current_mac_gpu_inventory() {
        let snapshot = GpuUsageCollector::new().sample();

        assert!(!snapshot.adapters.is_empty());
        assert!(snapshot
            .adapters
            .iter()
            .all(|adapter| !adapter.name.is_empty()));
    }

    #[test]
    fn reads_current_mac_memory_information() {
        let mut collector = MemoryInfoCollector::new();
        let info = collector.sample(16 * 1024_u64.pow(3), 8 * 1024_u64.pow(3));

        assert!(info.installed_bytes.is_some_and(|bytes| bytes > 0));
        assert!(info.compressed_bytes.is_some());
        assert!(info
            .form_factor
            .as_deref()
            .is_some_and(|value| value.contains("memory")));
    }

    #[test]
    fn reads_current_mac_network_inventory() {
        let snapshot = NetworkUsageCollector::new().sample();

        assert!(!snapshot.adapters.is_empty());
        assert!(snapshot
            .adapters
            .iter()
            .all(|adapter| adapter.name != "lo0"));
        assert!(snapshot.adapters.iter().any(
            |adapter| !adapter.ipv4_addresses.is_empty() || !adapter.ipv6_addresses.is_empty()
        ));
    }

    #[test]
    fn reads_current_mac_disk_inventory() {
        let snapshot = DiskUsageCollector::new().sample();
        let system_disk = snapshot
            .drives
            .iter()
            .find(|drive| drive.system_disk == Some(true))
            .expect("macOS root volume");

        assert!(system_disk.capacity_bytes.is_some_and(|bytes| bytes > 0));
        assert_eq!(system_disk.labels, vec!["/"]);
    }

    #[test]
    fn reads_current_mac_system_information() {
        let info = windows_info().expect("macOS system information");

        assert!(info.device_name.is_some());
        assert_eq!(info.manufacturer.as_deref(), Some("Apple Inc."));
        assert_eq!(info.os_edition.as_deref(), Some("macOS"));
        assert!(info.os_version.is_some());
        assert!(info.os_build.is_some());
    }

    #[test]
    fn parses_hardware_and_os_information() {
        let hardware = r#"{
            "SPHardwareDataType": [{
                "boot_rom_version": "18000.121.3",
                "chip_type": "Apple M5",
                "machine_model": "Mac17,3",
                "machine_name": "MacBook Air",
                "model_number": "MDHE4LL/A"
            }]
        }"#;
        let version = "ProductName: macOS\nProductVersion: 26.5.2\nBuildVersion: 25F84\n";

        let info = macos_info_from_outputs(
            Some(hardware),
            Some(version),
            Some("Andys-MacBook-Air".to_string()),
        )
        .expect("valid macOS information");

        assert_eq!(info.device_name.as_deref(), Some("Andys-MacBook-Air"));
        assert_eq!(info.manufacturer.as_deref(), Some("Apple Inc."));
        assert_eq!(info.model.as_deref(), Some("MacBook Air (Mac17,3)"));
        assert_eq!(info.system_type.as_deref(), Some("Apple M5"));
        assert_eq!(info.product_id.as_deref(), Some("MDHE4LL/A"));
        assert_eq!(info.os_edition.as_deref(), Some("macOS"));
        assert_eq!(info.os_version.as_deref(), Some("26.5.2"));
        assert_eq!(info.os_build.as_deref(), Some("25F84"));
        assert_eq!(info.experience.as_deref(), Some("18000.121.3"));
    }
}
