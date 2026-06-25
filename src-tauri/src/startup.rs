use crate::models::{CommandError, StartupApp};

pub struct StartupManager;

impl StartupManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apps(&self) -> Result<Vec<StartupApp>, CommandError> {
        Self::platform_apps()
    }

    #[cfg(windows)]
    fn platform_apps() -> Result<Vec<StartupApp>, CommandError> {
        use std::path::PathBuf;
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let mut apps = Vec::new();
        let registry_sources = [
            (HKEY_CURRENT_USER, "HKCU Run", "Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            (HKEY_LOCAL_MACHINE, "HKLM Run", "Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
        ];

        for (hive, source, path) in registry_sources {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(path, KEY_READ) {
                for value in key.enum_values().flatten() {
                    if let Ok(command) = key.get_value::<String, _>(&value.0) {
                        apps.push(Self::app_from_command(value.0, command, "Registry", source));
                    }
                }
            }
        }

        let startup_folders = [
            std::env::var_os("APPDATA").map(|path| PathBuf::from(path).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")).map(|path| (path, "Startup Folder")),
            std::env::var_os("PROGRAMDATA").map(|path| PathBuf::from(path).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")).map(|path| (path, "Common Startup Folder")),
        ];

        for folder in startup_folders.into_iter().flatten() {
            if let Ok(entries) = std::fs::read_dir(folder.0) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_stem().and_then(|name| name.to_str()).unwrap_or("Startup app").to_string();
                    apps.push(Self::app_from_command(name, path.to_string_lossy().into_owned(), "Startup Folder", folder.1));
                }
            }
        }

        apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        Ok(apps)
    }

    #[cfg(not(windows))]
    fn platform_apps() -> Result<Vec<StartupApp>, CommandError> {
        Ok(Vec::new())
    }

    fn app_from_command(name: String, command: String, startup_type: &str, source: &str) -> StartupApp {
        let path = Self::executable_path(&command);
        StartupApp {
            publisher: Self::publisher(&path),
            status: "Enabled".to_string(),
            impact: Self::impact(&command),
            startup_type: startup_type.to_string(),
            source: source.to_string(),
            command,
            path,
            delay_seconds: None,
            name,
        }
    }

    fn executable_path(command: &str) -> String {
        let trimmed = command.trim();
        if let Some(rest) = trimmed.strip_prefix('"') {
            return rest.split('"').next().unwrap_or(trimmed).to_string();
        }

        trimmed.split_whitespace().next().unwrap_or(trimmed).to_string()
    }

    fn publisher(path: &str) -> String {
        std::path::Path::new(path)
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .find(|component| !component.ends_with(':') && !component.eq_ignore_ascii_case("program files") && !component.eq_ignore_ascii_case("program files (x86)"))
            .unwrap_or("Unknown publisher")
            .to_string()
    }

    fn impact(command: &str) -> String {
        let lower = command.to_lowercase();
        if lower.contains("teams") || lower.contains("discord") || lower.contains("steam") || lower.contains("adobe") {
            return "High".to_string();
        }

        if lower.contains("onedrive") || lower.contains("update") || lower.contains("helper") || lower.contains("launcher") {
            return "Medium".to_string();
        }

        "Low".to_string()
    }
}