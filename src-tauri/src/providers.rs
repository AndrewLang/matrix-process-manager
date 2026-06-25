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
                        path: process
                            .exe()
                            .map(|path| path.to_string_lossy().into_owned())
                            .unwrap_or_default(),
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
