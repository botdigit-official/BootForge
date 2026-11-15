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

// ── ISO Analysis & Intelligence Types ─────────────────────────────

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
    /// Will this image boot on each class of computer, and how.
    #[serde(default)]
    pub device_guidance: Vec<TargetVerdict>,
}

// ── Target Device Guidance ────────────────────────────────────────

/// A class of computer, grouped by how its firmware boots external media.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceClass {
    /// 32-bit CPU, Legacy BIOS (Pentium 4/M, Atom N270 netbooks, ~2000–2008).
    LegacyBios32,
    /// 64-bit CPU, Legacy BIOS or UEFI with CSM (Core 2 era ThinkPads, desktops ~2006–2012).
    LegacyBios64,
    /// UEFI PC with Secure Boot off or switchable (most PCs 2012+).
    UefiPc,
    /// UEFI PC with Secure Boot enforced (Windows 11 era PCs, 2020+, no CSM).
    UefiSecureBoot,
    /// 64-bit CPU but 32-bit UEFI firmware (Bay Trail / Cherry Trail tablets and mini PCs).
    Uefi32,
    /// Intel Mac without T2 chip (2006–2017 models).
    IntelMac,
    /// Intel Mac with T2 security chip (2018–2020 models).
    IntelMacT2,
    /// Apple Silicon Mac (M1 and later).
    AppleSilicon,
    /// ARM64 UEFI machine (Snapdragon Windows laptops, ARM servers, Raspberry Pi with UEFI firmware).
    Arm64Uefi,
}

/// How well an image fits a device class.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BootVerdict {
    /// Boots directly from the BootForge drive.
    Works,
    /// Boots after the listed steps (firmware settings, key enrollment, separate drive...).
    WorksWithSteps,
    /// Cannot boot on this class of hardware.
    NotSupported,
}

impl std::fmt::Display for BootVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Works => write!(f, "Works"),
            Self::WorksWithSteps => write!(f, "Works with steps"),
            Self::NotSupported => write!(f, "Not supported"),
        }
    }
}

/// Guidance for one image on one device class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetVerdict {
    pub device: DeviceClass,
    pub device_name: String,
    pub verdict: BootVerdict,
    /// One-line explanation.
    pub summary: String,
    /// Ordered steps the user follows on that machine.
    pub steps: Vec<String>,
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
    pub uefi_x64_score: u8,
    pub bios_score: u8,
    pub secure_boot_score: u8,
    pub arm64_score: u8,
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
    AppleRecovery,
    Bsd,
    Specialist,
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
            Self::AppleRecovery => write!(f, "Apple Recovery"),
            Self::Bsd => write!(f, "BSD"),
            Self::Specialist => write!(f, "Specialist"),
            Self::Recovery => write!(f, "Recovery"),
            Self::Utility => write!(f, "Utilities"),
            Self::Firmware => write!(f, "Firmware"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

// ── Recovery Center Types ─────────────────────────────────────────

/// A categorized tool in the Recovery Suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryToolItem {
    pub id: String,
    pub name: String,
    pub category: RecoveryCategory,
    pub icon: String,
    pub description: String,
    pub size_estimate: String,
    pub default_filename: String,
    pub is_available: bool,
    pub download_url: Option<String>,
}

/// Recovery categories in the suite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryCategory {
    DiskRepair,
    MemoryDiagnostics,
    AccountPassword,
    DataRecovery,
    MalwareRescue,
    HardwareTesting,
}

impl std::fmt::Display for RecoveryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DiskRepair => write!(f, "Disk Repair"),
            Self::MemoryDiagnostics => write!(f, "Memory"),
            Self::AccountPassword => write!(f, "Password & Account"),
            Self::DataRecovery => write!(f, "Data Recovery"),
            Self::MalwareRescue => write!(f, "Malware Rescue"),
            Self::HardwareTesting => write!(f, "Hardware Diagnostics"),
        }
    }
}

// ── Hardware Telemetry Types ──────────────────────────────────────

/// System hardware telemetry snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTelemetrySnapshot {
    pub cpu_model: String,
    pub cpu_architecture: String,
    pub cpu_cores: usize,
    pub memory_gb: u64,
    pub memory_type: String,
    pub firmware_type: String, // "UEFI x86_64", "Apple Silicon EFI", "Legacy BIOS"
    pub secure_boot_active: bool,
    pub tpm_version: String, // "2.0 (Detected)", "Not Detected"
    pub display_resolution: String,
    pub storage_devices: Vec<String>,
    pub network_interfaces: Vec<String>,
}

// ── USB File Manager Types ────────────────────────────────────────

/// A file or directory inside the USB storage partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbFileNode {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub extension: String,
    pub is_bootable_iso: bool,
    pub category: Option<String>,
    pub sha256_hash: Option<String>,
    pub children: Vec<UsbFileNode>,
}

// ── Boot Repair Types ─────────────────────────────────────────────

/// Results of a 1-click non-destructive boot repair operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootRepairResult {
    pub success: bool,
    pub disk_id: String,
    pub mount_point: String,
    pub efi_restored: bool,
    pub grub_restored: bool,
    pub theme_restored: bool,
    pub isos_preserved_count: usize,
    pub message: String,
    pub logs: Vec<String>,
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

// ── Persistent Device Identity ────────────────────────────────────

/// Cryptographically/structurally pinned identity of a physical storage drive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceIdentity {
    pub os_path: String,
    pub vendor: String,
    pub model: String,
    pub serial_number: String,
    pub bus_type: UsbInterface,
    pub total_capacity_bytes: u64,
    pub partition_table_uuid: Option<String>,
    pub is_removable: bool,
    pub is_internal_system: bool,
}

// ── Safety & Transaction Engine Types ─────────────────────────────

/// User-selected safety posture for block-level operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SafetyLevel {
    Normal,
    Safe,
    Expert,
}

/// Transaction plan with verified preconditions, operations, and rollback data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPlan {
    pub id: String,
    pub target_device: DeviceIdentity,
    pub safety_level: SafetyLevel,
    pub created_at: DateTime<Utc>,
    pub pre_checks_passed: bool,
    pub operations: Vec<String>,
    pub metadata_backup_path: Option<PathBuf>,
    pub status: TransactionStatus,
}

/// Status of an active or past disk transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    PendingConfirmation,
    Executing,
    Verifying,
    Committed,
    RolledBack,
    Failed(String),
}

// ── OS Doctor & Host System Recovery Types ─────────────────────────

/// Operating system installed on host machine's internal disks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOsEntry {
    pub os_type: HostOsType,
    pub name: String,
    pub partition_id: String,
    pub mount_point: Option<String>,
    pub kernel_path: Option<String>,
    pub efi_entry_present: bool,
    pub bootloader_status: BootloaderStatus,
    pub is_bootable: bool,
}

/// Host operating system family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HostOsType {
    Windows,
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    GenericLinux,
    MacOs,
    Unknown,
}

/// Status of host bootloader integrity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootloaderStatus {
    Healthy,
    MissingEfiEntry,
    CorruptedBcd,
    BrokenGrubConfig,
    MissingKernel,
    Unknown,
}

/// Automated diagnostic evaluation by BootForge OS Doctor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsDoctorDiagnosis {
    pub scan_timestamp: DateTime<Utc>,
    pub detected_operating_systems: Vec<HostOsEntry>,
    pub issues_found: Vec<String>,
    pub proposed_repairs: Vec<String>,
    pub requires_user_confirmation: bool,
    pub can_auto_repair: bool,
}

// ── Boot Reliability & Diagnostic Stage Types ─────────────────────

/// Observable startup stages for diagnosing bootloader and kernel handoffs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BootStage {
    /// BF01: Early CPU mode transition and entry point
    Bf01KernelEntry,
    /// BF02: Physical memory map and page table setup
    Bf02MemoryMap,
    /// BF03: Console initialization (Serial / Text / GOP Framebuffer)
    Bf03ConsoleInit,
    /// BF04: Storage controller and bus device discovery
    Bf04StorageDiscovery,
    /// BF05: Initramfs unpacking and root pivot
    Bf05Initramfs,
    /// BF06: Standalone recovery userspace daemon active
    Bf06RecoveryUserspace,
    /// BF07: Interactive UI ready (Text / Framebuffer GUI)
    Bf07UiReady,
}

impl std::fmt::Display for BootStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bf01KernelEntry => write!(f, "[BF01] Kernel Entry Point"),
            Self::Bf02MemoryMap => write!(f, "[BF02] Memory & Page Tables"),
            Self::Bf03ConsoleInit => write!(f, "[BF03] Console (Serial/GOP/Text)"),
            Self::Bf04StorageDiscovery => write!(f, "[BF04] Storage Device Discovery"),
            Self::Bf05Initramfs => write!(f, "[BF05] Initramfs Environment"),
            Self::Bf06RecoveryUserspace => write!(f, "[BF06] Recovery Daemons"),
            Self::Bf07UiReady => write!(f, "[BF07] Interactive UI Ready"),
        }
    }
}

/// Target boot profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootProfile {
    /// Maximum compatibility: GPT + 2MiB BIOS_BOOT + FAT32 ESP + Data partition
    MaximumCompatibility,
    /// Modern UEFI-only GPT (FAT32 ESP + Data)
    ModernUefiOnly,
    /// Legacy MBR / BIOS only
    LegacyBiosOnly,
    /// Standalone Recovery OS only
    RecoveryOnly,
    /// Custom layout
    Custom,
}

/// Active console rendering mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsoleMode {
    SerialDebug,
    TextMode80x25,
    GopFramebuffer(FramebufferInfo),
}

/// Discovered UEFI GOP / VBE framebuffer parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FramebufferInfo {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub red_mask_size: u8,
    pub green_mask_size: u8,
    pub blue_mask_size: u8,
    pub base_address: u64,
}

/// Structured empirical boot test result captured from QEMU/hardware test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTestResult {
    pub image_fingerprint: String,
    pub firmware: String, // "UEFI 2.11", "Legacy BIOS (SeaBIOS)"
    pub architecture: Architecture,
    pub secure_boot_tested: bool,
    pub boot_success: bool,
    pub boot_time_ms: u64,
    pub last_completed_stage: BootStage,
    pub detected_loader: String,
    pub error: Option<String>,
}


