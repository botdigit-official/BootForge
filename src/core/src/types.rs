//! Shared type definitions for BootForge.
//!
//! These types are used across all BootForge components to ensure
//! consistent data representation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Storage Device Types ──────────────────────────────────────────

/// Represents a detected USB storage device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Unique identifier for this device session.
    pub id: String,
    /// Device vendor/manufacturer.
    pub vendor: String,
    /// Device model name.
    pub model: String,
    /// Device serial number.
    pub serial: String,
    /// Firmware version (if available).
    pub firmware_version: Option<String>,
    /// Total capacity in bytes.
    pub capacity: u64,
    /// Available (free) space in bytes.
    pub available: u64,
    /// USB interface type.
    pub interface: UsbInterface,
    /// Device classification.
    pub device_type: DeviceType,
    /// Whether the device is removable.
    pub removable: bool,
    /// Whether the device is read-only.
    pub read_only: bool,
    /// Health status (if S.M.A.R.T. data is available).
    pub health: Option<DeviceHealth>,
    /// Current partitions on the device.
    pub partitions: Vec<Partition>,
    /// Whether this is a system/boot drive (NEVER touch).
    pub is_system_drive: bool,
    /// Platform-specific device path (e.g., /dev/disk2).
    pub platform_path: String,
}

/// USB interface speed classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsbInterface {
    Usb20,
    Usb30,
    Usb31Gen2,
    Usb32Gen2x2,
    Usb40,
    Thunderbolt3,
    Thunderbolt4,
    Unknown,
}

impl std::fmt::Display for UsbInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usb20 => write!(f, "USB 2.0"),
            Self::Usb30 => write!(f, "USB 3.0"),
            Self::Usb31Gen2 => write!(f, "USB 3.1 Gen 2"),
            Self::Usb32Gen2x2 => write!(f, "USB 3.2 Gen 2x2"),
            Self::Usb40 => write!(f, "USB 4.0"),
            Self::Thunderbolt3 => write!(f, "Thunderbolt 3"),
            Self::Thunderbolt4 => write!(f, "Thunderbolt 4"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Device type classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    UsbFlashDrive,
    PortableSsd,
    ExternalHdd,
    SdCard,
    NvmeEnclosure,
    Unknown,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UsbFlashDrive => write!(f, "USB Flash Drive"),
            Self::PortableSsd => write!(f, "Portable SSD"),
            Self::ExternalHdd => write!(f, "External HDD"),
            Self::SdCard => write!(f, "SD Card"),
            Self::NvmeEnclosure => write!(f, "NVMe Enclosure"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Device health information from S.M.A.R.T. data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHealth {
    pub status: HealthStatus,
    pub wear_percentage: Option<u8>,
    pub power_on_hours: Option<u64>,
    pub error_count: u64,
    pub temperature_celsius: Option<f32>,
}

/// Overall health status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Warning,
    Critical,
    Failed,
    Unknown,
}

/// A partition on a storage device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub number: u32,
    pub label: Option<String>,
    pub filesystem: Option<String>,
    pub size: u64,
    pub used: Option<u64>,
    pub mount_point: Option<PathBuf>,
    pub uuid: Option<String>,
}

// ── ISO Analysis Types ────────────────────────────────────────────

/// Complete analysis result for a disk image file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoAnalysis {
    /// Unique ID for this analysis.
    pub id: String,
    /// Path to the ISO file.
    pub file_path: PathBuf,
    /// File name (for display).
    pub file_name: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Image format detected.
    pub format: ImageFormat,
    /// Hash verification result.
    pub hash: HashResult,
    /// Operating system information.
    pub os_info: OsInfo,
    /// Boot capability information.
    pub boot_info: BootInfo,
    /// Detected architecture(s).
    pub architecture: Architecture,
    /// Compatibility assessment.
    pub compatibility: CompatibilityScore,
    /// Known quirks from the compatibility database.
    pub quirks: Vec<String>,
    /// Recommended boot strategy.
    pub boot_strategy: BootStrategy,
    /// Whether persistence is supported.
    pub supports_persistence: bool,
    /// Analysis timestamp.
    pub analyzed_at: DateTime<Utc>,
    /// Any warnings generated during analysis.
    pub warnings: Vec<String>,
}

/// Disk image format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageFormat {
    Iso9660,
    Udf,
    HybridIso,
    Wim,
    Esd,
    RawImg,
    Vhd,
    Vhdx,
    Vmdk,
    Qcow2,
    Unknown,
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iso9660 => write!(f, "ISO 9660"),
            Self::Udf => write!(f, "UDF"),
            Self::HybridIso => write!(f, "Hybrid ISO"),
            Self::Wim => write!(f, "WIM"),
            Self::Esd => write!(f, "ESD"),
            Self::RawImg => write!(f, "Raw Image"),
            Self::Vhd => write!(f, "VHD"),
            Self::Vhdx => write!(f, "VHDX"),
            Self::Vmdk => write!(f, "VMDK"),
            Self::Qcow2 => write!(f, "QCOW2"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Hash verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashResult {
    pub sha256: String,
    pub sha512: Option<String>,
    pub md5: Option<String>,
    pub status: VerificationStatus,
}

/// Hash verification status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    /// Hash matches a known good value.
    Verified,
    /// Hash computed but no known value to compare.
    Unverified,
    /// Hash does NOT match — file may be corrupted.
    Mismatch,
    /// Hash computation in progress.
    Pending,
}

/// Operating system information extracted from an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub family: OsFamily,
    pub distribution: Option<String>,
    pub version: Option<String>,
    pub codename: Option<String>,
    pub image_type: ImageType,
    pub display_name: String,
}

/// OS family classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Type of disk image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageType {
    Installer,
    Live,
    Rescue,
    WindowsPe,
    NetInstall,
    Firmware,
    Utility,
    Unknown,
}

/// Boot capability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootInfo {
    pub uefi_capable: bool,
    pub bios_capable: bool,
    pub secure_boot: SecureBootStatus,
    pub efi_architectures: Vec<String>,
}

/// Secure Boot support status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecureBootStatus {
    Supported,
    NotSupported,
    Partial,
    Unknown,
}

/// CPU architecture of the OS image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Architecture {
    X86,
    X86_64,
    Arm64,
    Ia32,
    Multi(Vec<String>),
    Unknown,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X86 => write!(f, "x86"),
            Self::X86_64 => write!(f, "x86_64"),
            Self::Arm64 => write!(f, "ARM64"),
            Self::Ia32 => write!(f, "IA-32"),
            Self::Multi(archs) => write!(f, "Multi ({})", archs.join(", ")),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Compatibility score for an ISO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityScore {
    pub score: u32,
    pub level: CompatLevel,
    pub details: Vec<String>,
}

/// Compatibility level classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompatLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Untested,
    Incompatible,
}

impl std::fmt::Display for CompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Excellent => write!(f, "EXCELLENT"),
            Self::Good => write!(f, "GOOD"),
            Self::Fair => write!(f, "FAIR"),
            Self::Poor => write!(f, "POOR"),
            Self::Untested => write!(f, "UNTESTED"),
            Self::Incompatible => write!(f, "INCOMPATIBLE"),
        }
    }
}

// ── Boot Configuration Types ──────────────────────────────────────

/// Recommended boot strategy for an ISO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootStrategy {
    /// Standard GRUB2 loopback mount.
    GrubLoopback,
    /// Windows WIM boot method.
    WindowsWim,
    /// WIMBOOT for PE-based images.
    WimBoot,
    /// EFI chainload.
    EfiChainload,
    /// MEMDISK emulation (legacy).
    Memdisk,
    /// Direct Linux kernel boot.
    DirectLinux,
    /// Unknown — needs testing.
    Unknown,
}

/// Partition scheme for the boot drive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartitionScheme {
    GptUefiOnly,
    GptUefiPlusBios,
    MbrBiosOnly,
    MbrBiosWithEfi,
}

/// Filesystem type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Filesystem {
    Fat32,
    ExFat,
    Ntfs,
    Ext4,
    Btrfs,
    Xfs,
}

// ── Boot Category (for UI grouping) ───────────────────────────────

/// Category for organizing boot entries in the boot menu.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootCategory {
    Windows,
    Linux,
    Bsd,
    Recovery,
    Utility,
    Firmware,
    Custom,
}

impl std::fmt::Display for BootCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows"),
            Self::Linux => write!(f, "Linux"),
            Self::Bsd => write!(f, "BSD"),
            Self::Recovery => write!(f, "Recovery"),
            Self::Utility => write!(f, "Utilities"),
            Self::Firmware => write!(f, "Firmware"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

// ── Progress Types ────────────────────────────────────────────────

/// Progress update for long-running operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub phase: String,
    pub percent: f32,
    pub current_item: Option<String>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub bytes_written: u64,
    pub total_bytes: u64,
}

/// Phases of the drive creation process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WritePhase {
    Unmounting,
    Partitioning,
    Formatting,
    InstallingBootloader,
    CopyingImages,
    WritingConfig,
    Verifying,
    Complete,
    Failed,
}

impl std::fmt::Display for WritePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unmounting => write!(f, "Unmounting volumes"),
            Self::Partitioning => write!(f, "Creating partitions"),
            Self::Formatting => write!(f, "Formatting"),
            Self::InstallingBootloader => write!(f, "Installing bootloader"),
            Self::CopyingImages => write!(f, "Copying images"),
            Self::WritingConfig => write!(f, "Writing configuration"),
            Self::Verifying => write!(f, "Verifying"),
            Self::Complete => write!(f, "Complete"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}
