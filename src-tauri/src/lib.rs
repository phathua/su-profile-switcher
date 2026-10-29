pub mod changelog;
pub mod cli;
pub mod commands;
pub mod engine;
pub mod filter;
pub mod hash;
pub mod mcp;
pub mod models;
pub mod notification;
pub mod plugin_icon;
pub mod process;
pub mod shortcuts;
pub mod tray;
pub mod updater;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppStateWrapper(pub Mutex<models::AppState>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_state = engine::load_state();

    // Check CLI / MCP arguments for headless mode
    let args: Vec<String> = std::env::args().collect();
    if cli::is_cli_invocation(&args) {
        let code = cli::run_cli(&args);
        std::process::exit(code);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.show();
                let _ = main_win.unminimize();
                let _ = main_win.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppStateWrapper(Mutex::new(initial_state)))
        .setup(|app| {
            #[cfg(windows)]
            notification::ensure_aumid_registered();
            let _ = tray::setup_tray(app.handle());
            shortcuts::register_shortcuts(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Keep running in taskbar tray on close
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::get_installed_versions,
            commands::select_version,
            commands::switch_profile,
            commands::save_profile,
            commands::rename_profile,
            commands::delete_profile,
            commands::update_profile_metadata,
            commands::check_matching_hash,
            commands::is_sketchup_running,
            commands::close_sketchup,
            commands::launch_sketchup,
            commands::export_profile_zip,
            commands::check_for_updates,
            commands::download_update,
            commands::update_settings,
            commands::get_profile_components,
            commands::save_profile_custom,
            commands::analyze_import_source,
            commands::import_profile,
            commands::open_save_dialog,
            commands::get_profile_details,
            commands::apply_update_and_restart,
            commands::get_changelog,
            commands::get_live_changelog,
            commands::clear_changelog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

