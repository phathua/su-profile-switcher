use std::path::PathBuf;

pub const APP_ID: &str = "com.ktplugin.suprofileswitcher";
pub const APP_NAME: &str = "SketchUp Profile Switcher";

fn clean_path(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy().replace(r"\\?\", "");
    PathBuf::from(s)
}

pub fn resolve_app_icon_path() -> PathBuf {
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

    clean_path(std::env::current_exe().unwrap_or_default())
}

#[cfg(windows)]
pub fn ensure_aumid_registered() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REGISTERED: AtomicBool = AtomicBool::new(false);
    if REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }

    let icon_path = resolve_app_icon_path();
    let icon_str = icon_path.to_string_lossy();
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
}

pub fn show_profile_switched_toast(profile_name: &str) {
    #[cfg(windows)]
    {
        ensure_aumid_registered();
        let icon_path = resolve_app_icon_path();
        let icon_uri = if icon_path.exists() {
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
