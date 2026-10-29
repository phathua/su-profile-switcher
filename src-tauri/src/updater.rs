use std::fs;
use std::path::PathBuf;
use crate::models::UpdateInfo;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn check_for_updates(simulate: Option<&str>) -> UpdateInfo {
    let current = CURRENT_VERSION.to_string();

    if let Some(sim) = simulate {
        let has_update = is_newer(sim, &current);
        return UpdateInfo {
            has_update,
            current_version: current,
            latest_version: sim.to_string(),
            release_notes: "Bản phát hành mô phỏng kiểm thử tự động.".to_string(),
            download_url: "https://example.com/simulate.zip".to_string(),
        };
    }

    // Call GitHub Releases API
    let url = "https://api.github.com/repos/phathua/su-profile-switcher/releases/latest";
    let resp = ureq::get(url)
        .set("User-Agent", "su-profile-switcher-tauri")
        .timeout(std::time::Duration::from_secs(10))
        .call();

    if let Ok(res) = resp {
        if let Ok(val) = res.into_json::<serde_json::Value>() {
            let tag = val["tag_name"]
                .as_str()
                .unwrap_or("1.0.2")
                .trim_start_matches(|c| c == 'v' || c == 'V');
            let body = val["body"].as_str().unwrap_or("").to_string();
            let has_update = is_newer(tag, &current);

            let mut dl_url = String::new();
            if let Some(assets) = val["assets"].as_array() {
                let target_arch = get_current_arch();
                // 1. Priority: Find NSIS Setup executable matching architecture
                for asset in assets {
                    let name = asset["name"].as_str().unwrap_or("").to_lowercase();
                    if name.contains(&target_arch) && (name.contains("setup") || name.ends_with(".exe")) {
                        dl_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                        break;
                    }
                }

                // 2. Fallback: Find matching arch zip package
                if dl_url.is_empty() {
                    for asset in assets {
                        let name = asset["name"].as_str().unwrap_or("").to_lowercase();
                        if name.contains(&target_arch) && name.ends_with(".zip") {
                            dl_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                            break;
                        }
                    }
                }

                // 3. Fallback to any windows setup or exe/zip
                if dl_url.is_empty() {
                    for asset in assets {
                        let name = asset["name"].as_str().unwrap_or("").to_lowercase();
                        if name.ends_with(".exe") || name.ends_with(".zip") {
                            dl_url = asset["browser_download_url"].as_str().unwrap_or("").to_string();
                            break;
                        }
                    }
                }
            }

            return UpdateInfo {
                has_update,
                current_version: current,
                latest_version: tag.to_string(),
                release_notes: body,
                download_url: dl_url,
            };
        }
    }

    UpdateInfo {
        has_update: false,
        current_version: current.clone(),
        latest_version: current,
        release_notes: String::new(),
        download_url: String::new(),
    }
}

pub fn download_update(url: &str, latest_version: &str) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join("su-profile-switcher-update");
    let _ = fs::create_dir_all(&temp_dir);
    let ext = if url.to_lowercase().ends_with(".exe") { "exe" } else { "zip" };
    let dest = temp_dir.join(format!("update_{}.{}", latest_version, ext));

    if url.starts_with("http") {
        let resp = ureq::get(url)
            .timeout(std::time::Duration::from_secs(45))
            .call()
            .map_err(|e| e.to_string())?;

        let mut reader = resp.into_reader();
        let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
        std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    } else {
        fs::write(&dest, b"simulated update").map_err(|e| e.to_string())?;
    }

    Ok(dest)
}

pub fn apply_update_and_restart(app: &tauri::AppHandle, download_url: Option<&str>, latest_version: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let temp_dir = std::env::temp_dir().join("su-profile-switcher-update");
    let _ = fs::create_dir_all(&temp_dir);

    let downloaded_file = if let Some(url) = download_url.filter(|u| !u.is_empty()) {
        download_update(url, latest_version)?
    } else {
        // If simulation or empty URL, create a copy of current_exe to simulate
        let sim_exe = temp_dir.join("simulated_update.exe");
        let _ = fs::copy(&current_exe, &sim_exe);
        sim_exe
    };

    let is_zip = downloaded_file.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("zip")).unwrap_or(false);
    let new_exe = if is_zip {
        // Extract exe from zip
        let file = fs::File::open(&downloaded_file).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let extract_target = temp_dir.join("extracted_new_app.exe");
        let mut found = false;
        for i in 0..archive.len() {
            let mut item = archive.by_index(i).map_err(|e| e.to_string())?;
            if item.name().ends_with(".exe") {
                let mut out = fs::File::create(&extract_target).map_err(|e| e.to_string())?;
                std::io::copy(&mut item, &mut out).map_err(|e| e.to_string())?;
                found = true;
                break;
            }
        }
        if !found {
            return Err("Không tìm thấy tệp .exe trong gói cập nhật zip.".to_string());
        }
        extract_target
    } else {
        downloaded_file
    };

    let exe_name_lower = new_exe.to_string_lossy().to_lowercase();
    let is_installer = exe_name_lower.contains("setup") || exe_name_lower.contains("installer");

    // Windows update batch script
    let current_pid = std::process::id();
    let bat_path = temp_dir.join("apply_update.bat");
    let bat_content = if is_installer {
        format!(
            "@echo off\r\n\
            chcp 65001 > nul\r\n\
            :wait_loop\r\n\
            tasklist /fi \"PID eq {pid}\" | find \"{pid}\" > nul\r\n\
            if %ERRORLEVEL% == 0 (\r\n\
                timeout /t 1 /nobreak > nul\r\n\
                goto wait_loop\r\n\
            )\r\n\
            start \"\" \"{installer}\"\r\n\
            del \"%~f0\"\r\n",
            pid = current_pid,
            installer = new_exe.to_string_lossy()
        )
    } else {
        format!(
            "@echo off\r\n\
            chcp 65001 > nul\r\n\
            :wait_loop\r\n\
            tasklist /fi \"PID eq {pid}\" | find \"{pid}\" > nul\r\n\
            if %ERRORLEVEL% == 0 (\r\n\
                timeout /t 1 /nobreak > nul\r\n\
                goto wait_loop\r\n\
            )\r\n\
            copy /y \"{new_exe}\" \"{current_exe}\" > nul\r\n\
            start \"\" \"{current_exe}\"\r\n\
            del \"%~f0\"\r\n",
            pid = current_pid,
            new_exe = new_exe.to_string_lossy(),
            current_exe = current_exe.to_string_lossy()
        )
    };

    fs::write(&bat_path, bat_content).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .args(&["/C", &bat_path.to_string_lossy()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    app.exit(0);
    Ok(())
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parse_version = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|part| part.split('-').next().and_then(|p| p.parse().ok()))
            .collect()
    };

    let l = parse_version(latest);
    let c = parse_version(current);

    l > c
}

pub fn get_current_arch() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        "x64".to_string()
    }
    #[cfg(target_arch = "x86")]
    {
        "x86".to_string()
    }
    #[cfg(target_arch = "aarch64")]
    {
        "arm64".to_string()
    }
    #[cfg(target_arch = "arm")]
    {
        "arm".to_string()
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64", target_arch = "arm")))]
    {
        std::env::consts::ARCH.to_string()
    }
}
