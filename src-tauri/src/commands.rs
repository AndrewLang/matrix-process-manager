use crate::command_knowledge::models::{
    CommandAutocompleteRequest, CommandAutocompleteSuggestion, CommandIndexResult,
    FinishCommandExecutionRequest, InstalledApplicationScanResult, StartCommandExecutionRequest,
    StartCommandExecutionResponse,
};
use crate::models::{CommandError, ProcessSnapshot, StartupApp};
use crate::terminal::models::{
    TerminalResizeRequest, TerminalSessionInfo, TerminalSessionRequest, TerminalStartRequest,
    TerminalStartResponse, TerminalStopRequest, TerminalWriteRequest,
};
use crate::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_process_snapshot(state: State<'_, AppState>) -> Result<ProcessSnapshot, CommandError> {
    state.process_manager.snapshot()
}

#[tauri::command]
pub fn get_startup_apps(state: State<'_, AppState>) -> Result<Vec<StartupApp>, CommandError> {
    state.startup_manager.apps()
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
            .set_value("Matrix Process Manager", &format!("\"{}\"", path.display()))
            .map_err(|error| CommandError::settings_failed(error.to_string()))
    } else {
        match run_key.delete_value("Matrix Process Manager") {
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
fn open_native_tool_impl(tool_id: &str) -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;

    let (program, args): (&str, &[&str]) = match tool_id {
        "taskManager" => ("taskmgr.exe", &[]),
        "systemSettings" => ("explorer.exe", &["ms-settings:about"]),
        "diskManager" => ("cmd.exe", &["/C", "start", "", "diskmgmt.msc"]),
        "terminal" => ("wt.exe", &[]),
        "envVariables" => ("rundll32.exe", &["sysdm.cpl,EditEnvironmentVariables"]),
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
