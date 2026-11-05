use std::path::{Path, PathBuf};

pub fn app_storage_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return home().join("Library/Application Support");
    }
    #[cfg(not(target_os = "macos"))]
    {
        PathBuf::from(std::env::var_os("APPDATA").unwrap_or_else(|| ".".into()))
    }
}

#[cfg(target_os = "macos")]
pub fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
}

pub fn roaming_sketchup_path(version: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return app_storage_root().join(version).join("SketchUp");
    }
    #[cfg(not(target_os = "macos"))]
    {
        app_storage_root().join("SketchUp").join(version).join("SketchUp")
    }
}

pub fn local_sketchup_path(version: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return roaming_sketchup_path(version);
    }
    #[cfg(not(target_os = "macos"))]
    {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_else(|| ".".into()))
            .join("SketchUp").join(version).join("SketchUp")
    }
}

#[cfg(target_os = "macos")]
pub fn sketchup_app(version: &str) -> Option<PathBuf> {
    let candidates = [
        Path::new("/Applications").join(version).join("SketchUp.app"),
        home().join("Applications").join(version).join("SketchUp.app"),
        Path::new("/Applications").join("SketchUp.app"),
    ];
    candidates.into_iter().find(|p| {
        if !p.is_dir() { return false; }
        if p.parent().is_some_and(|parent| parent.file_name().is_some_and(|name| name == version)) {
            return true;
        }
        let info = p.join("Contents/Info.plist");
        let year = version.split_whitespace().last().unwrap_or("");
        std::process::Command::new("plutil")
            .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
            .arg(info).output().ok().is_some_and(|result| result.status.success() &&
                String::from_utf8_lossy(&result.stdout).trim().ends_with(&format!(".{year}")))
    })
}

#[cfg(target_os = "macos")]
pub fn toolbar_plist(version: &str) -> PathBuf {
    let year = version.split_whitespace().last().unwrap_or("");
    home().join("Library/Preferences").join(format!("com.sketchup.SketchUp.{year}.plist"))
}

pub fn installed_versions() -> Vec<String> {
    let mut versions = Vec::new();
    #[cfg(target_os = "macos")]
    {
        for year in 2021..=2026 {
            let name = format!("SketchUp {year}");
            if sketchup_app(&name).is_some() || roaming_sketchup_path(&name).is_dir() {
                versions.push(name);
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let root = app_storage_root().join("SketchUp");
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("SketchUp ") && entry.path().join("SketchUp").is_dir() {
                    versions.push(name);
                }
            }
        }
        if versions.is_empty() {
            versions = (2023..=2026).map(|year| format!("SketchUp {year}")).collect();
        }
    }
    versions.sort();
    versions.reverse();
    versions
}
