use crate::command_knowledge::models::{
    CommandAutocompleteRequest, CommandAutocompleteSuggestion, CommandIndexResult,
    FinishCommandExecutionRequest, InstalledApplicationScanResult, StartCommandExecutionRequest,
    StartCommandExecutionResponse,
};
use crate::disk_cleanup::DiskCleanupManager;
use crate::models::{
    CommandError, DiskCleanupRequest, DiskCleanupResult, DiskCleanupScan,
    DiskUsageInsightCleanupRequest, DiskUsageInsightCleanupResult, DockerAvailability,
    DockerContainer, DockerDashboard, DockerImage, DockerRegistryImage,
    DockerRegistryRequest, PortScan, PortUsage, ProcessSnapshot, SshKeyGenerationRequest,
    SshKeyInfo, StartupApp,
    StartupCommandUpdateRequest,
};
use reqwest::header::LINK;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
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
pub async fn list_ssh_keys() -> Result<Vec<SshKeyInfo>, CommandError> {
    tauri::async_runtime::spawn_blocking(list_ssh_keys_impl)
        .await
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?
}

#[tauri::command]
pub async fn generate_ssh_key(
    request: SshKeyGenerationRequest,
) -> Result<SshKeyInfo, CommandError> {
    tauri::async_runtime::spawn_blocking(move || generate_ssh_key_impl(request))
        .await
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?
}

#[tauri::command]
pub async fn get_docker_availability() -> Result<DockerAvailability, CommandError> {
    tauri::async_runtime::spawn_blocking(docker_availability_impl)
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn get_docker_dashboard() -> Result<DockerDashboard, CommandError> {
    tauri::async_runtime::spawn_blocking(docker_dashboard_impl)
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn run_docker_container_action(
    container_id: String,
    action: String,
) -> Result<(), CommandError> {
    tauri::async_runtime::spawn_blocking(move || docker_container_action_impl(&container_id, &action))
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn remove_docker_image(image_id: String) -> Result<(), CommandError> {
    tauri::async_runtime::spawn_blocking(move || run_docker_text(&["rmi", &image_id]).map(|_| ()))
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn get_docker_container_inspect(container_id: String) -> Result<String, CommandError> {
    tauri::async_runtime::spawn_blocking(move || run_docker_text(&["inspect", &container_id]))
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn get_docker_container_logs(container_id: String) -> Result<String, CommandError> {
    tauri::async_runtime::spawn_blocking(move || run_docker_text(&["logs", "--tail", "200", &container_id]))
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
}

#[tauri::command]
pub async fn list_docker_registry_images(
    request: DockerRegistryRequest,
) -> Result<Vec<DockerRegistryImage>, CommandError> {
    tauri::async_runtime::spawn_blocking(move || DockerRegistryClient::new(request)?.images())
        .await
        .map_err(|error| CommandError::docker_failed(error.to_string()))?
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

fn list_ssh_keys_impl() -> Result<Vec<SshKeyInfo>, CommandError> {
    let ssh_dir = ssh_directory()?;
    if !ssh_dir.exists() {
        return Ok(Vec::new());
    }

    let mut keys = Vec::new();
    for entry in std::fs::read_dir(&ssh_dir)
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?
    {
        let entry = entry.map_err(|error| CommandError::ssh_key_failed(error.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("pub") {
            continue;
        }

        if let Ok(key) = ssh_key_info_from_public_path(path) {
            keys.push(key);
        }
    }

    keys.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(keys)
}

fn generate_ssh_key_impl(request: SshKeyGenerationRequest) -> Result<SshKeyInfo, CommandError> {
    let ssh_dir = ssh_directory()?;
    std::fs::create_dir_all(&ssh_dir)
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?;

    let file_name = sanitize_ssh_key_file_name(&request.file_name)?;
    let key_type = match request.key_type.as_str() {
        "ed25519" => "ed25519",
        "rsa" => "rsa",
        _ => return Err(CommandError::ssh_key_failed("unsupported SSH key type")),
    };
    let private_path = ssh_dir.join(file_name);
    let public_path = private_path.with_extension("pub");
    if private_path.exists() || public_path.exists() {
        return Err(CommandError::ssh_key_failed("SSH key already exists"));
    }

    let mut command = std::process::Command::new("ssh-keygen");
    command
        .args(["-t", key_type, "-f"])
        .arg(&private_path)
        .args(["-C", request.comment.trim(), "-N", ""]);

    if key_type == "rsa" {
        command.args(["-b", "4096"]);
    }

    let output = command
        .output()
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?;

    if !output.status.success() {
        return Err(CommandError::ssh_key_failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    ssh_key_info_from_public_path(public_path)
}

fn ssh_directory() -> Result<std::path::PathBuf, CommandError> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| CommandError::ssh_key_failed("home directory is unavailable"))?;
    Ok(std::path::PathBuf::from(home).join(".ssh"))
}

fn sanitize_ssh_key_file_name(file_name: &str) -> Result<String, CommandError> {
    let trimmed = file_name.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed == "."
        || trimmed == ".."
        || !trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
    {
        return Err(CommandError::ssh_key_failed("invalid SSH key file name"));
    }

    Ok(trimmed.to_string())
}

fn ssh_key_info_from_public_path(public_path: std::path::PathBuf) -> Result<SshKeyInfo, CommandError> {
    let public_key = std::fs::read_to_string(&public_path)
        .map_err(|error| CommandError::ssh_key_failed(error.to_string()))?
        .trim()
        .to_string();
    let private_path = public_path.with_extension("");
    let name = private_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string();
    let mut parts = public_key.split_whitespace();
    let key_type = parts.next().unwrap_or("unknown").to_string();
    let _body = parts.next();
    let comment = parts.collect::<Vec<_>>().join(" ");
    let modified_at = std::fs::metadata(&public_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string());

    Ok(SshKeyInfo {
        name,
        key_type,
        public_key_path: public_path.display().to_string(),
        private_key_path: private_path.exists().then(|| private_path.display().to_string()),
        public_key,
        fingerprint: ssh_key_fingerprint(&public_path),
        comment: (!comment.is_empty()).then_some(comment),
        modified_at,
        has_private_key: private_path.exists(),
    })
}

fn ssh_key_fingerprint(public_path: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("ssh-keygen")
        .arg("-lf")
        .arg(public_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.trim().to_string()).filter(|value| !value.is_empty())
}

fn docker_availability_impl() -> Result<DockerAvailability, CommandError> {
    let output = std::process::Command::new("docker")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(DockerAvailability {
            installed: true,
            version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        }),
        Ok(_) => Ok(DockerAvailability {
            installed: false,
            version: None,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DockerAvailability {
            installed: false,
            version: None,
        }),
        Err(error) => Err(CommandError::docker_failed(error.to_string())),
    }
}

fn docker_dashboard_impl() -> Result<DockerDashboard, CommandError> {
    let availability = docker_availability_impl()?;
    if !availability.installed {
        return Ok(DockerDashboard {
            installed: false,
            running: false,
            version: None,
            server_version: None,
            error: Some("Docker CLI is not installed.".to_string()),
            containers: Vec::new(),
            images: Vec::new(),
        });
    }

    let server_version = run_docker_text(&["version", "--format", "{{.Server.Version}}"]);
    let running = server_version.is_ok();
    if !running {
        let error = match server_version.err() {
            Some(error) => error.message,
            None => "Docker daemon is not running.".to_string(),
        };
        return Ok(DockerDashboard {
            installed: true,
            running: false,
            version: availability.version,
            server_version: None,
            error: Some(error),
            containers: Vec::new(),
            images: Vec::new(),
        });
    }

    Ok(DockerDashboard {
        installed: true,
        running: true,
        version: availability.version,
        server_version: server_version.ok(),
        error: None,
        containers: docker_containers()?,
        images: docker_images()?,
    })
}

fn docker_container_action_impl(container_id: &str, action: &str) -> Result<(), CommandError> {
    let command = match action {
        "start" => "start",
        "stop" => "stop",
        "restart" => "restart",
        "remove" => "rm",
        "forceRemove" => "rm",
        _ => return Err(CommandError::docker_failed("unsupported Docker container action")),
    };

    if action == "forceRemove" {
        run_docker_text(&[command, "-f", container_id]).map(|_| ())
    } else {
        run_docker_text(&[command, container_id]).map(|_| ())
    }
}

fn docker_containers() -> Result<Vec<DockerContainer>, CommandError> {
    let text = run_docker_text(&["ps", "-a", "--format", "{{json .}}"])?;
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .map(|value| {
            let state = json_string(&value, "State");
            let labels = json_string(&value, "Labels");
            let parent_name = docker_label_value(&labels, "com.docker.compose.project");
            let service_name = docker_label_value(&labels, "com.docker.compose.service");
            DockerContainer {
                id: json_string(&value, "ID"),
                name: json_string(&value, "Names"),
                image: json_string(&value, "Image"),
                parent_name,
                service_name,
                state: state.clone(),
                status: json_string(&value, "Status"),
                ports: json_string(&value, "Ports"),
                created: json_string(&value, "CreatedAt"),
                running: state.eq_ignore_ascii_case("running"),
            }
        })
        .collect())
}

fn docker_images() -> Result<Vec<DockerImage>, CommandError> {
    let text = run_docker_text(&["images", "--format", "{{json .}}"])?;
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .map(|value| DockerImage {
            id: json_string(&value, "ID"),
            repository: json_string(&value, "Repository"),
            tag: json_string(&value, "Tag"),
            size: json_string(&value, "Size"),
            created: json_string(&value, "CreatedSince"),
        })
        .collect())
}

#[derive(Deserialize)]
struct DockerRegistryCatalogPage {
    repositories: Vec<String>,
}

#[derive(Deserialize)]
struct DockerRegistryTagsPage {
    tags: Option<Vec<String>>,
}

struct DockerRegistryClient {
    client: reqwest::blocking::Client,
    base_url: String,
    username: String,
    password: String,
}

impl DockerRegistryClient {
    fn new(request: DockerRegistryRequest) -> Result<Self, CommandError> {
        let registry = request.registry.trim().trim_end_matches('/');
        if registry.is_empty() {
            return Err(CommandError::docker_failed("registry URL is required"));
        }

        let base_url = if registry.starts_with("http://") || registry.starts_with("https://") {
            registry.to_string()
        } else {
            format!("https://{registry}")
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| CommandError::docker_failed(error.to_string()))?;

        Ok(Self {
            client,
            base_url,
            username: request.username,
            password: request.password,
        })
    }

    fn images(&self) -> Result<Vec<DockerRegistryImage>, CommandError> {
        self.repositories()?
            .into_iter()
            .map(|repository| {
                let tags = self.tags(&repository)?;
                Ok(DockerRegistryImage { repository, tags })
            })
            .collect()
    }

    fn repositories(&self) -> Result<Vec<String>, CommandError> {
        let mut repositories = Vec::new();
        let mut next_url = Some(format!("{}/v2/_catalog?n=100", self.base_url));

        while let Some(url) = next_url {
            let (page, link) = self.get_json_with_link::<DockerRegistryCatalogPage>(&url)?;
            repositories.extend(page.repositories);
            next_url = link.and_then(|value| self.next_url(&value));
        }

        repositories.sort();
        repositories.dedup();
        Ok(repositories)
    }

    fn tags(&self, repository: &str) -> Result<Vec<String>, CommandError> {
        let page = self.get_json::<DockerRegistryTagsPage>(&format!("{}/v2/{repository}/tags/list", self.base_url))?;
        let mut tags = page.tags.unwrap_or_default();
        tags.sort();
        Ok(tags)
    }

    fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, CommandError> {
        self.get_json_with_link(url).map(|(value, _)| value)
    }

    fn get_json_with_link<T: DeserializeOwned>(&self, url: &str) -> Result<(T, Option<String>), CommandError> {
        let mut request = self.client.get(url);
        if !self.username.trim().is_empty() || !self.password.is_empty() {
            request = request.basic_auth(self.username.trim(), Some(&self.password));
        }

        let response = request
            .send()
            .map_err(|error| CommandError::docker_failed(error.to_string()))?;

        let status = response.status();
        let link = response
            .headers()
            .get(LINK)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        if !status.is_success() {
            let message = response.text().unwrap_or_else(|_| status.to_string());
            return Err(CommandError::docker_failed(format!("registry request failed: {status} {message}")));
        }

        response
            .json::<T>()
            .map(|value| (value, link))
            .map_err(|error| CommandError::docker_failed(error.to_string()))
    }

    fn next_url(&self, link: &str) -> Option<String> {
        let target = link
            .split(',')
            .find(|item| item.contains("rel=\"next\""))?
            .split_once('<')?
            .1
            .split_once('>')?
            .0;

        if target.starts_with("http://") || target.starts_with("https://") {
            Some(target.to_string())
        } else if target.starts_with('/') {
            Some(format!("{}{}", self.base_url, target))
        } else {
            Some(format!("{}/{}", self.base_url, target))
        }
    }
}

fn run_docker_text(args: &[&str]) -> Result<String, CommandError> {
    let output = std::process::Command::new("docker")
        .args(args)
        .output()
        .map_err(|error| CommandError::docker_failed(error.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(CommandError::docker_failed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .to_string()
}

fn docker_label_value(labels: &str, key: &str) -> Option<String> {
    labels
        .split(',')
        .filter_map(|label| label.split_once('='))
        .find_map(|(label_key, label_value)| (label_key == key && !label_value.is_empty()).then(|| label_value.to_string()))
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
