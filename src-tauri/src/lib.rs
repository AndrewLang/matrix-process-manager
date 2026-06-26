mod commands;
mod managers;
mod models;
mod providers;
mod startup;

use managers::ProcessManager;
use providers::SysinfoProcessProvider;
use startup::StartupManager;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

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
        .setup(|app| {
            let icon = app.default_window_icon().cloned();
            let mut builder = TrayIconBuilder::with_id("main").tooltip("Matrix Process Manager");
            if let Some(icon) = icon {
                builder = builder.icon(icon);
            }

            builder
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_process_snapshot,
            commands::get_startup_apps,
            commands::open_native_tool,
            commands::set_start_with_windows,
            commands::terminate_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
