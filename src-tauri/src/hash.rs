use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::filter;

pub fn compute_configuration_hash(
    folder: &Path,
    local_folder: Option<&Path>,
    apply_exclusion: bool,
) -> String {
    if !folder.exists() {
        return String::new();
    }

    let mut hasher = Sha256::new();
    let mut buffer = String::new();

    // 1. SharedPreferences.json
    let shared = folder.join("SharedPreferences.json");
    if shared.exists() {
        if let Ok(bytes) = fs::read(&shared) {
            let mut file_hasher = Sha256::new();
            file_hasher.update(&bytes);
            buffer.push_str(&format!("shared:{:x};", file_hasher.finalize()));
        }
    }

    // 2. PrivatePreferences.json (in local_folder or folder)
    let priv_pref = if let Some(local) = local_folder {
        local.join("PrivatePreferences.json")
    } else {
        folder.join("PrivatePreferences.json")
    };

    if priv_pref.exists() {
        if let Ok(bytes) = fs::read(&priv_pref) {
            let mut file_hasher = Sha256::new();
            file_hasher.update(&bytes);
            buffer.push_str(&format!("priv:{:x};", file_hasher.finalize()));
        }
    }

    // 3. Plugins
    let plugins = if folder.join("Plugins").exists() {
        folder.join("Plugins")
    } else if folder.file_name().map_or(false, |n| n.eq_ignore_ascii_case("Plugins")) {
        folder.to_path_buf()
    } else {
        folder.join("Plugins")
    };

    if plugins.exists() {
        let mut entries: Vec<_> = WalkDir::new(&plugins)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        entries.sort_by_key(|e| e.path().to_path_buf());

        for entry in entries {
            if let Ok(rel) = entry.path().strip_prefix(&plugins) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                if apply_exclusion && filter::is_excluded(&rel_str) {
                    continue;
                }
                if let Ok(meta) = entry.metadata() {
                    buffer.push_str(&format!("{}:{};", rel_str, meta.len()));
                }
            }
        }
    }

    hasher.update(buffer.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn find_matching_profile(
    roaming: &Path,
    local: &Path,
    profiles_dir: &Path,
    apply_exclusion: bool,
) -> Option<String> {
    let active_hash = compute_configuration_hash(roaming, Some(local), apply_exclusion);
    if active_hash.is_empty() || !profiles_dir.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let p_name = entry.file_name().to_string_lossy().to_string();
                    let profile_hash = compute_configuration_hash(&entry.path(), None, apply_exclusion);
                    if !profile_hash.is_empty() && profile_hash.eq_ignore_ascii_case(&active_hash) {
                        return Some(p_name);
                    }
                }
            }
        }
    }

    None
}
