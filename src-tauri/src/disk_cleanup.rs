use crate::models::{
    CommandError, DiskCleanupRequest, DiskCleanupResult, DiskCleanupScan, DiskCleanupTarget,
    DiskUsageInsight, DiskUsageInsightCleanupRequest, DiskUsageInsightCleanupResult,
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

struct UsageInsightDefinition {
    id: String,
    name: String,
    path: PathBuf,
    category: String,
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
            usage_insights: Self::usage_insights()
                .into_iter()
                .map(Self::usage_insight_from_definition)
                .filter(|insight| insight.exists && insight.bytes > 0)
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

    pub fn clean_usage_insight(
        request: DiskUsageInsightCleanupRequest,
    ) -> Result<DiskUsageInsightCleanupResult, CommandError> {
        let Some(definition) = Self::usage_insights()
            .into_iter()
            .find(|insight| insight.id == request.insight_id)
        else {
            return Err(CommandError::disk_cleanup_failed("unknown usage insight"));
        };

        let before = Self::directory_size(&definition.path);
        Self::remove_directory_contents(&definition.path);
        let cleaned_insight = Self::usage_insight_from_definition(definition);

        Ok(DiskUsageInsightCleanupResult {
            released_bytes: before.saturating_sub(cleaned_insight.bytes),
            cleaned_insight,
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

    fn usage_insight_from_definition(definition: UsageInsightDefinition) -> DiskUsageInsight {
        let safe_to_clean = Self::safe_to_clean(&definition.id);
        DiskUsageInsight {
            safety: Self::safety_label(safe_to_clean).to_string(),
            safe_to_clean,
            id: definition.id,
            name: definition.name,
            path: definition.path.to_string_lossy().into_owned(),
            category: definition.category,
            description: definition.description,
            bytes: Self::directory_size(&definition.path),
            exists: definition.path.exists(),
        }
    }

    fn safe_to_clean(id: &str) -> bool {
        matches!(
            id,
            "nuget_packages"
                | "cargo_registry"
                | "cargo_git"
                | "maven_repository"
                | "gradle_cache"
                | "go_module_cache"
                | "conda_packages"
                | "poetry_cache"
                | "huggingface_cache"
                | "torch_cache"
                | "android_gradle_cache"
                | "npm_cache"
                | "pnpm_cache_macos"
                | "pnpm_store_macos"
                | "pnpm_store_roaming"
                | "composer_cache"
                | "pnpm_store_local"
                | "pnpm_store_local_alt"
                | "yarn_cache"
                | "pip_cache"
                | "electron_cache"
                | "electron_builder_cache"
                | "unity_cache"
                | "unreal_derived_data_cache"
                | "package_cache"
                | "chocolatey_cache"
                | "winget_cache"
        )
    }

    fn safety_label(safe_to_clean: bool) -> &'static str {
        if safe_to_clean {
            "Safe to clean; data can be redownloaded or rebuilt."
        } else {
            "Inspect first; deleting this can remove app data, tools, or installations."
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

    fn usage_insights() -> Vec<UsageInsightDefinition> {
        let mut insights = Vec::new();

        Self::push_known_usage_insights(&mut insights);
        Self::push_large_app_folders(&mut insights);

        insights
    }

    fn push_known_usage_insights(insights: &mut Vec<UsageInsightDefinition>) {
        #[cfg(target_os = "macos")]
        if let Some(home) = std::env::var_os("HOME") {
            Self::push_macos_usage_insights(insights, &PathBuf::from(home));
        }

        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let user_profile = PathBuf::from(user_profile);
            Self::push_usage_insight(
                insights,
                "nuget_packages",
                "NuGet packages",
                user_profile.join(".nuget").join("packages"),
                "Developer cache",
                "NuGet package cache used by .NET projects.",
            );
            Self::push_usage_insight(
                insights,
                "cargo_registry",
                "Cargo registry",
                user_profile.join(".cargo").join("registry"),
                "Developer cache",
                "Rust crate registry cache.",
            );
            Self::push_usage_insight(
                insights,
                "cargo_git",
                "Cargo git cache",
                user_profile.join(".cargo").join("git"),
                "Developer cache",
                "Rust git dependency cache.",
            );
            Self::push_usage_insight(
                insights,
                "rustup_toolchains",
                "Rust toolchains",
                user_profile.join(".rustup").join("toolchains"),
                "Developer tools",
                "Installed Rust toolchains managed by rustup.",
            );
            Self::push_usage_insight(
                insights,
                "vscode_extensions",
                "VS Code extensions",
                user_profile.join(".vscode").join("extensions"),
                "Developer tools",
                "Installed Visual Studio Code extensions.",
            );
            Self::push_usage_insight(
                insights,
                "docker_user",
                "Docker user data",
                user_profile.join(".docker"),
                "App data",
                "Docker CLI configuration and related user data.",
            );
            Self::push_usage_insight(
                insights,
                "maven_repository",
                "Maven repository",
                user_profile.join(".m2").join("repository"),
                "Developer cache",
                "Maven and Gradle dependency artifacts shared by Java projects.",
            );
            Self::push_usage_insight(
                insights,
                "gradle_cache",
                "Gradle cache",
                user_profile.join(".gradle").join("caches"),
                "Developer cache",
                "Gradle dependency and build caches.",
            );
            Self::push_usage_insight(
                insights,
                "go_module_cache",
                "Go module cache",
                user_profile.join("go").join("pkg").join("mod"),
                "Developer cache",
                "Downloaded Go modules used by Go projects.",
            );
            Self::push_usage_insight(
                insights,
                "conda_packages",
                "Conda packages",
                user_profile.join(".conda").join("pkgs"),
                "Developer cache",
                "Conda package cache used by Python environments.",
            );
            Self::push_usage_insight(
                insights,
                "poetry_cache",
                "Poetry cache",
                user_profile.join(".cache").join("pypoetry"),
                "Developer cache",
                "Poetry package and virtual environment cache.",
            );
            Self::push_usage_insight(
                insights,
                "huggingface_cache",
                "Hugging Face cache",
                user_profile.join(".cache").join("huggingface"),
                "AI model cache",
                "Downloaded datasets, tokenizers, and model files.",
            );
            Self::push_usage_insight(
                insights,
                "torch_cache",
                "PyTorch cache",
                user_profile.join(".cache").join("torch"),
                "AI model cache",
                "Downloaded PyTorch model and extension cache.",
            );
            Self::push_usage_insight(
                insights,
                "android_gradle_cache",
                "Android Gradle cache",
                user_profile.join(".android").join("build-cache"),
                "Developer cache",
                "Android build cache used by older Gradle Android builds.",
            );
        }

        if let Some(app_data) = std::env::var_os("APPDATA") {
            let app_data = PathBuf::from(app_data);
            Self::push_usage_insight(
                insights,
                "npm_cache",
                "npm cache",
                app_data.join("npm-cache"),
                "Developer cache",
                "npm package download cache.",
            );
            Self::push_usage_insight(
                insights,
                "pnpm_store_roaming",
                "pnpm store",
                app_data.join("pnpm").join("store"),
                "Developer cache",
                "pnpm content-addressable package store.",
            );
            Self::push_usage_insight(
                insights,
                "docker_roaming",
                "Docker roaming data",
                app_data.join("Docker"),
                "App data",
                "Docker Desktop roaming application data.",
            );
            Self::push_usage_insight(
                insights,
                "composer_cache",
                "Composer cache",
                app_data.join("Composer").join("cache"),
                "Developer cache",
                "PHP Composer package cache.",
            );
        }

        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let local_app_data = PathBuf::from(local_app_data);
            Self::push_usage_insight(
                insights,
                "pnpm_store_local",
                "pnpm local store",
                local_app_data.join("pnpm").join("store"),
                "Developer cache",
                "pnpm package store under local app data.",
            );
            Self::push_usage_insight(
                insights,
                "pnpm_store_local_alt",
                "pnpm store",
                local_app_data.join("pnpm-store"),
                "Developer cache",
                "Alternate pnpm package store location.",
            );
            Self::push_usage_insight(
                insights,
                "yarn_cache",
                "Yarn cache",
                local_app_data.join("Yarn").join("Cache"),
                "Developer cache",
                "Yarn package cache.",
            );
            Self::push_usage_insight(
                insights,
                "pip_cache",
                "pip cache",
                local_app_data.join("pip").join("Cache"),
                "Developer cache",
                "Python package download and wheel cache.",
            );
            Self::push_usage_insight(
                insights,
                "docker_local",
                "Docker Desktop data",
                local_app_data.join("Docker"),
                "App data",
                "Docker Desktop local application data.",
            );
            Self::push_usage_insight(
                insights,
                "docker_wsl",
                "Docker WSL data",
                local_app_data.join("Docker").join("wsl"),
                "App data",
                "Docker WSL distributions and disk images.",
            );
            Self::push_usage_insight(
                insights,
                "chrome_user_data",
                "Chrome user data",
                local_app_data
                    .join("Google")
                    .join("Chrome")
                    .join("User Data"),
                "App data",
                "Chrome profiles, cache, extensions, and browser data.",
            );
            Self::push_usage_insight(
                insights,
                "edge_user_data",
                "Edge user data",
                local_app_data
                    .join("Microsoft")
                    .join("Edge")
                    .join("User Data"),
                "App data",
                "Edge profiles, cache, extensions, and browser data.",
            );
            Self::push_usage_insight(
                insights,
                "teams_data",
                "Microsoft Teams data",
                local_app_data.join("Microsoft").join("Teams"),
                "App data",
                "Teams local application data and cache.",
            );
            Self::push_usage_insight(
                insights,
                "electron_cache",
                "Electron cache",
                local_app_data.join("electron").join("Cache"),
                "Developer cache",
                "Electron download and runtime cache.",
            );
            Self::push_usage_insight(
                insights,
                "electron_builder_cache",
                "Electron Builder cache",
                local_app_data.join("electron-builder").join("Cache"),
                "Developer cache",
                "Electron Builder downloaded tooling and target caches.",
            );
            Self::push_usage_insight(
                insights,
                "android_sdk",
                "Android SDK",
                local_app_data.join("Android").join("Sdk"),
                "Developer tools",
                "Installed Android SDK platforms, build tools, emulators, and system images.",
            );
            Self::push_usage_insight(
                insights,
                "jetbrains_caches",
                "JetBrains caches",
                local_app_data.join("JetBrains"),
                "Developer cache",
                "JetBrains IDE indexes, caches, plugins, and local metadata.",
            );
            Self::push_usage_insight(
                insights,
                "visual_studio_component_cache",
                "Visual Studio component cache",
                local_app_data
                    .join("Microsoft")
                    .join("VisualStudio")
                    .join("Packages"),
                "Installer cache",
                "Visual Studio component package cache.",
            );
            Self::push_usage_insight(
                insights,
                "wsl_packages",
                "WSL package data",
                local_app_data.join("Packages"),
                "App data",
                "Microsoft Store app package data, including WSL distributions and app containers.",
            );
            Self::push_usage_insight(
                insights,
                "unity_cache",
                "Unity cache",
                local_app_data.join("Unity").join("cache"),
                "Developer cache",
                "Unity asset, package, and editor cache.",
            );
            Self::push_usage_insight(
                insights,
                "unreal_derived_data_cache",
                "Unreal Derived Data Cache",
                local_app_data
                    .join("UnrealEngine")
                    .join("Common")
                    .join("DerivedDataCache"),
                "Developer cache",
                "Unreal Engine shader, asset, and derived data cache.",
            );
        }

        if let Some(program_data) = std::env::var_os("ProgramData") {
            let program_data = PathBuf::from(program_data);
            Self::push_usage_insight(
                insights,
                "docker_program_data",
                "Docker shared data",
                program_data.join("Docker"),
                "App data",
                "Docker shared images, layers, and runtime data.",
            );
            Self::push_usage_insight(
                insights,
                "docker_desktop_program_data",
                "Docker Desktop shared data",
                program_data.join("DockerDesktop"),
                "App data",
                "Docker Desktop shared application data.",
            );
            Self::push_usage_insight(
                insights,
                "package_cache",
                "Package Cache",
                program_data.join("Package Cache"),
                "Installer cache",
                "Installer package cache used by Visual Studio and other setup tools.",
            );
            Self::push_usage_insight(
                insights,
                "chocolatey_cache",
                "Chocolatey cache",
                program_data.join("chocolatey").join("cache"),
                "Package manager cache",
                "Chocolatey downloaded package cache.",
            );
            Self::push_usage_insight(
                insights,
                "winget_cache",
                "WinGet package cache",
                program_data
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Packages"),
                "Package manager cache",
                "WinGet package download and installer cache.",
            );
        }
    }

    #[cfg(target_os = "macos")]
    fn push_macos_usage_insights(insights: &mut Vec<UsageInsightDefinition>, home: &Path) {
        let library = home.join("Library");
        let caches = library.join("Caches");
        let definitions = [
            (
                "cargo_registry",
                "Cargo registry",
                home.join(".cargo/registry"),
                "Developer cache",
                "Rust crate registry cache.",
            ),
            (
                "cargo_git",
                "Cargo git cache",
                home.join(".cargo/git"),
                "Developer cache",
                "Rust git dependency cache.",
            ),
            (
                "rustup_toolchains",
                "Rust toolchains",
                home.join(".rustup/toolchains"),
                "Developer tools",
                "Installed Rust toolchains managed by rustup.",
            ),
            (
                "npm_cache",
                "npm cache",
                home.join(".npm"),
                "Developer cache",
                "npm package download cache.",
            ),
            (
                "pnpm_store_macos",
                "pnpm store",
                library.join("pnpm/store"),
                "Developer cache",
                "pnpm content-addressable package store.",
            ),
            (
                "pnpm_cache_macos",
                "pnpm cache",
                caches.join("pnpm"),
                "Developer cache",
                "pnpm metadata and downloaded package cache.",
            ),
            (
                "yarn_cache",
                "Yarn cache",
                caches.join("Yarn"),
                "Developer cache",
                "Yarn package cache.",
            ),
            (
                "pip_cache",
                "pip cache",
                caches.join("pip"),
                "Developer cache",
                "Python package download and wheel cache.",
            ),
            (
                "poetry_cache",
                "Poetry cache",
                caches.join("pypoetry"),
                "Developer cache",
                "Poetry package and virtual environment cache.",
            ),
            (
                "maven_repository",
                "Maven repository",
                home.join(".m2/repository"),
                "Developer cache",
                "Maven dependency artifacts used by Java projects.",
            ),
            (
                "gradle_cache",
                "Gradle cache",
                home.join(".gradle/caches"),
                "Developer cache",
                "Gradle dependency and build caches.",
            ),
            (
                "go_module_cache",
                "Go module cache",
                home.join("go/pkg/mod"),
                "Developer cache",
                "Downloaded Go modules used by Go projects.",
            ),
            (
                "vscode_extensions",
                "VS Code extensions",
                home.join(".vscode/extensions"),
                "Developer tools",
                "Installed Visual Studio Code extensions.",
            ),
            (
                "electron_cache",
                "Electron cache",
                caches.join("electron"),
                "Developer cache",
                "Electron download and runtime cache.",
            ),
            (
                "electron_builder_cache",
                "Electron Builder cache",
                caches.join("electron-builder"),
                "Developer cache",
                "Electron Builder downloaded tooling and target caches.",
            ),
        ];

        for (id, name, path, category, description) in definitions {
            Self::push_usage_insight(insights, id, name, path, category, description);
        }
    }

    fn push_large_app_folders(insights: &mut Vec<UsageInsightDefinition>) {
        let roots = [
            std::env::var_os("ProgramFiles").map(PathBuf::from),
            std::env::var_os("ProgramFiles(x86)").map(PathBuf::from),
            std::env::var_os("LOCALAPPDATA").map(|path| PathBuf::from(path).join("Programs")),
        ];

        for root in roots.into_iter().flatten() {
            let Ok(entries) = std::fs::read_dir(&root) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let bytes = Self::directory_size(&path);
                if bytes < 1024 * 1024 * 1024 {
                    continue;
                }

                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Application")
                    .to_string();
                let id = format!("large_app_{}", Self::id_fragment(&path));
                insights.push(UsageInsightDefinition {
                    id,
                    name,
                    path,
                    category: "Apps over 1 GB".to_string(),
                    description: "Top-level application folder using more than 1 GB.".to_string(),
                });
            }
        }
    }

    fn push_usage_insight(
        insights: &mut Vec<UsageInsightDefinition>,
        id: &str,
        name: &str,
        path: PathBuf,
        category: &str,
        description: &str,
    ) {
        insights.push(UsageInsightDefinition {
            id: id.to_string(),
            name: name.to_string(),
            path,
            category: category.to_string(),
            description: description.to_string(),
        });
    }

    fn id_fragment(path: &Path) -> String {
        path.to_string_lossy()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect()
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

    #[cfg(target_os = "macos")]
    fn volumes() -> Vec<DiskVolumeUsage> {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut volumes = disks
            .list()
            .iter()
            .filter_map(|disk| {
                let mount_point = disk.mount_point().to_string_lossy().into_owned();
                if disk.total_space() == 0
                    || !(mount_point == "/" || mount_point.starts_with("/Volumes/"))
                    || mount_point.starts_with("/System/Volumes/")
                {
                    return None;
                }

                let disk_name = disk.name().to_string_lossy().into_owned();
                Some(DiskVolumeUsage {
                    label: mount_point.clone(),
                    name: if disk_name.is_empty() {
                        mount_point.clone()
                    } else {
                        disk_name
                    },
                    total_bytes: disk.total_space(),
                    free_bytes: disk.available_space(),
                    system_drive: mount_point == "/",
                })
            })
            .collect::<Vec<_>>();
        volumes.sort_by(|left, right| {
            right
                .system_drive
                .cmp(&left.system_drive)
                .then_with(|| left.label.cmp(&right.label))
        });
        volumes
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    fn volumes() -> Vec<DiskVolumeUsage> {
        Vec::new()
    }

    #[cfg(windows)]
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::{DiskCleanupManager, UsageInsightDefinition};
    use std::path::Path;

    #[test]
    fn macos_usage_insights_use_native_developer_cache_paths() {
        let home = Path::new("/Users/example");
        let mut insights = Vec::<UsageInsightDefinition>::new();

        DiskCleanupManager::push_macos_usage_insights(&mut insights, home);

        let path_for = |id: &str| {
            insights
                .iter()
                .find(|insight| insight.id == id)
                .map(|insight| insight.path.as_path())
        };
        assert_eq!(
            path_for("cargo_registry"),
            Some(home.join(".cargo/registry").as_path())
        );
        assert_eq!(path_for("npm_cache"), Some(home.join(".npm").as_path()));
        assert_eq!(
            path_for("pnpm_store_macos"),
            Some(home.join("Library/pnpm/store").as_path())
        );
    }
}
