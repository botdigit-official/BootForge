# BootForge — Hardware Detection Specification

## Overview

The Hardware Detection module identifies, monitors, and assesses USB storage devices connected to the system. It provides real-time detection, health monitoring, speed benchmarking, and intelligent recommendations for drive configuration.

---

## Detection Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Hardware Detection                  │
│                                                     │
│  ┌─────────┐   ┌─────────┐   ┌──────────────────┐  │
│  │ Platform │   │ Device  │   │   Health         │  │
│  │ Watcher  │──▶│ Parser  │──▶│   Analyzer       │  │
│  │          │   │         │   │                  │  │
│  │ macOS:   │   │ Vendor  │   │ S.M.A.R.T.      │  │
│  │ DiskArb  │   │ Model   │   │ Wear Level      │  │
│  │          │   │ Serial  │   │ Error Count      │  │
│  │ Windows: │   │ Capacity│   │ Temperature      │  │
│  │ WMI      │   │ Speed   │   │ Power-On Hours   │  │
│  │          │   │ Type    │   │                  │  │
│  │ Linux:   │   │ Mount   │   │                  │  │
│  │ udev     │   │ Parts   │   │                  │  │
│  └─────────┘   └─────────┘   └──────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │              Speed Benchmarker               │    │
│  │                                             │    │
│  │  Sequential Read/Write (1MB blocks)         │    │
│  │  Random Read/Write (4KB blocks)             │    │
│  │  USB interface detection                     │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │         Safety & Recommendation              │    │
│  │                                             │    │
│  │  • System drive protection                  │    │
│  │  • Partition scheme recommendation          │    │
│  │  • Filesystem recommendation                │    │
│  │  • Boot compatibility assessment            │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

---

## Data Model

```rust
pub struct StorageDevice {
    // Identity
    pub id: DeviceId,
    pub vendor: String,
    pub model: String,
    pub serial: String,
    pub firmware_version: Option<String>,
    
    // Physical
    pub capacity: u64,                    // Total bytes
    pub block_size: u32,                  // Logical block size (512 or 4096)
    pub physical_block_size: u32,         // Physical block size
    
    // Interface
    pub interface: UsbInterface,
    pub bus_speed: BusSpeed,
    pub connection_type: ConnectionType,
    
    // Classification
    pub device_type: DeviceType,
    pub removable: bool,
    pub read_only: bool,
    
    // Health
    pub health: Option<DeviceHealth>,
    pub temperature: Option<f32>,         // Celsius
    
    // Current State
    pub partitions: Vec<Partition>,
    pub mount_points: Vec<MountPoint>,
    pub in_use: bool,
    
    // System Protection
    pub is_system_drive: bool,            // NEVER touch these
    pub is_boot_drive: bool,
    
    // Platform-specific
    pub platform_path: String,            // /dev/disk2, \\.\PhysicalDrive1, /dev/sdb
    pub platform_id: String,
}

pub enum UsbInterface {
    Usb20,           // 480 Mbps
    Usb30,           // 5 Gbps (USB 3.0 / USB 3.1 Gen 1 / USB 3.2 Gen 1)
    Usb31Gen2,       // 10 Gbps (USB 3.1 Gen 2 / USB 3.2 Gen 2)
    Usb32Gen2x2,     // 20 Gbps
    Usb40,           // 40 Gbps
    Thunderbolt3,    // 40 Gbps
    Thunderbolt4,    // 40 Gbps
    Unknown,
}

pub enum DeviceType {
    UsbFlashDrive,
    PortableSsd,
    ExternalHdd,
    SdCard,
    MicroSdCard,
    NvmeEnclosure,
    UsbHardDrive,
    CardReader,
    Unknown,
}

pub struct BusSpeed {
    pub max_bandwidth: u64,          // Bytes/sec theoretical
    pub measured_read: Option<u64>,  // Bytes/sec actual
    pub measured_write: Option<u64>, // Bytes/sec actual
}

pub struct Partition {
    pub number: u32,
    pub label: Option<String>,
    pub filesystem: Option<String>,
    pub size: u64,
    pub used: Option<u64>,
    pub mount_point: Option<PathBuf>,
    pub flags: Vec<PartitionFlag>,
    pub uuid: Option<String>,
    pub partition_type: Option<String>,
}

pub struct DeviceHealth {
    pub status: HealthStatus,
    pub wear_percentage: Option<u8>,      // 0-100, SSD wear
    pub power_on_hours: Option<u64>,
    pub power_on_count: Option<u64>,
    pub error_count: u64,
    pub reallocated_sectors: u64,
    pub pending_sectors: u64,
    pub smart_attributes: Vec<SmartAttribute>,
}

pub enum HealthStatus {
    Excellent,    // All metrics nominal
    Good,         // Minor wear, no errors
    Warning,      // Elevated wear or minor errors
    Critical,     // High wear or significant errors
    Failed,       // Drive is failing or has failed
    Unknown,      // S.M.A.R.T. not available
}
```

---

## Platform-Specific Detection

### macOS

```rust
// DiskArbitration Framework + IOKit
pub fn detect_devices_macos() -> Vec<StorageDevice> {
    // 1. Use DiskArbitration for real-time monitoring
    // DARegisterDiskAppearedCallback
    // DARegisterDiskDisappearedCallback
    
    // 2. Use IOKit for hardware details
    // IOServiceGetMatchingServices(kIOUSBHostDeviceClassName)
    
    // 3. Use diskutil for partition info
    // diskutil list -plist
    // diskutil info -plist /dev/disk2
    
    // 4. Use system_profiler for USB details
    // system_profiler SPUSBDataType -json
}
```

**Key macOS Commands**:
```bash
# List all disks
diskutil list -plist

# Get disk info
diskutil info -plist /dev/disk2

# USB device details
system_profiler SPUSBDataType -json

# S.M.A.R.T. status (built-in)
diskutil info /dev/disk2 | grep "SMART"

# Unmount before operations
diskutil unmountDisk /dev/disk2
```

### Windows

```rust
// WMI + SetupAPI + DeviceIoControl
pub fn detect_devices_windows() -> Vec<StorageDevice> {
    // 1. WMI for device enumeration
    // SELECT * FROM Win32_DiskDrive WHERE InterfaceType='USB'
    
    // 2. SetupAPI for USB details
    // SetupDiGetDeviceProperty(DEVPKEY_Device_BusReportedDeviceDesc)
    
    // 3. DeviceIoControl for S.M.A.R.T.
    // IOCTL_STORAGE_QUERY_PROPERTY
    // IOCTL_DISK_GET_DRIVE_GEOMETRY_EX
    
    // 4. WMI for real-time monitoring
    // WqlEventQuery on __InstanceCreationEvent
}
```

**Key Windows APIs**:
```
Win32_DiskDrive          → Disk enumeration
Win32_DiskPartition      → Partition info
Win32_LogicalDisk        → Mount points
MSFT_PhysicalDisk        → Modern disk info (PowerShell)
IOCTL_STORAGE_QUERY_PROPERTY → Storage properties
IOCTL_ATA_PASS_THROUGH   → S.M.A.R.T. data
```

### Linux

```rust
// udev + sysfs + ioctl
pub fn detect_devices_linux() -> Vec<StorageDevice> {
    // 1. udev for real-time monitoring
    // udev_monitor_new_from_netlink("udev")
    
    // 2. sysfs for device attributes
    // /sys/block/sdb/device/vendor
    // /sys/block/sdb/device/model
    // /sys/block/sdb/size
    
    // 3. lsblk for partition info
    // lsblk -Jpo NAME,SIZE,TYPE,FSTYPE,LABEL,MOUNTPOINT,UUID
    
    // 4. smartctl for S.M.A.R.T.
    // smartctl -a /dev/sdb
    
    // 5. udevadm for USB details
    // udevadm info --query=all --name=/dev/sdb
}
```

**Key Linux Paths**:
```
/sys/block/sdb/device/vendor        → Device vendor
/sys/block/sdb/device/model         → Device model
/sys/block/sdb/size                 → Size in sectors
/sys/block/sdb/removable            → 1 = removable
/sys/block/sdb/device/type          → Device type
/sys/bus/usb/devices/*/speed        → USB speed (Mbps)
/sys/bus/usb/devices/*/idVendor     → USB vendor ID
/sys/bus/usb/devices/*/idProduct    → USB product ID
```

---

## System Drive Protection

### Critical Safety Rule

> **BootForge MUST NEVER write to system drives.** This is the single most important safety feature.

### Detection Methods

```rust
pub fn is_system_drive(device: &StorageDevice) -> bool {
    // macOS
    if cfg!(target_os = "macos") {
        return device.platform_path == "/dev/disk0"
            || device.platform_path == "/dev/disk1"
            || device.mount_points.iter().any(|m| m.path == "/")
            || device.mount_points.iter().any(|m| m.path.starts_with("/System"));
    }
    
    // Windows
    if cfg!(target_os = "windows") {
        return device.platform_path.contains("PhysicalDrive0")
            || device.mount_points.iter().any(|m| m.path == "C:\\");
    }
    
    // Linux
    if cfg!(target_os = "linux") {
        return device.mount_points.iter().any(|m| m.path == "/")
            || device.mount_points.iter().any(|m| m.path == "/boot")
            || device.mount_points.iter().any(|m| m.path == "/home");
    }
    
    false
}
```

### Additional Safety Checks

| Check | Action |
|-------|--------|
| Drive has system root mounted | **Block** — never allow |
| Drive has boot partition | **Block** — never allow |
| Drive has user home mounted | **Block** — never allow |
| Drive > 2TB and internal | **Warn** — likely not a USB drive |
| Drive is not removable | **Warn** — confirm with user |
| Drive has data but user didn't confirm | **Block** — require explicit confirmation |

---

## Recommendation Engine

### Automatic Configuration Recommendations

```rust
pub fn recommend_configuration(
    device: &StorageDevice,
    isos: &[IsoAnalysis],
) -> DriveRecommendation {
    let total_iso_size: u64 = isos.iter().map(|i| i.file_size).sum();
    let has_windows = isos.iter().any(|i| i.os_info.family == OsFamily::Windows);
    let needs_legacy = isos.iter().any(|i| i.boot_info.bios_capable && !i.boot_info.uefi_capable);
    let needs_uefi = isos.iter().any(|i| i.boot_info.uefi_capable);
    let needs_secure_boot = isos.iter().any(|i| i.boot_info.secure_boot == SecureBootStatus::Supported);
    let needs_persistence = isos.iter().any(|i| i.supports_persistence);
    let any_iso_over_4gb = isos.iter().any(|i| i.file_size > 4 * 1024 * 1024 * 1024);
    
    DriveRecommendation {
        partition_scheme: if needs_uefi && needs_legacy {
            PartitionScheme::GptUefiPlusBios
        } else if needs_uefi {
            PartitionScheme::GptUefiOnly
        } else {
            PartitionScheme::MbrBiosOnly
        },
        
        data_filesystem: if any_iso_over_4gb || has_windows {
            Filesystem::ExFat  // No 4GB file size limit
        } else {
            Filesystem::ExFat  // Default to exFAT for compatibility
        },
        
        efi_partition_size: 512 * 1024 * 1024,  // 512MB
        
        persistence_size: if needs_persistence {
            Some(4 * 1024 * 1024 * 1024)  // 4GB default
        } else {
            None
        },
        
        secure_boot: needs_secure_boot,
        
        estimated_total: total_iso_size + 512 * 1024 * 1024
            + if needs_persistence { 4 * 1024 * 1024 * 1024 } else { 0 },
        
        fits_on_drive: device.capacity > total_iso_size + 1024 * 1024 * 1024, // + 1GB headroom
    }
}
```

---

## USB Speed Classes

### Speed Detection and Display

| Interface | Theoretical | Typical Real | Display |
|-----------|-------------|--------------|---------|
| USB 2.0 | 480 Mbps (60 MB/s) | 25-40 MB/s | USB 2.0 |
| USB 3.0 | 5 Gbps (625 MB/s) | 100-400 MB/s | USB 3.0 |
| USB 3.1 Gen 2 | 10 Gbps (1.25 GB/s) | 500-1000 MB/s | USB 3.1 |
| USB 3.2 Gen 2x2 | 20 Gbps (2.5 GB/s) | 1-2 GB/s | USB 3.2 |
| USB4 | 40 Gbps (5 GB/s) | 2-3 GB/s | USB4 |
| Thunderbolt 3/4 | 40 Gbps (5 GB/s) | 2-3 GB/s | TB3/TB4 |

### Speed Benchmark (Optional)

```rust
pub struct SpeedBenchmark {
    pub sequential_read: u64,   // Bytes/sec
    pub sequential_write: u64,  // Bytes/sec
    pub random_read_4k: u64,    // IOPS
    pub random_write_4k: u64,   // IOPS
    pub duration: Duration,
}
```

> **Note**: Speed benchmark is optional and non-destructive (reads only by default). Write benchmark requires explicit user consent and uses a temporary file.

---

## Real-Time Monitoring

### Device Events

```rust
pub enum DeviceEvent {
    Connected(StorageDevice),
    Disconnected(DeviceId),
    MountChanged(DeviceId, Vec<MountPoint>),
    HealthChanged(DeviceId, DeviceHealth),
    WriteProgress(DeviceId, WriteProgress),
}

pub struct WriteProgress {
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub current_file: String,
    pub speed: u64,           // Bytes/sec
    pub eta: Duration,
    pub phase: WritePhase,
}

pub enum WritePhase {
    Partitioning,
    FormattingEfi,
    FormattingData,
    InstallingBootloader,
    CopyingImages,
    WritingConfig,
    Verifying,
    Complete,
}
```

---

## Display Format

### Device Card (UI)

```
┌─────────────────────────────────────────────────┐
│  🔌 USB Device                                   │
│                                                   │
│  SanDisk Extreme Pro                              │
│  256 GB Portable SSD                              │
│                                                   │
│  Interface:  USB 3.2 Gen 2 (10 Gbps)             │
│  Speed:      Read ~1050 MB/s | Write ~1000 MB/s  │
│  Health:     ● Excellent (Wear: 2%)               │
│  Temp:       38°C                                  │
│                                                   │
│  Current Partitions:                               │
│  ├─ Part 1: EFI (FAT32, 512 MB)                  │
│  ├─ Part 2: BOOTFORGE (exFAT, 240 GB)            │
│  └─ Part 3: Persistence (ext4, 4 GB)             │
│                                                   │
│  Available: 238 GB                                 │
│                                                   │
│  [ CONFIGURE ]  [ FORMAT ]  [ EJECT ]             │
└─────────────────────────────────────────────────┘
```
