use std::collections::HashSet;
use std::fs;
use std::path::Path;
use serde_json::{Map, Value};

pub fn current_platform() -> &'static str {
    if cfg!(target_os = "macos") { "macos" } else { "windows" }
}

fn section(platform: &str) -> &'static str {
    if platform == "macos" { "Mac Only" } else { "Windows Only" }
}

pub fn infer_json_platform(bytes: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    let count = |platform| value.get(section(platform)).and_then(|v| v.get("Settings"))
        .and_then(Value::as_object).map_or(0, |v| v.len());
    match (count("windows") > 0, count("macos") > 0) {
        (true, false) => Some("windows".into()),
        (false, true) => Some("macos".into()),
        _ => None,
    }
}

pub fn infer_source_platform(path: &Path) -> Option<String> {
    if path.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        let archive = fs::File::open(path).ok()?;
        let mut archive = zip::ZipArchive::new(archive).ok()?;
        if let Ok(mut manifest_file) = archive.by_name("backup_manifest.json") {
            let mut content = String::new();
            use std::io::Read;
            if manifest_file.read_to_string(&mut content).is_ok() {
                if let Ok(manifest) = serde_json::from_str::<crate::models::BackupManifest>(&content) {
                    if manifest.source_platform.as_deref().is_some_and(|p| p == "windows" || p == "macos") {
                        return manifest.source_platform;
                    }
                }
            }
        }
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).ok()?;
            if file.name().to_lowercase().ends_with("sharedpreferences.json") {
                let mut bytes = Vec::new();
                std::io::copy(&mut file, &mut bytes).ok()?;
                if let Some(platform) = infer_json_platform(&bytes) { return Some(platform); }
            }
            if file.name().to_lowercase().ends_with("mactoolbar.plist") { return Some("macos".into()); }
        }
    } else if path.is_dir() {
        for entry in walkdir::WalkDir::new(path).max_depth(4).into_iter().flatten() {
            if entry.file_name().to_string_lossy().eq_ignore_ascii_case("SharedPreferences.json") {
                if let Ok(bytes) = fs::read(entry.path()) {
                    if let Some(platform) = infer_json_platform(&bytes) { return Some(platform); }
                }
            }
            if entry.file_name().to_string_lossy().eq_ignore_ascii_case("MacToolbar.plist") {
                return Some("macos".into());
            }
        }
    }
    None
}

#[derive(Default)]
pub struct ConversionReport {
    pub converted: usize,
    pub skipped: usize,
    pub duplicates: usize,
    pub issues: Vec<String>,
}

pub fn convert_shortcuts(imported: &Path, version: &str, source_platform: &str) -> Result<ConversionReport, String> {
    let destination = crate::platform::roaming_sketchup_path(version).join("SharedPreferences.json");
    convert_shortcuts_with_baseline(imported, &destination, source_platform)
}

fn convert_shortcuts_with_baseline(imported: &Path, destination: &Path, source_platform: &str) -> Result<ConversionReport, String> {
    let source: Value = serde_json::from_slice(&fs::read(imported).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let source_settings = source.get(section(source_platform)).and_then(|v| v.get("Settings"))
        .and_then(Value::as_object).ok_or("Không tìm thấy phím tắt của hệ nguồn")?;
    let mut target: Value = if destination.is_file() {
        serde_json::from_slice(&fs::read(destination).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        Value::Object(Map::new())
    };
    let target_platform = current_platform();
    let target_settings = target.as_object_mut().ok_or("JSON không hợp lệ")?
        .entry(section(target_platform)).or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut().ok_or("Mục hệ điều hành không hợp lệ")?
        .entry("Settings").or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut().ok_or("Mục phím tắt không hợp lệ")?;
    target_settings.retain(|key, _| !key.starts_with("Shortcut_") && key != "Num_Shortcuts");

    let mut report = ConversionReport::default();
    let mut seen = HashSet::new();
    for (key, value) in source_settings.iter().filter(|(key, _)| key.starts_with("Shortcut_")) {
        let Some(raw) = value.as_str() else {
            report.skipped += 1;
            report.issues.push(format!("{key}: giá trị không phải chuỗi"));
            continue;
        };
        let parts: Vec<&str> = raw.splitn(5, ' ').collect();
        if parts.len() != 5 || !parts[..3].iter().all(|part| *part == "0" || *part == "1") || parts[4].trim().is_empty() {
            report.skipped += 1;
            report.issues.push(format!("{key}: định dạng phím tắt không hợp lệ"));
            continue;
        }
        // SketchUp uses the same three modifier slots on both platforms. Slot one is
        // Ctrl on Windows and Command on macOS; slot two is Alt/Option.
        let signature = parts[..4].join(" ");
        if !seen.insert(signature.clone()) {
            report.duplicates += 1;
            report.issues.push(format!("{key}: tổ hợp {signature} trùng với mục khác"));
        }
        report.converted += 1;
        target_settings.insert(format!("Shortcut_{}", report.converted), Value::String(raw.to_string()));
    }
    target_settings.insert("Num_Shortcuts".into(), Value::from(report.converted));
    fs::write(imported, serde_json::to_vec_pretty(&target).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_old_archives_without_a_manifest() {
        let data = br#"{"Windows Only":{"Settings":{"Shortcut_1":"1 0 0 A Edit/Select All"}},"Mac Only":{}}"#;
        assert_eq!(infer_json_platform(data).as_deref(), Some("windows"));
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("old.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("profiles/Default/SharedPreferences.json", zip::write::SimpleFileOptions::default()).unwrap();
        use std::io::Write;
        zip.write_all(data).unwrap();
        zip.finish().unwrap();
        assert_eq!(infer_source_platform(&zip_path).as_deref(), Some("windows"));
    }

    #[test]
    fn reads_new_manifest_and_old_manifest_without_platform() {
        let temp = tempfile::tempdir().unwrap();
        let zip_path = temp.path().join("new.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let manifest = serde_json::json!({
            "app": "SketchUp Profile Switcher", "version": "1.0.8",
            "sketchupVersion": "SketchUp 2026", "backupDate": "now",
            "profiles": [], "sourcePlatform": "macos"
        });
        zip.start_file("backup_manifest.json", zip::write::SimpleFileOptions::default()).unwrap();
        use std::io::Write;
        zip.write_all(manifest.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();
        assert_eq!(infer_source_platform(&zip_path).as_deref(), Some("macos"));

        let mut old = manifest;
        old.as_object_mut().unwrap().remove("sourcePlatform");
        let parsed: crate::models::BackupManifest = serde_json::from_value(old).unwrap();
        assert!(parsed.source_platform.is_none());
    }

    #[test]
    fn converts_shortcuts_without_copying_foreign_preferences() {
        let temp = tempfile::tempdir().unwrap();
        let imported = temp.path().join("SharedPreferences.json");
        let baseline = temp.path().join("baseline.json");
        let source_os = if current_platform() == "macos" { "windows" } else { "macos" };
        let source = serde_json::json!({ section(source_os): { "Settings": {
            "Shortcut_1": "1 1 0 A Edit/Select All",
            "Shortcut_2": "0 0 1   selectLassoSelectionTool:",
            "Shortcut_3": "broken"
        }}});
        let target = serde_json::json!({ section(current_platform()): { "Settings": {
            "Shortcut_1": "0 0 0 Z selectZoomTool:", "Other": "keep"
        }}, "Keep": 42 });
        fs::write(&imported, source.to_string()).unwrap();
        fs::write(&baseline, target.to_string()).unwrap();
        let report = convert_shortcuts_with_baseline(&imported, &baseline, source_os).unwrap();
        assert_eq!((report.converted, report.skipped), (2, 1));
        let converted: Value = serde_json::from_slice(&fs::read(imported).unwrap()).unwrap();
        assert_eq!(converted["Keep"], 42);
        assert_eq!(converted[section(current_platform())]["Settings"]["Other"], "keep");
        assert_eq!(converted[section(current_platform())]["Settings"]["Shortcut_1"], "1 1 0 A Edit/Select All");
        assert!(converted.get(section(source_os)).is_none());
    }
}
