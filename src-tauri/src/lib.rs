mod command_knowledge;
mod commands;
mod disk_cleanup;
mod managers;
mod models;
mod providers;
mod startup;
mod terminal;

use command_knowledge::service::CommandKnowledgeService;
use managers::ProcessManager;
use providers::SysinfoProcessProvider;
use startup::StartupManager;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use terminal::service::TerminalService;

pub struct AppState {
    process_manager: ProcessManager<SysinfoProcessProvider>,
    startup_manager: StartupManager,
    terminal_service: TerminalService,
    command_knowledge_service: CommandKnowledgeService,
}

impl AppState {
    fn new() -> Self {
        Self {
            process_manager: ProcessManager::new(SysinfoProcessProvider::new()),
            startup_manager: StartupManager::new(),
            terminal_service: TerminalService::new().expect("terminal service is unavailable"),
            command_knowledge_service: CommandKnowledgeService::new()
                .expect("command knowledge service is unavailable"),
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
            let mut builder = TrayIconBuilder::with_id("main").tooltip("Workstation Console");
            if let Some(icon) = icon.clone() {
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

            if let (Some(window), Some(icon)) = (app.get_webview_window("main"), icon) {
                let _ = window.set_icon(icon);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::autocomplete_commands,
            commands::clean_disk,
            commands::clean_disk_usage_insight,
            commands::finish_command_execution,
            commands::get_disk_cleanup_scan,
            commands::get_port_scan,
            commands::get_process_snapshot,
            commands::get_active_terminal_session,
            commands::get_startup_apps,
            commands::get_terminal_session,
            commands::index_commands,
            commands::open_native_tool,
            commands::refresh_window_icon,
            commands::scan_installed_applications,
            commands::set_start_with_windows,
            commands::set_active_terminal_session,
            commands::start_command_execution,
            commands::start_terminal_session,
            commands::stop_terminal_session,
            commands::terminate_process,
            commands::update_startup_command,
            commands::resize_terminal_session,
            commands::write_terminal_input,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
