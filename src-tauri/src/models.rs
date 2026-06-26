use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub publisher: String,
    pub status: String,
    pub user: String,
    pub path: String,
    pub has_visible_window: bool,
    pub icon_data_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub cpu_percent: f32,
    pub gpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRow {
    pub info: ProcessInfo,
    pub metrics: ProcessMetrics,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub model: String,
    pub current_speed_mhz: u64,
    pub base_speed_mhz: u64,
    pub sockets: usize,
    pub cores: usize,
    pub logical_processors: usize,
    pub uptime_seconds: u64,
    pub total_threads: usize,
    pub total_handles: Option<usize>,
    pub virtualization: Option<String>,
    pub l1_cache_bytes: Option<u64>,
    pub l2_cache_bytes: Option<u64>,
    pub l3_cache_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub installed_bytes: Option<u64>,
    pub in_use_bytes: u64,
    pub compressed_bytes: Option<u64>,
    pub available_bytes: u64,
    pub committed_bytes: u64,
    pub commit_limit_bytes: u64,
    pub cached_bytes: u64,
    pub paged_pool_bytes: u64,
    pub non_paged_pool_bytes: u64,
    pub speed_mhz: Option<u64>,
    pub slots_used: Option<usize>,
    pub slots_total: Option<usize>,
    pub form_factor: Option<String>,
    pub hardware_reserved_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuEngineUsage {
    pub name: String,
    pub utilization_percent: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuAdapterUsage {
    pub name: String,
    pub adapter_index: usize,
    pub utilization_percent: f32,
    pub engines: Vec<GpuEngineUsage>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskDriveUsage {
    pub name: String,
    pub labels: Vec<String>,
    pub disk_index: usize,
    pub active_time_percent: f32,
    pub average_response_time_ms: f32,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub capacity_bytes: Option<u64>,
    pub formatted_bytes: Option<u64>,
    pub system_disk: Option<bool>,
    pub page_file: Option<bool>,
    pub disk_type: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapterUsage {
    pub name: String,
    pub adapter_index: usize,
    pub utilization_percent: f32,
    pub receive_bytes_per_sec: u64,
    pub send_bytes_per_sec: u64,
    pub link_speed_bits_per_sec: Option<u64>,
    pub connection_name: Option<String>,
    pub mac_address: Option<String>,
    pub adapter_type: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsInfo {
    pub device_name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub system_type: Option<String>,
    pub device_id: Option<String>,
    pub product_id: Option<String>,
    pub os_edition: Option<String>,
    pub os_version: Option<String>,
    pub installed_on: Option<String>,
    pub os_build: Option<String>,
    pub experience: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSnapshot {
    pub processes: Vec<ProcessRow>,
    pub total_processes: usize,
    pub total_cpu_percent: f32,
    pub total_gpu_percent: f32,
    pub total_disk_percent: f32,
    pub total_network_percent: f32,
    pub used_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub gpu_adapters: Vec<GpuAdapterUsage>,
    pub disk_drives: Vec<DiskDriveUsage>,
    pub network_adapters: Vec<NetworkAdapterUsage>,
    pub windows_info: WindowsInfo,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupApp {
    pub name: String,
    pub publisher: String,
    pub icon_data_url: Option<String>,
    pub status: String,
    pub impact: String,
    pub startup_type: String,
    pub source: String,
    pub command: String,
    pub path: String,
    pub delay_seconds: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn process_snapshot_failed(message: impl Into<String>) -> Self {
        Self {
            code: "processSnapshotFailed".to_string(),
            message: message.into(),
        }
    }

    pub fn native_tool_failed(message: impl Into<String>) -> Self {
        Self {
            code: "nativeToolFailed".to_string(),
            message: message.into(),
        }
    }

    pub fn settings_failed(message: impl Into<String>) -> Self {
        Self {
            code: "settingsFailed".to_string(),
            message: message.into(),
        }
    }

    pub fn process_action_failed(message: impl Into<String>) -> Self {
        Self {
            code: "processActionFailed".to_string(),
            message: message.into(),
        }
    }
}
