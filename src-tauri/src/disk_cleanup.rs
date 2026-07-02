use crate::models::{
    CommandError, DiskCleanupRequest, DiskCleanupResult, DiskCleanupScan, DiskCleanupTarget,
    DiskVolumeUsage,
};
use std::path::{Path, PathBuf};

pub struct DiskCleanupManager;

struct CleanupTargetDefinition {
    id: String,
    name: String,
    path: PathBuf,
    description: String,
}

impl DiskCleanupManager {
    pub fn scan() -> Result<DiskCleanupScan, CommandError> {
        let volumes = Self::volumes();
        Ok(DiskCleanupScan {
            targets: Self::targets(&volumes)
                .into_iter()
                .map(Self::target_from_definition)
                .collect::<Vec<_>>(),
            volumes,
        })
    }

    pub fn clean(request: DiskCleanupRequest) -> Result<DiskCleanupResult, CommandError> {
        let volumes = Self::volumes();
        let definitions = Self::targets(&volumes);
        let mut released_bytes = 0;
        let mut cleaned_targets = Vec::new();

        for target_id in request.target_ids {
            let Some(definition) = definitions.iter().find(|target| target.id == target_id) else {
                return Err(CommandError::disk_cleanup_failed("unknown cleanup target"));
            };

            let before = Self::directory_size(&definition.path);
            Self::remove_directory_contents(&definition.path);
            let after = Self::directory_size(&definition.path);
            released_bytes += before.saturating_sub(after);
            cleaned_targets.push(Self::target_from_definition_ref(definition));
        }

        Ok(DiskCleanupResult {
            released_bytes,
            cleaned_targets,
        })
    }

    fn target_from_definition(definition: CleanupTargetDefinition) -> DiskCleanupTarget {
        Self::target_from_definition_ref(&definition)
    }

    fn target_from_definition_ref(definition: &CleanupTargetDefinition) -> DiskCleanupTarget {
        DiskCleanupTarget {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            path: definition.path.to_string_lossy().into_owned(),
            description: definition.description.to_string(),
            bytes: Self::directory_size(&definition.path),
            exists: definition.path.exists(),
        }
    }

    fn targets(volumes: &[DiskVolumeUsage]) -> Vec<CleanupTargetDefinition> {
        let mut targets = Vec::new();

        for volume in volumes {
            let volume_id = volume
                .label
                .trim_end_matches(':')
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            let root = PathBuf::from(format!("{}\\", volume.label));
            targets.push(CleanupTargetDefinition {
                id: format!("recycle_bin_{volume_id}"),
                name: "Recycle Bin".to_string(),
                path: root.join("$Recycle.Bin"),
                description: "Deleted files waiting in this volume's Recycle Bin.".to_string(),
            });

            targets.push(CleanupTargetDefinition {
                id: format!("windows_old_{volume_id}"),
                name: "Previous Windows installation".to_string(),
                path: root.join("Windows.old"),
                description: "Files kept after a Windows upgrade on this volume.".to_string(),
            });
        }

        targets.push(CleanupTargetDefinition {
            id: "user_temp".to_string(),
            name: "User temporary files".to_string(),
            path: std::env::temp_dir(),
            description: "Temporary files created by apps for the current user.".to_string(),
        });

        if let Some(system_root) = std::env::var_os("SystemRoot") {
            let system_root = PathBuf::from(system_root);
            targets.push(CleanupTargetDefinition {
                id: "windows_temp".to_string(),
                name: "Windows temporary files".to_string(),
                path: system_root.join("Temp"),
                description: "Temporary files under the Windows folder.".to_string(),
            });
            targets.push(CleanupTargetDefinition {
                id: "windows_update_cache".to_string(),
                name: "Windows Update downloads".to_string(),
                path: system_root.join("SoftwareDistribution").join("Download"),
                description: "Downloaded update packages that Windows can fetch again.".to_string(),
            });
        }

        if let Some(program_data) = std::env::var_os("ProgramData") {
            let program_data = PathBuf::from(program_data);
            targets.push(CleanupTargetDefinition {
                id: "delivery_optimization".to_string(),
                name: "Delivery Optimization cache".to_string(),
                path: program_data
                    .join("Microsoft")
                    .join("Windows")
                    .join("DeliveryOptimization")
                    .join("Cache"),
                description: "Windows delivery cache for updates and Store downloads.".to_string(),
            });
            targets.push(CleanupTargetDefinition {
                id: "wer_reports".to_string(),
                name: "Windows error reports".to_string(),
                path: program_data
                    .join("Microsoft")
                    .join("Windows")
                    .join("WER")
                    .join("ReportArchive"),
                description: "Archived Windows Error Reporting data.".to_string(),
            });
        }

        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let local_app_data = PathBuf::from(local_app_data);
            targets.push(CleanupTargetDefinition {
                id: "crash_dumps".to_string(),
                name: "Crash dumps".to_string(),
                path: local_app_data.join("CrashDumps"),
                description: "Application crash dumps saved for diagnostics.".to_string(),
            });
            targets.push(CleanupTargetDefinition {
                id: "chrome_cache".to_string(),
                name: "Chrome cache".to_string(),
                path: local_app_data
                    .join("Google")
                    .join("Chrome")
                    .join("User Data")
                    .join("Default")
                    .join("Cache")
                    .join("Cache_Data"),
                description: "Cached web content from the default Chrome profile.".to_string(),
            });
            targets.push(CleanupTargetDefinition {
                id: "edge_cache".to_string(),
                name: "Edge cache".to_string(),
                path: local_app_data
                    .join("Microsoft")
                    .join("Edge")
                    .join("User Data")
                    .join("Default")
                    .join("Cache")
                    .join("Cache_Data"),
                description: "Cached web content from the default Edge profile.".to_string(),
            });
        }

        targets
    }

    fn directory_size(path: &Path) -> u64 {
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return 0;
        };

        if metadata.is_file() {
            return metadata.len();
        }

        if !metadata.is_dir() {
            return 0;
        }

        let Ok(entries) = std::fs::read_dir(path) else {
            return 0;
        };

        entries
            .flatten()
            .map(|entry| Self::directory_size(&entry.path()))
            .sum()
    }

    fn remove_directory_contents(path: &Path) {
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return;
        };

        if !metadata.is_dir() {
            let _ = std::fs::remove_file(path);
            return;
        }

        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };

        for entry in entries.flatten() {
            let child_path = entry.path();
            let Ok(child_metadata) = std::fs::symlink_metadata(&child_path) else {
                continue;
            };

            if child_metadata.is_dir() {
                let _ = std::fs::remove_dir_all(&child_path);
            } else {
                let _ = std::fs::remove_file(&child_path);
            }
        }
    }

    #[cfg(windows)]
    fn volumes() -> Vec<DiskVolumeUsage> {
        use std::os::windows::process::CommandExt;

        let script = "$system=$env:SystemDrive;Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3'|Sort-Object DeviceID|ForEach-Object{\"VOL|$($_.DeviceID)|$($_.VolumeName)|$($_.Size)|$($_.FreeSpace)|$($_.DeviceID -eq $system)\"}";
        let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(0x08000000)
            .output()
        else {
            return Vec::new();
        };

        if !output.status.success() {
            return Vec::new();
        }

        Self::parse_volumes(&String::from_utf8_lossy(&output.stdout))
    }

    #[cfg(not(windows))]
    fn volumes() -> Vec<DiskVolumeUsage> {
        Vec::new()
    }

    fn parse_volumes(output: &str) -> Vec<DiskVolumeUsage> {
        output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let parts = line.split('|').collect::<Vec<_>>();
                let ["VOL", label, name, total, free, system_drive] = parts.as_slice() else {
                    return None;
                };

                Some(DiskVolumeUsage {
                    label: label.trim().to_string(),
                    name: name.trim().to_string(),
                    total_bytes: total.parse().ok()?,
                    free_bytes: free.parse().ok()?,
                    system_drive: system_drive.eq_ignore_ascii_case("true"),
                })
            })
            .collect()
    }
}
