# BootForge OS — Custom Distribution Specification

## Overview

BootForge OS is a lightweight, purpose-built Linux distribution designed to be a universal rescue, diagnostic, and utility environment. It lives alongside other ISOs on a BootForge multi-boot drive and provides tools for system recovery, disk management, file operations, and hardware diagnostics.

> **BootForge OS is NOT a daily-use operating system.** It is a specialized tool environment — like a Swiss Army knife for computer maintenance.

---

## Design Philosophy

```
┌──────────────────────────────────────────────────────────┐
│                    DESIGN PRINCIPLES                     │
│                                                          │
│  1. BOOT FAST      — Under 15 seconds to usable desktop │
│  2. JUST WORK      — Hardware detection out of the box   │
│  3. DO NO HARM     — Read-only by default, explicit write│
│  4. STAY SMALL     — Under 1GB ISO, runs in 512MB RAM    │
│  5. LOOK GOOD      — Modern, polished UI                 │
│  6. BE USEFUL      — Tools that solve real problems      │
└──────────────────────────────────────────────────────────┘
```

---

## Editions

### BootForge OS Lite (Free)
> Minimal rescue environment

- Boot to terminal
- Basic file operations (cp, mv, ls, cat)
- Disk tools (fdisk, parted, mkfs)
- Network tools (ip, ping, ssh, curl)
- 250MB ISO size
- Runs on 256MB RAM

### BootForge OS Standard (Free)
> Full graphical rescue environment

Everything in Lite, plus:
- Graphical desktop (lightweight window manager)
- File Explorer with GUI
- Disk Manager with GUI
- Terminal emulator
- Web browser (lightweight)
- 600MB ISO size
- Runs on 512MB RAM

### BootForge OS Pro (Paid)
> Professional rescue and diagnostic suite

Everything in Standard, plus:
- Advanced backup tools (disk imaging, incremental backup)
- Data recovery tools
- Hardware diagnostics suite
- Network diagnostics
- Remote access tools
- Automated repair scripts
- 900MB ISO size
- Runs on 1GB RAM

---

## System Architecture

```
┌──────────────────────────────────────────────────┐
│                BootForge OS Stack                  │
│                                                    │
│  ┌────────────────────────────────────────────┐   │
│  │            User Applications                │   │
│  │                                            │   │
│  │  File Explorer │ Disk Manager │ Terminal    │   │
│  │  Backup Tool  │ Network Tools │ Browser    │   │
│  │  Hardware Info │ Recovery     │ Formatter   │   │
│  └──────────────────────┬─────────────────────┘   │
│                         │                          │
│  ┌──────────────────────┴─────────────────────┐   │
│  │         Desktop Environment                 │   │
│  │                                            │   │
│  │  Window Manager: labwc (Wayland) or IceWM  │   │
│  │  Panel: Waybar or tint2                    │   │
│  │  File Manager: PCManFM-Qt (lite)           │   │
│  │  Terminal: foot or sakura                  │   │
│  └──────────────────────┬─────────────────────┘   │
│                         │                          │
│  ┌──────────────────────┴─────────────────────┐   │
│  │            System Layer                     │   │
│  │                                            │   │
│  │  Init: OpenRC or BusyBox init              │   │
│  │  Shell: Bash + BusyBox                     │   │
│  │  Package: Alpine APK or Void XBPS          │   │
│  └──────────────────────┬─────────────────────┘   │
│                         │                          │
│  ┌──────────────────────┴─────────────────────┐   │
│  │            Linux Kernel                     │   │
│  │                                            │   │
│  │  Custom config: broad hardware support     │   │
│  │  Modules: USB, NVMe, SATA, NIC, WiFi, GPU │   │
│  │  Firmware: linux-firmware package          │   │
│  └────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
```

### Base Distribution Choice

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **Alpine Linux** | Tiny (~5MB base), musl libc, fast boot | Limited software, musl compatibility | ✅ Best for Lite |
| **Void Linux** | Fast, runit init, musl option | Smaller community | ✅ Good alternative |
| **Debian minimal** | Huge software repo, stable | Larger base size | Good for Pro |
| **Arch** | Rolling, latest packages | Complex, no installer | Not ideal for embedded |
| **Buildroot** | Custom everything | High effort | For advanced custom needs |

**Recommendation**: Start with **Alpine Linux** as the base for all editions. Its tiny footprint and fast boot time align perfectly with BootForge OS goals.

---

## Built-in Applications

### 1. File Explorer

**Purpose**: Browse, copy, move, and manage files across all detected drives.

```
┌──────────────────────────────────────────────────┐
│  BootForge File Explorer               ─ □ ✕    │
├──────────────────────────────────────────────────┤
│  ← → ↑  /mnt/sda1/Users/Documents               │
├────────────────┬─────────────────────────────────┤
│                │                                  │
│  📁 Devices    │  Name          Size     Modified │
│  ├─ 💾 sda     │  ────────────────────────────── │
│  │  ├─ sda1 C: │  📁 Documents  —        Jun 15 │
│  │  └─ sda2 D: │  📁 Pictures   —        Jun 10 │
│  ├─ 💾 sdb     │  📄 resume.pdf 2.1 MB   Jun 14 │
│  │  └─ sdb1    │  📄 notes.txt  12 KB    Jun 13 │
│  └─ 💾 nvme0n1 │  📄 backup.zip 856 MB   Jun 12 │
│     ├─ p1      │                                  │
│     └─ p2      │                                  │
│                │                                  │
│  📁 Network    │                                  │
│  └─ + Add SMB  │                                  │
│                │                                  │
├────────────────┴─────────────────────────────────┤
│  3 files, 858 MB | Free: 45 GB                    │
└──────────────────────────────────────────────────┘
```

**Features**:
- Auto-mount all detected partitions (read-only by default)
- Support: NTFS, exFAT, FAT32, ext2/3/4, Btrfs, XFS, HFS+, APFS (read-only)
- Copy/Paste across different filesystems
- Drag and drop support
- Search functionality
- Bulk file operations
- SMB/NFS network share browsing
- ZIP/TAR extraction

### 2. Disk Manager

**Purpose**: Partition, format, and manage storage devices.

```
┌──────────────────────────────────────────────────┐
│  BootForge Disk Manager                ─ □ ✕    │
├──────────────────────────────────────────────────┤
│                                                   │
│  ┌─ /dev/sda ─ Samsung SSD 970 EVO 500GB ──────┐│
│  │                                               ││
│  │ ████████████████████████████ ░░░░░░░░░░░░░░  ││
│  │ [  sda1 EFI  ] [    sda2 Windows (NTFS)    ] ││
│  │   512 MB          465 GB                      ││
│  │                                               ││
│  └───────────────────────────────────────────────┘│
│                                                   │
│  ┌─ /dev/sdb ─ SanDisk Extreme 256GB ───────────┐│
│  │                                               ││
│  │ ████████████████████████████████████████████  ││
│  │ [       sdb1 BOOTFORGE (exFAT)             ] ││
│  │   256 GB                                      ││
│  │                                               ││
│  └───────────────────────────────────────────────┘│
│                                                   │
│  [ New Partition ]  [ Format ]  [ Delete ]        │
│  [ Resize ]  [ Clone ]  [ Wipe ]                  │
│                                                   │
└──────────────────────────────────────────────────┘
```

**Features**:
- Visual partition layout
- Create/Delete/Resize partitions
- Format with: FAT32, exFAT, NTFS, ext4, Btrfs, XFS
- Partition table: GPT, MBR
- Partition cloning
- Secure wipe (zero-fill, random, DoD)
- S.M.A.R.T. health display
- Benchmark tools

### 3. Backup Tool

**Purpose**: Create and restore disk images and file backups.

```
┌──────────────────────────────────────────────────┐
│  BootForge Backup                      ─ □ ✕    │
├──────────────────────────────────────────────────┤
│                                                   │
│  BACKUP MODE                                      │
│                                                   │
│  ● Full Disk Image                                │
│  ○ Partition Image                                │
│  ○ File/Folder Backup                            │
│  ○ Clone Disk-to-Disk                            │
│                                                   │
│  Source: [  /dev/sda (Samsung SSD 500GB)       ▼] │
│  Target: [  /dev/sdb/backups/                  ▼] │
│                                                   │
│  Options:                                         │
│  ☑ Compress (gzip)                               │
│  ☐ Split into 4GB files                          │
│  ☑ Verify after backup                           │
│  ☐ Encrypt (AES-256)                             │
│                                                   │
│  Estimated size: ~120 GB (used space only)        │
│  Available space: 180 GB                          │
│                                                   │
│  [ START BACKUP ]                                 │
│                                                   │
│  Progress: ████████████░░░░░░░░ 60%              │
│  Speed: 450 MB/s | ETA: 3 min                    │
│                                                   │
└──────────────────────────────────────────────────┘
```

**Features**:
- Full disk imaging (dd + compression)
- Partition-level imaging
- File-level backup (rsync)
- Disk cloning (disk-to-disk)
- Compression: gzip, zstd, lz4
- Encryption: AES-256
- Split large images for FAT32 compatibility
- Verify after backup
- Restore from image
- Incremental backup support (Pro)

### 4. Format Tool

**Purpose**: Safe, easy disk formatting with multiple options.

```
┌──────────────────────────────────────────────────┐
│  BootForge Format                      ─ □ ✕    │
├──────────────────────────────────────────────────┤
│                                                   │
│  Device: SanDisk Extreme 256 GB                   │
│                                                   │
│  FORMAT TYPE                                      │
│                                                   │
│  ● Quick Format (seconds)                         │
│  ○ Full Format (writes zeros, minutes-hours)      │
│  ○ Secure Erase (cryptographic, SSD only)        │
│                                                   │
│  FILESYSTEM                                       │
│                                                   │
│  ● exFAT     (Best for USB, Mac + Windows + Linux)│
│  ○ FAT32     (Universal, 4GB file limit)          │
│  ○ NTFS      (Windows, large files)              │
│  ○ ext4      (Linux)                             │
│  ○ APFS      (macOS, read-only creation)         │
│  ○ Btrfs     (Linux, snapshots)                  │
│                                                   │
│  PARTITION TABLE                                  │
│                                                   │
│  ● GPT       (Modern, recommended)               │
│  ○ MBR       (Legacy compatibility)              │
│                                                   │
│  Label: [ MYUSB                              ]    │
│                                                   │
│  ⚠ WARNING: All data will be erased!             │
│                                                   │
│  [ FORMAT DRIVE ]                                 │
│                                                   │
└──────────────────────────────────────────────────┘
```

**Filesystem Recommendation Engine**:

| Use Case | Recommended | Why |
|----------|-------------|-----|
| USB for Mac + Windows | exFAT | Native read/write on both |
| USB for Windows only | NTFS | Best Windows support |
| USB for Linux only | ext4 | Best Linux performance |
| Boot drive (BootForge) | GPT + exFAT + EFI | Multi-boot compatible |
| SD card for camera | FAT32 | Universal camera support |
| External backup drive | ext4 or Btrfs | Reliability + snapshots |
| Sharing files < 4GB | FAT32 | Most compatible |

### 5. Hardware Info

**Purpose**: Full hardware detection and reporting.

```
┌──────────────────────────────────────────────────┐
│  BootForge Hardware Info               ─ □ ✕    │
├──────────────────────────────────────────────────┤
│                                                   │
│  SYSTEM                                           │
│  Manufacturer: Dell                               │
│  Model:        OptiPlex 7090                      │
│  BIOS/UEFI:    Dell BIOS 2.18.0                   │
│  Secure Boot:  Enabled                            │
│                                                   │
│  CPU                                              │
│  Processor:    Intel Core i7-11700 @ 2.50GHz     │
│  Cores/Threads: 8 / 16                           │
│  Architecture: x86_64                            │
│                                                   │
│  MEMORY                                           │
│  Total:        32 GB DDR4 3200MHz                │
│  Modules:      2x 16GB                           │
│                                                   │
│  STORAGE                                          │
│  /dev/nvme0n1: Samsung 970 EVO 500GB (Healthy)   │
│  /dev/sda:     WD Blue 2TB HDD (Healthy)         │
│  /dev/sdb:     SanDisk USB 3.2 256GB             │
│                                                   │
│  NETWORK                                          │
│  eth0:         Intel I219-LM (Connected)         │
│  wlan0:        Intel AX201 (Available)           │
│                                                   │
│  GPU                                              │
│  Intel UHD 750 (integrated)                      │
│                                                   │
│  [ EXPORT REPORT ]  [ COPY TO CLIPBOARD ]        │
│                                                   │
└──────────────────────────────────────────────────┘
```

**Detection Sources**:
- `/sys/class/dmi/id/` — System manufacturer, model
- `lscpu` — CPU info
- `/proc/meminfo` + `dmidecode` — Memory details
- `lsblk` + `smartctl` — Storage
- `lspci` — PCI devices (GPU, NIC, etc.)
- `lsusb` — USB devices
- `ip link` — Network interfaces
- `efivar` — UEFI variables, Secure Boot status

### 6. Network Tools

**Purpose**: Basic network diagnostics and file transfer.

**Features**:
- IP configuration (DHCP + static)
- WiFi connection manager
- Ping / traceroute / DNS lookup
- SSH client and server
- SCP / SFTP file transfer
- SMB/CIFS share mounting
- NFS share mounting
- HTTP/FTP download
- Speed test

### 7. Recovery Tools

**Purpose**: Fix broken systems.

**Features**:
- Boot repair (fix GRUB, fix Windows BCD)
- Partition recovery (testdisk)
- File recovery (photorec)
- Password reset (chntpw for Windows)
- Registry editor (read-only, for diagnostics)
- Filesystem repair (fsck, chkdsk equivalent)
- MBR/GPT repair

---

## Build System

### ISO Build Process

```
Alpine Base (5MB)
      │
      ├── Add kernel + firmware (~200MB)
      ├── Add desktop environment (~100MB)
      ├── Add BootForge tools (~50MB)
      ├── Add system utilities (~50MB)
      ├── Add drivers + firmware (~150MB)
      │
      ▼
BootForge OS ISO (~600MB for Standard)
```

### Build Script (Conceptual)

```bash
#!/bin/bash
# build-bootforge-os.sh

EDITION="${1:-standard}"  # lite, standard, pro
ARCH="${2:-x86_64}"

# Start from Alpine base
alpine_setup_base

# Install kernel with broad hardware support
install_kernel_with_firmware

# Install edition-specific packages
case "$EDITION" in
    lite)
        install_cli_tools
        ;;
    standard)
        install_cli_tools
        install_desktop
        install_gui_tools
        ;;
    pro)
        install_cli_tools
        install_desktop
        install_gui_tools
        install_pro_tools
        ;;
esac

# Apply BootForge branding
apply_branding

# Generate ISO
generate_iso "bootforge-os-${EDITION}-${ARCH}.iso"
```

---

## Boot Configuration

### BootForge OS GRUB Entry

```grub
menuentry "BootForge OS" --class bootforge --class linux {
    set iso_path="/images/bootforge/bootforge-os-standard.iso"
    loopback loop $iso_path
    linux (loop)/boot/vmlinuz-lts boot=live \
        modprobe.blacklist=nouveau \
        quiet splash
    initrd (loop)/boot/initramfs-lts
}

menuentry "BootForge OS (Safe Mode)" --class bootforge --class linux {
    set iso_path="/images/bootforge/bootforge-os-standard.iso"
    loopback loop $iso_path
    linux (loop)/boot/vmlinuz-lts boot=live \
        nomodeset single
    initrd (loop)/boot/initramfs-lts
}

menuentry "BootForge OS (RAM Mode)" --class bootforge --class linux {
    set iso_path="/images/bootforge/bootforge-os-standard.iso"
    loopback loop $iso_path
    linux (loop)/boot/vmlinuz-lts boot=live \
        toram quiet splash
    initrd (loop)/boot/initramfs-lts
}
```

---

## Filesystem Support Matrix

| Filesystem | Read | Write | Create | Resize | Repair | Package |
|------------|------|-------|--------|--------|--------|---------|
| FAT32 | ✅ | ✅ | ✅ | ✅ | ✅ | dosfstools |
| exFAT | ✅ | ✅ | ✅ | ❌ | ✅ | exfatprogs |
| NTFS | ✅ | ✅ | ✅ | ✅ | ✅ | ntfs-3g |
| ext2/3/4 | ✅ | ✅ | ✅ | ✅ | ✅ | e2fsprogs |
| Btrfs | ✅ | ✅ | ✅ | ✅ | ✅ | btrfs-progs |
| XFS | ✅ | ✅ | ✅ | ✅ | ✅ | xfsprogs |
| ZFS | ✅ | ✅ | ✅ | ❌ | ✅ | zfs-utils |
| HFS+ | ✅ | ⚠️ | ❌ | ❌ | ❌ | hfsprogs |
| APFS | ✅ | ❌ | ❌ | ❌ | ❌ | apfs-fuse |
| ReiserFS | ✅ | ✅ | ✅ | ✅ | ✅ | reiserfsprogs |
| F2FS | ✅ | ✅ | ✅ | ❌ | ✅ | f2fs-tools |

---

## Version Plan

### V3.0 — Initial Release
- Alpine-based minimal environment
- CLI tools + basic GUI (labwc/IceWM)
- File explorer (PCManFM-Qt)
- Disk management (parted/fdisk + GUI wrapper)
- Basic backup (dd + compression)
- Hardware info
- Terminal
- x86_64 architecture

### V3.5 — Enhanced
- WiFi manager GUI
- Advanced backup (incremental)
- Data recovery (testdisk + photorec)
- Boot repair (GRUB + Windows BCD)
- Network file sharing
- ARM64 architecture support

### V4.0 — Professional
- Full diagnostics suite
- Automated repair scripts
- Remote management
- Custom tool plugins
- Network boot (PXE) server mode
- Forensic mode (read-only everything)
