//! Storage Partition Management and Layout module for BootForge.
//! Implements maximum-compatibility GPT + BIOS_BOOT (2 MiB) + FAT32 ESP + Data partition schemes.

use bootforge_core::types::PartitionScheme;

/// Represents a planned partition table structure before execution.
#[derive(Debug, Clone)]
pub struct PlannedPartitionLayout {
    pub scheme: PartitionScheme,
    pub bios_boot_size_mb: u64, // 2 MiB dedicated BIOS boot code area
    pub efi_size_mb: u64,       // 512 MiB FAT32 ESP for UEFI bootloaders & themes
    pub efi_fs: String,
    pub efi_label: String,
    pub data_label: String,
    pub data_fs: String,
    pub align_4k: bool,
}

impl Default for PlannedPartitionLayout {
    fn default() -> Self {
        Self {
            scheme: PartitionScheme::GptUefiPlusBios,
            bios_boot_size_mb: 2, // 2 MiB BIOS boot partition for GPT
            efi_size_mb: 512,     // 512 MiB ESP for UEFI 2.11 compliant FAT32
            efi_fs: "FAT32".to_string(),
            efi_label: "BOOTFORGE_EFI".to_string(),
            data_label: "BOOTFORGE".to_string(),
            data_fs: "ExFAT".to_string(),
            align_4k: true,
        }
    }
}

/// Calculated sector bounds for all planned partitions on the target drive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionSectorMap {
    pub bios_boot_sectors: u64,
    pub efi_sectors: u64,
    pub data_sectors: u64,
    pub total_sectors: u64,
    pub sector_size: u64,
}

/// Calculate partition offsets and sector boundaries for a given drive capacity.
pub fn calculate_partition_boundaries(
    capacity_bytes: u64,
    layout: &PlannedPartitionLayout,
) -> PartitionSectorMap {
    let sector_size = 512u64;
    let bios_boot_sectors = (layout.bios_boot_size_mb * 1024 * 1024) / sector_size;
    let efi_sectors = (layout.efi_size_mb * 1024 * 1024) / sector_size;
    let total_sectors = capacity_bytes / sector_size;
    let overhead_sectors = 2048 + bios_boot_sectors + efi_sectors + 34; // GPT headers + alignment
    let data_sectors = total_sectors.saturating_sub(overhead_sectors);

    PartitionSectorMap {
        bios_boot_sectors,
        efi_sectors,
        data_sectors,
        total_sectors,
        sector_size,
    }
}
