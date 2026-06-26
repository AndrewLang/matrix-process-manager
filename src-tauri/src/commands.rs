use crate::models::{CommandError, ProcessSnapshot, StartupApp};
use crate::AppState;
use tauri::State;

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

#[cfg(windows)]
fn open_native_tool_impl(tool_id: &str) -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;

    let (program, args): (&str, &[&str]) = match tool_id {
        "taskManager" => ("taskmgr.exe", &[]),
        "systemSettings" => ("explorer.exe", &["ms-settings:about"]),
        "diskManager" => ("diskmgmt.msc", &[]),
        "terminal" => ("wt.exe", &[]),
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
