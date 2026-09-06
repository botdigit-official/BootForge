//! Bootloader, El Torito, UEFI, BIOS, and Secure Boot detection module.

use bootforge_core::types::{BootInfo, SecureBootStatus};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// El Torito Platform ID enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElToritoPlatform {
    X86Bios = 0x00,
    PowerPc = 0x01,
    Mac = 0x02,
    Efi = 0xEF,
    Unknown = 0xFF,
}

/// Information extracted from El Torito boot catalog
#[derive(Debug, Clone)]
pub struct ElToritoCatalog {
    pub validation_header_ok: bool,
    pub platform: ElToritoPlatform,
    pub has_bios_boot: bool,
    pub has_efi_boot: bool,
    pub boot_catalog_sector: u32,
    pub initial_entry_lba: u32,
}

/// Analyze boot structures and bootloaders present inside an ISO image.
pub fn detect_boot_capabilities(path: &Path, file_name: &str) -> BootInfo {
    let lower_name = file_name.to_lowercase();
    let mut uefi_capable = false;
    let mut bios_capable = false;
    let mut secure_boot = SecureBootStatus::NotSupported;
    let mut efi_architectures = Vec::new();

    // 1. Inspect El Torito boot catalog and ISO sectors
    if let Ok(file) = File::open(path) {
        if let Ok(Some(catalog)) = parse_el_torito_catalog(file) {
            if catalog.has_efi_boot {
                uefi_capable = true;
                efi_architectures.push("x86_64 (UEFI)".to_string());
            }
            if catalog.has_bios_boot {
                bios_capable = true;
            }
        }
    }

    // 2. Scan file content samples for EFI and Boot signatures
    if let Ok(mut file) = File::open(path) {
        let mut sample = vec![0u8; 1024 * 1024.min(file.metadata().map(|m| m.len()).unwrap_or(0) as usize)];
        if file.read_exact(&mut sample).is_ok() {
            // Check for EFI signatures
            if contains_subslice(&sample, b"EFI/BOOT/BOOTX64.EFI")
                || contains_subslice(&sample, b"efi/boot/bootx64.efi")
                || contains_subslice(&sample, b"BOOTX64.EFI")
            {
                uefi_capable = true;
                if !efi_architectures.iter().any(|a| a.contains("x86_64")) {
                    efi_architectures.push("x86_64 (UEFI)".to_string());
                }
            }

            if contains_subslice(&sample, b"BOOTAA64.EFI") || contains_subslice(&sample, b"bootaa64.efi") {
                uefi_capable = true;
                if !efi_architectures.iter().any(|a| a.contains("ARM64")) {
                    efi_architectures.push("ARM64 (UEFI)".to_string());
                }
            }

            if contains_subslice(&sample, b"BOOTIA32.EFI") || contains_subslice(&sample, b"bootia32.efi") {
                uefi_capable = true;
                if !efi_architectures.iter().any(|a| a.contains("IA-32")) {
                    efi_architectures.push("IA-32 (UEFI 32-bit)".to_string());
                }
            }

            // Check for Secure Boot Shim signatures
            if contains_subslice(&sample, b"shimx64.efi")
                || contains_subslice(&sample, b"SHIMX64.EFI")
                || contains_subslice(&sample, b"Microsoft Corporation UEFI CA")
                || contains_subslice(&sample, b"MokManager")
            {
                secure_boot = SecureBootStatus::Supported;
            }

            // Check for ISOLINUX / SYSLINUX / GRUB BIOS bootloaders
            if contains_subslice(&sample, b"ISOLINUX")
                || contains_subslice(&sample, b"isolinux.bin")
                || contains_subslice(&sample, b"syslinux")
                || contains_subslice(&sample, b"grub.cfg")
                || contains_subslice(&sample, b"GRUB")
            {
                bios_capable = true;
            }
        }
    }

    // 3. Apply Heuristics based on standard OS distributions if needed
    if !uefi_capable {
        // Modern OS distributions are almost all UEFI capable
        if lower_name.contains("ubuntu")
            || lower_name.contains("debian")
            || lower_name.contains("mint")
            || lower_name.contains("fedora")
            || lower_name.contains("arch")
            || lower_name.contains("win11")
            || lower_name.contains("win10")
            || lower_name.contains("windows")
            || lower_name.contains("alpine")
            || lower_name.contains("clonezilla")
            || lower_name.contains("gparted")
        {
            uefi_capable = true;
            if efi_architectures.is_empty() {
                efi_architectures.push("x86_64 (UEFI)".to_string());
            }
        }
    }

    if !bios_capable {
        if lower_name.contains("ubuntu")
            || lower_name.contains("debian")
            || lower_name.contains("mint")
            || lower_name.contains("fedora")
            || lower_name.contains("arch")
            || lower_name.contains("win10")
            || lower_name.contains("alpine")
            || lower_name.contains("clonezilla")
            || lower_name.contains("gparted")
            || lower_name.contains("memtest")
        {
            bios_capable = true;
        }
    }

    if secure_boot == SecureBootStatus::NotSupported {
        if lower_name.contains("win11")
            || lower_name.contains("win10")
            || lower_name.contains("windows")
            || lower_name.contains("ubuntu")
            || lower_name.contains("fedora")
            || lower_name.contains("debian")
            || lower_name.contains("mint")
        {
            secure_boot = SecureBootStatus::Supported;
        }
    }

    if efi_architectures.is_empty() {
        if lower_name.contains("arm64") || lower_name.contains("aarch64") {
            efi_architectures.push("ARM64 (UEFI)".to_string());
        } else {
            efi_architectures.push("x86_64 (UEFI)".to_string());
        }
    }

    BootInfo {
        uefi_capable,
        bios_capable,
        secure_boot,
        efi_architectures,
    }
}

/// Parse El Torito Boot Record and Catalog from an ISO 9660 file.
pub fn parse_el_torito_catalog(mut file: File) -> io::Result<Option<ElToritoCatalog>> {
    let file_size = file.metadata()?.len();
    if file_size < 32768 + 2048 {
        return Ok(None);
    }

    // El Torito Boot Record Volume Descriptor is typically at sector 17 (0x8800)
    for sector in 16..20 {
        let offset = sector * 2048;
        if file.seek(SeekFrom::Start(offset)).is_err() {
            break;
        }

        let mut desc = [0u8; 2048];
        if file.read_exact(&mut desc).is_err() {
            break;
        }

        if desc[0] == 0x00 && &desc[1..6] == b"CD001" {
            let boot_system = String::from_utf8_lossy(&desc[7..39]);
            if boot_system.to_uppercase().contains("EL TORITO") {
                // Boot catalog pointer is at bytes 0x47-0x4A (71-74)
                let catalog_sector = u32::from_le_bytes([desc[71], desc[72], desc[73], desc[74]]);
                if catalog_sector == 0 || (catalog_sector as u64 * 2048) >= file_size {
                    continue;
                }

                // Seek to boot catalog
                file.seek(SeekFrom::Start(catalog_sector as u64 * 2048))?;
                let mut cat_bytes = [0u8; 2048];
                if file.read_exact(&mut cat_bytes).is_err() {
                    continue;
                }

                // Validation Header: Byte 0 is 0x01, Byte 1 is Platform ID, Bytes 30-31 is 0x55AA
                let validation_header_ok = cat_bytes[0] == 0x01 && cat_bytes[30] == 0x55 && cat_bytes[31] == 0xAA;
                let platform_raw = cat_bytes[1];
                let platform = match platform_raw {
                    0x00 => ElToritoPlatform::X86Bios,
                    0x01 => ElToritoPlatform::PowerPc,
                    0x02 => ElToritoPlatform::Mac,
                    0xEF => ElToritoPlatform::Efi,
                    _ => ElToritoPlatform::Unknown,
                };

                // Initial/Default Entry starts at offset 32
                let boot_indicator = cat_bytes[32]; // 0x88 = bootable, 0x00 = not bootable
                let has_bios_boot = boot_indicator == 0x88 || platform == ElToritoPlatform::X86Bios;
                let initial_entry_lba = u32::from_le_bytes([cat_bytes[40], cat_bytes[41], cat_bytes[42], cat_bytes[43]]);

                // Check Section Headers (starting at offset 64) for EFI platform 0xEF
                let mut has_efi_boot = platform == ElToritoPlatform::Efi;
                for entry_offset in (64..cat_bytes.len()).step_by(32) {
                    if entry_offset + 32 > cat_bytes.len() {
                        break;
                    }
                    let header_id = cat_bytes[entry_offset];
                    if header_id == 0x90 || header_id == 0x91 {
                        // Section Header
                        let section_platform = cat_bytes[entry_offset + 1];
                        if section_platform == 0xEF {
                            has_efi_boot = true;
                        }
                    }
                }

                return Ok(Some(ElToritoCatalog {
                    validation_header_ok,
                    platform,
                    has_bios_boot,
                    has_efi_boot,
                    boot_catalog_sector: catalog_sector,
                    initial_entry_lba,
                }));
            }
        }
    }

    Ok(None)
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
