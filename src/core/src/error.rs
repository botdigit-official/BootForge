//! Error types for BootForge.
//!
//! Provides structured error types with context for every failure mode.
//! Every error includes what happened, why, and what the user can do.

use std::path::PathBuf;
use thiserror::Error;

/// Convenience type alias for BootForge results.
pub type Result<T> = std::result::Result<T, BootForgeError>;

/// Top-level error type for all BootForge operations.
#[derive(Debug, Error)]
pub enum BootForgeError {
    // ── Device Errors ──────────────────────────────────────────────

    /// No USB device found at the specified path or ID.
    #[error("Device not found: {device_id}")]
    DeviceNotFound { device_id: String },

    /// Device is currently in use by another process.
    #[error("Device is busy: {device_id}. Close any applications using it and try again.")]
    DeviceBusy { device_id: String },

    /// Device does not have enough space for the requested operation.
    #[error("Not enough space on device. Required: {required} bytes, Available: {available} bytes")]
    DeviceTooSmall { required: u64, available: u64 },

    /// Device is write-protected (physical switch or software lock).
    #[error("Device is write-protected: {device_id}. Remove write protection and try again.")]
    DeviceWriteProtected { device_id: String },

    /// Attempted to operate on a system drive — BLOCKED for safety.
    #[error("SAFETY: Refused to operate on system drive: {device_path}. BootForge never writes to system drives.")]
    SystemDriveDetected { device_path: String },

    // ── ISO Errors ─────────────────────────────────────────────────

    /// ISO file hash does not match the expected value — possible corruption.
    #[error("ISO hash mismatch for {path:?}. Expected: {expected}, Got: {actual}. File may be corrupted.")]
    IsoCorrupted {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    /// ISO format or content is not supported by BootForge.
    #[error("Unsupported ISO: {path:?}. Reason: {reason}")]
    IsoUnsupported { path: PathBuf, reason: String },

    /// ISO file is larger than available space on the target device.
    #[error("ISO too large: {path:?} ({size} bytes). Available: {available} bytes")]
    IsoTooLarge {
        path: PathBuf,
        size: u64,
        available: u64,
    },

    /// ISO file could not be read or parsed.
    #[error("Failed to read ISO: {path:?}. {detail}")]
    IsoReadFailed { path: PathBuf, detail: String },

    // ── Boot Errors ────────────────────────────────────────────────

    /// Bootloader installation failed during drive creation.
    #[error("Bootloader installation failed at stage '{stage}': {detail}")]
    BootloaderInstallFailed { stage: String, detail: String },

    /// Architecture mismatch between ISO and requested boot target.
    #[error("Architecture mismatch: ISO is {iso_arch}, but {requested} was requested")]
    IncompatibleArchitecture { iso_arch: String, requested: String },

    /// Secure Boot is required but not available or configured.
    #[error("Secure Boot support unavailable for this configuration")]
    SecureBootUnavailable,

    // ── Platform Errors ────────────────────────────────────────────

    /// Operation requires elevated permissions.
    #[error("Permission denied for operation: {operation}. Run with elevated privileges.")]
    PermissionDenied { operation: String },

    /// Feature is not available on this platform.
    #[error("Feature '{feature}' is not supported on this platform")]
    PlatformUnsupported { feature: String },

    // ── Network Errors ─────────────────────────────────────────────

    /// File download failed.
    #[error("Download failed from {url}: HTTP {status}")]
    DownloadFailed { url: String, status: u16 },

    /// Downloaded file hash does not match.
    #[error("Downloaded file hash mismatch. Expected: {expected}, Got: {actual}")]
    DownloadHashMismatch { expected: String, actual: String },

    // ── General Errors ─────────────────────────────────────────────

    /// I/O error wrapper.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration file error.
    #[error("Configuration error: {detail}")]
    ConfigError { detail: String },

    /// Generic internal error — should be rare.
    #[error("Internal error: {0}")]
    Internal(String),
}
