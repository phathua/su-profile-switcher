use std::path::PathBuf;

pub const APP_ID: &str = "com.ktplugin.suprofileswitcher";
pub const APP_NAME: &str = "SketchUp Profile Switcher";

const APP_ICON_PNG: &[u8] = include_bytes!("../icons/128x128.png");
const APP_ICON_ICO: &[u8] = include_bytes!("../icons/icon.ico");

fn clean_path(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy().replace(r"\\?\", "");
    PathBuf::from(s)
}

pub fn resolve_app_icon_path() -> PathBuf {
    // 1. Kiểm tra file ảnh cạnh binary hoặc thư mục dev
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("icons").join("128x128.png");
            if p1.exists() { return clean_path(p1); }
            let p2 = parent.join("128x128.png");
            if p2.exists() { return clean_path(p2); }
            let p3 = parent.join("icons").join("icon.ico");
            if p3.exists() { return clean_path(p3); }
            let p4 = parent.join("icon.ico");
            if p4.exists() { return clean_path(p4); }
            let p5 = parent.join("app-icon.png");
            if p5.exists() { return clean_path(p5); }
        }
    }

    let cwd_icon = PathBuf::from("src-tauri/icons/128x128.png");
    if cwd_icon.exists() {
        if let Ok(abs) = cwd_icon.canonicalize() {
            return clean_path(abs);
        }
    }
    let cwd_ico = PathBuf::from("src-tauri/icons/icon.ico");
    if cwd_ico.exists() {
        if let Ok(abs) = cwd_ico.canonicalize() {
            return clean_path(abs);
        }
    }

    // 2. Tự động trích xuất icon vào AppData nếu chưa tồn tại
    let target_dir = if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("SketchUp Profile Switcher")
    } else if let Ok(app_data) = std::env::var("APPDATA") {
        PathBuf::from(app_data).join("SketchUp Profile Switcher")
    } else {
        std::env::temp_dir().join("SketchUp Profile Switcher")
    };

    let _ = std::fs::create_dir_all(&target_dir);
    let target_png = target_dir.join("app-icon.png");
    if !target_png.exists() || std::fs::metadata(&target_png).map(|m| m.len()).unwrap_or(0) == 0 {
        let _ = std::fs::write(&target_png, APP_ICON_PNG);
    }
    let target_ico = target_dir.join("app-icon.ico");
    if !target_ico.exists() || std::fs::metadata(&target_ico).map(|m| m.len()).unwrap_or(0) == 0 {
        let _ = std::fs::write(&target_ico, APP_ICON_ICO);
    }

    if target_png.exists() {
        clean_path(target_png)
    } else {
        PathBuf::new()
    }
}

#[cfg(windows)]
pub fn ensure_aumid_registered() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REGISTERED: AtomicBool = AtomicBool::new(false);
    if REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }

    let icon_path = resolve_app_icon_path();
    let ico_candidate = icon_path.with_extension("ico");
    let reg_icon = if ico_candidate.exists() {
        ico_candidate
    } else {
        icon_path.clone()
    };
    let icon_str = reg_icon.to_string_lossy();
    let base_key = format!(r#"HKCU\Software\Classes\AppUserModelId\{}"#, APP_ID);

    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let _ = std::process::Command::new("reg")
        .args(&["add", &base_key, "/v", "DisplayName", "/d", APP_NAME, "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if !icon_str.is_empty() {
        let _ = std::process::Command::new("reg")
            .args(&["add", &base_key, "/v", "IconUri", "/d", &icon_str, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }

    let _ = std::process::Command::new("reg")
        .args(&["add", &base_key, "/v", "ShowInSettings", "/t", "REG_DWORD", "/d", "1", "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    // Dọn dẹp key com.opensource cũ nếu còn tồn tại trong Registry
    let old_key = r#"HKCU\Software\Classes\AppUserModelId\com.opensource.suprofileswitcher"#;
    let _ = std::process::Command::new("reg")
        .args(&["delete", old_key, "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}

pub fn show_profile_switched_toast(profile_name: &str) {
    #[cfg(windows)]
    {
        ensure_aumid_registered();
        let icon_path = resolve_app_icon_path();
        let is_image = icon_path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("jpg"));
        let icon_uri = if icon_path.exists() && is_image {
            format!("file:///{}", icon_path.to_string_lossy().replace('\\', "/"))
        } else {
            String::new()
        };

        let image_xml = if !icon_uri.is_empty() {
            format!(r#"<image placement="appLogoOverride" hint-crop="circle" src="{}" alt="Logo"/>"#, icon_uri)
        } else {
            String::new()
        };

        let xml_payload = format!(
            r#"<toast>
                <visual>
                    <binding template="ToastGeneric">
                        <text id="1">{}</text>
                        <text id="2">Đã chuyển sang hồ sơ '{}' thành công!</text>
                        {}
                    </binding>
                </visual>
            </toast>"#,
            APP_NAME,
            profile_name,
            image_xml
        );

        use windows::core::HSTRING;
        use windows::Data::Xml::Dom::XmlDocument;
        use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

        if let Ok(xml) = XmlDocument::new() {
            if xml.LoadXml(&HSTRING::from(xml_payload)).is_ok() {
                if let Ok(toast) = ToastNotification::CreateToastNotification(&xml) {
                    if let Ok(notifier) = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID)) {
                        let _ = notifier.Show(&toast);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_app_icon_path() {
        let p = resolve_app_icon_path();
        assert!(p.exists(), "Resolved icon path must exist on disk: {:?}", p);
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        assert!(ext == "png" || ext == "ico", "Icon must be png or ico, got: {}", ext);
        assert_ne!(ext, "exe", "Icon must NEVER be an executable binary!");
    }

    #[test]
    fn test_ensure_aumid_registered() {
        ensure_aumid_registered();
        let base_key = format!(r#"HKCU\Software\Classes\AppUserModelId\{}"#, APP_ID);
        let out = std::process::Command::new("reg")
            .args(&["query", &base_key, "/v", "IconUri"])
            .output()
            .expect("failed to run reg query");
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(!stdout.contains(".exe"), "Registry IconUri must not be .exe: {}", stdout);
        assert!(stdout.contains(".ico") || stdout.contains(".png"), "Registry IconUri must be .ico or .png: {}", stdout);

        let old_key = r#"HKCU\Software\Classes\AppUserModelId\com.opensource.suprofileswitcher"#;
        let out_old = std::process::Command::new("reg")
            .args(&["query", old_key])
            .output()
            .expect("failed to run reg query");
        assert!(!out_old.status.success(), "Old registry key should be deleted");
    }
}
