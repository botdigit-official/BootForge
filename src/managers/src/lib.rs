//! # BootForge Managers
//!
//! System managers for BootForge.
//!
//! Provides:
//! - Device detection and monitoring (USB hot-plug)
//! - Partition management (create, format, resize)
//! - Disk writer (copy ISOs, verify writes)
//! - OS image downloader (official sources)
//! - QEMU VM tester

pub mod device;
pub mod download;
pub mod format;
pub mod partition;
pub mod vm;
pub mod writer;
