use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;

use crate::engine;
use crate::filter;
use crate::models::{AppState, ChangelogEntry, LiveDiff};

pub fn get_changelog_file_path() -> PathBuf {
    engine::get_app_storage_dir().join("changelog.json")
}

pub fn load_changelog() -> Vec<ChangelogEntry> {
    let path = get_changelog_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str::<Vec<ChangelogEntry>>(&content) {
                return entries;
            }
        }
    }
    Vec::new()
}

pub fn save_changelog(entries: &[ChangelogEntry]) -> Result<(), String> {
    let path = get_changelog_file_path();
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Scan a plugins directory and return clean names of non-excluded user plugins.
pub fn scan_user_plugin_names(plugins_dir: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    if !plugins_dir.exists() {
        return names;
    }

    if let Ok(entries) = fs::read_dir(plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let fname = entry.file_name().to_string_lossy().to_string();

            if filter::is_excluded(&fname) {
                continue;
            }

            if path.is_file() && fname.ends_with(".rb") {
                let clean = fname.trim_end_matches(".rb").to_string();
                names.insert(clean);
            } else if path.is_dir() {
                let rb_check = plugins_dir.join(format!("{}.rb", fname));
                // Only consider directory if there is no corresponding .rb file
                if !rb_check.exists() {
                    names.insert(fname);
                }
            }
        }
    }

    names
}

/// Computes (added_plugins, removed_plugins) by comparing active plugins dir vs stored profile plugins dir.
pub fn compute_plugins_diff(active_plugins_dir: &Path, stored_plugins_dir: &Path) -> (Vec<String>, Vec<String>) {
    let active_names = scan_user_plugin_names(active_plugins_dir);
    let stored_names = scan_user_plugin_names(stored_plugins_dir);

    let mut added: Vec<String> = active_names
        .difference(&stored_names)
        .cloned()
        .collect();
    let mut removed: Vec<String> = stored_names
        .difference(&active_names)
        .cloned()
        .collect();

    added.sort();
    removed.sort();

    (added, removed)
}

/// Records a new change event to changelog.json
pub fn record_change_event(
    version: &str,
    profile: &str,
    event_type: &str,
    added: Vec<String>,
    removed: Vec<String>,
    details: &str,
) -> Result<(), String> {
    if added.is_empty() && removed.is_empty() {
        return Ok(());
    }

    let mut entries = load_changelog();
    let now = Local::now();
    let timestamp = now.format("%d/%m/%Y %H:%M:%S").to_string();
    let id = format!("log_{}", now.timestamp_millis());

    let entry = ChangelogEntry {
        id,
        timestamp,
        version: version.to_string(),
        profile: profile.to_string(),
        event_type: event_type.to_string(),
        added_plugins: added,
        removed_plugins: removed,
        details: details.to_string(),
    };

    entries.insert(0, entry);
    if entries.len() > 300 {
        entries.truncate(300);
    }

    save_changelog(&entries)
}

/// Compute live diff of the currently active profile
pub fn get_live_diff(version: &str, state: &AppState) -> LiveDiff {
    let config = match state.versions.get(version) {
        Some(c) => c,
        None => return LiveDiff::default(),
    };

    let active_profile = config.active_profile.clone();
    let roaming = engine::get_roaming_sketchup_path(version);
    let profiles_dir = engine::get_profiles_dir(version);

    let active_plugins_dir = roaming.join("Plugins");
    let stored_plugins_dir = profiles_dir.join(&active_profile).join("Plugins");

    let (added, removed) = compute_plugins_diff(&active_plugins_dir, &stored_plugins_dir);
    let has_changes = !added.is_empty() || !removed.is_empty();

    LiveDiff {
        active_profile,
        added_plugins: added,
        removed_plugins: removed,
        has_changes,
    }
}

pub fn clear_changelog() -> Result<(), String> {
    save_changelog(&[])
}
