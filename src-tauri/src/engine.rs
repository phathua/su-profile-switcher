use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use std::collections::HashMap;
use crate::filter;
use crate::models::{AppState, ImportAnalysis, PluginItem, ProfileComponents, ProfileDetails, ShortcutItem, VersionConfig};
use zip::read::ZipArchive;
use crate::process;

pub fn get_app_storage_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = Path::new(&appdata).join("su-profile-switcher");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_state_file_path() -> PathBuf {
    get_app_storage_dir().join("app_state.json")
}

pub fn load_state() -> AppState {
    let path = get_state_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                return state;
            }
        }
    }
    AppState::default()
}

pub fn save_state(state: &AppState) {
    let path = get_state_file_path();
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}

pub fn get_roaming_sketchup_path(version_display: &str) -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    Path::new(&appdata)
        .join("SketchUp")
        .join(version_display)
        .join("SketchUp")
}

pub fn get_local_sketchup_path(version_display: &str) -> PathBuf {
    let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    Path::new(&localappdata)
        .join("SketchUp")
        .join(version_display)
        .join("SketchUp")
}

pub fn get_profiles_dir(version_display: &str) -> PathBuf {
    get_roaming_sketchup_path(version_display).join("Profiles")
}

pub fn remap_preferences_paths(file_path: &Path) {
    if !file_path.exists() {
        return;
    }
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let user_profile = match std::env::var("USERPROFILE") {
        Ok(p) => p,
        Err(_) => return,
    };
    let norm_current_profile = user_profile.replace('/', "\\").trim_end_matches('\\').to_string();

    let mut val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut modified = false;

    fn remap_value(v: &mut serde_json::Value, current_user_dir: &str, modified: &mut bool) {
        match v {
            serde_json::Value::String(s) => {
                let norm = s.replace('/', "\\");
                let lower = norm.to_lowercase();
                if let Some(idx) = lower.find(":\\users\\") {
                    let after_users = &norm[idx + 8..];
                    if let Some(slash_idx) = after_users.find('\\') {
                        let old_user = &after_users[..slash_idx];
                        let rest_of_path = &after_users[slash_idx..];
                        let current_username = current_user_dir.split('\\').last().unwrap_or("");
                        if !old_user.eq_ignore_ascii_case(current_username) {
                            let mut new_path = format!("{}{}", current_user_dir, rest_of_path);
                            if s.ends_with('\\') && !new_path.ends_with('\\') {
                                new_path.push('\\');
                            }
                            *s = new_path.clone();
                            *modified = true;

                            let p = PathBuf::from(&new_path);
                            let dir_to_create = if p.extension().is_some() {
                                p.parent().map(|p| p.to_path_buf())
                            } else {
                                Some(p)
                            };
                            if let Some(d) = dir_to_create {
                                let _ = fs::create_dir_all(&d);
                            }
                        }
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    remap_value(item, current_user_dir, modified);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, item) in map {
                    remap_value(item, current_user_dir, modified);
                }
            }
            _ => {}
        }
    }

    remap_value(&mut val, &norm_current_profile, &mut modified);

    if modified {
        if let Ok(serialized) = serde_json::to_string_pretty(&val) {
            let _ = fs::write(file_path, serialized);
        }
    }
}

pub fn detect_installed_versions() -> Vec<String> {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let su_root = Path::new(&appdata).join("SketchUp");
    let mut versions = Vec::new();

    if su_root.exists() {
        if let Ok(entries) = fs::read_dir(&su_root) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("SketchUp ") && entry.path().join("SketchUp").exists() {
                            versions.push(name);
                        }
                    }
                }
            }
        }
    }

    if versions.is_empty() {
        versions.push("SketchUp 2026".to_string());
        versions.push("SketchUp 2025".to_string());
        versions.push("SketchUp 2024".to_string());
        versions.push("SketchUp 2023".to_string());
    } else {
        versions.sort();
        versions.reverse();
    }

    versions
}

pub fn ensure_initialized(version_display: &str, state: &mut AppState) {
    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let profiles_dir = get_profiles_dir(version_display);

    let _ = fs::create_dir_all(&roaming);
    let _ = fs::create_dir_all(&local);
    let _ = fs::create_dir_all(&profiles_dir);

    let config = state
        .versions
        .entry(version_display.to_string())
        .or_insert_with(VersionConfig::default);

    // Read real profile directories on disk
    let mut dir_profiles = Vec::new();
    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.trim().is_empty() {
                        dir_profiles.push(name);
                    }
                }
            }
        }
    }

    // Check .current_profile marker
    let marker = roaming.join(".current_profile");
    if marker.exists() {
        if let Ok(marked) = fs::read_to_string(&marker) {
            let trimmed = marked.trim().to_string();
            if !trimmed.is_empty() {
                config.active_profile = trimmed;
            }
        }
    }

    if config.active_profile.is_empty() {
        config.active_profile = dir_profiles.first().cloned().unwrap_or_else(|| "Default".to_string());
    }

    // Prune profiles that no longer exist on disk
    config.profiles.retain(|p| dir_profiles.contains(p) || p == &config.active_profile);

    // Add existing directories to config profiles
    for p in &dir_profiles {
        if !config.profiles.contains(p) {
            config.profiles.push(p.clone());
        }
    }

    if config.profiles.is_empty() {
        config.profiles.push(config.active_profile.clone());
        let _ = fs::create_dir_all(profiles_dir.join(&config.active_profile));
    } else if !config.profiles.contains(&config.active_profile) {
        config.profiles.insert(0, config.active_profile.clone());
    }

    // Ensure recent profiles are clean
    config.recent_profiles.retain(|p| config.profiles.contains(p));
    if !config.recent_profiles.contains(&config.active_profile) {
        config.recent_profiles.insert(0, config.active_profile.clone());
    }

    save_state(state);
}

pub fn switch_profile(version_display: &str, target_profile: &str, state: &mut AppState) -> Result<(), String> {
    let target = target_profile.trim().to_string();
    if target.is_empty() {
        return Err("Tên hồ sơ không hợp lệ.".to_string());
    }

    ensure_initialized(version_display, state);
    let config = state.versions.get_mut(version_display).ok_or("Không tìm thấy cấu hình phiên bản.")?;

    if config.active_profile.eq_ignore_ascii_case(&target) {
        mark_recent(config, &target);
        save_state(state);
        return Ok(());
    }

    // Gracefully close SketchUp if running
    if process::is_sketchup_running() {
        if !process::close_sketchup_gracefully() {
            return Err("SketchUp đang chạy và không thể đóng tự động.".to_string());
        }
    }

    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let profiles_dir = get_profiles_dir(version_display);

    let active_plugins = roaming.join("Plugins");
    let active_shared = roaming.join("SharedPreferences.json");
    let active_priv = local.join("PrivatePreferences.json");

    let current_profile = config.active_profile.clone();
    let current_dir = profiles_dir.join(&current_profile);
    let target_dir = profiles_dir.join(&target);

    let _ = fs::create_dir_all(&current_dir);
    let _ = fs::create_dir_all(&target_dir);

    // 1. Move active user plugins to current profile dir (leave excluded plugins intact!)
    let current_plugins = current_dir.join("Plugins");
    let _ = fs::create_dir_all(&current_plugins);
    if active_plugins.exists() {
        copy_plugins_filtered(&active_plugins, &current_plugins, state.exclude_render_and_default_plugins, true)?;
    }

    if active_shared.exists() {
        let _ = fs::copy(&active_shared, current_dir.join("SharedPreferences.json"));
    }
    if active_priv.exists() {
        let _ = fs::copy(&active_priv, current_dir.join("PrivatePreferences.json"));
    }

    // 2. Restore target profile plugins to active
    let target_plugins = target_dir.join("Plugins");
    let _ = fs::create_dir_all(&active_plugins);
    if target_plugins.exists() {
        copy_plugins_filtered(&target_plugins, &active_plugins, false, false)?;
    }

    let target_shared = target_dir.join("SharedPreferences.json");
    if target_shared.exists() {
        let _ = fs::copy(&target_shared, &active_shared);
        if state.remap_user_paths {
            remap_preferences_paths(&active_shared);
        }
    }
    let target_priv = target_dir.join("PrivatePreferences.json");
    if target_priv.exists() {
        let _ = fs::copy(&target_priv, &active_priv);
        if state.remap_user_paths {
            remap_preferences_paths(&active_priv);
        }
    }

    // 3. Update state
    config.active_profile = target.clone();
    mark_recent(config, &target);
    save_state(state);

    let marker = roaming.join(".current_profile");
    let _ = fs::write(marker, &target);

    Ok(())
}

fn mark_recent(config: &mut VersionConfig, profile: &str) {
    config.recent_profiles.retain(|p| p != profile);
    config.recent_profiles.insert(0, profile.to_string());
    if config.recent_profiles.len() > 10 {
        config.recent_profiles.truncate(10);
    }
}

pub fn save_current_profile_as(
    version_display: &str,
    name: &str,
    save_plugins: bool,
    save_shortcuts: bool,
    state: &mut AppState,
) -> Result<(), String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("Tên hồ sơ không được để trống.".to_string());
    }

    ensure_initialized(version_display, state);
    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let target_dir = get_profiles_dir(version_display).join(&trimmed);
    let _ = fs::create_dir_all(&target_dir);

    if save_plugins {
        let active_plugins = roaming.join("Plugins");
        let dest_plugins = target_dir.join("Plugins");
        if active_plugins.exists() {
            let _ = fs::create_dir_all(&dest_plugins);
            copy_plugins_filtered(&active_plugins, &dest_plugins, state.exclude_render_and_default_plugins, false)?;
        }
    }

    if save_shortcuts {
        let active_shared = roaming.join("SharedPreferences.json");
        let active_priv = local.join("PrivatePreferences.json");
        if active_shared.exists() {
            let _ = fs::copy(&active_shared, target_dir.join("SharedPreferences.json"));
        }
        if active_priv.exists() {
            let _ = fs::copy(&active_priv, target_dir.join("PrivatePreferences.json"));
        }
    }

    let config = state.versions.get_mut(version_display).ok_or("Config not found")?;
    if !config.profiles.contains(&trimmed) {
        config.profiles.push(trimmed.clone());
    }
    mark_recent(config, &trimmed);
    save_state(state);

    Ok(())
}

pub fn rename_profile(
    version_display: &str,
    old_name: &str,
    new_name: &str,
    state: &mut AppState,
) -> Result<(), String> {
    let old_t = old_name.trim();
    let new_t = new_name.trim();
    if new_t.is_empty() || old_t == new_t {
        return Ok(());
    }

    ensure_initialized(version_display, state);
    let profiles_dir = get_profiles_dir(version_display);
    let old_dir = profiles_dir.join(old_t);
    let new_dir = profiles_dir.join(new_t);

    // 1. Rename directory in place if exists
    if old_dir.exists() {
        fs::rename(&old_dir, &new_dir).map_err(|e| format!("Không thể đổi tên thư mục: {}", e))?;
    }

    let config = state.versions.get_mut(version_display).ok_or("Config not found")?;

    // 2. In-place replace in profiles list (DO NOT DUPLICATE!)
    if let Some(pos) = config.profiles.iter().position(|p| p == old_t) {
        config.profiles[pos] = new_t.to_string();
    } else {
        config.profiles.push(new_t.to_string());
    }

    // 3. In-place replace in recent profiles
    if let Some(pos) = config.recent_profiles.iter().position(|p| p == old_t) {
        config.recent_profiles[pos] = new_t.to_string();
    }

    // 4. If renaming active profile, update active profile and marker
    if config.active_profile == old_t {
        config.active_profile = new_t.to_string();
        let roaming = get_roaming_sketchup_path(version_display);
        let _ = fs::write(roaming.join(".current_profile"), new_t);
    }

    save_state(state);
    Ok(())
}

pub fn delete_profile(version_display: &str, name: &str, state: &mut AppState) -> Result<(), String> {
    ensure_initialized(version_display, state);
    let config = state.versions.get_mut(version_display).ok_or("Config not found")?;

    if config.active_profile.eq_ignore_ascii_case(name) {
        return Err("Không thể xóa hồ sơ đang kích hoạt.".to_string());
    }

    config.profiles.retain(|p| p != name);
    config.recent_profiles.retain(|p| p != name);

    let target_dir = get_profiles_dir(version_display).join(name);
    if target_dir.exists() {
        let _ = fs::remove_dir_all(target_dir);
    }

    save_state(state);
    Ok(())
}

pub fn export_profile_zip(version_display: &str, name: &str, zip_dest: &Path, state: &AppState) -> Result<(), String> {
    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let profiles_dir = get_profiles_dir(version_display);

    let is_active = state.versions.get(version_display).map_or(false, |cfg| cfg.active_profile == name);

    let plugins_src = if is_active { roaming.join("Plugins") } else { profiles_dir.join(name).join("Plugins") };
    let shared_src = if is_active { roaming.join("SharedPreferences.json") } else { profiles_dir.join(name).join("SharedPreferences.json") };
    let priv_src = if is_active { local.join("PrivatePreferences.json") } else { profiles_dir.join(name).join("PrivatePreferences.json") };

    let file = fs::File::create(zip_dest).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    if plugins_src.exists() {
        for entry in WalkDir::new(&plugins_src).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&plugins_src) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                if rel_str.is_empty() {
                    continue;
                }
                if state.exclude_render_and_default_plugins && filter::is_excluded(&rel_str) {
                    continue;
                }

                let zip_entry_name = format!("Plugins/{}", rel_str);
                if path.is_file() {
                    zip.start_file(zip_entry_name, options).map_err(|e| e.to_string())?;
                    let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                } else if path.is_dir() {
                    zip.add_directory(zip_entry_name, options).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    if shared_src.exists() {
        zip.start_file("SharedPreferences.json", options).map_err(|e| e.to_string())?;
        let mut f = fs::File::open(shared_src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    if priv_src.exists() {
        zip.start_file("PrivatePreferences.json", options).map_err(|e| e.to_string())?;
        let mut f = fs::File::open(priv_src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_plugins_filtered(source: &Path, destination: &Path, apply_exclusion: bool, move_items: bool) -> Result<(), String> {
    let _ = fs::create_dir_all(destination);

    if let Ok(entries) = fs::read_dir(source) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if apply_exclusion && filter::is_excluded(&name) {
                continue;
            }

            let dest_item = destination.join(&name);
            let src_item = entry.path();

            if src_item.is_dir() {
                if move_items {
                    if dest_item.exists() {
                        let _ = fs::remove_dir_all(&dest_item);
                    }
                    let _ = fs::rename(&src_item, &dest_item);
                } else {
                    copy_dir_recursive(&src_item, &dest_item)?;
                }
            } else if src_item.is_file() {
                if move_items {
                    if dest_item.exists() {
                        let _ = fs::remove_file(&dest_item);
                    }
                    let _ = fs::rename(&src_item, &dest_item);
                } else {
                    let _ = fs::copy(&src_item, &dest_item);
                }
            }
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    let _ = fs::create_dir_all(dst);
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Ok(rel) = path.strip_prefix(src) {
            let target = dst.join(rel);
            if path.is_dir() {
                let _ = fs::create_dir_all(target);
            } else if path.is_file() {
                let _ = fs::copy(path, target);
            }
        }
    }
    Ok(())
}


pub fn get_profile_components(version_display: &str, profile_name: &str, state: &AppState) -> ProfileComponents {
    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let profiles_dir = get_profiles_dir(version_display);
    let profile_dir = profiles_dir.join(profile_name);

    let is_active = state
        .versions
        .get(version_display)
        .map_or(false, |c| c.active_profile == profile_name);

    let (plugins_dir, shared_file, priv_file, mat_dir, tmpl_dir) = if is_active {
        (
            roaming.join("Plugins"),
            roaming.join("SharedPreferences.json"),
            local.join("PrivatePreferences.json"),
            roaming.join("Materials"),
            roaming.join("Templates"),
        )
    } else {
        (
            profile_dir.join("Plugins"),
            profile_dir.join("SharedPreferences.json"),
            profile_dir.join("PrivatePreferences.json"),
            profile_dir.join("Materials"),
            profile_dir.join("Templates"),
        )
    };

    let has_plugins = plugins_dir.is_dir() && fs::read_dir(&plugins_dir).map(|mut it| it.next().is_some()).unwrap_or(false);
    let has_shortcuts = shared_file.is_file() || profile_dir.join("PhimTat").exists();
    let has_toolbars = priv_file.is_file() || profile_dir.join("ViTriThanhCongCu").exists();
    let has_materials = mat_dir.is_dir() && fs::read_dir(&mat_dir).map(|mut it| it.next().is_some()).unwrap_or(false);
    let has_templates = tmpl_dir.is_dir() && fs::read_dir(&tmpl_dir).map(|mut it| it.next().is_some()).unwrap_or(false);

    ProfileComponents {
        name: profile_name.to_string(),
        has_plugins,
        has_shortcuts,
        has_toolbars,
        has_materials,
        has_templates,
    }
}

pub fn get_all_profile_components(version_display: &str, state: &AppState) -> Vec<ProfileComponents> {
    if let Some(config) = state.versions.get(version_display) {
        config
            .profiles
            .iter()
            .map(|p| get_profile_components(version_display, p, state))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn save_profile_custom(
    version_display: &str,
    name: &str,
    save_plugins: bool,
    save_shortcuts: bool,
    save_toolbars: bool,
    save_materials: bool,
    save_templates: bool,
    icon: Option<String>,
    color: Option<String>,
    state: &mut AppState,
) -> Result<(), String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("Tên hồ sơ không được để trống.".to_string());
    }

    ensure_initialized(version_display, state);
    let roaming = get_roaming_sketchup_path(version_display);
    let local = get_local_sketchup_path(version_display);
    let target_dir = get_profiles_dir(version_display).join(&trimmed);
    let _ = fs::create_dir_all(&target_dir);

    if save_plugins {
        let active_plugins = roaming.join("Plugins");
        let dest_plugins = target_dir.join("Plugins");
        if active_plugins.exists() {
            let _ = fs::create_dir_all(&dest_plugins);
            copy_plugins_filtered(&active_plugins, &dest_plugins, state.exclude_render_and_default_plugins, false)?;
        }
    }

    if save_shortcuts {
        let active_shared = roaming.join("SharedPreferences.json");
        if active_shared.exists() {
            let _ = fs::copy(&active_shared, target_dir.join("SharedPreferences.json"));
        }
    }

    if save_toolbars {
        let active_priv = local.join("PrivatePreferences.json");
        if active_priv.exists() {
            let _ = fs::copy(&active_priv, target_dir.join("PrivatePreferences.json"));
        }
    }

    if save_materials {
        let active_mat = roaming.join("Materials");
        if active_mat.exists() {
            let _ = copy_dir_recursive(&active_mat, &target_dir.join("Materials"));
        }
    }

    if save_templates {
        let active_tmpl = roaming.join("Templates");
        if active_tmpl.exists() {
            let _ = copy_dir_recursive(&active_tmpl, &target_dir.join("Templates"));
        }
    }

    let config = state.versions.get_mut(version_display).ok_or("Config not found")?;
    if !config.profiles.contains(&trimmed) {
        config.profiles.push(trimmed.clone());
    }

    if icon.is_some() || color.is_some() {
        let meta = config.profiles_metadata.entry(trimmed.clone()).or_default();
        if icon.is_some() {
            meta.icon = icon;
        }
        if color.is_some() {
            meta.color = color;
        }
    }

    mark_recent(config, &trimmed);
    save_state(state);

    Ok(())
}

pub fn parse_shortcut_string(raw: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = raw.splitn(5, ' ').collect();
    if parts.len() < 5 {
        return None;
    }
    let ctrl = parts[0] == "1";
    let alt = parts[1] == "1";
    let shift = parts[2] == "1";
    let key = parts[3];
    let cmd = parts[4].to_string();

    let mut mods = Vec::new();
    if ctrl { mods.push("Ctrl"); }
    if alt { mods.push("Alt"); }
    if shift { mods.push("Shift"); }
    mods.push(key);

    let key_display = mods.join(" + ");
    Some((key_display, cmd))
}

pub fn load_default_shortcuts_map() -> HashMap<String, String> {
    let default_path = get_roaming_sketchup_path("SketchUp 2026").join("SharedPreferences.json");
    let mut map = HashMap::new();
    if default_path.exists() {
        if let Ok(content) = fs::read_to_string(&default_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(settings) = val.get("Windows Only").and_then(|w| w.get("Settings")).and_then(|s| s.as_object()) {
                    for (k, v) in settings {
                        if k.starts_with("Shortcut_") {
                            if let Some(raw) = v.as_str() {
                                if let Some((key_display, cmd)) = parse_shortcut_string(raw) {
                                    map.insert(cmd, key_display);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if map.is_empty() {
        let hardcoded_defaults = [
            ("selectDollyTool:", "H"),
            ("selectImageIglooTool:", "I"),
            ("selectOrbitTool:", "O"),
            ("selectZoomTool:", "Z"),
            ("viewZoomExtents:", "Shift + Z"),
            ("selectArcTool:", "A"),
            ("selectCircleTool:", "C"),
            ("selectEraseTool:", "E"),
            ("selectPaintTool:", "B"),
            ("selectPushPullTool:", "P"),
            ("selectRectangleTool:", "R"),
            ("selectSelectionTool:", "Space"),
            ("selectLassoSelectionTool:", "Shift + Space"),
            ("selectLineTool:", "L"),
            ("View/Edge Style/Back Edges", "K"),
            ("selectMoveTool:", "M"),
            ("Edit/Make Component...", "G"),
            ("selectMeasureTool:", "T"),
            ("selectOffsetTool:", "F"),
            ("selectRotateTool:", "Q"),
            ("selectScaleTool:", "S"),
            ("Help/Search SketchUp", "Shift + S"),
        ];
        for (cmd, key) in hardcoded_defaults {
            map.insert(cmd.to_string(), key.to_string());
        }
    }
    map
}

pub fn get_profile_details(
    version_display: &str,
    profile_name: &str,
    _state: &AppState,
) -> Result<ProfileDetails, String> {
    let roaming = get_roaming_sketchup_path(version_display);
    let target_dir = get_profiles_dir(version_display).join(profile_name);

    let (plugins_dir, shared_pref_path) = if target_dir.exists() {
        (target_dir.join("Plugins"), target_dir.join("SharedPreferences.json"))
    } else {
        (roaming.join("Plugins"), roaming.join("SharedPreferences.json"))
    };

    // 1. Scan Plugins
    let mut plugins = Vec::new();
    if plugins_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                if path.is_file() && fname.ends_with(".rb") {
                    let clean_name = fname.trim_end_matches(".rb").to_string();
                    let is_ex = filter::is_excluded(&fname);
                    plugins.push(PluginItem {
                        name: clean_name,
                        filename: fname,
                        is_excluded: is_ex,
                    });
                } else if path.is_dir() {
                    let rb_check = plugins_dir.join(format!("{}.rb", fname));
                    if !rb_check.exists() {
                        let is_ex = filter::is_excluded(&fname);
                        plugins.push(PluginItem {
                            name: fname.clone(),
                            filename: fname,
                            is_excluded: is_ex,
                        });
                    }
                }
            }
        }
    }
    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // 2. Parse Shortcuts & Diff
    let default_map = load_default_shortcuts_map();
    let mut custom_shortcuts = Vec::new();
    let mut total_shortcuts_count = 0;

    if shared_pref_path.exists() {
        if let Ok(content) = fs::read_to_string(&shared_pref_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(settings) = val.get("Windows Only").and_then(|w| w.get("Settings")).and_then(|s| s.as_object()) {
                    for (k, v) in settings {
                        if k.starts_with("Shortcut_") {
                            if let Some(raw) = v.as_str() {
                                total_shortcuts_count += 1;
                                if let Some((key_display, cmd)) = parse_shortcut_string(raw) {
                                    match default_map.get(&cmd) {
                                        None => {
                                            custom_shortcuts.push(ShortcutItem {
                                                raw: raw.to_string(),
                                                key_display,
                                                command: cmd,
                                                is_custom: true,
                                                original_key: None,
                                            });
                                        }
                                        Some(orig_key) => {
                                            if orig_key != &key_display {
                                                custom_shortcuts.push(ShortcutItem {
                                                    raw: raw.to_string(),
                                                    key_display,
                                                    command: cmd,
                                                    is_custom: true,
                                                    original_key: Some(orig_key.clone()),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    custom_shortcuts.sort_by(|a, b| a.key_display.cmp(&b.key_display));

    Ok(ProfileDetails {
        profile_name: profile_name.to_string(),
        plugins,
        custom_shortcuts,
        total_shortcuts_count,
        default_shortcuts_count: default_map.len(),
    })
}

pub fn analyze_import_source(source_path: &str) -> Result<ImportAnalysis, String> {
    let path = Path::new(source_path);
    if !path.exists() {
        return Err("Đường dẫn nguồn không tồn tại.".to_string());
    }

    let mut detected_version: Option<String> = None;
    let mut has_plugins = false;
    let mut plugin_count: usize = 0;
    let mut has_shortcuts = false;
    let mut has_toolbars = false;
    let mut has_materials = false;
    let mut has_templates = false;

    let path_str_lower = source_path.to_lowercase();
    for v in &["2026", "2025", "2024", "2023", "2022", "2021"] {
        if path_str_lower.contains(v) {
            detected_version = Some(format!("SketchUp {}", v));
            break;
        }
    }

    let suggested_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported_Profile")
        .replace([' ', '-'], "_");

    if path.is_file() && source_path.to_lowercase().ends_with(".zip") {
        let file = fs::File::open(path).map_err(|e| format!("Không thể mở file zip: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("File zip không hợp lệ: {}", e))?;

        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name().to_lowercase().replace('\\', "/");

                if detected_version.is_none() {
                    for v in &["2026", "2025", "2024", "2023", "2022", "2021"] {
                        if name.contains(v) {
                            detected_version = Some(format!("SketchUp {}", v));
                            break;
                        }
                    }
                }

                if name.starts_with("plugins/") || name.ends_with(".rb") || name.ends_with(".rbe") {
                    let rel = if name.starts_with("plugins/") {
                        name.trim_start_matches("plugins/").to_string()
                    } else {
                        name.clone()
                    };
                    if !rel.is_empty() && !filter::is_excluded(&rel) {
                        has_plugins = true;
                        if rel.ends_with(".rb") || rel.ends_with(".rbe") {
                            plugin_count += 1;
                        }
                    }
                }
                if name.contains("sharedpreferences") || name.contains("phimtat") || (name.ends_with(".dat") && !name.contains("private")) {
                    has_shortcuts = true;
                }
                if name.contains("privatepreferences") || name.contains("vitrithanhcongcu") || name.contains("toolbar") {
                    has_toolbars = true;
                }
                if name.contains("materials/") || name.contains("vatlieu/") {
                    has_materials = true;
                }
                if name.contains("templates/") || name.contains("maudunghinh/") {
                    has_templates = true;
                }
            }
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                let fname_lower = fname.to_lowercase();

                if detected_version.is_none() {
                    for v in &["2026", "2025", "2024", "2023", "2022", "2021"] {
                        if fname_lower.contains(v) {
                            detected_version = Some(format!("SketchUp {}", v));
                            break;
                        }
                    }
                }

                if fname_lower == "plugins" && entry.path().is_dir() {
                    has_plugins = true;
                    if let Ok(sub) = fs::read_dir(entry.path()) {
                        plugin_count = sub
                            .flatten()
                            .filter(|e| !filter::is_excluded(&e.file_name().to_string_lossy()))
                            .count();
                    }
                } else if fname_lower.ends_with(".rb") || fname_lower.ends_with(".rbe") {
                    if !filter::is_excluded(&fname) {
                        has_plugins = true;
                        plugin_count += 1;
                    }
                }

                if fname_lower.contains("sharedpreferences") || fname_lower == "phimtat" || fname_lower.ends_with(".dat") {
                    has_shortcuts = true;
                    if entry.path().is_dir() {
                        if let Ok(sub) = fs::read_dir(entry.path()) {
                            for s in sub.flatten() {
                                let sname = s.file_name().to_string_lossy().to_lowercase();
                                if detected_version.is_none() {
                                    for v in &["2026", "2025", "2024", "2023", "2022", "2021"] {
                                        if sname.contains(v) {
                                            detected_version = Some(format!("SketchUp {}", v));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if fname_lower.contains("privatepreferences") || fname_lower == "vitrithanhcongcu" || fname_lower.contains("toolbar") {
                    has_toolbars = true;
                    if entry.path().is_dir() {
                        if let Ok(sub) = fs::read_dir(entry.path()) {
                            for s in sub.flatten() {
                                let sname = s.file_name().to_string_lossy().to_lowercase();
                                if detected_version.is_none() {
                                    for v in &["2026", "2025", "2024", "2023", "2022", "2021"] {
                                        if sname.contains(v) {
                                            detected_version = Some(format!("SketchUp {}", v));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if fname_lower == "materials" || fname_lower == "vatlieu" {
                    has_materials = true;
                }

                if fname_lower == "templates" || fname_lower == "maudunghinh" {
                    has_templates = true;
                }
            }
        }
    }

    Ok(ImportAnalysis {
        source_path: source_path.to_string(),
        detected_version,
        has_plugins,
        has_shortcuts,
        has_toolbars,
        has_materials,
        has_templates,
        plugin_count,
        suggested_profile_name: suggested_name,
    })
}

pub fn import_profile(
    version_display: &str,
    profile_name: &str,
    source_path: &str,
    import_plugins: bool,
    import_shortcuts: bool,
    import_toolbars: bool,
    import_materials: bool,
    import_templates: bool,
    apply_immediately: bool,
    state: &mut AppState,
) -> Result<(), String> {
    let name_trimmed = profile_name.trim();
    if name_trimmed.is_empty() {
        return Err("Tên hồ sơ không hợp lệ.".to_string());
    }

    ensure_initialized(version_display, state);
    let dest_dir = get_profiles_dir(version_display).join(name_trimmed);
    let _ = fs::create_dir_all(&dest_dir);

    let src_path = Path::new(source_path);
    if !src_path.exists() {
        return Err("Nguồn nhập không tồn tại.".to_string());
    }

    if src_path.is_file() && source_path.to_lowercase().ends_with(".zip") {
        let file = fs::File::open(src_path).map_err(|e| format!("Không thể mở zip: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Lỗi đọc zip: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let raw_name = file.name().replace('\\', "/");
            let name_lower = raw_name.to_lowercase();

            if import_plugins && (name_lower.starts_with("plugins/") || name_lower.ends_with(".rb") || name_lower.ends_with(".rbe")) {
                let rel = if name_lower.starts_with("plugins/") {
                    raw_name.trim_start_matches("plugins/").trim_start_matches("Plugins/").to_string()
                } else {
                    raw_name.clone()
                };
                if !rel.is_empty() {
                    if state.exclude_render_and_default_plugins && filter::is_excluded(&rel) {
                        continue;
                    }
                    let outpath = dest_dir.join("Plugins").join(&rel);
                    if file.is_dir() {
                        let _ = fs::create_dir_all(&outpath);
                    } else {
                        if let Some(p) = outpath.parent() {
                            let _ = fs::create_dir_all(p);
                        }
                        let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }

            if import_shortcuts && (name_lower.contains("sharedpreferences") || (name_lower.ends_with(".dat") && !name_lower.contains("private"))) {
                let outpath = dest_dir.join("SharedPreferences.json");
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }

            if import_toolbars && (name_lower.contains("privatepreferences") || name_lower.contains("vitrithanhcongcu")) {
                let outpath = dest_dir.join("PrivatePreferences.json");
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }

            if import_materials && (name_lower.starts_with("materials/") || name_lower.starts_with("vatlieu/")) {
                let rel = raw_name.trim_start_matches("materials/").trim_start_matches("vatlieu/").to_string();
                if !rel.is_empty() {
                    let outpath = dest_dir.join("Materials").join(rel);
                    if file.is_dir() {
                        let _ = fs::create_dir_all(&outpath);
                    } else {
                        if let Some(p) = outpath.parent() {
                            let _ = fs::create_dir_all(p);
                        }
                        let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }

            if import_templates && (name_lower.starts_with("templates/") || name_lower.starts_with("maudunghinh/")) {
                let rel = raw_name.trim_start_matches("templates/").trim_start_matches("maudunghinh/").to_string();
                if !rel.is_empty() {
                    let outpath = dest_dir.join("Templates").join(rel);
                    if file.is_dir() {
                        let _ = fs::create_dir_all(&outpath);
                    } else {
                        if let Some(p) = outpath.parent() {
                            let _ = fs::create_dir_all(p);
                        }
                        let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    } else if src_path.is_dir() {
        if import_plugins {
            let plugins_src = if src_path.join("Plugins").is_dir() {
                src_path.join("Plugins")
            } else {
                src_path.to_path_buf()
            };
            let dest_plugins = dest_dir.join("Plugins");
            let _ = copy_plugins_filtered(
                &plugins_src,
                &dest_plugins,
                state.exclude_render_and_default_plugins,
                false,
            );
        }

        if import_shortcuts {
            let mut found_pref = None;
            let check_dirs = [src_path.to_path_buf(), src_path.join("PhimTat"), src_path.join("Shortcuts")];
            for d in &check_dirs {
                if d.is_dir() {
                    if let Ok(entries) = fs::read_dir(d) {
                        for e in entries.flatten() {
                            let fname = e.file_name().to_string_lossy().to_string();
                            let fname_lower = fname.to_lowercase();
                            if fname_lower.contains("sharedpreferences") && fname_lower.ends_with(".json") {
                                found_pref = Some(e.path());
                                break;
                            }
                            if fname_lower.contains("phimtat") && fname_lower.ends_with(".json") {
                                found_pref = Some(e.path());
                            } else if found_pref.is_none() && fname_lower.ends_with(".json") && !fname_lower.contains("private") {
                                found_pref = Some(e.path());
                            }
                        }
                    }
                }
                if found_pref.is_some() {
                    break;
                }
            }
            if let Some(p) = found_pref {
                let dest_file = dest_dir.join("SharedPreferences.json");
                let _ = fs::copy(p, &dest_file);
                if state.remap_user_paths {
                    remap_preferences_paths(&dest_file);
                }
            }
        }

        if import_toolbars {
            let mut found_priv = None;
            let check_dirs = [src_path.to_path_buf(), src_path.join("ViTriThanhCongCu"), src_path.join("Toolbars")];
            for d in &check_dirs {
                if d.is_dir() {
                    if let Ok(entries) = fs::read_dir(d) {
                        for e in entries.flatten() {
                            let fname = e.file_name().to_string_lossy().to_string();
                            let fname_lower = fname.to_lowercase();
                            if fname_lower.contains("privatepreferences") && fname_lower.ends_with(".json") {
                                found_priv = Some(e.path());
                                break;
                            }
                        }
                    }
                }
                if found_priv.is_some() {
                    break;
                }
            }
            if let Some(p) = found_priv {
                let dest_file = dest_dir.join("PrivatePreferences.json");
                let _ = fs::copy(p, &dest_file);
                if state.remap_user_paths {
                    remap_preferences_paths(&dest_file);
                }
            }
        }

        if import_materials {
            let mat_src = if src_path.join("Materials").is_dir() {
                Some(src_path.join("Materials"))
            } else if src_path.join("VatLieu").is_dir() {
                Some(src_path.join("VatLieu"))
            } else {
                None
            };
            if let Some(msrc) = mat_src {
                let _ = copy_dir_recursive(&msrc, &dest_dir.join("Materials"));
            }
        }

        if import_templates {
            let tmpl_src = if src_path.join("Templates").is_dir() {
                Some(src_path.join("Templates"))
            } else if src_path.join("MauDungHinh").is_dir() {
                Some(src_path.join("MauDungHinh"))
            } else {
                None
            };
            if let Some(tsrc) = tmpl_src {
                let _ = copy_dir_recursive(&tsrc, &dest_dir.join("Templates"));
            }
        }
    }

    let config = state.versions.get_mut(version_display).ok_or("Config not found")?;
    if !config.profiles.contains(&name_trimmed.to_string()) {
        config.profiles.push(name_trimmed.to_string());
    }
    mark_recent(config, name_trimmed);
    save_state(state);

    if apply_immediately {
        switch_profile(version_display, name_trimmed, state)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcut() {
        let (key, cmd) = parse_shortcut_string("0 0 0 H selectDollyTool:").unwrap();
        assert_eq!(key, "H");
        assert_eq!(cmd, "selectDollyTool:");

        let (key, cmd) = parse_shortcut_string("1 0 1 O Edit/Item/TIG.Smart_offset").unwrap();
        assert_eq!(key, "Ctrl + Shift + O");
        assert_eq!(cmd, "Edit/Item/TIG.Smart_offset");

        let (key, cmd) = parse_shortcut_string("1 1 0 F Extensions/Erode").unwrap();
        assert_eq!(key, "Ctrl + Alt + F");
        assert_eq!(cmd, "Extensions/Erode");
    }

    #[test]
    fn test_default_shortcuts_map() {
        let map = load_default_shortcuts_map();
        assert!(!map.is_empty());
        assert!(map.contains_key("selectDollyTool:"));
    }
}
