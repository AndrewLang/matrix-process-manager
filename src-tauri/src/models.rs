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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub cpu_percent: f32,
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
pub struct ProcessSnapshot {
    pub processes: Vec<ProcessRow>,
    pub total_processes: usize,
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
}
