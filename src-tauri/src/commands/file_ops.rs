use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize, Deserialize)]
pub struct SaveResult {
    pub success: bool,
    pub path: Option<String>,
    pub message: String,
}

/// Lưu dữ liệu base64 (video blob từ MediaRecorder) vào file tạm
/// Trả về đường dẫn file tạm
#[tauri::command]
pub async fn save_temp_file(
    _app: AppHandle,
    base64_data: String,
    extension: String,
) -> Result<String, String> {
    // Decode base64 → bytes
    let bytes = BASE64
        .decode(&base64_data)
        .map_err(|e| format!("Lỗi decode base64: {}", e))?;

    // Tạo file tạm trong thư mục temp của hệ thống
    let temp_dir = std::env::temp_dir();
    let filename = format!(
        "hoclongtieng_temp_{}.{}",
        chrono_timestamp(),
        extension.trim_start_matches('.')
    );
    let temp_path = temp_dir.join(&filename);

    fs::write(&temp_path, &bytes)
        .map_err(|e| format!("Không thể ghi file tạm: {}", e))?;

    log::info!(
        "Saved temp file: {} ({} bytes)",
        temp_path.display(),
        bytes.len()
    );

    Ok(temp_path.to_string_lossy().to_string())
}

/// Mở native Save Dialog → trả về đường dẫn user chọn
#[tauri::command]
pub async fn open_save_dialog(
    app: AppHandle,
    default_filename: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::FilePath;

    let path = app
        .dialog()
        .file()
        .set_title("Lưu video của bé 🎬")
        .set_file_name(&default_filename)
        .add_filter("Video MP4", &["mp4"])
        .blocking_save_file();

    match path {
        Some(file_path) => {
            let path_str = match file_path {
                FilePath::Path(p) => p.to_string_lossy().to_string(),
                FilePath::Url(u) => u.to_string(),
            };
            Ok(Some(path_str))
        }
        None => Ok(None), // User cancel
    }
}

/// Copy file từ temp → vị trí user chọn
#[tauri::command]
pub async fn save_file_to_path(
    source_path: String,
    dest_path: String,
) -> Result<SaveResult, String> {
    let src = PathBuf::from(&source_path);
    let dst = PathBuf::from(&dest_path);

    if !src.exists() {
        return Ok(SaveResult {
            success: false,
            path: None,
            message: "File gốc không tồn tại".to_string(),
        });
    }

    fs::copy(&src, &dst).map_err(|e| format!("Không thể lưu file: {}", e))?;

    log::info!("Saved file to: {}", dst.display());

    Ok(SaveResult {
        success: true,
        path: Some(dst.to_string_lossy().to_string()),
        message: "Lưu thành công! 🎉".to_string(),
    })
}

/// Xóa file tạm sau khi đã lưu
#[tauri::command]
pub async fn delete_temp_file(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if p.exists() {
        fs::remove_file(&p).map_err(|e| format!("Không thể xóa file tạm: {}", e))?;
        log::info!("Deleted temp file: {}", p.display());
    }
    Ok(())
}

/// Tạo tên file mặc định theo timestamp
#[tauri::command]
pub fn generate_filename() -> String {
    format!("video_hoclongtieng_{}.mp4", chrono_timestamp())
}

/// Helper: timestamp đơn giản (không dùng chrono để tránh dependency nặng)
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Tạo format YYYYMMDD_HHmmss đơn giản từ unix timestamp
    // Đây là tính toán đơn giản, không phải timezone-aware
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    // Tính ngày tháng từ Julian Day Number
    let jdn = days_since_epoch + 2440588; // Unix epoch = JDN 2440588
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let year = 100 * b + d - 4800 + m / 10;
    let month = m + 3 - 12 * (m / 10);
    let day = e - (153 * m + 2) / 5 + 1;

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hour, minute, second
    )
}

/// Lấy thông tin file (kích thước)
#[tauri::command]
pub async fn get_file_size(path: String) -> Result<u64, String> {
    let metadata =
        fs::metadata(&path).map_err(|e| format!("Không thể đọc thông tin file: {}", e))?;
    Ok(metadata.len())
}
