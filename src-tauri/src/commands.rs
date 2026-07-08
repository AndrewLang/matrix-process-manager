use crate::command_knowledge::models::{
    CommandAutocompleteRequest, CommandAutocompleteSuggestion, CommandIndexResult,
    FinishCommandExecutionRequest, InstalledApplicationScanResult, StartCommandExecutionRequest,
    StartCommandExecutionResponse,
};
use crate::disk_cleanup::DiskCleanupManager;
use crate::models::{
    CommandError, DiskCleanupRequest, DiskCleanupResult, DiskCleanupScan,
    DiskUsageInsightCleanupRequest, DiskUsageInsightCleanupResult, PortScan, PortUsage,
    ProcessSnapshot, StartupApp,
    StartupCommandUpdateRequest,
};
use crate::terminal::models::{
    TerminalResizeRequest, TerminalSessionInfo, TerminalSessionRequest, TerminalStartRequest,
    TerminalStartResponse, TerminalStopRequest, TerminalWriteRequest,
};
use crate::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_process_snapshot(state: State<'_, AppState>) -> Result<ProcessSnapshot, CommandError> {
    state.process_manager.snapshot()
}

#[tauri::command]
pub fn get_startup_apps(state: State<'_, AppState>) -> Result<Vec<StartupApp>, CommandError> {
    state.startup_manager.apps()
}

#[tauri::command]
pub fn update_startup_command(
    state: State<'_, AppState>,
    request: StartupCommandUpdateRequest,
) -> Result<(), CommandError> {
    state.startup_manager.update_command(request)
}

#[tauri::command]
pub fn refresh_window_icon(app_handle: AppHandle) -> Result<(), CommandError> {
    let Some(window) = app_handle.get_webview_window("main") else {
        return Err(CommandError::settings_failed("main window is unavailable"));
    };

    let Some(icon) = app_handle.default_window_icon().cloned() else {
        return Err(CommandError::settings_failed("default window icon is unavailable"));
    };

    window
        .set_icon(icon)
        .map_err(|error| CommandError::settings_failed(error.to_string()))
}

#[tauri::command]
pub async fn get_disk_cleanup_scan() -> Result<DiskCleanupScan, CommandError> {
    tauri::async_runtime::spawn_blocking(DiskCleanupManager::scan)
        .await
        .map_err(|error| CommandError::disk_cleanup_failed(error.to_string()))?
}

#[tauri::command]
pub async fn clean_disk(request: DiskCleanupRequest) -> Result<DiskCleanupResult, CommandError> {
    tauri::async_runtime::spawn_blocking(move || DiskCleanupManager::clean(request))
        .await
        .map_err(|error| CommandError::disk_cleanup_failed(error.to_string()))?
}

#[tauri::command]
pub async fn clean_disk_usage_insight(
    request: DiskUsageInsightCleanupRequest,
) -> Result<DiskUsageInsightCleanupResult, CommandError> {
    tauri::async_runtime::spawn_blocking(move || DiskCleanupManager::clean_usage_insight(request))
        .await
        .map_err(|error| CommandError::disk_cleanup_failed(error.to_string()))?
}

#[tauri::command]
pub async fn get_port_scan() -> Result<PortScan, CommandError> {
    tauri::async_runtime::spawn_blocking(scan_ports_impl)
        .await
        .map_err(|error| CommandError::port_scan_failed(error.to_string()))?
}

#[tauri::command]
pub fn open_native_tool(tool_id: String) -> Result<(), CommandError> {
    open_native_tool_impl(&tool_id)
}

#[tauri::command]
pub fn set_start_with_windows(enabled: bool) -> Result<(), CommandError> {
    set_start_with_windows_impl(enabled)
}

#[tauri::command]
pub fn terminate_process(pid: u32) -> Result<(), CommandError> {
    terminate_process_impl(pid)
}

#[tauri::command]
pub fn start_terminal_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    request: TerminalStartRequest,
) -> Result<TerminalStartResponse, CommandError> {
    state.terminal_service.start_session(request, app_handle)
}

#[tauri::command]
pub fn write_terminal_input(
    state: State<'_, AppState>,
    request: TerminalWriteRequest,
) -> Result<(), CommandError> {
    state.terminal_service.write_input(request)
}

#[tauri::command]
pub fn resize_terminal_session(
    state: State<'_, AppState>,
    request: TerminalResizeRequest,
) -> Result<(), CommandError> {
    state.terminal_service.resize_session(request)
}

#[tauri::command]
pub fn stop_terminal_session(
    state: State<'_, AppState>,
    request: TerminalStopRequest,
) -> Result<(), CommandError> {
    state.terminal_service.stop_session(request)
}

#[tauri::command]
pub fn get_terminal_session(
    state: State<'_, AppState>,
    request: TerminalSessionRequest,
) -> Result<TerminalSessionInfo, CommandError> {
    state.terminal_service.get_session(request)
}

#[tauri::command]
pub fn get_active_terminal_session(
    state: State<'_, AppState>,
) -> Result<Option<TerminalSessionInfo>, CommandError> {
    state.terminal_service.active_session()
}

#[tauri::command]
pub fn set_active_terminal_session(
    state: State<'_, AppState>,
    request: TerminalSessionRequest,
) -> Result<(), CommandError> {
    state.terminal_service.set_active_session(request)
}

#[tauri::command]
pub fn scan_installed_applications(
    state: State<'_, AppState>,
) -> Result<InstalledApplicationScanResult, CommandError> {
    state
        .command_knowledge_service
        .scan_installed_applications()
}

#[tauri::command]
pub fn index_commands(state: State<'_, AppState>) -> Result<CommandIndexResult, CommandError> {
    state.command_knowledge_service.index_commands()
}

#[tauri::command]
pub fn autocomplete_commands(
    state: State<'_, AppState>,
    request: CommandAutocompleteRequest,
) -> Result<Vec<CommandAutocompleteSuggestion>, CommandError> {
    state
        .command_knowledge_service
        .autocomplete_commands(request)
}

#[tauri::command]
pub fn start_command_execution(
    state: State<'_, AppState>,
    request: StartCommandExecutionRequest,
) -> Result<StartCommandExecutionResponse, CommandError> {
    state
        .command_knowledge_service
        .start_command_execution(request)
}

#[tauri::command]
pub fn finish_command_execution(
    state: State<'_, AppState>,
    request: FinishCommandExecutionRequest,
) -> Result<(), CommandError> {
    state
        .command_knowledge_service
        .finish_command_execution(request)
}

#[cfg(windows)]
fn set_start_with_windows_impl(enabled: bool) -> Result<(), CommandError> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let run_key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            winreg::enums::KEY_SET_VALUE,
        )
        .map_err(|error| CommandError::settings_failed(error.to_string()))?;

    if enabled {
        let path = std::env::current_exe()
            .map_err(|error| CommandError::settings_failed(error.to_string()))?;
        run_key
            .set_value("Workstation Console", &format!("\"{}\"", path.display()))
            .map_err(|error| CommandError::settings_failed(error.to_string()))
    } else {
        let _ = run_key.delete_value("Matrix Process Manager");
        match run_key.delete_value("Workstation Console") {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CommandError::settings_failed(error.to_string())),
        }
    }
}

#[cfg(not(windows))]
fn set_start_with_windows_impl(_: bool) -> Result<(), CommandError> {
    Err(CommandError::settings_failed(
        "start with Windows is only available on Windows",
    ))
}

#[cfg(windows)]
fn terminate_process_impl(pid: u32) -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;

    let status = std::process::Command::new("taskkill.exe")
        .args(["/PID", &pid.to_string(), "/F"])
        .creation_flags(0x08000000)
        .status()
        .map_err(|error| CommandError::process_action_failed(error.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(CommandError::process_action_failed(format!(
            "taskkill exited with {status}"
        )))
    }
}

#[cfg(not(windows))]
fn terminate_process_impl(_: u32) -> Result<(), CommandError> {
    Err(CommandError::process_action_failed(
        "process termination is only available on Windows",
    ))
}

#[cfg(windows)]
fn scan_ports_impl() -> Result<PortScan, CommandError> {
    use serde::Deserialize;
    use std::os::windows::process::CommandExt;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct WindowsPortRow {
        protocol: String,
        local_address: String,
        local_port: u16,
        remote_address: Option<String>,
        remote_port: Option<u16>,
        state: String,
        pid: Option<u32>,
        process_name: Option<String>,
        process_path: Option<String>,
    }

    let script = r#"
$tcp = Get-NetTCPConnection | ForEach-Object {
  $proc = if ($_.OwningProcess) { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue } else { $null }
  [pscustomobject]@{
    Protocol = 'TCP'
    LocalAddress = $_.LocalAddress
    LocalPort = [int]$_.LocalPort
    RemoteAddress = $_.RemoteAddress
    RemotePort = if ($_.RemotePort -ne $null) { [int]$_.RemotePort } else { $null }
    State = $_.State.ToString()
    Pid = $_.OwningProcess
    ProcessName = if ($proc) { $proc.ProcessName } else { 'System' }
    ProcessPath = if ($proc) { $proc.Path } else { $null }
  }
}
$udp = Get-NetUDPEndpoint | ForEach-Object {
  $proc = if ($_.OwningProcess) { Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue } else { $null }
  [pscustomobject]@{
    Protocol = 'UDP'
    LocalAddress = $_.LocalAddress
    LocalPort = [int]$_.LocalPort
    RemoteAddress = $null
    RemotePort = $null
    State = 'Open'
    Pid = $_.OwningProcess
    ProcessName = if ($proc) { $proc.ProcessName } else { 'System' }
    ProcessPath = if ($proc) { $proc.Path } else { $null }
  }
}
@($tcp + $udp) | Sort-Object LocalPort, Protocol | ConvertTo-Json -Compress -Depth 3
"#;

    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(0x08000000)
        .output()
        .map_err(|error| CommandError::port_scan_failed(error.to_string()))?;

    if !output.status.success() {
        return Err(CommandError::port_scan_failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows: Vec<WindowsPortRow> = match serde_json::from_str(stdout.trim()) {
        Ok(rows) => rows,
        Err(_) => {
            let row: WindowsPortRow = serde_json::from_str(stdout.trim())
                .map_err(|error| CommandError::port_scan_failed(error.to_string()))?;
            vec![row]
        }
    };

    Ok(PortScan {
        scanned_at: unix_timestamp_string(),
        ports: rows
            .into_iter()
            .map(|row| PortUsage {
                protocol: row.protocol,
                local_address: normalize_address(row.local_address),
                local_port: row.local_port,
                remote_address: row.remote_address.map(normalize_address),
                remote_port: row.remote_port,
                state: row.state,
                pid: row.pid,
                process_name: row.process_name.unwrap_or_else(|| "Unknown".to_string()),
                process_path: row.process_path,
            })
            .collect(),
    })
}

#[cfg(not(windows))]
fn scan_ports_impl() -> Result<PortScan, CommandError> {
    let output = std::process::Command::new("lsof")
        .args(["-nP", "-iTCP", "-iUDP"])
        .output()
        .map_err(|error| CommandError::port_scan_failed(error.to_string()))?;

    if !output.status.success() {
        return Err(CommandError::port_scan_failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(PortScan {
        scanned_at: unix_timestamp_string(),
        ports: String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter_map(parse_lsof_port_line)
            .collect(),
    })
}

#[cfg(not(windows))]
fn parse_lsof_port_line(line: &str) -> Option<PortUsage> {
    let columns: Vec<&str> = line.split_whitespace().collect();
    if columns.len() < 9 {
        return None;
    }

    let protocol = columns[7].to_string();
    let endpoint = columns[8].split("->").next().unwrap_or(columns[8]);
    let (local_address, local_port) = split_endpoint(endpoint)?;
    let state = line
        .split('(')
        .nth(1)
        .and_then(|value| value.split(')').next())
        .unwrap_or(if protocol.contains("UDP") { "Open" } else { "Unknown" })
        .to_string();

    Some(PortUsage {
        protocol: if protocol.contains("UDP") { "UDP" } else { "TCP" }.to_string(),
        local_address,
        local_port,
        remote_address: None,
        remote_port: None,
        state,
        pid: columns[1].parse::<u32>().ok(),
        process_name: columns[0].to_string(),
        process_path: None,
    })
}

#[cfg(not(windows))]
fn split_endpoint(endpoint: &str) -> Option<(String, u16)> {
    let (address, port) = endpoint.rsplit_once(':')?;
    Some((normalize_address(address.to_string()), port.parse().ok()?))
}

fn normalize_address(address: String) -> String {
    match address.as_str() {
        "0.0.0.0" | "::" | "*" => "All interfaces".to_string(),
        "127.0.0.1" | "::1" => "localhost".to_string(),
        _ => address,
    }
}

fn unix_timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(windows)]
fn open_native_tool_impl(tool_id: &str) -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;

    if tool_id == "envVariables" {
        return std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "Start-Process rundll32.exe -ArgumentList 'sysdm.cpl,EditEnvironmentVariables' -Verb RunAs",
            ])
            .creation_flags(0x08000000)
            .spawn()
            .map(|_| ())
            .map_err(|error| CommandError::native_tool_failed(error.to_string()));
    }

    let (program, args): (&str, &[&str]) = match tool_id {
        "taskManager" => ("taskmgr.exe", &[]),
        "systemSettings" => ("explorer.exe", &["ms-settings:about"]),
        "diskManager" => ("cmd.exe", &["/C", "start", "", "diskmgmt.msc"]),
        "terminal" => ("wt.exe", &[]),
        "snippingTool" => (
            "explorer.exe",
            &["shell:AppsFolder\\Microsoft.ScreenSketch_8wekyb3d8bbwe!App"],
        ),
        _ => return Err(CommandError::native_tool_failed("unknown native tool")),
    };

    std::process::Command::new(program)
        .args(args)
        .creation_flags(0x08000000)
        .spawn()
        .map(|_| ())
        .map_err(|error| CommandError::native_tool_failed(error.to_string()))
}

#[cfg(not(windows))]
fn open_native_tool_impl(_: &str) -> Result<(), CommandError> {
    Err(CommandError::native_tool_failed(
        "native tools are only available on Windows",
    ))
}
