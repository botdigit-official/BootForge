#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DIR"

echo "🔨 Building BootForge Engine Release Binary for macOS..."
cargo build --release

echo "🎨 Ensuring App Icons are generated..."
if [ ! -f "assets/icons/AppIcon.icns" ]; then
    python3 scripts/create_icon.py
fi

DIST_DIR="$DIR/dist"
APP_NAME="Botdigit BootForge"
APP_DIR="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "📦 Creating native macOS App Bundle: $APP_DIR..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# 1. Compile native Swift Cocoa GUI Launcher
echo "🍏 Compiling native standalone macOS Cocoa WebKit App..."
swiftc -O -target arm64-apple-macos11.0 src/native/mac_app.swift -o "$MACOS_DIR/$APP_NAME" -framework Cocoa -framework WebKit || \
swiftc -O src/native/mac_app.swift -o "$MACOS_DIR/$APP_NAME" -framework Cocoa -framework WebKit

chmod +x "$MACOS_DIR/$APP_NAME"

# 2. Copy Rust Backend Engine & MBR writer
cp "target/release/bootforge" "$MACOS_DIR/bootforge_engine"
chmod +x "$MACOS_DIR/bootforge_engine"
cp "target/release/bootforge" "$MACOS_DIR/bootforge"
chmod +x "$MACOS_DIR/bootforge"
if [ -f "target/release/write_mbr_core" ]; then
    cp "target/release/write_mbr_core" "$MACOS_DIR/write_mbr_core"
    chmod +x "$MACOS_DIR/write_mbr_core"
fi

# 3. Copy App Icon and Boot Assets
cp "assets/icons/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
cp -R "src/managers/assets" "$RESOURCES_DIR/assets"

# 4. Create Info.plist
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.botdigit.bootforge</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 5. Create Root Clickable Command for Terminal/Finder double-click
cat > "$DIR/Launch Botdigit BootForge.command" << 'EOF'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "=========================================================="
echo "      ⚡ Botdigit BootForge — Universal Boot Media Studio   "
echo "=========================================================="
echo "Launching Botdigit BootForge Native macOS App..."

if [ -d "./dist/Botdigit BootForge.app" ]; then
    open "./dist/Botdigit BootForge.app"
elif [ -f "./target/release/bootforge" ]; then
    "./target/release/bootforge" ui --port 4242
else
    cargo run --release -- ui --port 4242
fi
EOF
chmod +x "$DIR/Launch Botdigit BootForge.command"

# 6. Create DMG package
DMG_NAME="Botdigit-BootForge-v0.1.0-macOS.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"
DMG_TMP="$DIST_DIR/dmg_tmp"

echo "💿 Packaging DMG: $DMG_PATH..."
rm -rf "$DMG_TMP" "$DMG_PATH"
mkdir -p "$DMG_TMP"
cp -R "$APP_DIR" "$DMG_TMP/"
ln -s /Applications "$DMG_TMP/Applications"

hdiutil create -volname "Botdigit BootForge" -srcfolder "$DMG_TMP" -ov -format UDZO "$DMG_PATH"
rm -rf "$DMG_TMP"

# 7. Create Zip package
cd "$DIST_DIR"
rm -f "Botdigit-BootForge-v0.1.0-macOS.zip"
zip -r -q "Botdigit-BootForge-v0.1.0-macOS.zip" "$APP_NAME.app"
cd "$DIR"

echo "✅ Ready-to-use Botdigit macOS files created successfully!"
echo "   - Native macOS App: dist/$APP_NAME.app"
echo "   - DMG Installer: dist/$DMG_NAME"
echo "   - Zip Archive: dist/Botdigit-BootForge-v0.1.0-macOS.zip"
echo "   - Finder 1-Click Launcher: Launch Botdigit BootForge.command"
