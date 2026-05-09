use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

/// Payload gửi về frontend khi đang convert
#[derive(Clone, Serialize, Deserialize)]
pub struct ConvertProgress {
    pub percent: f64,
    pub current_time: String,
    pub total_time: String,
    pub status: String, // "converting" | "done" | "error"
}

/// Parse thời gian "HH:MM:SS.xx" → giây (f64)
fn parse_time_to_seconds(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.trim().split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().ok()?;
        let m: f64 = parts[1].parse().ok()?;
        let s: f64 = parts[2].parse().ok()?;
        Some(h * 3600.0 + m * 60.0 + s)
    } else {
        None
    }
}

/// Format giây → "MM:SS"
fn format_seconds(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}

/// Command chính: Convert video .webm → .mp4 bằng FFmpeg Sidecar
/// Progress được emit qua Tauri event "convert-progress"
#[tauri::command]
pub async fn convert_video(
    app: AppHandle,
    input_path: String,
    output_path: String,
) -> Result<String, String> {
    log::info!("Starting conversion: {} → {}", input_path, output_path);

    // Lấy FFmpeg sidecar
    let sidecar_command = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("Không tìm thấy FFmpeg: {}", e))?
        .args([
            "-i",
            &input_path,
            "-vcodec",
            "libx264",
            "-crf",
            "28",
            "-preset",
            "medium",
            "-acodec",
            "aac",
            "-movflags",
            "+faststart", // Tốt hơn để web playback
            "-y",         // Overwrite nếu tồn tại
            &output_path,
        ]);

    let (mut rx, _child) = sidecar_command
        .spawn()
        .map_err(|e| format!("Không thể khởi động FFmpeg: {}", e))?;

    // Regex parse duration và progress từ FFmpeg stderr
    let duration_re = Regex::new(r"Duration:\s*(\d+:\d+:\d+\.\d+)").unwrap();
    let time_re = Regex::new(r"time=(\d+:\d+:\d+\.\d+)").unwrap();

    let mut total_duration_secs: f64 = 0.0;
    let mut total_time_str = String::from("00:00");

    while let Some(event) = rx.recv().await {
        use tauri_plugin_shell::process::CommandEvent;
        match event {
            // FFmpeg ghi progress vào stderr
            CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes).to_string();

                // Lấy tổng duration khi FFmpeg đọc file đầu vào
                if let Some(caps) = duration_re.captures(&line) {
                    if let Some(dur_str) = caps.get(1) {
                        if let Some(secs) = parse_time_to_seconds(dur_str.as_str()) {
                            total_duration_secs = secs;
                            total_time_str = format_seconds(secs);
                            log::info!("Video duration: {}s", secs);
                        }
                    }
                }

                // Parse tiến độ hiện tại
                if let Some(caps) = time_re.captures(&line) {
                    if let Some(time_str) = caps.get(1) {
                        if let Some(current_secs) = parse_time_to_seconds(time_str.as_str()) {
                            let percent = if total_duration_secs > 0.0 {
                                (current_secs / total_duration_secs * 100.0).min(99.0)
                            } else {
                                0.0
                            };

                            let progress = ConvertProgress {
                                percent,
                                current_time: format_seconds(current_secs),
                                total_time: total_time_str.clone(),
                                status: "converting".to_string(),
                            };

                            app.emit("convert-progress", &progress).ok();
                        }
                    }
                }
            }

            CommandEvent::Stdout(bytes) => {
                let line = String::from_utf8_lossy(&bytes).to_string();
                log::debug!("FFmpeg stdout: {}", line.trim());
            }

            CommandEvent::Terminated(status) => {
                log::info!("FFmpeg terminated with code: {:?}", status.code);
                if status.code == Some(0) {
                    // Emit 100% done
                    let done = ConvertProgress {
                        percent: 100.0,
                        current_time: total_time_str.clone(),
                        total_time: total_time_str.clone(),
                        status: "done".to_string(),
                    };
                    app.emit("convert-progress", &done).ok();
                    return Ok(output_path);
                } else {
                    let err_progress = ConvertProgress {
                        percent: 0.0,
                        current_time: "00:00".to_string(),
                        total_time: total_time_str.clone(),
                        status: "error".to_string(),
                    };
                    app.emit("convert-progress", &err_progress).ok();
                    return Err("FFmpeg chuyển đổi thất bại. Vui lòng thử lại.".to_string());
                }
            }

            CommandEvent::Error(err) => {
                log::error!("FFmpeg error: {}", err);
                return Err(format!("Lỗi FFmpeg: {}", err));
            }

            _ => {}
        }
    }

    Err("FFmpeg kết thúc bất ngờ".to_string())
}

/// Kiểm tra FFmpeg sidecar có tồn tại không
#[tauri::command]
pub async fn check_ffmpeg(app: AppHandle) -> Result<bool, String> {
    let result = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(["-version"])
        .output()
        .await;

    match result {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}
