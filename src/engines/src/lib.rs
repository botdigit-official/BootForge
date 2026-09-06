//! # BootForge Boot Engines
//!
//! Universal boot configuration and bootloader deployment engines for BootForge.
//!
//! Provides:
//! - Limine Universal Bootloader engine (BIOS MBR/GPT + UEFI x64/ia32/aa64)
//! - GRUB2 configuration generator
//! - EFI boot entry and layout management
//! - Windows boot handling (WIM, Split SWM, BCD bypasses)
//! - Persistence configuration for Live Linux

pub mod config_gen;
pub mod efi;
pub mod grub;
pub mod limine;
pub mod persistence;
pub mod windows;
