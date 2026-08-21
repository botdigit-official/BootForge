//! Event system for BootForge.
//!
//! Defines all events that flow between components and to the frontend UI.

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Events emitted by the backend to the frontend UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    // ── Device Events ──────────────────────────────────────────
    /// A new USB device was connected.
    DeviceConnected(StorageDevice),
    /// A USB device was disconnected.
    DeviceDisconnected { device_id: String },
    /// Device health changed.
    DeviceHealthChanged {
        device_id: String,
        health: DeviceHealth,
    },

    // ── ISO Events ─────────────────────────────────────────────
    /// ISO analysis started.
    IsoAnalysisStarted { file_name: String },
    /// ISO analysis progress (hash computation).
    IsoAnalysisProgress { file_name: String, percent: f32 },
    /// ISO analysis completed.
    IsoAnalysisCompleted(IsoAnalysis),
    /// ISO analysis failed.
    IsoAnalysisFailed {
        file_name: String,
        error: String,
    },

    // ── Write Events ───────────────────────────────────────────
    /// Drive creation progress.
    WriteProgress(Progress),
    /// Drive creation completed successfully.
    WriteCompleted,
    /// Drive creation failed.
    WriteFailed { error: String, phase: String },

    // ── Download Events ────────────────────────────────────────
    /// Download progress.
    DownloadProgress {
        file_name: String,
        percent: f32,
        speed_bytes_per_sec: u64,
        eta_seconds: u64,
    },
    /// Download completed.
    DownloadCompleted { file_name: String, path: String },
    /// Download failed.
    DownloadFailed {
        file_name: String,
        error: String,
    },

    // ── VM Test Events ─────────────────────────────────────────
    /// VM boot test started.
    VmTestStarted { iso_name: String },
    /// VM boot test result.
    VmTestCompleted {
        iso_name: String,
        success: bool,
        boot_time_ms: u64,
        log: String,
    },

    // ── General Events ─────────────────────────────────────────
    /// Informational notification.
    Notification {
        level: NotificationLevel,
        title: String,
        message: String,
    },
}

/// Notification severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}
