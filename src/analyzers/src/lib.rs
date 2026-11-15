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
pub mod targets;

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

    // 1. Detect format using magic bytes and volume descriptor parser
    let format = iso::classify_image_format(path);

    // 2. Quick fingerprint of the first 1 MiB (a full checksum of a multi-GB image is done on request)
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; (1024 * 1024).min(file_size as usize)];
    let bytes_read = file.read(&mut buffer).unwrap_or(0);
    let sample_hashes = hash::compute_hashes_from_bytes(&buffer[..bytes_read]);

    let hash_result = HashResult {
        sha256: sample_hashes.sha256,
        sha512: sample_hashes.sha512,
        md5: sample_hashes.md5,
        // Nothing has been compared against the publisher's checksum yet.
        status: VerificationStatus::Unverified,
    };

    // 3. Identify OS (from volume label if ISO, or file name)
    let label = if let Ok(Some(desc)) = iso::inspect_iso_descriptors(path) {
        if !desc.volume_id.is_empty() {
            format!("{} {}", desc.volume_id, file_name)
        } else {
            file_name.clone()
        }
    } else {
        file_name.clone()
    };

    let os_info = os_detect::identify_os_from_label(&label);

    // 4. Architecture
    let architecture = arch_detect::detect_architecture_from_str(&label);

    // 5. Boot capability analysis (El Torito, UEFI, BIOS, Secure Boot Shim)
    let boot_info = boot_detect::detect_boot_capabilities(path, &file_name);

    // 6. Compatibility Score
    let (compatibility, boot_strategy) =
        compat::evaluate_compatibility(&os_info.family, &boot_info, &hash_result, 0);

    let supports_persistence = os_info.family == OsFamily::Linux
        && (os_info.display_name.contains("Ubuntu")
            || os_info.display_name.contains("Mint")
            || os_info.display_name.contains("Debian")
            || os_info.display_name.contains("Pop"));

    let device_guidance = targets::evaluate_targets(&os_info, &architecture, &boot_info);

    let mut warnings = Vec::new();
    if os_info.family == OsFamily::MacOs {
        warnings.push(
            "macOS installers cannot be started from the multi-boot menu. Create a dedicated installer drive \
             with `bootforge macos-installer` (uses Apple's createinstallmedia)."
                .to_string(),
        );
    }
    if file_size == 0 {
        warnings.push("The file is empty (0 bytes); copy it again.".to_string());
    }

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
        warnings,
        device_guidance,
    })
}
