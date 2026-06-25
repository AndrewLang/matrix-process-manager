mod commands;
mod managers;
mod models;
mod providers;
mod startup;

use managers::ProcessManager;
use providers::SysinfoProcessProvider;
use startup::StartupManager;

pub struct AppState {
    process_manager: ProcessManager<SysinfoProcessProvider>,
    startup_manager: StartupManager,
}

impl AppState {
    fn new() -> Self {
        Self {
            process_manager: ProcessManager::new(SysinfoProcessProvider::new()),
            startup_manager: StartupManager::new(),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_process_snapshot,
            commands::get_startup_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
