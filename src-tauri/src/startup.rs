use crate::models::{CommandError, StartupApp, StartupCommandUpdateRequest};
use crate::providers::file_icon_data_url;

pub struct StartupManager;

impl StartupManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apps(&self) -> Result<Vec<StartupApp>, CommandError> {
        Self::platform_apps()
    }

    pub fn update_command(&self, request: StartupCommandUpdateRequest) -> Result<(), CommandError> {
        Self::platform_update_command(request)
    }

    #[cfg(windows)]
    fn platform_apps() -> Result<Vec<StartupApp>, CommandError> {
        use std::path::PathBuf;
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let mut apps = Vec::new();
        let registry_sources = [
            (
                HKEY_CURRENT_USER,
                "HKCU Run",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            ),
            (
                HKEY_LOCAL_MACHINE,
                "HKLM Run",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            ),
        ];

        for (hive, source, path, approved_path) in registry_sources {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(path, KEY_READ) {
                for value in key.enum_values().flatten() {
                    if let Ok(command) = key.get_value::<String, _>(&value.0) {
                        let status = Self::startup_status(&root, approved_path, &value.0);
                        let value_name = value.0.clone();
                        apps.push(Self::app_from_command(
                            value.0,
                            command,
                            "Registry",
                            source,
                            status,
                            Some(value_name),
                        ));
                    }
                }
            }
        }

        let startup_folders = [
            std::env::var_os("APPDATA")
                .map(|path| {
                    PathBuf::from(path).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
                })
                .map(|path| (path, "Startup Folder")),
            std::env::var_os("PROGRAMDATA")
                .map(|path| {
                    PathBuf::from(path).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
                })
                .map(|path| (path, "Common Startup Folder")),
        ];

        for folder in startup_folders.into_iter().flatten() {
            if let Ok(entries) = std::fs::read_dir(folder.0) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.eq_ignore_ascii_case("desktop.ini"))
                    {
                        continue;
                    }

                    let name = path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Startup app")
                        .to_string();
                    let value_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(&name);
                    let status = Self::startup_status(&RegKey::predef(HKEY_CURRENT_USER), "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder", value_name);
                    apps.push(Self::app_from_command(
                        name,
                        path.to_string_lossy().into_owned(),
                        "Startup Folder",
                        folder.1,
                        status,
                        None,
                    ));
                }
            }
        }

        apps.extend(Self::packaged_apps());

        apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        Ok(apps)
    }

    #[cfg(not(windows))]
    fn platform_apps() -> Result<Vec<StartupApp>, CommandError> {
        Ok(Vec::new())
    }

    #[cfg(windows)]
    fn platform_update_command(request: StartupCommandUpdateRequest) -> Result<(), CommandError> {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE};
        use winreg::RegKey;

        let command = request.command.trim();
        if command.is_empty() {
            return Err(CommandError::settings_failed(
                "startup command cannot be empty",
            ));
        }

        let (hive, path) = match request.source.as_str() {
            "HKCU Run" => (
                HKEY_CURRENT_USER,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            ),
            "HKLM Run" => (
                HKEY_LOCAL_MACHINE,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            ),
            _ => {
                return Err(CommandError::settings_failed(
                    "only registry startup entries can be edited",
                ))
            }
        };

        RegKey::predef(hive)
            .open_subkey_with_flags(path, KEY_SET_VALUE)
            .and_then(|key| key.set_value(&request.value_name, &command))
            .map_err(|error| CommandError::settings_failed(error.to_string()))
    }

    #[cfg(not(windows))]
    fn platform_update_command(_: StartupCommandUpdateRequest) -> Result<(), CommandError> {
        Err(CommandError::settings_failed(
            "startup command editing is only available on Windows",
        ))
    }

    fn app_from_command(
        name: String,
        command: String,
        startup_type: &str,
        source: &str,
        status: String,
        value_name: Option<String>,
    ) -> StartupApp {
        let path = Self::expand_environment_variables(&Self::executable_path(&command));
        let name = Self::display_name(&path, &name);
        StartupApp {
            publisher: Self::publisher(&path),
            icon_data_url: file_icon_data_url(&path),
            status,
            impact: Self::impact(&command),
            startup_type: startup_type.to_string(),
            source: source.to_string(),
            command,
            path,
            value_name,
            delay_seconds: None,
            name,
        }
    }

    fn executable_path(command: &str) -> String {
        let trimmed = command.trim();
        if let Some(rest) = trimmed.strip_prefix('"') {
            return rest.split('"').next().unwrap_or(trimmed).to_string();
        }

        trimmed
            .split_whitespace()
            .next()
            .unwrap_or(trimmed)
            .to_string()
    }

    fn publisher(path: &str) -> String {
        Self::file_company_name(path).unwrap_or_else(|| Self::path_publisher(path))
    }

    fn display_name(path: &str, fallback: &str) -> String {
        Self::file_version_string(path, "FileDescription")
            .or_else(|| Self::file_version_string(path, "ProductName"))
            .unwrap_or_else(|| Self::spaced_name(fallback))
    }

    fn spaced_name(value: &str) -> String {
        let mut name = String::new();
        let mut previous_lower = false;
        for character in value.replace('_', " ").chars() {
            if previous_lower && character.is_uppercase() {
                name.push(' ');
            }

            previous_lower = character.is_lowercase() || character.is_ascii_digit();
            name.push(character);
        }

        name.trim().to_string()
    }

    fn expand_environment_variables(value: &str) -> String {
        let mut expanded = String::new();
        let mut chars = value.chars().peekable();
        while let Some(character) = chars.next() {
            if character != '%' {
                expanded.push(character);
                continue;
            }

            let mut variable = String::new();
            while let Some(next) = chars.peek().copied() {
                chars.next();
                if next == '%' {
                    break;
                }

                variable.push(next);
            }

            if variable.is_empty() {
                expanded.push('%');
            } else if let Ok(value) = std::env::var(&variable) {
                expanded.push_str(&value);
            } else {
                expanded.push('%');
                expanded.push_str(&variable);
                expanded.push('%');
            }
        }

        expanded
    }

    fn path_publisher(path: &str) -> String {
        std::path::Path::new(path)
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .find(|component| {
                !component.ends_with(':')
                    && !component.eq_ignore_ascii_case("program files")
                    && !component.eq_ignore_ascii_case("program files (x86)")
            })
            .unwrap_or("Unknown publisher")
            .to_string()
    }

    #[cfg(windows)]
    fn startup_status(root: &winreg::RegKey, approved_path: &str, value_name: &str) -> String {
        use winreg::enums::KEY_READ;

        root.open_subkey_with_flags(approved_path, KEY_READ)
            .ok()
            .and_then(|key| key.get_raw_value(value_name).ok())
            .and_then(|value| value.bytes.first().copied())
            .map(|flag| if flag == 3 { "Disabled" } else { "Enabled" }.to_string())
            .unwrap_or_else(|| "Enabled".to_string())
    }

    #[cfg(windows)]
    fn packaged_apps() -> Vec<StartupApp> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
        use winreg::RegKey;

        let root = RegKey::predef(HKEY_CURRENT_USER);
        let Ok(system_app_data) = root.open_subkey_with_flags("Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\SystemAppData", KEY_READ) else {
            return Vec::new();
        };

        let mut apps = Vec::new();
        for package in system_app_data.enum_keys().flatten() {
            let Ok(package_key) = system_app_data.open_subkey_with_flags(&package, KEY_READ) else {
                continue;
            };

            for task in package_key.enum_keys().flatten() {
                if !Self::is_packaged_startup_task(&task) {
                    continue;
                }

                let Ok(task_key) = package_key.open_subkey_with_flags(&task, KEY_READ) else {
                    continue;
                };

                let Ok(state) = task_key.get_value::<u32, _>("State") else {
                    continue;
                };

                let last_disabled_time = Self::registry_number(&task_key, "LastDisabledTime");
                let user_enabled_startup_once =
                    task_key.get_value::<u32, _>("UserEnabledStartupOnce").ok();
                let show_notification = task_key.get_value::<u32, _>("ShowNotification").ok();
                let status = Self::packaged_status(
                    state,
                    last_disabled_time,
                    user_enabled_startup_once,
                    show_notification,
                );
                let name = Self::packaged_name(&package, &task);
                apps.push(StartupApp {
                    publisher: Self::packaged_publisher(&package),
                    icon_data_url: None,
                    status,
                    impact: "Low".to_string(),
                    startup_type: "Startup Task".to_string(),
                    source: "Packaged app".to_string(),
                    command: format!("{package}!{task}"),
                    path: String::new(),
                    value_name: None,
                    delay_seconds: None,
                    name,
                });
            }
        }

        apps
    }

    #[cfg(windows)]
    fn is_packaged_startup_task(task: &str) -> bool {
        let lower = task.to_lowercase();
        lower.contains("startup")
            || lower.contains("start")
            || lower.contains("autostart")
            || lower == "microsoftdefender"
            || lower == "cmdpalstartup"
            || lower == "webviewhoststartupid"
    }

    #[cfg(windows)]
    fn packaged_status(
        state: u32,
        last_disabled_time: Option<u64>,
        user_enabled_startup_once: Option<u32>,
        show_notification: Option<u32>,
    ) -> String {
        if state == 1
            || (state != 2
                && (last_disabled_time.is_some() || user_enabled_startup_once == Some(1)))
            || show_notification == Some(1)
        {
            "Disabled".to_string()
        } else {
            "Enabled".to_string()
        }
    }

    #[cfg(windows)]
    fn registry_number(key: &winreg::RegKey, value_name: &str) -> Option<u64> {
        key.get_value::<u64, _>(value_name)
            .ok()
            .or_else(|| key.get_value::<u32, _>(value_name).ok().map(u64::from))
    }

    #[cfg(windows)]
    fn packaged_name(package: &str, task: &str) -> String {
        if package.contains("WhatsAppDesktop") {
            return "WhatsApp".to_string();
        }

        if package.starts_with("Microsoft.6365217CE6EB4") {
            return "Microsoft Defender".to_string();
        }

        if package.starts_with("Microsoft.CommandPalette") {
            return "Command Palette".to_string();
        }

        if package.starts_with("Microsoft.MicrosoftOfficeHub") {
            return "Microsoft 365 Copilot".to_string();
        }

        if package.starts_with("Microsoft.WindowsTerminal") {
            return "Terminal".to_string();
        }

        if package.starts_with("Microsoft.YourPhone") {
            return "Phone Link".to_string();
        }

        if package.starts_with("MicrosoftWindows.CrossDevice") {
            return "Mobile devices".to_string();
        }

        if package.starts_with("MSTeams") {
            return "Microsoft Teams".to_string();
        }

        Self::spaced_name(task)
    }

    #[cfg(windows)]
    fn packaged_publisher(package: &str) -> String {
        if package.contains("WhatsAppDesktop") {
            return "WhatsApp Inc.".to_string();
        }

        if package.starts_with("MicrosoftWindows.CrossDevice") {
            return "Microsoft Windows".to_string();
        }

        if package.starts_with("Microsoft.")
            || package.starts_with("MicrosoftWindows.")
            || package.starts_with("MSTeams")
        {
            return "Microsoft Corporation".to_string();
        }

        "Unknown publisher".to_string()
    }

    #[cfg(windows)]
    fn file_company_name(path: &str) -> Option<String> {
        use std::ffi::c_void;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW,
        };

        let wide_path = Self::wide(path);
        let mut handle = 0;
        let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if unsafe {
            GetFileVersionInfoW(
                wide_path.as_ptr(),
                0,
                size,
                buffer.as_mut_ptr().cast::<c_void>(),
            )
        } == 0
        {
            return None;
        }

        for (language, codepage) in Self::version_translations(&buffer) {
            let query = format!("\\StringFileInfo\\{language:04x}{codepage:04x}\\CompanyName");
            if let Some(company) = Self::version_string(&buffer, &query) {
                return Some(company);
            }
        }

        Self::version_string(&buffer, "\\StringFileInfo\\040904b0\\CompanyName")
    }

    #[cfg(windows)]
    fn file_version_string(path: &str, key: &str) -> Option<String> {
        use std::ffi::c_void;
        use windows_sys::Win32::Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW,
        };

        let wide_path = Self::wide(path);
        let mut handle = 0;
        let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if unsafe {
            GetFileVersionInfoW(
                wide_path.as_ptr(),
                0,
                size,
                buffer.as_mut_ptr().cast::<c_void>(),
            )
        } == 0
        {
            return None;
        }

        for (language, codepage) in Self::version_translations(&buffer) {
            let query = format!("\\StringFileInfo\\{language:04x}{codepage:04x}\\{key}");
            if let Some(value) = Self::version_string(&buffer, &query) {
                return Some(value);
            }
        }

        Self::version_string(&buffer, &format!("\\StringFileInfo\\040904b0\\{key}"))
    }

    #[cfg(not(windows))]
    fn file_version_string(_path: &str, _key: &str) -> Option<String> {
        None
    }

    #[cfg(not(windows))]
    fn file_company_name(_path: &str) -> Option<String> {
        None
    }

    #[cfg(windows)]
    fn version_translations(buffer: &[u8]) -> Vec<(u16, u16)> {
        use std::ffi::c_void;
        use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

        let mut translation_ptr: *mut c_void = std::ptr::null_mut();
        let mut translation_len = 0;
        let query = Self::wide("\\VarFileInfo\\Translation");
        if unsafe {
            VerQueryValueW(
                buffer.as_ptr().cast::<c_void>(),
                query.as_ptr(),
                &mut translation_ptr,
                &mut translation_len,
            )
        } == 0
            || translation_ptr.is_null()
            || translation_len < 4
        {
            return vec![(0x0409, 0x04b0)];
        }

        let words = unsafe {
            std::slice::from_raw_parts(translation_ptr.cast::<u16>(), translation_len as usize / 2)
        };
        words
            .chunks_exact(2)
            .map(|translation| (translation[0], translation[1]))
            .collect()
    }

    #[cfg(windows)]
    fn version_string(buffer: &[u8], query: &str) -> Option<String> {
        use std::ffi::c_void;
        use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

        let wide_query = Self::wide(query);
        let mut value_ptr: *mut c_void = std::ptr::null_mut();
        let mut value_len = 0;
        if unsafe {
            VerQueryValueW(
                buffer.as_ptr().cast::<c_void>(),
                wide_query.as_ptr(),
                &mut value_ptr,
                &mut value_len,
            )
        } == 0
            || value_ptr.is_null()
            || value_len == 0
        {
            return None;
        }

        let value =
            unsafe { std::slice::from_raw_parts(value_ptr.cast::<u16>(), value_len as usize) };
        let company = String::from_utf16_lossy(value)
            .trim_end_matches('\0')
            .trim()
            .to_string();
        (!company.is_empty()).then_some(company)
    }

    #[cfg(windows)]
    fn wide(value: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn impact(command: &str) -> String {
        let lower = command.to_lowercase();
        if lower.contains("teams")
            || lower.contains("discord")
            || lower.contains("steam")
            || lower.contains("adobe")
        {
            return "High".to_string();
        }

        if lower.contains("onedrive")
            || lower.contains("update")
            || lower.contains("helper")
            || lower.contains("launcher")
        {
            return "Medium".to_string();
        }

        "Low".to_string()
    }
}
