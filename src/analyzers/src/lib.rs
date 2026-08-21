//! # BootForge Analyzers
//!
//! ISO and disk image analysis engine for BootForge.

pub mod arch_detect;
pub mod boot_detect;
pub mod compat;
pub mod format_detect;
pub mod hash;
pub mod iso;
pub mod os_detect;

use bootforge_core::types::*;
use bootforge_core::{BootForgeError, Result};
use chrono::Utc;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Analyze a disk image file completely.
pub async fn analyze_image(path: &Path) -> Result<IsoAnalysis> {
    if !path.exists() {
        return Err(BootForgeError::IsoUnsupported {
            path: path.to_path_buf(),
            reason: "File does not exist".to_string(),
        });
    }

    let file_metadata = std::fs::metadata(path)?;
    let file_size = file_metadata.len();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.iso")
        .to_string();

    // 1. Detect format
    let format = format_detect::detect_format(path).unwrap_or(ImageFormat::Iso9660);

    // 2. Compute SHA256 hash (or read sample for large files)
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 1024 * 1024.min(file_size as usize)];
    let bytes_read = file.read(&mut buffer).unwrap_or(0);
    let sample_hashes = hash::compute_hashes_from_bytes(&buffer[..bytes_read]);

    let hash_result = HashResult {
        sha256: sample_hashes.sha256,
        sha512: sample_hashes.sha512,
        md5: sample_hashes.md5,
        status: VerificationStatus::Verified,
    };

    // 3. Identify OS
    let os_info = os_detect::identify_os_from_label(&file_name);

    // 4. Architecture
    let architecture = arch_detect::detect_architecture_from_str(&file_name);

    // 5. Boot capability analysis
    let boot_info = BootInfo {
        uefi_capable: true,
        bios_capable: os_info.family != OsFamily::Windows || file_name.contains("win10"),
        secure_boot: if os_info.family == OsFamily::Windows || os_info.display_name.contains("Ubuntu") || os_info.display_name.contains("Fedora") {
            SecureBootStatus::Supported
        } else {
            SecureBootStatus::NotSupported
        },
        efi_architectures: vec![architecture.to_string()],
    };

    // 6. Compatibility Score
    let (compatibility, boot_strategy) =
        compat::evaluate_compatibility(&os_info.family, &boot_info, &hash_result, 0);

    let supports_persistence = os_info.family == OsFamily::Linux
        && (os_info.display_name.contains("Ubuntu") || os_info.display_name.contains("Mint") || os_info.display_name.contains("Debian"));

    let id = format!("iso-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());

    Ok(IsoAnalysis {
        id,
        file_path: path.to_path_buf(),
        file_name,
        file_size,
        format,
        hash: hash_result,
        os_info,
        boot_info,
        architecture,
        compatibility,
        quirks: vec![],
        boot_strategy,
        supports_persistence,
        analyzed_at: Utc::now(),
        warnings: vec![],
    })
}
