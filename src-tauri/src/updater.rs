use crate::models::UpdateInfo;
use tauri_plugin_updater::UpdaterExt;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check_for_updates(app: &tauri::AppHandle, simulate: Option<&str>) -> Result<UpdateInfo, String> {
    if let Some(version) = simulate {
        let parse = |value: &str| -> Option<Vec<u64>> {
            value.trim_start_matches('v').split('.').map(|part| part.parse().ok()).collect()
        };
        return Ok(UpdateInfo {
            has_update: parse(version).zip(parse(CURRENT_VERSION))
                .is_some_and(|(candidate, current)| candidate > current),
            current_version: CURRENT_VERSION.into(),
            latest_version: version.into(),
            release_notes: "Bản phát hành mô phỏng.".into(),
            download_url: String::new(),
        });
    }

    let update = app.updater().map_err(|e| e.to_string())?
        .check().await.map_err(|e| e.to_string())?;
    Ok(match update {
        Some(update) => UpdateInfo {
            has_update: true,
            current_version: update.current_version,
            latest_version: update.version,
            release_notes: update.body.unwrap_or_default(),
            download_url: update.download_url.to_string(),
        },
        None => UpdateInfo {
            has_update: false,
            current_version: CURRENT_VERSION.into(),
            latest_version: CURRENT_VERSION.into(),
            release_notes: String::new(),
            download_url: String::new(),
        },
    })
}

pub async fn apply_update_and_restart(
    app: &tauri::AppHandle,
    latest_version: &str,
    restart_on_finish: bool,
) -> Result<(), String> {
    let update = app.updater().map_err(|e| e.to_string())?
        .check().await.map_err(|e| e.to_string())?
        .ok_or("Không có bản cập nhật mới")?;
    if update.version != latest_version { return Err("Bản cập nhật đã thay đổi; vui lòng kiểm tra lại".into()); }
    update.restart_after_install(restart_on_finish)
        .download_and_install(|_, _| {}, || {})
        .await.map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    if restart_on_finish { app.restart(); }
    Ok(())
}
