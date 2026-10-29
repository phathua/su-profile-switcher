use base64::Engine;
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_plugin_icon_data(plugins_dir: &Path, plugin_name: &str) -> Option<String> {
    let matched_dir = find_plugin_dir(plugins_dir, plugin_name)?;
    let icon_path = find_icon_file(&matched_dir)?;
    encode_icon_to_data_url(&icon_path)
}

fn find_plugin_dir(plugins_dir: &Path, plugin_name: &str) -> Option<PathBuf> {
    let exact = plugins_dir.join(plugin_name);
    if exact.is_dir() {
        return Some(exact);
    }

    let unstripped = plugin_name.trim_start_matches('_');
    let unstrip_path = plugins_dir.join(unstripped);
    if unstrip_path.is_dir() {
        return Some(unstrip_path);
    }

    let clean_lower = unstripped.to_lowercase();
    if let Ok(entries) = fs::read_dir(plugins_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let name_lower = name.to_lowercase();
                if name_lower == clean_lower
                    || (name_lower.len() >= 3 && clean_lower.starts_with(&name_lower))
                    || (clean_lower.len() >= 3 && name_lower.starts_with(&clean_lower))
                {
                    return Some(entry.path());
                }
            }
        }
    }

    None
}

fn find_icon_file(plugin_dir: &Path) -> Option<PathBuf> {
    let icon_names = ["icon", "logo", "plugin", "app", "main", "default"];
    let subdirs = ["", "icons", "images", "img", "resources", "assets", "ui", "dialogs", "dialogs/images"];
    let extensions = ["svg", "png", "jpg", "jpeg"];

    // Phase 1: High-confidence direct matches
    for sub in &subdirs {
        let base = if sub.is_empty() {
            plugin_dir.to_path_buf()
        } else {
            plugin_dir.join(sub)
        };
        for name in &icon_names {
            for ext in &extensions {
                let candidate = base.join(format!("{}.{}", name, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // Phase 2: Any image file inside standard icon folders
    for sub in &["icons", "images", "img", "resources", "assets"] {
        let sdir = plugin_dir.join(sub);
        if sdir.is_dir() {
            if let Ok(entries) = fs::read_dir(&sdir) {
                let mut valid_imgs = Vec::new();
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let fname = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if (fname.ends_with(".svg") || fname.ends_with(".png") || fname.ends_with(".jpg"))
                            && !fname.starts_with("cursor")
                        {
                            valid_imgs.push(p);
                        }
                    }
                }
                valid_imgs.sort();
                if let Some(first) = valid_imgs.into_iter().next() {
                    return Some(first);
                }
            }
        }
    }

    // Phase 3: Recursive scan (max depth 2)
    for entry in walkdir::WalkDir::new(plugin_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() {
            let fname = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            if (fname.ends_with(".svg") || fname.ends_with(".png") || fname.ends_with(".jpg"))
                && !fname.starts_with("cursor")
            {
                if fname.contains("icon")
                    || fname.contains("logo")
                    || fname.contains("tool")
                    || fname.contains("btn")
                    || fname.contains("tb")
                {
                    return Some(p.to_path_buf());
                }
            }
        }
    }

    None
}

fn encode_icon_to_data_url(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    // Cap size at 500KB to keep JSON payload fast and light
    if metadata.len() > 500 * 1024 {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    let mime = match ext.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => return None,
    };

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, b64))
}
