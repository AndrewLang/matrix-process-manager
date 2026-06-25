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
