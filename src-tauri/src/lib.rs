mod commands;

use tauri::Manager;

use commands::convert::{check_ffmpeg, convert_video};
use commands::file_ops::{
    delete_temp_file, generate_filename, get_file_size, open_save_dialog, save_file_to_path,
    save_temp_file,
};

/// JavaScript toolbar được inject vào trang web https://littlecat.vn
/// UI thân thiện với trẻ em 3-8 tuổi: button lớn, màu sắc vui, emoji
const TOOLBAR_SCRIPT: &str = r#"
(function() {
  'use strict';

  // Không inject nếu đã inject rồi
  if (document.getElementById('hlt-toolbar')) return;

  // ── Inject Google Fonts (Nunito - font tròn dễ thương) ──
  const fontLink = document.createElement('link');
  fontLink.rel = 'stylesheet';
  fontLink.href = 'https://fonts.googleapis.com/css2?family=Nunito:wght@700;800;900&display=swap';
  document.head.appendChild(fontLink);

  // ── CSS Styles ──
  const style = document.createElement('style');
  style.textContent = `
    #hlt-toolbar {
      position: fixed;
      bottom: 24px;
      left: 50%;
      transform: translateX(-50%);
      z-index: 2147483647;
      font-family: 'Nunito', 'Arial Rounded MT Bold', Arial, sans-serif;
      user-select: none;
    }

    #hlt-toolbar-inner {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      border-radius: 32px;
      padding: 12px 20px;
      display: flex;
      align-items: center;
      gap: 12px;
      box-shadow: 0 8px 32px rgba(102, 126, 234, 0.5), 0 2px 8px rgba(0,0,0,0.3);
      border: 3px solid rgba(255,255,255,0.3);
      backdrop-filter: blur(10px);
      transition: all 0.3s ease;
    }

    #hlt-toolbar-inner:hover {
      box-shadow: 0 12px 40px rgba(102, 126, 234, 0.7), 0 4px 12px rgba(0,0,0,0.4);
      transform: translateY(-2px);
    }

    .hlt-btn {
      border: none;
      border-radius: 20px;
      cursor: pointer;
      font-family: inherit;
      font-weight: 900;
      font-size: 15px;
      padding: 10px 20px;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      gap: 8px;
      white-space: nowrap;
      outline: none;
      -webkit-tap-highlight-color: transparent;
    }

    .hlt-btn:active {
      transform: scale(0.93);
    }

    .hlt-btn:focus-visible {
      outline: 3px solid #fff;
      outline-offset: 2px;
    }

    #hlt-btn-record {
      background: linear-gradient(135deg, #f5576c, #f093fb);
      color: white;
      box-shadow: 0 4px 15px rgba(245, 87, 108, 0.5);
      font-size: 16px;
      padding: 12px 24px;
    }

    #hlt-btn-record:hover {
      background: linear-gradient(135deg, #ee0979, #ff6a00);
      box-shadow: 0 6px 20px rgba(245, 87, 108, 0.7);
    }

    #hlt-btn-stop {
      background: linear-gradient(135deg, #f7971e, #ffd200);
      color: #333;
      box-shadow: 0 4px 15px rgba(255, 200, 0, 0.5);
      font-size: 16px;
      padding: 12px 24px;
      display: none;
    }

    #hlt-btn-stop:hover {
      background: linear-gradient(135deg, #f46b45, #eea849);
    }

    #hlt-btn-minimize {
      background: rgba(255,255,255,0.2);
      color: white;
      font-size: 13px;
      padding: 8px 14px;
      border-radius: 16px;
    }

    #hlt-btn-minimize:hover {
      background: rgba(255,255,255,0.35);
    }

    /* Chấm đỏ nhấp nháy khi đang quay */
    #hlt-rec-dot {
      width: 14px;
      height: 14px;
      background: #ff4757;
      border-radius: 50%;
      display: none;
      box-shadow: 0 0 0 0 rgba(255, 71, 87, 0.7);
      animation: hlt-pulse 1.4s infinite;
      flex-shrink: 0;
    }

    @keyframes hlt-pulse {
      0%   { box-shadow: 0 0 0 0 rgba(255, 71, 87, 0.7); }
      70%  { box-shadow: 0 0 0 10px rgba(255, 71, 87, 0); }
      100% { box-shadow: 0 0 0 0 rgba(255, 71, 87, 0); }
    }

    #hlt-timer {
      color: white;
      font-size: 15px;
      font-weight: 800;
      min-width: 52px;
      text-align: center;
      display: none;
      text-shadow: 0 1px 4px rgba(0,0,0,0.4);
    }

    /* Progress bar khi đang convert */
    #hlt-progress-wrap {
      display: none;
      flex-direction: column;
      gap: 4px;
      min-width: 200px;
    }

    #hlt-progress-label {
      color: white;
      font-size: 13px;
      font-weight: 800;
      text-align: center;
    }

    #hlt-progress-bar-bg {
      background: rgba(255,255,255,0.25);
      border-radius: 10px;
      height: 12px;
      overflow: hidden;
    }

    #hlt-progress-bar {
      height: 100%;
      background: linear-gradient(90deg, #43e97b, #38f9d7);
      border-radius: 10px;
      width: 0%;
      transition: width 0.4s ease;
      box-shadow: 0 0 8px rgba(67, 233, 123, 0.6);
    }

    /* Thông báo popup */
    #hlt-toast {
      position: fixed;
      bottom: 110px;
      left: 50%;
      transform: translateX(-50%);
      background: rgba(30, 30, 60, 0.95);
      color: white;
      padding: 14px 24px;
      border-radius: 20px;
      font-family: 'Nunito', Arial, sans-serif;
      font-size: 15px;
      font-weight: 800;
      z-index: 2147483647;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.3s ease;
      border: 2px solid rgba(255,255,255,0.2);
      box-shadow: 0 8px 24px rgba(0,0,0,0.4);
      text-align: center;
      max-width: 380px;
      white-space: pre-line;
    }

    #hlt-toast.show {
      opacity: 1;
    }

    /* Trạng thái thu gọn toolbar */
    #hlt-toolbar.minimized #hlt-toolbar-inner {
      padding: 10px 16px;
    }

    #hlt-toolbar.minimized .hlt-label-text {
      display: none;
    }

    .hlt-logo {
      font-size: 22px;
      line-height: 1;
    }

    .hlt-separator {
      width: 1px;
      height: 32px;
      background: rgba(255,255,255,0.25);
      flex-shrink: 0;
    }
  `;
  document.head.appendChild(style);

  // ── HTML Toolbar ──
  const toolbar = document.createElement('div');
  toolbar.id = 'hlt-toolbar';
  toolbar.innerHTML = `
    <div id="hlt-toolbar-inner">
      <span class="hlt-logo">🐱</span>
      <div class="hlt-separator"></div>

      <button class="hlt-btn" id="hlt-btn-record" title="Bắt đầu quay màn hình">
        🎬 <span class="hlt-label-text">Bắt đầu Quay</span>
      </button>

      <button class="hlt-btn" id="hlt-btn-stop" title="Dừng và lưu video">
        ⏹ <span class="hlt-label-text">Dừng Quay</span>
      </button>

      <div id="hlt-rec-dot"></div>
      <div id="hlt-timer">00:00</div>

      <div id="hlt-progress-wrap">
        <div id="hlt-progress-label">⚙️ Đang xử lý... 0%</div>
        <div id="hlt-progress-bar-bg">
          <div id="hlt-progress-bar"></div>
        </div>
      </div>

      <div class="hlt-separator"></div>
      <button class="hlt-btn" id="hlt-btn-minimize" title="Thu gọn">⇣</button>
    </div>
  `;
  document.body.appendChild(toolbar);

  // Toast notification
  const toast = document.createElement('div');
  toast.id = 'hlt-toast';
  document.body.appendChild(toast);

  // ── State ──
  let mediaRecorder = null;
  let recordedChunks = [];
  let timerInterval = null;
  let timerSeconds = 0;
  let isConverting = false;
  let tempFilePath = null;
  let isMinimized = false;

  // ── DOM refs ──
  const btnRecord    = document.getElementById('hlt-btn-record');
  const btnStop      = document.getElementById('hlt-btn-stop');
  const btnMinimize  = document.getElementById('hlt-btn-minimize');
  const recDot       = document.getElementById('hlt-rec-dot');
  const timerEl      = document.getElementById('hlt-timer');
  const progressWrap = document.getElementById('hlt-progress-wrap');
  const progressBar  = document.getElementById('hlt-progress-bar');
  const progressLabel= document.getElementById('hlt-progress-label');

  // ── Toast helper ──
  let toastTimeout = null;
  function showToast(msg, duration = 3500) {
    toast.textContent = msg;
    toast.classList.add('show');
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => toast.classList.remove('show'), duration);
  }

  // ── Timer ──
  function formatTimer(secs) {
    const m = Math.floor(secs / 60).toString().padStart(2, '0');
    const s = (secs % 60).toString().padStart(2, '0');
    return `${m}:${s}`;
  }

  function startTimer() {
    timerSeconds = 0;
    timerEl.textContent = '00:00';
    timerEl.style.display = 'block';
    timerInterval = setInterval(() => {
      timerSeconds++;
      timerEl.textContent = formatTimer(timerSeconds);
    }, 1000);
  }

  function stopTimer() {
    clearInterval(timerInterval);
    timerEl.style.display = 'none';
  }

  // ── Bắt đầu quay ──
  async function startRecording() {
    try {
      showToast('📋 Chọn màn hình bạn muốn quay nhé bé ơi!', 4000);

      const displayStream = await navigator.mediaDevices.getDisplayMedia({
        video: { mediaSource: 'screen', frameRate: 30, width: { ideal: 1920 }, height: { ideal: 1080 } },
        audio: { echoCancellation: true, noiseSuppression: true },
      });

      // Cố gắng thêm mic audio
      let combinedStream = displayStream;
      try {
        const micStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
        const audioContext = new AudioContext();
        const dest = audioContext.createMediaStreamDestination();
        if (displayStream.getAudioTracks().length > 0) {
          audioContext.createMediaStreamSource(displayStream).connect(dest);
        }
        audioContext.createMediaStreamSource(micStream).connect(dest);
        combinedStream = new MediaStream([
          ...displayStream.getVideoTracks(),
          ...dest.stream.getTracks(),
        ]);
      } catch (micErr) {
        console.warn('[HLT] Không lấy được mic:', micErr.message);
      }

      recordedChunks = [];
      const mimeType = MediaRecorder.isTypeSupported('video/webm;codecs=vp9,opus')
        ? 'video/webm;codecs=vp9,opus'
        : 'video/webm;codecs=vp8,opus';

      mediaRecorder = new MediaRecorder(combinedStream, {
        mimeType,
        videoBitsPerSecond: 3_000_000,
        audioBitsPerSecond: 128_000,
      });

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) recordedChunks.push(e.data);
      };

      mediaRecorder.onstop = handleRecordingStop;

      // Nếu user stop share từ browser
      displayStream.getVideoTracks()[0].onended = () => {
        if (mediaRecorder && mediaRecorder.state === 'recording') {
          mediaRecorder.stop();
        }
      };

      mediaRecorder.start(1000); // collect mỗi 1 giây

      // Cập nhật UI → đang quay
      btnRecord.style.display = 'none';
      btnStop.style.display = 'flex';
      recDot.style.display = 'block';
      startTimer();

      showToast('🎬 Đang quay màn hình rồi!\n📌 Bấm ⏹ Dừng Quay khi bé xong nhé!', 4000);
    } catch (err) {
      console.error('[HLT] Lỗi bắt đầu quay:', err);
      if (err.name === 'NotAllowedError') {
        showToast('🙈 Bé chưa cho phép quay màn hình.\nHãy nhấn "Cho phép" nhé!', 4000);
      } else {
        showToast('😢 Có lỗi xảy ra: ' + err.message, 4000);
      }
    }
  }

  // ── Dừng quay ──
  function stopRecording() {
    if (mediaRecorder && mediaRecorder.state === 'recording') {
      mediaRecorder.stop();
      mediaRecorder.stream.getTracks().forEach(t => t.stop());
    }
    stopTimer();
    btnStop.style.display = 'none';
    recDot.style.display = 'none';
    showToast('⏳ Đang chuẩn bị video cho bé...', 3000);
  }

  // ── Xử lý sau khi dừng quay ──
  async function handleRecordingStop() {
    if (recordedChunks.length === 0) {
      showToast('😅 Không có dữ liệu quay. Thử lại nhé bé!', 3000);
      resetUI();
      return;
    }

    isConverting = true;
    progressWrap.style.display = 'flex';
    progressBar.style.width = '0%';
    progressLabel.textContent = '⚙️ Đang nén video... 0%';

    try {
      // Blob → ArrayBuffer → base64
      const blob = new Blob(recordedChunks, { type: 'video/webm' });
      const buffer = await blob.arrayBuffer();
      const uint8 = new Uint8Array(buffer);

      // Chia nhỏ để encode base64 (tránh stack overflow)
      let binary = '';
      const chunkSize = 8192;
      for (let i = 0; i < uint8.length; i += chunkSize) {
        binary += String.fromCharCode(...uint8.subarray(i, i + chunkSize));
      }
      const base64Data = btoa(binary);

      // 1. Lưu file .webm tạm
      progressLabel.textContent = '💾 Đang lưu file tạm...';
      tempFilePath = await window.__TAURI__.core.invoke('save_temp_file', {
        base64Data,
        extension: 'webm',
      });

      // Lấy tên file output
      const defaultName = await window.__TAURI__.core.invoke('generate_filename');
      const outputPath = tempFilePath.replace(/\.webm$/, '_out.mp4');

      // 2. Lắng nghe progress từ Rust
      const unlisten = await window.__TAURI__.event.listen('convert-progress', (event) => {
        const { percent, status } = event.payload;
        progressBar.style.width = Math.min(percent, 100) + '%';

        if (status === 'converting') {
          progressLabel.textContent = `⚙️ Đang xử lý... ${Math.round(percent)}%`;
        } else if (status === 'done') {
          progressLabel.textContent = '✅ Xong rồi! 🎉';
          unlisten();
        } else if (status === 'error') {
          progressLabel.textContent = '❌ Có lỗi xảy ra';
          unlisten();
        }
      });

      // 3. Invoke convert
      await window.__TAURI__.core.invoke('convert_video', {
        inputPath: tempFilePath,
        outputPath,
      });

      // 4. Mở Save Dialog
      progressLabel.textContent = '📁 Chọn nơi lưu video...';
      const savePath = await window.__TAURI__.core.invoke('open_save_dialog', {
        defaultFilename: defaultName,
      });

      if (savePath) {
        // Copy file đến vị trí user chọn
        const result = await window.__TAURI__.core.invoke('save_file_to_path', {
          sourcePath: outputPath,
          destPath: savePath,
        });

        if (result.success) {
          showToast('🎉 Lưu video thành công!\n📍 ' + savePath, 5000);
        } else {
          showToast('😢 Lưu thất bại: ' + result.message, 4000);
        }
      } else {
        showToast('💡 Bé chưa lưu file.\nBấm Quay lại để lưu sau nhé!', 4000);
      }

      // 5. Xóa file tạm
      await window.__TAURI__.core.invoke('delete_temp_file', { path: tempFilePath }).catch(() => {});
      await window.__TAURI__.core.invoke('delete_temp_file', { path: outputPath }).catch(() => {});
      tempFilePath = null;

    } catch (err) {
      console.error('[HLT] Lỗi xử lý video:', err);
      showToast('😢 Lỗi xử lý video:\n' + err, 4000);
    } finally {
      isConverting = false;
      resetUI();
    }
  }

  // ── Reset UI về trạng thái ban đầu ──
  function resetUI() {
    btnRecord.style.display = 'flex';
    btnStop.style.display = 'none';
    recDot.style.display = 'none';
    progressWrap.style.display = 'none';
    progressBar.style.width = '0%';
    timerEl.style.display = 'none';
    isConverting = false;
  }

  // ── Minimize toolbar ──
  btnMinimize.addEventListener('click', () => {
    isMinimized = !isMinimized;
    toolbar.classList.toggle('minimized', isMinimized);
    btnMinimize.textContent = isMinimized ? '⇡' : '⇣';
    btnMinimize.title = isMinimized ? 'Mở rộng' : 'Thu gọn';
  });

  // ── Event listeners ──
  btnRecord.addEventListener('click', () => {
    if (isConverting) {
      showToast('⏳ Đang xử lý video, chờ bé một chút nhé!', 2500);
      return;
    }
    startRecording();
  });

  btnStop.addEventListener('click', () => {
    stopRecording();
  });

  // ── Draggable toolbar ──
  let isDragging = false;
  let dragStartX = 0, dragStartY = 0;
  let toolbarX = 0, toolbarY = 0;

  const inner = document.getElementById('hlt-toolbar-inner');
  inner.addEventListener('mousedown', (e) => {
    if (e.target.closest('button')) return;
    isDragging = true;
    const rect = toolbar.getBoundingClientRect();
    dragStartX = e.clientX - rect.left;
    dragStartY = e.clientY - rect.top;
    toolbar.style.transition = 'none';
    document.body.style.cursor = 'grabbing';
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    const x = e.clientX - dragStartX;
    const y = e.clientY - dragStartY;
    toolbar.style.left = x + 'px';
    toolbar.style.top = y + 'px';
    toolbar.style.transform = 'none';
    toolbar.style.bottom = 'auto';
  });

  document.addEventListener('mouseup', () => {
    if (isDragging) {
      isDragging = false;
      toolbar.style.transition = '';
      document.body.style.cursor = '';
    }
  });

  // Kiểm tra __TAURI__ sẵn sàng
  if (typeof window.__TAURI__ === 'undefined') {
    console.warn('[HLT] Tauri API chưa sẵn sàng, toolbar sẽ ở chế độ preview');
    btnRecord.disabled = true;
    btnRecord.style.opacity = '0.5';
  }

  console.log('[HLT] 🎬 Học Lồng Tiếng toolbar đã khởi động!');
})();
"#;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Lấy main window và inject toolbar script
            if let Some(window) = app.get_webview_window("main") {
                window
                    .eval(TOOLBAR_SCRIPT)
                    .unwrap_or_else(|e| log::error!("Lỗi inject toolbar script: {}", e));
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Convert commands
            convert_video,
            check_ffmpeg,
            // File operations
            save_temp_file,
            open_save_dialog,
            save_file_to_path,
            delete_temp_file,
            generate_filename,
            get_file_size,
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khởi động ứng dụng Học Lồng Tiếng");
}
