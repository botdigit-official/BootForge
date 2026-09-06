#!/bin/bash
# BootForge Universal Installer Script
set -e

echo "=========================================="
echo "⚡ Botdigit BootForge Installer"
echo "Universal Boot-Media Engineering Platform"
echo "=========================================="

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    echo "Detected macOS ($ARCH)"
    echo "Downloading BootForge Universal DMG..."
    TMP_DMG="/tmp/BootForge-macOS.dmg"
    curl -fsSL "https://boot.botdigit.site/assets/BootForge-macOS.dmg" -o "$TMP_DMG"
    echo "Mounting DMG..."
    MOUNT_DIR=$(hdiutil attach "$TMP_DMG" -nobrowse | grep -E '/Volumes/' | awk '{print $3}')
    echo "Installing to /Applications..."
    cp -R "$MOUNT_DIR/Botdigit BootForge.app" /Applications/ 2>/dev/null || cp -R "$MOUNT_DIR/"*.app /Applications/
    hdiutil detach "$MOUNT_DIR" -quiet
    rm -f "$TMP_DMG"
    echo "✓ BootForge successfully installed to /Applications!"
    echo "Run 'open /Applications/Botdigit\ BootForge.app' to launch."
    ;;
  Linux)
    echo "Detected Linux ($ARCH)"
    echo "Downloading BootForge binary..."
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
    curl -fsSL "https://github.com/botdigit-official/BootForge/releases/download/v0.1.0/bootforge-v0.1.0-linux-x86_64.tar.gz" -o "/tmp/bootforge.tar.gz" 2>/dev/null || true
    if [ -f "/tmp/bootforge.tar.gz" ]; then
      tar -xzf "/tmp/bootforge.tar.gz" -C "$INSTALL_DIR" 2>/dev/null || true
      chmod +x "$INSTALL_DIR/bootforge" 2>/dev/null || true
      echo "✓ BootForge installed to $INSTALL_DIR/bootforge"
    else
      echo "For Linux, install via cargo: cargo install bootforge-cli"
    fi
    ;;
  *)
    echo "Unsupported OS: $OS. Please visit https://boot.botdigit.site for manual downloads."
    ;;
esac
