//! ISO and boot media disk writer module with live progress metrics.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub is_running: bool,
    pub is_completed: bool,
    pub is_error: bool,
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
    pub phase: String,
    pub message: String,
    pub logs: Vec<String>,
}

pub struct WriteSession {
    pub is_active: Arc<AtomicBool>,
    pub bytes_written: Arc<AtomicU64>,
    pub total_bytes: Arc<AtomicU64>,
}

/// Safely transfer an ISO file to a destination folder / drive mount with real-time stats
pub fn copy_image_with_progress<F>(
    src_path: &Path,
    dest_path: &Path,
    mut on_progress: F,
) -> std::io::Result<()>
where
    F: FnMut(u64, u64, f32, u64), // bytes_written, total_bytes, speed_mbps, eta_seconds
{
    let mut src = File::open(src_path)?;
    let total_bytes = src.metadata()?.len();
    let mut dest = File::create(dest_path)?;

    let buffer_size = 4 * 1024 * 1024; // 4MB chunks
    let mut buffer = vec![0u8; buffer_size];
    let mut written: u64 = 0;

    let start_time = Instant::now();
    let mut last_update = Instant::now();

    loop {
        let n = src.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        dest.write_all(&buffer[..n])?;
        written += n as u64;

        if last_update.elapsed().as_millis() >= 100 || written == total_bytes {
            let elapsed_secs = start_time.elapsed().as_secs_f32().max(0.001);
            let speed_mbps = (written as f32 / (1024.0 * 1024.0)) / elapsed_secs;
            let remaining_bytes = total_bytes.saturating_sub(written);
            let eta_seconds = if speed_mbps > 0.0 {
                (remaining_bytes as f32 / (speed_mbps * 1024.0 * 1024.0)) as u64
            } else {
                0
            };

            on_progress(written, total_bytes, speed_mbps, eta_seconds);
            last_update = Instant::now();
        }
    }

    dest.flush()?;
    Ok(())
}
