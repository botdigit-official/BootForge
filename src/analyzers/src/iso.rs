//! Deep ISO 9660, UDF, and Hybrid disk image parser.

use bootforge_core::types::ImageFormat;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// ISO 9660 Primary Volume Descriptor summary.
#[derive(Debug, Clone)]
pub struct IsoDescriptorInfo {
    pub system_id: String,
    pub volume_id: String,
    pub publisher_id: String,
    pub data_preparer_id: String,
    pub application_id: String,
    pub volume_size_bytes: u64,
    pub logical_block_size: u32,
    pub is_hybrid: bool,
    pub has_joliet: bool,
    pub has_el_torito: bool,
}

/// Parse ISO 9660 headers and descriptors.
pub fn inspect_iso_descriptors(path: &Path) -> io::Result<Option<IsoDescriptorInfo>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    // Check minimum size for ISO sector 16 (16 * 2048 = 32,768 bytes)
    if file_size < 32768 + 2048 {
        return Ok(None);
    }

    // 1. Check for Hybrid MBR signature at sector 0
    let mut mbr = [0u8; 512];
    let is_hybrid = if file.read_exact(&mut mbr).is_ok() && mbr[510] == 0x55 && mbr[511] == 0xAA {
        // Has valid MBR signature
        let mut has_part = false;
        for i in 0..4 {
            let offset = 446 + (i * 16);
            let part_type = mbr[offset + 4];
            if part_type != 0 {
                has_part = true;
                break;
            }
        }
        has_part
    } else {
        false
    };

    // 2. Read Volume Descriptors starting at sector 16 (0x8000)
    let mut system_id = String::new();
    let mut volume_id = String::new();
    let mut publisher_id = String::new();
    let mut data_preparer_id = String::new();
    let mut application_id = String::new();
    let mut volume_size_bytes = 0u64;
    let mut logical_block_size = 2048u32;
    let mut found_pvd = false;
    let mut has_joliet = false;
    let mut has_el_torito = false;

    for sector in 16..32 {
        let offset = sector * 2048;
        if offset + 2048 > file_size {
            break;
        }

        if file.seek(SeekFrom::Start(offset)).is_err() {
            break;
        }

        let mut desc = [0u8; 2048];
        if file.read_exact(&mut desc).is_err() {
            break;
        }

        let desc_type = desc[0];
        let magic = &desc[1..6];

        if magic != b"CD001" {
            // Check for UDF or non-standard descriptor
            if magic == b"BEA01" || magic == b"NSR02" || magic == b"NSR03" {
                // UDF descriptor found
                continue;
            }
            break;
        }

        match desc_type {
            0x00 => {
                // Boot Record Descriptor (El Torito)
                let boot_sys_id = String::from_utf8_lossy(&desc[7..39]).trim().to_string();
                if boot_sys_id.to_uppercase().contains("EL TORITO") {
                    has_el_torito = true;
                }
            }
            0x01 => {
                // Primary Volume Descriptor (PVD)
                found_pvd = true;
                system_id = String::from_utf8_lossy(&desc[8..40]).trim().to_string();
                volume_id = String::from_utf8_lossy(&desc[40..72]).trim().to_string();
                publisher_id = String::from_utf8_lossy(&desc[318..446]).trim().to_string();
                data_preparer_id = String::from_utf8_lossy(&desc[446..574]).trim().to_string();
                application_id = String::from_utf8_lossy(&desc[574..702]).trim().to_string();

                let vol_space_sectors = u32::from_le_bytes([desc[80], desc[81], desc[82], desc[83]]) as u64;
                let block_size = u16::from_le_bytes([desc[128], desc[129]]) as u32;
                logical_block_size = if block_size > 0 { block_size } else { 2048 };
                volume_size_bytes = vol_space_sectors * logical_block_size as u64;
            }
            0x02 => {
                // Supplementary Volume Descriptor (Joliet / UTF-16)
                has_joliet = true;
                if volume_id.is_empty() {
                    // Try decoding UCS-2 / UTF-16BE volume id from Joliet descriptor
                    let u16_chars: Vec<u16> = desc[40..72]
                        .chunks_exact(2)
                        .map(|c| u16::from_be_bytes([c[0], c[1]]))
                        .filter(|&c| c != 0 && c != 0x20)
                        .collect();
                    if let Ok(s) = String::from_utf16(&u16_chars) {
                        let trimmed = s.trim().to_string();
                        if !trimmed.is_empty() {
                            volume_id = trimmed;
                        }
                    }
                }
            }
            0xFF => {
                // Volume Descriptor Set Terminator
                break;
            }
            _ => {}
        }
    }

    if found_pvd {
        Ok(Some(IsoDescriptorInfo {
            system_id,
            volume_id,
            publisher_id,
            data_preparer_id,
            application_id,
            volume_size_bytes,
            logical_block_size,
            is_hybrid,
            has_joliet,
            has_el_torito,
        }))
    } else {
        Ok(None)
    }
}

/// Detect image format and sub-type directly from filesystem analysis.
pub fn classify_image_format(path: &Path) -> ImageFormat {
    if let Ok(Some(info)) = inspect_iso_descriptors(path) {
        if info.is_hybrid {
            ImageFormat::HybridIso
        } else {
            ImageFormat::Iso9660
        }
    } else {
        crate::format_detect::detect_format(path).unwrap_or(ImageFormat::Unknown)
    }
}
