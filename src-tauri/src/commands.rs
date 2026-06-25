use crate::models::{CommandError, ProcessSnapshot};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_process_snapshot(state: State<'_, AppState>) -> Result<ProcessSnapshot, CommandError> {
    state.process_manager.snapshot()
}
