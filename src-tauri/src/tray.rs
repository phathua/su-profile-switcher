use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::engine;

pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let state = engine::load_state();
    let installed_versions = engine::detect_installed_versions();
    let current_ver = if !state.selected_version.is_empty() && installed_versions.contains(&state.selected_version) {
        state.selected_version.clone()
    } else {
        installed_versions.first().cloned().unwrap_or_else(|| "SketchUp 2026".to_string())
    };

    let cfg = state.versions.get(&current_ver).cloned().unwrap_or_default();
    let active_str = format!("Hồ sơ: {} ({})", cfg.active_profile, current_ver.replace("SketchUp ", ""));

    let menu = Menu::new(app)?;

    let active_item = MenuItem::with_id(app, "active_info", &active_str, false, None::<&str>)?;
    menu.append(&active_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Submenu for switching SketchUp version (hover to view & select)
    let ver_label = format!("Phiên bản ({})", current_ver.replace("SketchUp ", ""));
    let version_sub = Submenu::with_id(app, "version_sub", &ver_label, true)?;
    for ver in &installed_versions {
        let is_current = ver == &current_ver;
        let ver_item = CheckMenuItem::with_id(
            app,
            format!("set_ver_{}", ver),
            ver,
            true,
            is_current,
            None::<&str>,
        )?;
        version_sub.append(&ver_item)?;
    }
    menu.append(&version_sub)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Recent profiles (up to 3) with real native Windows checkmark
    for (i, p) in cfg.recent_profiles.iter().take(3).enumerate() {
        let is_active = p == &cfg.active_profile;
        let id = format!("recent_{}", i);
        let item = CheckMenuItem::with_id(app, &id, p, true, is_active, None::<&str>)?;
        menu.append(&item)?;
    }
    if !cfg.recent_profiles.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    let save_item = MenuItem::with_id(app, "save_profile", "Lưu hồ sơ", true, None::<&str>)?;
    let auto_update_item = CheckMenuItem::with_id(
        app,
        "toggle_auto_update",
        "Tự kiểm tra cập nhật",
        true,
        state.auto_check_updates,
        None::<&str>,
    )?;
    let open_item = MenuItem::with_id(app, "open_window", "Mở", true, None::<&str>)?;
    let exit_item = MenuItem::with_id(app, "exit_app", "Thoát", true, None::<&str>)?;

    menu.append(&save_item)?;
    menu.append(&auto_update_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&open_item)?;
    menu.append(&exit_item)?;

    Ok(menu)
}

pub fn update_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main_tray") {
        if let Ok(new_menu) = build_tray_menu(app) {
            let _ = tray.set_menu(Some(new_menu));
        }
    }
}

pub fn open_save_dialog(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("save_dialog") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(
            app,
            "save_dialog",
            WebviewUrl::App("index.html?mode=save".into()),
        )
        .title("Lưu hồ sơ")
        .inner_size(360.0, 310.0)
        .resizable(false)
        .decorations(false)
        .center()
        .always_on_top(true)
        .build();
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let menu = build_tray_menu(app)?;
    let icon_image = app.default_window_icon().cloned().unwrap();

    let tray = TrayIconBuilder::with_id("main_tray")
        .icon(icon_image)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "open_window" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                "exit_app" => {
                    app.exit(0);
                }
                "save_profile" => {
                    open_save_dialog(app);
                }
                "toggle_auto_update" => {
                    let mut s = engine::load_state();
                    s.auto_check_updates = !s.auto_check_updates;
                    engine::save_state(&s);
                    update_tray_menu(app);
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.emit("settings-updated", ());
                    }
                }
                s if s.starts_with("set_ver_") => {
                    let target_ver = s["set_ver_".len()..].to_string();
                    let mut s = engine::load_state();
                    s.selected_version = target_ver.clone();
                    engine::ensure_initialized(&target_ver, &mut s);
                    engine::save_state(&s);
                    update_tray_menu(app);
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.emit("version-changed", target_ver);
                    }
                }
                s if s.starts_with("recent_") => {
                    if let Ok(idx) = s["recent_".len()..].parse::<usize>() {
                        let state = engine::load_state();
                        let installed_versions = engine::detect_installed_versions();
                        let current_ver = if !state.selected_version.is_empty() && installed_versions.contains(&state.selected_version) {
                            state.selected_version.clone()
                        } else {
                            installed_versions.first().cloned().unwrap_or_else(|| "SketchUp 2026".to_string())
                        };

                        if let Some(cfg) = state.versions.get(&current_ver) {
                            if let Some(target) = cfg.recent_profiles.iter().take(3).nth(idx) {
                                let target_clone = target.clone();
                                let mut mutable_state = state.clone();
                                let _ = engine::switch_profile(&current_ver, &target_clone, &mut mutable_state);
                                update_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("profile-switched", target_clone);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
