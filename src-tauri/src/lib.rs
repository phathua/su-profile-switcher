pub mod commands;
pub mod engine;
pub mod filter;
pub mod hash;
pub mod models;
pub mod process;
pub mod tray;
pub mod updater;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppStateWrapper(pub Mutex<models::AppState>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_state = engine::load_state();

    // Check CLI arguments for headless test mode
    let args: Vec<String> = std::env::args().collect();
    if handle_cli_args(&args) {
        return;
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
        .manage(AppStateWrapper(Mutex::new(initial_state)))
        .setup(|app| {
            let _ = tray::setup_tray(app.handle());
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
            commands::apply_update_and_restart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_cli_args(args: &[String]) -> bool {
    let mut switch_profile: Option<String> = None;
    let mut save_profile: Option<String> = None;
    let mut target_version: Option<String> = None;
    let mut check_update = false;
    let mut simulate_update: Option<String> = None;
    let mut hash_check = false;
    let mut list_profiles = false;
    let mut save_plugins = true;
    let mut save_shortcuts = true;

    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg.eq_ignore_ascii_case("--switch") && i + 1 < args.len() {
            switch_profile = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--save-profile") && i + 1 < args.len() {
            save_profile = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--version") && i + 1 < args.len() {
            target_version = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--check-update") {
            check_update = true;
            i += 1;
        } else if arg.eq_ignore_ascii_case("--simulate-update") && i + 1 < args.len() {
            simulate_update = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--hash-check") {
            hash_check = true;
            i += 1;
        } else if arg.eq_ignore_ascii_case("--list-profiles") {
            list_profiles = true;
            i += 1;
        } else if arg.eq_ignore_ascii_case("--plugins") && i + 1 < args.len() {
            save_plugins = args[i + 1].parse().unwrap_or(true);
            i += 2;
        } else if arg.eq_ignore_ascii_case("--shortcuts") && i + 1 < args.len() {
            save_shortcuts = args[i + 1].parse().unwrap_or(true);
            i += 2;
        } else {
            i += 1;
        }
    }

    let is_cli = switch_profile.is_some()
        || save_profile.is_some()
        || check_update
        || simulate_update.is_some()
        || hash_check
        || list_profiles;

    if !is_cli {
        return false;
    }

    let mut state = engine::load_state();
    let ver = target_version.unwrap_or_else(|| {
        if !state.selected_version.is_empty() {
            state.selected_version.clone()
        } else {
            engine::detect_installed_versions().first().cloned().unwrap_or_else(|| "SketchUp 2026".to_string())
        }
    });

    engine::ensure_initialized(&ver, &mut state);

    if check_update || simulate_update.is_some() {
        let info = updater::check_for_updates(simulate_update.as_deref());
        println!("[UPDATE] Current: v{}, Latest: v{}, HasUpdate: {}", info.current_version, info.latest_version, info.has_update);
    }

    if hash_check {
        let roaming = engine::get_roaming_sketchup_path(&ver);
        let local = engine::get_local_sketchup_path(&ver);
        let profiles_dir = engine::get_profiles_dir(&ver);
        let active_hash = hash::compute_configuration_hash(&roaming, Some(&local), state.exclude_render_and_default_plugins);
        let matching = hash::find_matching_profile(&roaming, &local, &profiles_dir, state.exclude_render_and_default_plugins);
        println!("[HASH_CHECK] Version: {}, ActiveHash: {}, MatchingProfile: {}", ver, active_hash, matching.as_deref().unwrap_or("None"));
    }

    if let Some(save_name) = save_profile {
        match engine::save_current_profile_as(&ver, &save_name, save_plugins, save_shortcuts, &mut state) {
            Ok(_) => println!("[SAVE_PROFILE] Saved '{}' for {} (Plugins: {}, Shortcuts: {})", save_name, ver, save_plugins, save_shortcuts),
            Err(e) => eprintln!("[ERROR] {}", e),
        }
    }

    if let Some(target) = switch_profile {
        match engine::switch_profile(&ver, &target, &mut state) {
            Ok(_) => println!("[SWITCH_PROFILE] Switched to '{}' for {}", target, ver),
            Err(e) => eprintln!("[ERROR] {}", e),
        }
    }

    if list_profiles {
        if let Some(cfg) = state.versions.get(&ver) {
            println!("[LIST_PROFILES] Version: {}, Active: {}", ver, cfg.active_profile);
            println!("[ALL_PROFILES] {}", cfg.profiles.join(", "));
            println!("[RECENT_3] {}", cfg.recent_profiles.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
        }
    }

    true
}
