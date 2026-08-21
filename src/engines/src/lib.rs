//! # BootForge Boot Engines
//!
//! Boot configuration generators for BootForge.
//!
//! Provides:
//! - GRUB2 configuration generation
//! - EFI boot entry management
//! - Windows boot handling (WIM, BCD)
//! - Persistence configuration
//! - Boot config generation from analysis results

pub mod config_gen;
pub mod efi;
pub mod grub;
pub mod persistence;
pub mod windows;
