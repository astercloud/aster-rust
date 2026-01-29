//! Aster Tauri Desktop Application
//!
//! Tauri 版本的 Aster 桌面应用，提供与 Electron 版本相同的功能。

mod commands;
mod state;
mod tray;

use tauri::Manager;

pub use commands::*;
pub use state::*;

/// 运行 Tauri 应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化应用状态
            app.manage(AppState::new());
            
            // 设置系统托盘
            #[cfg(desktop)]
            tray::setup_tray(app)?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::start_session,
            commands::stop_session,
            commands::send_message,
            commands::get_sessions,
            commands::get_session_messages,
            commands::get_providers,
            commands::get_extensions,
            commands::install_extension,
            commands::uninstall_extension,
            commands::get_server_status,
            commands::start_server,
            commands::stop_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
