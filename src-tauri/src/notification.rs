pub fn show_profile_switched_toast(profile_name: &str) {
    #[cfg(windows)]
    {
        use winrt_notification::{Toast, Duration};
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title("SketchUp Profile Switcher")
            .text1(&format!("Đã chuyển sang hồ sơ '{}' thành công!", profile_name))
            .duration(Duration::Short)
            .show();
    }
}
