use tauri::{Emitter, State};
use crate::models::{AppState, ChangelogEntry, ImportAnalysis, LiveDiff, ProfileComponents, ProfileDetails, UpdateInfo};
use crate::{changelog, engine, hash, process, updater, AppStateWrapper};

#[tauri::command]
pub fn get_app_state(state_wrap: State<'_, AppStateWrapper>) -> AppState {
    let mut state = state_wrap.0.lock().unwrap();
    *state = engine::load_state();
    state.clone()
}

#[tauri::command]
pub fn get_installed_versions() -> Vec<String> {
    engine::detect_installed_versions()
}

#[tauri::command]
pub fn select_version(
    version: String,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    state.selected_version = version.clone();
    engine::ensure_initialized(&version, &mut state);
    engine::save_state(&state);
    crate::tray::update_tray_menu(&app);
    Ok(state.clone())
}

#[tauri::command]
pub fn switch_profile(
    version: String,
    target_profile: String,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::switch_profile(&version, &target_profile, &mut state)?;
    crate::tray::update_tray_menu(&app);
    crate::notification::show_profile_switched_toast(&target_profile);
    Ok(state.clone())
}

#[tauri::command]
pub fn save_profile(
    version: String,
    name: String,
    save_plugins: bool,
    save_shortcuts: bool,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::save_current_profile_as(&version, &name, save_plugins, save_shortcuts, &mut state)?;
    crate::tray::update_tray_menu(&app);
    Ok(state.clone())
}

#[tauri::command]
pub fn rename_profile(
    version: String,
    old_name: String,
    new_name: String,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::rename_profile(&version, &old_name, &new_name, &mut state)?;
    crate::tray::update_tray_menu(&app);
    Ok(state.clone())
}

#[tauri::command]
pub fn delete_profile(
    version: String,
    name: String,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::delete_profile(&version, &name, &mut state)?;
    crate::tray::update_tray_menu(&app);
    Ok(state.clone())
}

#[tauri::command]
pub fn check_matching_hash(version: String, state_wrap: State<'_, AppStateWrapper>) -> Option<String> {
    let state = state_wrap.0.lock().unwrap();
    let roaming = engine::get_roaming_sketchup_path(&version);
    let local = engine::get_local_sketchup_path(&version);
    let profiles_dir = engine::get_profiles_dir(&version);

    hash::find_matching_profile(&roaming, &local, &profiles_dir, state.exclude_render_and_default_plugins)
}

#[tauri::command]
pub fn is_sketchup_running() -> bool {
    process::is_sketchup_running()
}

#[tauri::command]
pub fn close_sketchup() -> bool {
    process::close_sketchup_gracefully()
}

#[tauri::command]
pub fn launch_sketchup(version: String, state_wrap: State<'_, AppStateWrapper>) -> bool {
    let state = state_wrap.0.lock().unwrap();
    let custom = state.versions.get(&version).and_then(|c| c.custom_exe_path.as_deref());
    process::launch_sketchup(&version, custom)
}

#[tauri::command]
pub fn export_profile_zip(
    version: String,
    profile_name: String,
    zip_path: String,
    state_wrap: State<'_, AppStateWrapper>,
) -> Result<(), String> {
    let state = state_wrap.0.lock().unwrap();
    engine::export_profile_zip(&version, &profile_name, std::path::Path::new(&zip_path), &state)
}

#[tauri::command]
pub fn check_for_updates(simulate: Option<String>) -> UpdateInfo {
    updater::check_for_updates(simulate.as_deref())
}

#[tauri::command]
pub fn download_update(url: String, latest_version: String) -> Result<String, String> {
    let path = updater::download_update(&url, &latest_version)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn apply_update_and_restart(
    app: tauri::AppHandle,
    download_url: Option<String>,
    latest_version: String,
    restart_on_finish: Option<bool>,
) -> Result<(), String> {
    updater::apply_update_and_restart(
        &app,
        download_url.as_deref(),
        &latest_version,
        restart_on_finish.unwrap_or(true),
    )
}

#[tauri::command]
pub fn update_settings(
    auto_check_updates: bool,
    exclude_render: bool,
    language: String,
    theme: Option<String>,
    ignored_update_version: Option<String>,
    remind_later: bool,
    remap_user_paths: Option<bool>,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> AppState {
    let mut state = state_wrap.0.lock().unwrap();
    state.auto_check_updates = auto_check_updates;
    state.exclude_render_and_default_plugins = exclude_render;
    if let Some(r) = remap_user_paths {
        state.remap_user_paths = r;
    }
    state.language = language;
    if let Some(t) = theme {
        state.theme = t;
    }
    if let Some(ignored) = ignored_update_version {
        state.ignored_update_version = ignored;
    }
    if remind_later {
        let tomorrow = chrono::Utc::now() + chrono::Duration::days(1);
        state.remind_later_until = Some(tomorrow.to_rfc3339());
    }
    engine::save_state(&state);
    crate::tray::update_tray_menu(&app);
    state.clone()
}

#[tauri::command]
pub fn update_profile_metadata(
    version: String,
    profile: String,
    icon: Option<String>,
    color: Option<String>,
    shortcut: Option<String>,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    if let Some(cfg) = state.versions.get_mut(&version) {
        let meta = cfg.profiles_metadata.entry(profile).or_default();
        meta.icon = icon;
        meta.color = color;
        meta.shortcut = shortcut;
    }
    engine::save_state(&state);
    crate::tray::update_tray_menu(&app);
    crate::shortcuts::register_shortcuts(&app);
    Ok(state.clone())
}

#[tauri::command]
pub fn get_profile_components(version: String, state_wrap: State<'_, AppStateWrapper>) -> Vec<ProfileComponents> {
    let state = state_wrap.0.lock().unwrap();
    engine::get_all_profile_components(&version, &state)
}

#[tauri::command]
pub fn save_profile_custom(
    version: String,
    name: String,
    save_plugins: bool,
    save_shortcuts: bool,
    save_toolbars: bool,
    save_materials: bool,
    save_templates: bool,
    icon: Option<String>,
    color: Option<String>,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::save_profile_custom(
        &version,
        &name,
        save_plugins,
        save_shortcuts,
        save_toolbars,
        save_materials,
        save_templates,
        icon,
        color,
        &mut state,
    )?;
    crate::tray::update_tray_menu(&app);
    let _ = app.emit("profiles-updated", &*state);
    Ok(state.clone())
}

#[tauri::command]
pub fn get_profile_details(
    version: String,
    profile_name: String,
    state_wrap: State<'_, AppStateWrapper>,
) -> Result<ProfileDetails, String> {
    let state = state_wrap.0.lock().unwrap();
    engine::get_profile_details(&version, &profile_name, &state)
}

#[tauri::command]
pub fn analyze_import_source(source_path: String) -> Result<ImportAnalysis, String> {
    engine::analyze_import_source(&source_path)
}

#[tauri::command]
pub fn import_profile(
    version: String,
    profile_name: String,
    source_path: String,
    import_plugins: bool,
    import_shortcuts: bool,
    import_toolbars: bool,
    import_materials: bool,
    import_templates: bool,
    apply_immediately: bool,
    state_wrap: State<'_, AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<AppState, String> {
    let mut state = state_wrap.0.lock().unwrap();
    engine::import_profile(
        &version,
        &profile_name,
        &source_path,
        import_plugins,
        import_shortcuts,
        import_toolbars,
        import_materials,
        import_templates,
        apply_immediately,
        &mut state,
    )?;
    crate::tray::update_tray_menu(&app);
    Ok(state.clone())
}


#[tauri::command]
pub fn open_save_dialog(app: tauri::AppHandle) {
    crate::tray::open_save_dialog(&app);
}

#[tauri::command]
pub fn get_changelog() -> Vec<ChangelogEntry> {
    changelog::load_changelog()
}

#[tauri::command]
pub fn get_live_changelog(version: String, state_wrap: State<'_, AppStateWrapper>) -> LiveDiff {
    let state = state_wrap.0.lock().unwrap();
    changelog::get_live_diff(&version, &state)
}

#[tauri::command]
pub fn clear_changelog() -> Result<(), String> {
    changelog::clear_changelog()
}
