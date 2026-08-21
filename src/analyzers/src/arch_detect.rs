//! Architecture detection module.

use bootforge_core::types::Architecture;

/// Detect CPU architecture from filename, volume label, or binary headers.
pub fn detect_architecture_from_str(text: &str) -> Architecture {
    let lower = text.to_lowercase();

    if lower.contains("arm64") || lower.contains("aarch64") {
        Architecture::Arm64
    } else if lower.contains("x86_64") || lower.contains("amd64") || lower.contains("x64") {
        Architecture::X86_64
    } else if lower.contains("i386") || lower.contains("i686") || lower.contains("x86") {
        Architecture::X86
    } else if lower.contains("ia32") {
        Architecture::Ia32
    } else {
        // Most modern ISOs default to x86_64 if unspecified
        Architecture::X86_64
    }
}

/// Detect architecture from PE machine header bytes.
pub fn detect_from_pe_machine(machine: u16) -> Architecture {
    match machine {
        0x8664 => Architecture::X86_64,
        0xAA64 => Architecture::Arm64,
        0x014C => Architecture::X86,
        0x0200 => Architecture::Ia32,
        _ => Architecture::Unknown,
    }
}
