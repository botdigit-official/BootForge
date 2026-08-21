# BootForge — Build Instructions

## Quick Build

```bash
# Development (hot-reload)
cargo tauri dev

# Production build (current platform)
cargo tauri build

# Core library only (no UI)
cargo build --release
```

---

## Platform-Specific Builds

### macOS

**Universal Binary (Intel + Apple Silicon)**:
```bash
# Add both targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build universal binary
cargo tauri build --target universal-apple-darwin
```

Output:
```
target/release/bundle/
├── macos/BootForge.app
└── dmg/BootForge_0.1.0_universal.dmg
```

**Code Signing** (for distribution):
```bash
# Sign the app
codesign --force --deep --sign "Developer ID Application: Your Name" \
    target/release/bundle/macos/BootForge.app

# Notarize
xcrun notarytool submit target/release/bundle/dmg/BootForge.dmg \
    --apple-id your@email.com \
    --team-id TEAMID \
    --password @keychain:notarytool-password
```

### Windows

```bash
# Requires Visual Studio Build Tools
cargo tauri build --target x86_64-pc-windows-msvc
```

Output:
```
target/release/bundle/
├── msi/BootForge_0.1.0_x64_en-US.msi
└── nsis/BootForge_0.1.0_x64-setup.exe
```

### Linux

```bash
# AppImage + DEB + RPM
cargo tauri build --target x86_64-unknown-linux-gnu
```

Output:
```
target/release/bundle/
├── appimage/BootForge_0.1.0_amd64.AppImage
├── deb/bootforge_0.1.0_amd64.deb
└── rpm/bootforge-0.1.0-1.x86_64.rpm
```

---

## Cross-Compilation

Cross-compiling from macOS to other platforms requires additional setup.

### macOS → Linux

```bash
# Install cross-compilation tools
cargo install cross

# Build Linux binary
cross build --release --target x86_64-unknown-linux-gnu
```

### macOS → Windows

Windows cross-compilation from macOS is complex. Recommended approach:
- Use GitHub Actions CI for Windows builds
- Or use a Windows VM

---

## CI/CD

See `.github/workflows/` for automated build pipelines.

### Release Workflow

1. Tag a version: `git tag v0.1.0`
2. Push tag: `git push origin v0.1.0`
3. CI builds for all platforms
4. Creates GitHub Release with binaries
5. Updates Homebrew tap / Winget / AUR package

---

## Build Configuration

### Tauri Config (`src-tauri/tauri.conf.json`)

```json
{
    "productName": "BootForge",
    "version": "0.1.0",
    "identifier": "dev.bootforge.app",
    "build": {
        "frontendDist": "../src/ui/dist"
    },
    "app": {
        "windows": [
            {
                "title": "BootForge — One Drive. Every OS.",
                "width": 900,
                "height": 700,
                "minWidth": 600,
                "minHeight": 500,
                "resizable": true,
                "decorations": true
            }
        ],
        "security": {
            "csp": "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com"
        }
    },
    "bundle": {
        "active": true,
        "icon": [
            "icons/32x32.png",
            "icons/128x128.png",
            "icons/128x128@2x.png",
            "icons/icon.icns",
            "icons/icon.ico"
        ],
        "macOS": {
            "minimumSystemVersion": "12.0"
        },
        "windows": {
            "certificateThumbprint": null,
            "digestAlgorithm": "sha256"
        }
    }
}
```

---

## Troubleshooting

### Build Errors

| Error | Solution |
|-------|----------|
| `missing webkit2gtk` | Install `libwebkit2gtk-4.1-dev` (Linux) |
| `MSVC not found` | Install Visual Studio Build Tools (Windows) |
| `xcrun: error` | Install Xcode CLI: `xcode-select --install` |
| `OpenSSL not found` | Install `libssl-dev` or use vendored: `openssl = { version = "0.10", features = ["vendored"] }` |
| `linking failed` | Check platform-specific dependencies are installed |
