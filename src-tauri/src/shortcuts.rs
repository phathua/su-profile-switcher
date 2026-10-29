use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::str::FromStr;
use crate::{engine, notification, process};

pub fn register_shortcuts(app: &AppHandle) {
    let global_shortcut = app.global_shortcut();
    let _ = global_shortcut.unregister_all();

    let state = engine::load_state();
    let installed = engine::detect_installed_versions();
    let current_ver = if !state.selected_version.is_empty() && installed.contains(&state.selected_version) {
        state.selected_version.clone()
    } else {
        installed.first().cloned().unwrap_or_else(|| "SketchUp 2026".to_string())
    };

    if let Some(cfg) = state.versions.get(&current_ver) {
        for (profile_name, meta) in &cfg.profiles_metadata {
            if let Some(sc_str) = &meta.shortcut {
                let trimmed = sc_str.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(shortcut) = Shortcut::from_str(trimmed) {
                    let p_name = profile_name.clone();
                    let ver = current_ver.clone();
                    let app_handle = app.clone();

                    let _ = global_shortcut.on_shortcut(shortcut, move |_app, _shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                            handle_shortcut_switch(&app_handle, &ver, &p_name);
                        }
                    });
                }
            }
        }
    }
}

fn handle_shortcut_switch(app: &AppHandle, version: &str, target_profile: &str) {
    // If SketchUp is running, we notify the frontend to show the confirmation dialog
    if process::is_sketchup_running() {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.set_focus();
            let _ = w.emit("request-confirm-switch", serde_json::json!({
                "version": version,
                "targetProfile": target_profile,
            }));
        }
        return;
    }

    let mut state = engine::load_state();
    if engine::switch_profile(version, target_profile, &mut state).is_ok() {
        crate::tray::update_tray_menu(app);
        notification::show_profile_switched_toast(target_profile);
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.emit("profile-switched", target_profile);
        }
    }
}
