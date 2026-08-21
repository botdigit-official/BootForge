//! Compatibility scoring and evaluation.

use bootforge_core::types::{
    BootInfo, BootStrategy, CompatLevel, CompatibilityScore, HashResult, OsFamily,
    SecureBootStatus, VerificationStatus,
};

/// Calculate compatibility score based on verified image traits.
pub fn evaluate_compatibility(
    os_family: &OsFamily,
    boot_info: &BootInfo,
    hash: &HashResult,
    quirk_count: usize,
) -> (CompatibilityScore, BootStrategy) {
    let mut score: i32 = 100;
    let mut details = Vec::new();

    // 1. Hash status check
    if hash.status == VerificationStatus::Mismatch {
        score = 0;
        details.push("Integrity hash mismatch — file is likely corrupted".to_string());
    } else if hash.status == VerificationStatus::Verified {
        details.push("SHA256 signature verified against known official source".to_string());
    }

    // 2. Bootloader capability check
    if !boot_info.uefi_capable && !boot_info.bios_capable {
        score -= 40;
        details.push("No standard bootloader signature detected".to_string());
    } else {
        if boot_info.uefi_capable {
            details.push("UEFI boot structure detected".to_string());
        }
        if boot_info.bios_capable {
            details.push("Legacy BIOS bootloader detected".to_string());
        }
    }

    // 3. Secure Boot status
    match boot_info.secure_boot {
        SecureBootStatus::Supported => {
            details.push("Secure Boot compatible shim present".to_string());
        }
        SecureBootStatus::NotSupported => {
            details.push("Requires Secure Boot disabled in system firmware".to_string());
        }
        _ => {}
    }

    // 4. OS Family & Quirks
    if *os_family == OsFamily::Unknown {
        score -= 20;
        details.push("Generic OS profile applied".to_string());
    }

    if quirk_count > 0 {
        score -= (quirk_count as i32) * 5;
        details.push(format!("{} known compatibility quirk(s) handled", quirk_count));
    }

    let clamped = score.clamp(0, 100) as u32;
    let level = match clamped {
        90..=100 => CompatLevel::Excellent,
        70..=89 => CompatLevel::Good,
        50..=69 => CompatLevel::Fair,
        1..=49 => CompatLevel::Poor,
        _ => CompatLevel::Incompatible,
    };

    // Determine recommended boot strategy
    let strategy = match os_family {
        OsFamily::Windows => BootStrategy::WindowsWim,
        OsFamily::Linux => BootStrategy::GrubLoopback,
        OsFamily::Recovery => BootStrategy::GrubLoopback,
        OsFamily::Utility => BootStrategy::GrubLoopback,
        _ => {
            if boot_info.uefi_capable {
                BootStrategy::EfiChainload
            } else {
                BootStrategy::GrubLoopback
            }
        }
    };

    let uefi_x64 = if boot_info.uefi_capable { 100 } else { 0 };
    let bios = if boot_info.bios_capable { 100 } else { 0 };
    let secure_boot = match boot_info.secure_boot {
        SecureBootStatus::Supported => 95,
        SecureBootStatus::Partial => 60,
        _ => 20,
    };
    let arm64 = if boot_info.efi_architectures.iter().any(|a| a.contains("aa64") || a.contains("arm64")) { 90 } else { 0 };

    (
        CompatibilityScore {
            score: clamped,
            level,
            details,
            uefi_x64_score: uefi_x64,
            bios_score: bios,
            secure_boot_score: secure_boot,
            arm64_score: arm64,
        },
        strategy,
    )
}
