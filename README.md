# ⚡ Botdigit BootForge
### *Universal Boot Media & Multi-OS USB Studio for macOS*

[![macOS](https://img.shields.io/badge/Platform-macOS%2011%2B%20(Universal)-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/botdigit/bootforge)
[![Rust](https://img.shields.io/badge/Engine-Rust%201.80%2B-orange?style=for-the-badge&logo=rust&logoColor=white)](https://github.com/botdigit/bootforge)
[![Swift](https://img.shields.io/badge/GUI-Native%20Swift%20Cocoa-F05138?style=for-the-badge&logo=swift&logoColor=white)](https://github.com/botdigit/bootforge)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

> **One Drive. Every OS.**  
> Create multi-boot USB drives on macOS without terminal headaches. Boot Windows 11/10, Ubuntu, Debian, Linux Mint, Clonezilla, Alpine, and recovery utilities from a single flash drive or external SSD.

---

## ✨ Features

* 🍏 **Native Standalone macOS Desktop App**: Runs in its own dedicated Cocoa window with dark glassmorphism, dock icon, and native window controls.
* ⚡ **Drag & Drop Multi-Boot Engine**: Drop ISOs, WIMs, and disk images directly to build a bootable USB with live SHA-256 verification.
* 🔍 **Deep Storage & S.M.A.R.T. Health Analyzer**: Inspect partition maps, ESP/UEFI slices, bus speeds (USB 3.2 Gen 2x2 @ 20Gbps), and wear levels.
* 📦 **Installed OS Image Scanner**: Automatically detects all operating system images already residing on connected USB drives.
* 📍 **Drive Mount & Volume Manager**: 1-click `📂 Open in Finder`, `⚡ Mount Volume`, and `⏏️ Safely Eject`.
* 🛡️ **Zero-Accident Safety Shield**: Internal Apple SSDs are strictly locked. Drives larger than 100 GB require explicit typed disk confirmation.
* 🚀 **Real-time Live Transfer Stream**: High-throughput 4MB chunk streaming with live MB/s speedometer, byte counter, ETA timer, and terminal logging.

---

## 🚀 Getting Started on macOS

### 1. Download DMG Installer
1. Open [`dist/Botdigit-BootForge-v0.1.0-macOS.dmg`](dist/Botdigit-BootForge-v0.1.0-macOS.dmg).
2. Drag **Botdigit BootForge** into your **Applications** folder.
3. Launch **Botdigit BootForge** from Applications or Spotlight.

### 2. Or Launch with 1-Click Command
Double-click [`Launch Botdigit BootForge.command`](Launch%20Botdigit%20BootForge.command) in the project directory.

---

## 💻 CLI Commands Reference

Botdigit BootForge also provides a full-featured CLI:

```bash
# Scan and list all physical & USB storage drives
bootforge devices

# Deep inspect partition tables and health of a specific drive
bootforge analyze-device disk9

# Analyze an ISO image for UEFI / Secure Boot compatibility
bootforge analyze /path/to/ubuntu-24.04.iso

# Launch the backend UI server on a custom port
bootforge ui --port 4242
```

---

## 🔨 Building from Source

```bash
# Clone the repository
git clone https://github.com/botdigit/bootforge.git
cd bootforge

# Build native release app and DMG installer
./scripts/package_mac.sh
```

---

## 📜 License
Botdigit BootForge is open source software licensed under the [MIT License](LICENSE).
