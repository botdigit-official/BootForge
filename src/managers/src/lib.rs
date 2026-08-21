//! # BootForge Managers
//!
//! System managers for BootForge Universal Boot & Recovery Studio.
//!
//! Provides:
//! - Device detection and monitoring (USB hot-plug)
//! - Partition management (create, format, resize)
//! - Disk writer (copy ISOs, verify writes)
//! - OS image downloader (official sources)
//! - QEMU VM tester
//! - Hardware Telemetry and System Explorer
//! - 1-Click Non-Destructive Boot Repair
//! - USB Micro File Manager

pub mod device;
pub mod download;
pub mod file_manager;
pub mod format;
pub mod hardware;
pub mod partition;
pub mod repair;
pub mod vm;
pub mod writer;
