use base64::Engine;
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_plugin_icon_data(plugins_dir: &Path, plugin_name: &str) -> Option<String> {
    let matched_dir = find_plugin_dir(plugins_dir, plugin_name)?;
    let icon_path = find_icon_file(plugins_dir, &matched_dir, plugin_name)?;
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

fn is_negative_asset(filename: &str) -> bool {
    let fl = filename.to_lowercase();
    fl.contains("cursor")
        || fl.contains("texture")
        || fl.contains("tile")
        || fl.contains("basemap")
        || fl.contains("folder.jpg")
}

fn extract_image_literals(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    for quote in ['"', '\''] {
        let mut start_idx = None;
        for (i, ch) in text.char_indices() {
            if ch == quote {
                if let Some(start) = start_idx {
                    let literal = &text[start + 1..i];
                    let lit_lower = literal.to_lowercase();
                    if (lit_lower.ends_with(".png")
                        || lit_lower.ends_with(".svg")
                        || lit_lower.ends_with(".jpg")
                        || lit_lower.ends_with(".jpeg"))
                        && !is_negative_asset(&lit_lower)
                    {
                        results.push(literal.to_string());
                    }
                    start_idx = None;
                } else {
                    start_idx = Some(i);
                }
            }
        }
    }
    results
}

fn find_icon_file(plugins_dir: &Path, plugin_dir: &Path, plugin_name: &str) -> Option<PathBuf> {
    // Phase 0: Parse Ruby loader if present for explicit small_icon / large_icon declarations
    let loader_rb = plugins_dir.join(format!("{}.rb", plugin_name));
    if loader_rb.is_file() {
        if let Ok(content) = fs::read_to_string(&loader_rb) {
            for line in content.lines() {
                let lower = line.to_lowercase();
                if lower.contains("small_icon") || lower.contains("large_icon") || lower.contains(".icon") || lower.contains("apply_icons") {
                    let literals = extract_image_literals(line);
                    for lit in literals {
                        let direct = plugin_dir.join(&lit);
                        if direct.is_file() {
                            return Some(direct);
                        }
                        if let Some(file_name) = Path::new(&lit).file_name() {
                            let in_base = plugin_dir.join(file_name);
                            if in_base.is_file() {
                                return Some(in_base);
                            }
                            // Deep check inside plugin_dir for matching file name
                            for entry in walkdir::WalkDir::new(plugin_dir).max_depth(4).into_iter().flatten() {
                                if entry.file_name() == file_name && entry.path().is_file() {
                                    return Some(entry.path().to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Phase 1: High-confidence direct matches (names & plugin-derived names)
    let p_clean = plugin_name.trim_start_matches('_').to_lowercase();
    let mut name_candidates = vec![
        "icon".to_string(),
        "logo".to_string(),
        "plugin".to_string(),
        "app".to_string(),
        "main".to_string(),
        "default".to_string(),
        p_clean.clone(),
        plugin_name.to_lowercase(),
    ];

    for prefix in ["s4u_", "curic_", "ae_", "clf_", "ene_", "tt_", "fredo6_", "dm_"] {
        if p_clean.starts_with(prefix) {
            name_candidates.push(p_clean[prefix.len()..].to_string());
        }
    }

    let subdirs = [
        "", "icons", "_icons", "_ICONS", "images", "img", "resources", "assets", "ui", "dialogs", "data", "Data",
    ];
    let extensions = ["svg", "png", "jpg", "jpeg"];

    for sub in &subdirs {
        let base = if sub.is_empty() {
            plugin_dir.to_path_buf()
        } else {
            plugin_dir.join(sub)
        };
        if !base.is_dir() {
            continue;
        }

        // SVG first (vector sharpness), then PNG
        for ext in &extensions {
            for name in &name_candidates {
                let candidate = base.join(format!("{}.{}", name, ext));
                if candidate.is_file() && !is_negative_asset(&candidate.to_string_lossy()) {
                    return Some(candidate);
                }
                let candidate_large = base.join(format!("{}_large.{}", name, ext));
                if candidate_large.is_file() && !is_negative_asset(&candidate_large.to_string_lossy()) {
                    return Some(candidate_large);
                }
            }
        }
    }

    // Phase 2: First valid icon in standard subdirs
    for sub in &["icons", "_icons", "_ICONS", "images", "resources", "assets", "data", "Data"] {
        let sdir = plugin_dir.join(sub);
        if sdir.is_dir() {
            if let Ok(entries) = fs::read_dir(&sdir) {
                let mut valid_imgs = Vec::new();
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let fname = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if (fname.ends_with(".svg") || fname.ends_with(".png") || fname.ends_with(".jpg"))
                            && !is_negative_asset(&fname)
                        {
                            valid_imgs.push(p);
                        }
                    }
                }
                // Sort preferring svg first
                valid_imgs.sort_by(|a, b| {
                    let a_is_svg = a.extension().map(|e| e == "svg").unwrap_or(false);
                    let b_is_svg = b.extension().map(|e| e == "svg").unwrap_or(false);
                    match (a_is_svg, b_is_svg) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.cmp(b),
                    }
                });
                if let Some(first) = valid_imgs.into_iter().next() {
                    return Some(first);
                }
            }
        }
    }

    // Phase 3: Root folder images
    if let Ok(entries) = fs::read_dir(plugin_dir) {
        let mut root_imgs = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let fname = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                if (fname.ends_with(".svg") || fname.ends_with(".png") || fname.ends_with(".jpg"))
                    && !is_negative_asset(&fname)
                {
                    root_imgs.push(p);
                }
            }
        }
        root_imgs.sort_by(|a, b| {
            let a_is_svg = a.extension().map(|e| e == "svg").unwrap_or(false);
            let b_is_svg = b.extension().map(|e| e == "svg").unwrap_or(false);
            match (a_is_svg, b_is_svg) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });
        if let Some(first) = root_imgs.into_iter().next() {
            return Some(first);
        }
    }

    // Phase 4: Recursive scan with depth up to 4
    let mut candidates: Vec<(i32, PathBuf)> = Vec::new();
    for entry in walkdir::WalkDir::new(plugin_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() {
            let fname = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            if (fname.ends_with(".svg") || fname.ends_with(".png") || fname.ends_with(".jpg"))
                && !is_negative_asset(&fname)
            {
                let mut score = 0;
                if fname.contains("icon")
                    || fname.contains("logo")
                    || fname.contains("tool")
                    || fname.contains("btn")
                    || fname.contains("tb")
                    || fname.contains("large")
                    || fname.contains("24")
                    || fname.contains("32")
                {
                    score += 10;
                }
                if fname.ends_with(".svg") {
                    score += 5;
                }
                if score > 0 {
                    candidates.push((score, p.to_path_buf()));
                }
            }
        }
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    if let Some((_, best)) = candidates.into_iter().next() {
        return Some(best);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dulieu_sketchup_icons() {
        let plugins_dir = Path::new(r"C:\Users\phathua\AppData\Roaming\SketchUp\SketchUp 2025\SketchUp\Profiles\DuLieu_SketchUp\Plugins");
        if !plugins_dir.exists() {
            return;
        }

        let mut found_count = 0;
        let mut total_count = 0;

        if let Ok(entries) = fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                let clean_name = if p.is_file() && fname.ends_with(".rb") {
                    Some(fname.trim_end_matches(".rb").to_string())
                } else if p.is_dir() && !plugins_dir.join(format!("{}.rb", fname)).exists() {
                    Some(fname)
                } else {
                    None
                };

                if let Some(name) = clean_name {
                    total_count += 1;
                    if let Some(icon) = find_plugin_icon_data(plugins_dir, &name) {
                        assert!(icon.starts_with("data:image/"));
                        found_count += 1;
                    }
                }
            }
        }

        println!("Test DuLieu_SketchUp: found {}/{} icons", found_count, total_count);
        assert_eq!(total_count, 134);
        assert!(found_count >= 98, "Expected at least 98 icons, got {}", found_count);
    }
}
