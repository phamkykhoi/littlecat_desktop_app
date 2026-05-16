use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: String,
    pub current_version: String,
    pub notes: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: f64,
}

/// Kiểm tra xem có bản cập nhật mới không
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();

    let update = app
        .updater_builder()
        .build()
        .map_err(|e| format!("Lỗi build updater: {}", e))?
        .check()
        .await
        .map_err(|e| format!("Lỗi kiểm tra cập nhật: {}", e))?;

    match update {
        Some(update) => Ok(UpdateInfo {
            available: true,
            version: update.version.clone(),
            current_version: current,
            notes: update.body.clone().unwrap_or_default(),
        }),
        None => Ok(UpdateInfo {
            available: false,
            version: current.clone(),
            current_version: current,
            notes: String::new(),
        }),
    }
}

/// Download + cài đặt bản cập nhật, emit progress về frontend
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    use tauri::Emitter;

    let update = app
        .updater_builder()
        .build()
        .map_err(|e| format!("Lỗi build updater: {}", e))?
        .check()
        .await
        .map_err(|e| format!("Lỗi kiểm tra cập nhật: {}", e))?;

    match update {
        Some(update) => {
            let app_clone = app.clone();
            let mut downloaded_total: u64 = 0;

            // Download với progress callback
            update
                .download_and_install(
                    |chunk_length, content_length| {
                        downloaded_total += chunk_length as u64;
                        let percent = content_length
                            .map(|total| {
                                if total > 0 {
                                    (downloaded_total as f64 / total as f64) * 100.0
                                } else {
                                    0.0
                                }
                            })
                            .unwrap_or(0.0);

                        let progress = UpdateProgress {
                            downloaded: downloaded_total,
                            total: content_length,
                            percent,
                        };
                        app_clone.emit("update-progress", &progress).ok();
                    },
                    || {
                        log::info!("Cập nhật xong, đang restart...");
                    },
                )
                .await
                .map_err(|e| format!("Lỗi cài đặt cập nhật: {}", e))?;

            Ok(())
        }
        None => Err("Không tìm thấy bản cập nhật".to_string()),
    }
}
