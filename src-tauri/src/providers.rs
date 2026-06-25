use crate::models::{
    CommandError, CpuInfo, ProcessInfo, ProcessMetrics, ProcessRow, ProcessSnapshot,
};
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
    memory_usage: Mutex<MemoryUsageCollector>,
}

impl SysinfoProcessProvider {
    pub fn new() -> Self {
        Self {
            system: Mutex::new(System::new_all()),
            cpu_usage: Mutex::new(CpuUsageCollector::new()),
            gpu_usage: Mutex::new(GpuUsageCollector::new()),
            disk_usage: Mutex::new(DiskUsageCollector::new()),
            memory_usage: Mutex::new(MemoryUsageCollector::new()),
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
        let cache_info = CpuCacheInfo::read();
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
            virtualization: cpu_virtualization_status(),
            l1_cache_bytes: cache_info.l1_bytes,
            l2_cache_bytes: cache_info.l2_bytes,
            l3_cache_bytes: cache_info.l3_bytes,
        };
        let gpu_usage = self
            .gpu_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let total_disk_percent = self
            .disk_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
        let private_memory = self
            .memory_usage
            .lock()
            .map(|mut collector| collector.sample())
            .unwrap_or_default();
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
                        icon_data_url: process_icon_data_url(&path),
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
            total_disk_percent,
            used_memory_bytes: system.used_memory(),
            total_memory_bytes: system.total_memory(),
            cpu_info,
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

#[derive(Default)]
struct CpuCacheInfo {
    l1_bytes: Option<u64>,
    l2_bytes: Option<u64>,
    l3_bytes: Option<u64>,
}

impl CpuCacheInfo {
    fn read() -> Self {
        windows_cpu_cache_info().unwrap_or_default()
    }
}

#[cfg(windows)]
fn windows_cpu_cache_info() -> Option<CpuCacheInfo> {
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

#[cfg(not(windows))]
fn windows_cpu_cache_info() -> Option<CpuCacheInfo> {
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

#[cfg(not(windows))]
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
            total_percent: by_engine
                .values()
                .copied()
                .fold(0.0, f32::max)
                .clamp(0.0, 100.0),
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

#[cfg(windows)]
struct DiskUsageCollector {
    query: windows_sys::Win32::System::Performance::PDH_HQUERY,
    counter: windows_sys::Win32::System::Performance::PDH_HCOUNTER,
    ready: bool,
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
        let mut counter = std::ptr::null_mut();
        let path = GpuUsageCollector::wide("\\PhysicalDisk(_Total)\\% Disk Time");
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
        }
    }

    fn sample(&mut self) -> f32 {
        use windows_sys::Win32::System::Performance::{
            PdhCollectQueryData, PdhGetFormattedCounterValue, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
        };

        if self.query.is_null() || self.counter.is_null() {
            return 0.0;
        }

        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return 0.0;
        }

        if !self.ready {
            self.ready = true;
            return 0.0;
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
            return 0.0;
        }

        unsafe { value.Anonymous.doubleValue }.max(0.0).min(100.0) as f32
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

#[cfg(not(windows))]
struct DiskUsageCollector;

#[cfg(not(windows))]
impl DiskUsageCollector {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> f32 {
        0.0
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
