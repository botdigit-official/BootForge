//! Disk image format detection module.

use bootforge_core::types::ImageFormat;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// Detect image format by reading magic bytes and file structure.
pub fn detect_format_from_bytes(header: &[u8], _file_size: u64, footer: Option<&[u8]>) -> ImageFormat {
    // 1. WIM Header check
    if header.len() >= 8 && &header[0..8] == b"MSWIM\x00\x00\x00" {
        return ImageFormat::Wim;
    }

    // 2. VHDX check
    if header.len() >= 8 && &header[0..8] == b"vhdxfile" {
        return ImageFormat::Vhdx;
    }

    // 3. QCOW2 check
    if header.len() >= 4 && &header[0..4] == &[0x51, 0x46, 0x49, 0xFB] {
        return ImageFormat::Qcow2;
    }

    // 4. ISO 9660 Primary Volume Descriptor: "CD001" at sector 16 (offset 0x8001 or 32769)
    if header.len() >= 0x8006 && &header[0x8001..0x8006] == b"CD001" {
        return ImageFormat::Iso9660;
    }

    // 5. VHD footer check ("conectix" at the last 512 bytes)
    if let Some(foot) = footer {
        if foot.len() >= 8 && &foot[0..8] == b"conectix" {
            return ImageFormat::Vhd;
        }
    }

    // 6. Partition table check for raw images
    if header.len() >= 512 && header[510] == 0x55 && header[511] == 0xAA {
        if header.len() >= 520 && &header[512..520] == b"EFI PART" {
            return ImageFormat::RawImg;
        }
        return ImageFormat::RawImg;
    }

    ImageFormat::Unknown
}

/// Detect image format from a file path.
pub fn detect_format(path: &Path) -> io::Result<ImageFormat> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    // Read initial 64KB for header checks (covers sector 16 ISO descriptor at 32KB)
    let read_size = (file_size as usize).min(64 * 1024);
    let mut header = vec![0u8; read_size];
    file.read_exact(&mut header)?;

    let mut footer = None;
    if file_size >= 512 {
        file.seek(SeekFrom::End(-512))?;
        let mut foot = [0u8; 512];
        if file.read_exact(&mut foot).is_ok() {
            footer = Some(foot);
        }
    }

    let detected = detect_format_from_bytes(&header, file_size, footer.as_ref().map(|f| &f[..]));
    if detected != ImageFormat::Unknown {
        return Ok(detected);
    }

    // Fallback to file extension
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
        Some("iso") => Ok(ImageFormat::Iso9660),
        Some("wim") => Ok(ImageFormat::Wim),
        Some("esd") => Ok(ImageFormat::Esd),
        Some("img") => Ok(ImageFormat::RawImg),
        Some("vhd") => Ok(ImageFormat::Vhd),
        Some("vhdx") => Ok(ImageFormat::Vhdx),
        Some("vmdk") => Ok(ImageFormat::Vmdk),
        Some("qcow2") => Ok(ImageFormat::Qcow2),
        _ => Ok(ImageFormat::Unknown),
    }
}
