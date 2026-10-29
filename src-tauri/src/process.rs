use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

pub fn is_sketchup_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_lowercase();
        if name == "sketchup.exe" || name.starts_with("sketchup") {
            return true;
        }
    }
    false
}

pub fn close_sketchup_gracefully() -> bool {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut found = false;
    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_lowercase();
        if name == "sketchup.exe" || name.starts_with("sketchup") {
            proc_.kill();
            found = true;
        }
    }

    if found {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    !is_sketchup_running()
}

pub fn find_sketchup_exe(version_display: &str) -> Option<PathBuf> {
    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    let year = version_display.split_whitespace().last().unwrap_or(version_display);

    let candidates = [
        // Trimble SketchUp 2024, 2025, 2026 subfolder layout:
        Path::new(&program_files).join("SketchUp").join(version_display).join("SketchUp").join("SketchUp.exe"),
        Path::new(&program_files).join("SketchUp").join(format!("SketchUp {}", year)).join("SketchUp").join("SketchUp.exe"),
        // Direct version folder:
        Path::new(&program_files).join("SketchUp").join(version_display).join("SketchUp.exe"),
        Path::new(&program_files).join("SketchUp").join(format!("SketchUp {}", year)).join("SketchUp.exe"),
    ];

    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }

    None
}

pub fn launch_sketchup(version_display: &str, custom_path: Option<&str>) -> bool {
    let exe_path = if let Some(custom) = custom_path {
        let p = PathBuf::from(custom);
        if p.exists() {
            p
        } else {
            match find_sketchup_exe(version_display) {
                Some(found) => found,
                None => return false,
            }
        }
    } else {
        match find_sketchup_exe(version_display) {
            Some(found) => found,
            None => return false,
        }
    };

    Command::new(exe_path)
        .spawn()
        .is_ok()
}
