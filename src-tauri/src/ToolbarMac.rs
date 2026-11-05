use std::path::Path;

#[cfg(target_os = "macos")]
fn toolbar_key(version: &str) -> String {
    let year = version.split_whitespace().last().unwrap_or("");
    format!("NSToolbar Configuration SketchUpToolbar{}", &year[year.len().saturating_sub(2)..])
}

#[cfg(target_os = "macos")]
pub fn capture(version: &str, profile_dir: &Path) -> Result<(), String> {
    if let Some(bytes) = snapshot_bytes(version)? {
        std::fs::write(profile_dir.join("MacToolbar.plist"), bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn snapshot_bytes(version: &str) -> Result<Option<Vec<u8>>, String> {
    let source = crate::platform::toolbar_plist(version);
    snapshot_bytes_from(&source, &toolbar_key(version))
}

#[cfg(target_os = "macos")]
fn snapshot_bytes_from(source: &Path, key: &str) -> Result<Option<Vec<u8>>, String> {
    if !source.is_file() { return Ok(None); }
    let output = std::process::Command::new("plutil")
        .args(["-extract", key, "xml1", "-o", "-"])
        .arg(&source).output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Ok(None); }
    Ok(Some(output.stdout))
}

#[cfg(target_os = "macos")]
pub fn restore(version: &str, profile_dir: &Path) -> Result<(), String> {
    let snapshot = profile_dir.join("MacToolbar.plist");
    if !snapshot.is_file() { return Ok(()); }
    let target = crate::platform::toolbar_plist(version);
    let xml = std::fs::read_to_string(snapshot).map_err(|e| e.to_string())?;
    restore_into(&target, &toolbar_key(version), &xml)
}

#[cfg(target_os = "macos")]
fn restore_into(target: &Path, key: &str, xml: &str) -> Result<(), String> {
    if !target.is_file() {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let created = std::process::Command::new("plutil")
            .args(["-create", "xml1"]).arg(target).status().map_err(|e| e.to_string())?;
        if !created.success() { return Err("Không thể tạo plist thanh công cụ macOS".into()); }
    }
    let status = std::process::Command::new("plutil")
        .args(["-replace", key, "-xml", xml])
        .arg(target).status().map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err("Không thể khôi phục thanh công cụ macOS".into()) }
}

#[cfg(not(target_os = "macos"))]
pub fn capture(_version: &str, _profile_dir: &Path) -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "macos"))]
pub fn restore(_version: &str, _profile_dir: &Path) -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "macos"))]
pub fn snapshot_bytes(_version: &str) -> Result<Option<Vec<u8>>, String> { Ok(None) }

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn restores_only_toolbar_key() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.plist");
        let target = temp.path().join("target.plist");
        std::fs::write(&source, r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>Toolbar</key><dict><key>item</key><string>new</string></dict></dict></plist>"#).unwrap();
        std::fs::write(&target, r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>Toolbar</key><dict><key>item</key><string>old</string></dict><key>Keep</key><string>yes</string></dict></plist>"#).unwrap();
        let xml = String::from_utf8(snapshot_bytes_from(&source, "Toolbar").unwrap().unwrap()).unwrap();
        restore_into(&target, "Toolbar", &xml).unwrap();
        let keep = std::process::Command::new("plutil").args(["-extract", "Keep", "raw", "-o", "-"]).arg(&target).output().unwrap();
        let item = std::process::Command::new("plutil").args(["-extract", "Toolbar.item", "raw", "-o", "-"]).arg(&target).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&keep.stdout).trim(), "yes");
        assert_eq!(String::from_utf8_lossy(&item.stdout).trim(), "new");

        let missing = temp.path().join("missing.plist");
        restore_into(&missing, "Toolbar", &xml).unwrap();
        let fresh = std::process::Command::new("plutil").args(["-extract", "Toolbar.item", "raw", "-o", "-"]).arg(&missing).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&fresh.stdout).trim(), "new");
    }
}
