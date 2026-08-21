# BootForge — System Architecture

## Overview

BootForge is architected as a modular, cross-platform application with a Rust core engine and a lightweight Tauri-based desktop UI. The system is designed for reliability, extensibility, and user simplicity.

---

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        USER INTERFACE                         │
│                     (Tauri + HTML/CSS/JS)                     │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐ │
│  │  Device   │  │   ISO    │  │  Boot    │  │  Settings    │ │
│  │  Panel    │  │  Manager │  │  Config  │  │  & Prefs     │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬───────┘ │
│       │              │              │               │         │
├───────┴──────────────┴──────────────┴───────────────┴────────┤
│                        TAURI BRIDGE                           │
│                   (IPC Commands / Events)                     │
├──────────────────────────────────────────────────────────────┤
│                        RUST CORE ENGINE                       │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Device     │  │    ISO      │  │   Boot Engine       │  │
│  │   Manager    │  │   Analyzer  │  │                     │  │
│  │             │  │             │  │  ┌─────┐ ┌───────┐  │  │
│  │  • detect   │  │  • parse    │  │  │GRUB2│ │ EFI   │  │  │
│  │  • monitor  │  │  • verify   │  │  └─────┘ └───────┘  │  │
│  │  • health   │  │  • identify │  │  ┌─────┐ ┌───────┐  │  │
│  │  • format   │  │  • compat   │  │  │ WIM │ │Legacy │  │  │
│  │             │  │             │  │  └─────┘ └───────┘  │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────────┴──────────┐  │
│  │  Partition   │  │ Compat DB   │  │  Config Generator   │  │
│  │  Manager     │  │             │  │                     │  │
│  │             │  │  • quirks   │  │  • grub.cfg         │  │
│  │  • GPT/MBR  │  │  • modes    │  │  • EFI entries      │  │
│  │  • FAT32    │  │  • tested   │  │  • boot menu        │  │
│  │  • exFAT    │  │  • patches  │  │  • persistence      │  │
│  │  • NTFS     │  │             │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Downloader  │  │  VM Tester  │  │  Logger / Telemetry │  │
│  │             │  │             │  │                     │  │
│  │  • catalog  │  │  • QEMU     │  │  • structured logs  │  │
│  │  • verify   │  │  • config   │  │  • error reports    │  │
│  │  • progress │  │  • monitor  │  │  • opt-in metrics   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                    PLATFORM ABSTRACTION                       │
│                                                              │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐                  │
│  │  macOS    │  │  Windows   │  │  Linux   │                  │
│  │          │  │           │  │          │                  │
│  │  diskutil │  │  WMI      │  │  udev    │                  │
│  │  DiskArb  │  │  DeviceIO │  │  lsblk   │                  │
│  │  hdiutil  │  │  VDS      │  │  sysfs   │                  │
│  └──────────┘  └───────────┘  └──────────┘                  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Module Specifications

### 1. Device Manager

**Purpose**: Detect, monitor, and manage USB storage devices.

```rust
pub struct DeviceManager {
    watcher: DeviceWatcher,
    devices: Vec<StorageDevice>,
}

pub struct StorageDevice {
    pub id: String,
    pub vendor: String,
    pub model: String,
    pub serial: String,
    pub capacity: u64,
    pub interface: UsbInterface,
    pub speed: TransferSpeed,
    pub device_type: DeviceType,
    pub health: Option<SmartHealth>,
    pub partitions: Vec<Partition>,
    pub mount_points: Vec<PathBuf>,
}

pub enum UsbInterface {
    Usb20,
    Usb30,
    Usb31Gen1,
    Usb31Gen2,
    Usb32Gen2x2,
    Usb40,
    Thunderbolt3,
    Thunderbolt4,
}

pub enum DeviceType {
    FlashDrive,
    PortableSsd,
    ExternalHdd,
    SdCard,
    NvmeEnclosure,
    Unknown,
}
```

**Platform Implementation**:

| Platform | API | Method |
|----------|-----|--------|
| macOS | DiskArbitration + IOKit | `DARegisterDiskAppearedCallback` |
| Windows | WMI + SetupAPI | `WqlEventQuery` on `__InstanceCreationEvent` |
| Linux | udev + sysfs | `udev_monitor_new_from_netlink` |

**Capabilities**:
- Real-time device insertion/removal monitoring
- Device identification (vendor, model, serial, capacity)
- USB interface and speed detection
- S.M.A.R.T. health monitoring (where supported)
- Existing partition table reading
- Safe unmount before operations
- Write verification

---

### 2. ISO Analyzer

**Purpose**: Analyze, identify, and validate disk image files.

```rust
pub struct IsoAnalyzer;

pub struct IsoAnalysis {
    pub file_path: PathBuf,
    pub file_size: u64,
    pub hash: HashResult,
    pub os_info: OsInfo,
    pub boot_info: BootInfo,
    pub architecture: Architecture,
    pub compatibility: CompatibilityScore,
    pub quirks: Vec<KnownQuirk>,
    pub recommended_strategy: BootStrategy,
}

pub struct OsInfo {
    pub family: OsFamily,
    pub distribution: Option<String>,
    pub version: Option<String>,
    pub codename: Option<String>,
    pub image_type: ImageType,
}

pub enum OsFamily {
    Windows,
    Linux,
    Bsd,
    MacOs,
    Recovery,
    Utility,
    Firmware,
    Unknown,
}

pub enum ImageType {
    Installer,
    Live,
    Rescue,
    Pe,           // Windows PE
    NetInstall,
    MinimalInstall,
    FullInstall,
    Firmware,
    Utility,
}

pub struct BootInfo {
    pub uefi_capable: bool,
    pub bios_capable: bool,
    pub secure_boot: SecureBootStatus,
    pub el_torito: Option<ElToritoInfo>,
    pub efi_files: Vec<EfiFile>,
    pub boot_catalog: Option<BootCatalog>,
}

pub enum Architecture {
    X86,
    X86_64,
    Arm64,
    Ia32,
    Mips64el,
    Multi(Vec<Architecture>),
    Unknown,
}

pub enum SecureBootStatus {
    Supported,
    NotSupported,
    Partial,
    Unknown,
}
```

**Detection Methods**:

| OS | Detection Strategy |
|----|-------------------|
| **Windows** | Scan for `sources/install.wim` or `install.esd`, read `AutoUnattend.xml`, check `boot/bcd` |
| **Ubuntu/Debian** | Read `.disk/info`, check `casper/` or `live/` directory |
| **Fedora** | Read `.discinfo`, check `LiveOS/`, `images/pxeboot/` |
| **Arch** | Check `arch/` directory, read `pkglist.*.txt` |
| **Alpine** | Check `apks/` directory structure |
| **Kali** | Read `.disk/info` for Kali identifier |
| **Rescue** | Heuristic: look for diagnostic tools, PE environment markers |
| **MemTest** | Check for `memtest` binary or known signatures |

---

### 3. Boot Engine

**Purpose**: Generate boot configurations and manage bootloader setup.

```rust
pub struct BootEngine {
    grub_engine: GrubEngine,
    efi_engine: EfiEngine,
    windows_engine: WindowsEngine,
    config_generator: ConfigGenerator,
}

pub struct BootConfiguration {
    pub partition_scheme: PartitionScheme,
    pub boot_entries: Vec<BootEntry>,
    pub grub_config: String,
    pub efi_entries: Vec<EfiBootEntry>,
    pub boot_menu: BootMenu,
}

pub struct BootEntry {
    pub label: String,
    pub iso_path: PathBuf,
    pub os_info: OsInfo,
    pub boot_method: BootMethod,
    pub boot_flags: Vec<BootFlag>,
    pub category: BootCategory,
    pub icon: String,
}

pub enum BootMethod {
    Normal,              // Standard ISO boot
    Grub2Direct,         // GRUB2 loopback mount
    WimBoot,             // Windows WIM boot
    MemdiskEmulation,    // MEMDISK for legacy ISOs
    Chainload,           // Chainload existing bootloader
    LinuxVmlinuz,        // Direct Linux kernel boot
}

pub enum BootCategory {
    Windows,
    Linux,
    Bsd,
    Recovery,
    Utility,
    Firmware,
    Custom,
}

pub enum PartitionScheme {
    GptUefiOnly,
    GptUefiPlusBios,    // Recommended: hybrid approach
    MbrBiosOnly,
    MbrBiosWithEfi,
}
```

**Boot Engine Decision Tree**:

```
ISO Analyzed
    │
    ├─ Windows ISO?
    │   ├─ Yes → Use WIM boot method
    │   │   ├─ Extract boot.wim + install.wim
    │   │   ├─ Generate BCD entries
    │   │   └─ Configure EFI + Legacy
    │   └─ No ↓
    │
    ├─ Standard Linux?
    │   ├─ Yes → Use GRUB2 loopback
    │   │   ├─ Generate GRUB menuentry
    │   │   ├─ Configure iso-scan/filename
    │   │   └─ Handle persistence if supported
    │   └─ No ↓
    │
    ├─ Rescue/Utility ISO?
    │   ├─ Yes → Check compatibility DB
    │   │   ├─ Known ISO → Use recommended method
    │   │   └─ Unknown → Try standard, flag for testing
    │   └─ No ↓
    │
    └─ Unknown ISO
        ├─ Analyze boot catalog
        ├─ Try standard ISO boot
        └─ Flag as "may require testing"
```

---

### 4. Partition Manager

**Purpose**: Create and manage disk partitions for boot drives.

```rust
pub struct PartitionManager;

pub struct PartitionLayout {
    pub scheme: PartitionScheme,
    pub partitions: Vec<PartitionSpec>,
}

pub struct PartitionSpec {
    pub label: String,
    pub filesystem: Filesystem,
    pub size: PartitionSize,
    pub flags: Vec<PartitionFlag>,
    pub purpose: PartitionPurpose,
}

pub enum Filesystem {
    Fat32,
    ExFat,
    Ntfs,
    Ext4,
    Btrfs,
}

pub enum PartitionPurpose {
    EfiSystem,          // EFI System Partition (FAT32, ~512MB)
    BiosGrub,           // BIOS GRUB partition (~1MB)
    ImageStorage,       // Main partition for ISO files
    Persistence,        // Linux persistence partition
    DataExchange,       // Optional data partition for file exchange
}
```

**Recommended Layout for Universal Boot Drive**:

```
┌────────────────────────────────────────────────┐
│ GPT Partition Table                            │
│                                                │
│ Part 1: EFI System (FAT32)        512 MB       │
│   ├── EFI/BOOT/BOOTx64.EFI                    │
│   ├── EFI/BOOT/grubx64.efi                    │
│   └── EFI/BOOT/shimx64.efi (Secure Boot)      │
│                                                │
│ Part 2: BIOS GRUB                 1 MB         │
│   └── core.img (GRUB BIOS)                    │
│                                                │
│ Part 3: BootForge Data (exFAT)    Remaining    │
│   ├── bootforge/                               │
│   │   ├── config.json                          │
│   │   ├── grub/grub.cfg                        │
│   │   └── themes/                              │
│   ├── images/                                  │
│   │   ├── windows/                             │
│   │   ├── linux/                               │
│   │   ├── rescue/                              │
│   │   └── utilities/                           │
│   └── persistence/ (optional)                  │
│                                                │
│ Part 4: Persistence (ext4)        Optional     │
│   └── Linux persistence data                   │
└────────────────────────────────────────────────┘
```

---

### 5. Compatibility Database

**Purpose**: Store known ISO compatibility information and quirks.

```rust
pub struct CompatibilityDb {
    entries: HashMap<String, CompatEntry>,
}

pub struct CompatEntry {
    pub iso_identifier: IsoIdentifier,
    pub tested: bool,
    pub compatibility: CompatLevel,
    pub boot_methods: Vec<BootMethodCompat>,
    pub quirks: Vec<Quirk>,
    pub notes: Option<String>,
    pub last_tested: Option<DateTime>,
}

pub enum CompatLevel {
    Excellent,   // Tested, works perfectly
    Good,        // Tested, minor issues
    Fair,        // Works with workarounds
    Poor,        // Significant issues
    Untested,    // Not yet tested
    Incompatible,// Known to not work
}

pub struct Quirk {
    pub id: String,
    pub description: String,
    pub workaround: QuirkWorkaround,
    pub auto_apply: bool,
}

pub enum QuirkWorkaround {
    UseGrub2Mode,
    UseWimBoot,
    UseMemdisk,
    InjectDriver(String),
    ModifyBootParams(String),
    UseAlternativeLoader,
    RequiresLegacyBios,
    Custom(String),
}
```

**Database Format** (TOML):

```toml
[windows-11-x64]
name = "Windows 11"
family = "windows"
architectures = ["x86_64"]
boot_methods = ["uefi", "legacy"]
secure_boot = true
wim_boot = true
compatibility = "excellent"
tested_versions = ["23H2", "24H2"]

[ubuntu-24.04]
name = "Ubuntu 24.04 LTS"
family = "linux"
distribution = "ubuntu"
architectures = ["x86_64", "arm64"]
boot_methods = ["uefi", "legacy"]
secure_boot = true
persistence = true
compatibility = "excellent"
grub_entry = "linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$iso_path"

[hirens-boot-pe]
name = "Hiren's Boot CD PE"
family = "rescue"
architectures = ["x86_64"]
boot_methods = ["uefi"]
secure_boot = false
compatibility = "good"
quirks = ["wimboot-required"]
notes = "Requires WIMBOOT mode for reliable boot"
```

---

### 6. Downloader

**Purpose**: Download OS images from official sources.

```rust
pub struct OsDownloader {
    catalog: DownloadCatalog,
    download_manager: DownloadManager,
}

pub struct DownloadCatalog {
    pub categories: Vec<OsCategory>,
}

pub struct DownloadEntry {
    pub name: String,
    pub version: String,
    pub url: String,               // Official download URL
    pub hash_url: Option<String>,   // URL to hash file
    pub expected_hash: Option<String>,
    pub size: u64,
    pub architecture: Architecture,
    pub mirror_urls: Vec<String>,
}
```

**Supported Download Sources (V1)**:

| OS | Source | Method |
|----|--------|--------|
| Ubuntu | releases.ubuntu.com | Direct HTTP |
| Debian | cdimage.debian.org | Direct HTTP |
| Fedora | download.fedoraproject.org | Direct HTTP |
| Linux Mint | linuxmint.com | Direct HTTP |
| Alpine | alpinelinux.org | Direct HTTP |
| Arch | archlinux.org | Direct HTTP |
| Kali | kali.org | Direct HTTP |
| Windows | microsoft.com | Media Creation Tool API |
| Clonezilla | clonezilla.org | Direct HTTP |
| GParted | gparted.org | Direct HTTP |

> **Important**: All downloads are from official sources only. No third-party mirrors without user consent.

---

### 7. VM Tester

**Purpose**: Test ISO boot capability without physical hardware.

```rust
pub struct VmTester {
    qemu_path: PathBuf,
    ovmf_path: PathBuf,  // UEFI firmware for QEMU
}

pub struct VmTestConfig {
    pub iso_path: PathBuf,
    pub firmware: VmFirmware,
    pub memory: u64,        // MB
    pub cpu_cores: u32,
    pub display: bool,
    pub timeout: Duration,
}

pub enum VmFirmware {
    Bios,
    UefiX64,
    UefiAarch64,
    UefiSecureBoot,
}

pub struct VmTestResult {
    pub success: bool,
    pub boot_time: Duration,
    pub firmware: VmFirmware,
    pub screenshot: Option<PathBuf>,
    pub log: String,
    pub errors: Vec<String>,
}
```

---

## Data Flow

### Create Boot Drive Flow

```
User drops ISOs
       │
       ▼
┌─────────────┐
│ ISO Analyzer │──── Hash verification
│             │──── OS identification
│             │──── Architecture detection
│             │──── Boot method analysis
│             │──── Compatibility check
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Compat DB   │──── Look up known quirks
│             │──── Determine boot strategy
│             │──── Flag warnings
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Boot Engine  │──── Generate partition layout
│             │──── Create GRUB config
│             │──── Generate EFI entries
│             │──── Build boot menu
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Partition    │──── Create partition table
│ Manager      │──── Format partitions
│             │──── Install bootloader
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ File Writer  │──── Copy ISOs to drive
│             │──── Write boot files
│             │──── Write config files
│             │──── Verify writes
└──────┬──────┘
       │
       ▼
   Boot drive ready!
```

---

## Security Considerations

### Privilege Escalation
- USB operations require elevated privileges
- macOS: Request authorization via `AuthorizationCreate`
- Windows: UAC prompt for disk operations
- Linux: polkit or sudo for block device access

### Safe Operations
- **Drive Safety**: Never operate on system drives — detect and block
- **Confirmation**: Double-confirm destructive operations with device details
- **Verification**: Post-write verification of all data
- **Rollback**: Backup partition table before modification

### Download Security
- All downloads over HTTPS only
- SHA256 verification of all downloaded images
- GPG signature verification where available
- Certificate pinning for critical download sources

---

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| USB detection | < 2s | After plugin event |
| ISO analysis | < 5s | For standard ISOs up to 10GB |
| Hash verification | Line speed | SHA256, hardware accelerated |
| Write speed | 90%+ of USB max | Direct I/O, large block writes |
| Boot time (menu) | < 3s | GRUB menu load time |
| App launch | < 1s | Tauri cold start |
| Memory usage | < 100MB | During idle |

---

## Error Handling Strategy

```rust
pub enum BootForgeError {
    // Device errors
    DeviceNotFound,
    DeviceBusy,
    DeviceTooSmall { required: u64, available: u64 },
    DeviceWriteProtected,
    SystemDriveDetected,

    // ISO errors
    IsoCorrupted { path: PathBuf, expected_hash: String, actual_hash: String },
    IsoUnsupported { path: PathBuf, reason: String },
    IsoTooLarge { path: PathBuf, size: u64, available: u64 },

    // Boot errors
    BootloaderInstallFailed { stage: String, detail: String },
    IncompatibleArchitecture { iso_arch: Architecture, requested: Architecture },
    SecureBootUnavailable,

    // Platform errors
    PermissionDenied { operation: String },
    PlatformUnsupported { feature: String },

    // Network errors
    DownloadFailed { url: String, status: u16 },
    HashMismatch { expected: String, actual: String },
}
```

Every error provides:
1. **What happened** — clear description
2. **Why it happened** — root cause
3. **What to do** — actionable recovery steps
