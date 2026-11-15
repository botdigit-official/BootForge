//! Architecture detection module.

use bootforge_core::types::Architecture;

/// Detect CPU architecture from filename or volume label.
///
/// Token based: `x86_64` → 64-bit, `i386` / `i686` / `386` / `x86` / `32bit` → 32-bit,
/// `arm64` / `aarch64` / `a64fre` → ARM64. Unlabelled images are assumed 64-bit x86.
pub fn detect_architecture_from_str(text: &str) -> Architecture {
    let lower = text.to_lowercase().replace("x86_64", "x86-64").replace("x86-64", "amd64");
    let tokens: Vec<&str> = lower.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty()).collect();
    let has = |names: &[&str]| tokens.iter().any(|t| names.contains(t));

    if has(&["arm64", "aarch64", "a64fre", "armv8"]) || lower.contains("a64fre") {
        Architecture::Arm64
    } else if has(&["amd64", "x64", "64bit", "win64"]) || lower.contains("x64fre") {
        Architecture::X86_64
    } else if has(&["i386", "i486", "i586", "i686", "386", "686", "x86", "32bit", "win32"])
        || lower.contains("x86fre")
    {
        Architecture::X86
    } else if has(&["ia32"]) {
        Architecture::Ia32
    } else {
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
