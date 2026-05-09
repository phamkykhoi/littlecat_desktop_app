#!/usr/bin/env bash
# =============================================================================
# Download FFmpeg binaries cho Tauri Sidecar
# Học Lồng Tiếng App
# =============================================================================

set -e

BINARIES_DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

echo "📦 Downloading FFmpeg binaries vào: $BINARIES_DIR"
echo ""

# ── Detect current platform ──────────────────────────────────────────────────
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

# ── Download function ─────────────────────────────────────────────────────────
download_ffmpeg() {
  local url="$1"
  local dest="$2"
  local platform="$3"

  echo "⬇️  Đang tải FFmpeg cho $platform..."
  echo "   URL: $url"
  echo "   → $dest"

  if command -v curl &>/dev/null; then
    curl -L --progress-bar "$url" -o "$dest"
  elif command -v wget &>/dev/null; then
    wget --show-progress -O "$dest" "$url"
  else
    echo "❌ Cần cài curl hoặc wget!"
    exit 1
  fi

  chmod +x "$dest"
  echo "   ✅ Xong! ($(du -sh "$dest" | cut -f1))"
  echo ""
}

# =============================================================================
# macOS Apple Silicon (aarch64)
# =============================================================================
download_macos_arm() {
  local dest="$BINARIES_DIR/ffmpeg-aarch64-apple-darwin"

  if [ -f "$dest" ]; then
    echo "✅ FFmpeg macOS ARM64 đã tồn tại, bỏ qua."
    return
  fi

  # Sử dụng FFmpeg static build từ evermeet.cx (uy tín, cho macOS)
  # Hoặc từ github.com/eugeneware/ffmpeg-static
  local url="https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip"

  echo "⬇️  Đang tải FFmpeg cho macOS Apple Silicon..."

  # Tải zip và extract
  local tmp_zip="/tmp/ffmpeg-macos-arm.zip"
  if command -v curl &>/dev/null; then
    curl -L --progress-bar "$url" -o "$tmp_zip"
  fi

  if [ -f "$tmp_zip" ]; then
    unzip -o "$tmp_zip" -d /tmp/ffmpeg-extract-arm/
    local extracted=$(find /tmp/ffmpeg-extract-arm/ -name "ffmpeg" -type f | head -1)
    if [ -n "$extracted" ]; then
      cp "$extracted" "$dest"
      chmod +x "$dest"
      echo "   ✅ Xong! ($(du -sh "$dest" | cut -f1))"
    else
      echo "   ❌ Không extract được ffmpeg binary"
    fi
    rm -rf "$tmp_zip" /tmp/ffmpeg-extract-arm/
  fi
  echo ""
}

# =============================================================================
# macOS Intel (x86_64)
# =============================================================================
download_macos_intel() {
  local dest="$BINARIES_DIR/ffmpeg-x86_64-apple-darwin"

  if [ -f "$dest" ]; then
    echo "✅ FFmpeg macOS Intel đã tồn tại, bỏ qua."
    return
  fi

  local url="https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip"
  local tmp_zip="/tmp/ffmpeg-macos-intel.zip"

  echo "⬇️  Đang tải FFmpeg cho macOS Intel..."
  if command -v curl &>/dev/null; then
    curl -L --progress-bar "$url" -o "$tmp_zip"
  fi

  if [ -f "$tmp_zip" ]; then
    unzip -o "$tmp_zip" -d /tmp/ffmpeg-extract-intel/
    local extracted=$(find /tmp/ffmpeg-extract-intel/ -name "ffmpeg" -type f | head -1)
    if [ -n "$extracted" ]; then
      cp "$extracted" "$dest"
      chmod +x "$dest"
      echo "   ✅ Xong! ($(du -sh "$dest" | cut -f1))"
    fi
    rm -rf "$tmp_zip" /tmp/ffmpeg-extract-intel/
  fi
  echo ""
}

# =============================================================================
# Windows x86_64
# Lưu ý: Script này chạy trên macOS/Linux để cross-prepare Windows binary
# =============================================================================
download_windows() {
  local dest="$BINARIES_DIR/ffmpeg-x86_64-pc-windows-msvc.exe"

  if [ -f "$dest" ]; then
    echo "✅ FFmpeg Windows đã tồn tại, bỏ qua."
    return
  fi

  # Dùng gyan.dev - build ffmpeg tĩnh cho Windows
  local url="https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-essentials_build.zip"
  local tmp_zip="/tmp/ffmpeg-windows.zip"

  echo "⬇️  Đang tải FFmpeg cho Windows x64..."
  if command -v curl &>/dev/null; then
    curl -L --progress-bar "$url" -o "$tmp_zip"
  fi

  if [ -f "$tmp_zip" ]; then
    unzip -o "$tmp_zip" "*/bin/ffmpeg.exe" -d /tmp/ffmpeg-extract-win/ 2>/dev/null || true
    local extracted=$(find /tmp/ffmpeg-extract-win/ -name "ffmpeg.exe" -type f | head -1)
    if [ -n "$extracted" ]; then
      cp "$extracted" "$dest"
      echo "   ✅ Xong! ($(du -sh "$dest" | cut -f1))"
    else
      echo "   ❌ Không extract được ffmpeg.exe"
    fi
    rm -rf "$tmp_zip" /tmp/ffmpeg-extract-win/
  fi
  echo ""
}

# =============================================================================
# MAIN — chọn theo platform
# =============================================================================
echo "🖥️  Detected: $CURRENT_OS / $CURRENT_ARCH"
echo ""

# Luôn download tất cả platforms để cross-build
DOWNLOAD_ALL="${DOWNLOAD_ALL:-false}"

if [ "$DOWNLOAD_ALL" = "true" ]; then
  echo "📦 Chế độ: Download ALL platforms"
  download_macos_arm
  download_macos_intel
  download_windows
else
  # Chỉ download cho platform hiện tại
  if [ "$CURRENT_OS" = "Darwin" ]; then
    if [ "$CURRENT_ARCH" = "arm64" ]; then
      download_macos_arm
      # Symlink cho Tauri (platform hiện tại cần binary đúng tên)
      if [ -f "$BINARIES_DIR/ffmpeg-aarch64-apple-darwin" ]; then
        echo "🔗 Tauri sẽ dùng: ffmpeg-aarch64-apple-darwin"
      fi
    else
      download_macos_intel
    fi
  elif [ "$CURRENT_OS" = "MINGW64_NT"* ] || [ "$CURRENT_OS" = "MSYS_NT"* ]; then
    echo "⚠️  Đang chạy trên Windows Git Bash"
    echo "   Vui lòng download thủ công từ: https://www.gyan.dev/ffmpeg/builds/"
    echo "   Đặt vào: src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe"
  fi

  echo "💡 Để download tất cả platforms: DOWNLOAD_ALL=true bash scripts/download-ffmpeg.sh"
fi

# =============================================================================
# Kiểm tra kết quả
# =============================================================================
echo ""
echo "📁 Binaries hiện có:"
ls -lh "$BINARIES_DIR/" 2>/dev/null || echo "   (thư mục trống)"
echo ""

# Verify binary hoạt động
echo "🧪 Kiểm tra FFmpeg binary..."
if [ "$CURRENT_OS" = "Darwin" ] && [ "$CURRENT_ARCH" = "arm64" ]; then
  BIN="$BINARIES_DIR/ffmpeg-aarch64-apple-darwin"
elif [ "$CURRENT_OS" = "Darwin" ]; then
  BIN="$BINARIES_DIR/ffmpeg-x86_64-apple-darwin"
fi

if [ -n "$BIN" ] && [ -f "$BIN" ]; then
  "$BIN" -version 2>&1 | head -1
  echo "✅ FFmpeg hoạt động tốt!"
else
  echo "⚠️  Binary chưa sẵn sàng, vui lòng kiểm tra lại."
fi

echo ""
echo "🎉 Hoàn tất! Bây giờ chạy: npm run dev"
