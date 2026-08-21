# BootForge — Development Guide

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.75+ (stable) | Core engine |
| **Node.js** | 20+ LTS | Tauri frontend build |
| **Tauri CLI** | 2.x | Desktop app framework |
| **Git** | 2.x | Version control |

### Platform-Specific Dependencies

#### macOS
```bash
# Xcode Command Line Tools
xcode-select --install

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (via Homebrew)
brew install node@20

# Tauri dependencies (macOS has most things built-in)
# No additional dependencies needed on macOS
```

#### Windows
```powershell
# Rust
winget install Rustlang.Rustup

# Node.js
winget install OpenJS.NodeJS.LTS

# Visual Studio Build Tools (C++ workload)
winget install Microsoft.VisualStudio.2022.BuildTools
# Select "Desktop development with C++" workload

# WebView2 (usually pre-installed on Windows 11)
# Download from https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

#### Linux (Ubuntu/Debian)
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install nodejs

# Tauri system dependencies
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

---

## Project Setup

### Clone and Initialize

```bash
# Clone repository
git clone https://github.com/botdigit/bootforge.git
cd bootforge

# Install Rust toolchain
rustup default stable
rustup target add x86_64-apple-darwin    # macOS Intel
rustup target add aarch64-apple-darwin   # macOS ARM
rustup target add x86_64-pc-windows-msvc # Windows
rustup target add x86_64-unknown-linux-gnu # Linux

# Install Tauri CLI
cargo install tauri-cli

# Install frontend dependencies
cd src/ui
npm install
cd ../..

# Build and run in development mode
cargo tauri dev
```

### Workspace Structure

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "src/core",          # Core library
    "src/engines",       # Boot engines
    "src/analyzers",     # ISO/format analyzers
    "src/managers",      # Device/partition managers
    "src-tauri",         # Tauri backend
]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
thiserror = "1.0"
tracing = "0.1"
sha2 = "0.10"
uuid = { version = "1", features = ["v4"] }
```

---

## Development Workflow

### Daily Development

```bash
# Start development server (hot-reload)
cargo tauri dev

# Run core tests only (no UI)
cargo test -p bootforge-core

# Run specific test
cargo test -p bootforge-analyzers -- iso_analysis::test_ubuntu_detection

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Build release
cargo tauri build
```

### Code Organization

```
src/core/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types
│   ├── events.rs           # Event system
│   └── types.rs            # Shared types

src/analyzers/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── iso.rs              # ISO 9660 analysis
│   ├── wim.rs              # WIM format analysis
│   ├── img.rs              # Raw image analysis
│   ├── hash.rs             # Hash computation
│   ├── os_detect.rs        # OS identification
│   ├── arch_detect.rs      # Architecture detection
│   ├── boot_detect.rs      # Boot capability detection
│   └── compat.rs           # Compatibility scoring

src/engines/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── grub.rs             # GRUB2 configuration
│   ├── efi.rs              # EFI boot setup
│   ├── windows.rs          # Windows boot handling
│   ├── persistence.rs      # Persistence engine
│   └── config_gen.rs       # Boot config generation

src/managers/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── device.rs           # Device detection & monitoring
│   ├── partition.rs        # Partition management
│   ├── format.rs           # Filesystem formatting
│   ├── writer.rs           # Disk writer (ISO copy + verify)
│   ├── download.rs         # OS downloader
│   └── vm.rs               # QEMU VM tester

src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs             # Tauri entry point
│   ├── commands.rs         # IPC command handlers
│   ├── state.rs            # App state management
│   └── events.rs           # Frontend event emitters

src/ui/
├── package.json
├── index.html
├── src/
│   ├── main.js             # Entry point
│   ├── app.js              # Main app component
│   ├── styles/
│   │   ├── index.css       # Global styles
│   │   ├── variables.css   # CSS variables / design tokens
│   │   └── components.css  # Component styles
│   ├── components/
│   │   ├── DevicePanel.js
│   │   ├── IsoCard.js
│   │   ├── IsoList.js
│   │   ├── BootMenu.js
│   │   ├── ProgressBar.js
│   │   ├── DownloadPanel.js
│   │   └── VmTester.js
│   ├── views/
│   │   ├── MainView.js
│   │   ├── SettingsView.js
│   │   └── AboutView.js
│   └── utils/
│       ├── tauri.js        # Tauri IPC helpers
│       ├── format.js       # Formatting utilities
│       └── icons.js        # Icon system
```

---

## IPC API (Tauri Commands)

### Device Commands

```rust
#[tauri::command]
async fn list_devices(state: State<'_, AppState>) -> Result<Vec<StorageDevice>, String>;

#[tauri::command]
async fn get_device_health(device_id: String) -> Result<DeviceHealth, String>;

#[tauri::command]
async fn format_device(
    device_id: String,
    filesystem: String,
    label: String,
    partition_scheme: String,
) -> Result<(), String>;

#[tauri::command]
async fn eject_device(device_id: String) -> Result<(), String>;
```

### ISO Commands

```rust
#[tauri::command]
async fn analyze_iso(path: String) -> Result<IsoAnalysis, String>;

#[tauri::command]
async fn verify_iso_hash(path: String) -> Result<HashResult, String>;

#[tauri::command]
async fn add_iso(
    state: State<'_, AppState>,
    path: String,
) -> Result<IsoAnalysis, String>;

#[tauri::command]
async fn remove_iso(
    state: State<'_, AppState>,
    iso_id: String,
) -> Result<(), String>;
```

### Boot Drive Commands

```rust
#[tauri::command]
async fn create_boot_drive(
    state: State<'_, AppState>,
    device_id: String,
    iso_ids: Vec<String>,
    options: CreateOptions,
    window: Window,
) -> Result<(), String>;

// Progress events emitted to frontend:
// "write-progress" → { phase, percent, speed, eta, current_file }
```

### Download Commands

```rust
#[tauri::command]
async fn get_download_catalog() -> Result<DownloadCatalog, String>;

#[tauri::command]
async fn download_iso(
    url: String,
    target_path: String,
    window: Window,
) -> Result<String, String>;

// Progress events:
// "download-progress" → { percent, speed, downloaded, total, eta }
```

### VM Test Commands

```rust
#[tauri::command]
async fn check_qemu_available() -> Result<bool, String>;

#[tauri::command]
async fn test_iso_in_vm(
    iso_path: String,
    firmware: String, // "bios" | "uefi" | "secure-boot"
) -> Result<VmTestResult, String>;
```

---

## Testing Strategy

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test -p bootforge-analyzers
cargo test -p bootforge-engines
cargo test -p bootforge-managers
```

### Test Data

Keep test ISOs and sample data in `tests/fixtures/`:

```
tests/
├── fixtures/
│   ├── iso/
│   │   ├── minimal-linux.iso      # Tiny valid ISO for testing
│   │   ├── minimal-windows.iso    # Tiny Windows-like ISO
│   │   └── corrupt.iso            # Intentionally corrupted
│   ├── wim/
│   │   └── sample.wim             # Minimal WIM file
│   └── compat/
│       └── test-compat-db.toml    # Test compatibility DB
├── integration/
│   ├── test_iso_analysis.rs
│   ├── test_device_detection.rs
│   └── test_boot_config.rs
└── e2e/
    └── test_full_workflow.rs
```

### Integration Tests

Integration tests require actual hardware (USB drives). These run in CI with specific hardware or locally:

```bash
# Run integration tests (requires USB drive at /dev/disk99 or similar test device)
BOOTFORGE_TEST_DEVICE=/dev/disk99 cargo test --test integration
```

---

## Debugging

### Rust Backend

```bash
# Enable debug logging
RUST_LOG=bootforge=debug cargo tauri dev

# Verbose logging
RUST_LOG=bootforge=trace cargo tauri dev

# Backtrace on panic
RUST_BACKTRACE=1 cargo tauri dev
```

### Frontend

- Open DevTools: `Cmd+Opt+I` (macOS) / `Ctrl+Shift+I` (Windows/Linux)
- Console logs from Tauri backend appear in DevTools console
- Use `console.log` for frontend debugging

---

## Release Process

### Build for All Platforms

```bash
# macOS (Universal binary: Intel + Apple Silicon)
cargo tauri build --target universal-apple-darwin

# Windows
cargo tauri build --target x86_64-pc-windows-msvc

# Linux
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Output

```
target/release/bundle/
├── macos/
│   └── BootForge.app
├── dmg/
│   └── BootForge_1.0.0_universal.dmg
├── msi/
│   └── BootForge_1.0.0_x64.msi
├── deb/
│   └── bootforge_1.0.0_amd64.deb
└── appimage/
    └── BootForge_1.0.0_amd64.AppImage
```

---

## Code Style

### Rust

- Follow standard Rust style (`cargo fmt`)
- Use `clippy` for linting
- Document all public APIs with `///` doc comments
- Use `thiserror` for error types
- Use `tracing` for logging (not `println!`)
- Async where possible (Tokio runtime)

### Frontend (JS/CSS)

- Vanilla JS (no framework in V1, consider Svelte for V2)
- BEM naming for CSS classes
- CSS custom properties for theming
- No external CSS frameworks
- Semantic HTML5

### Commit Messages

```
feat: add ISO analysis for WIM format
fix: correct partition alignment on GPT drives
docs: add hardware detection specification
test: add unit tests for architecture detection
refactor: extract boot config generation
chore: update dependencies
```
