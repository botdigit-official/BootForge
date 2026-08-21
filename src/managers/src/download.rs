//! OS & Recovery Image Downloader Engine with real-time progress tracking.

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub name: String,
    pub category: String, // "linux", "windows", "rescue", "utility"
    pub version: String,
    pub filename: String,
    pub direct_url: String,
    pub size_estimate_mb: u64,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatus {
    pub is_downloading: bool,
    pub is_completed: bool,
    pub is_error: bool,
    pub item_id: String,
    pub file_name: String,
    pub saved_path: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_mbps: f64,
    pub eta_seconds: u64,
    pub message: String,
}

pub fn get_curated_download_catalog() -> Vec<DownloadItem> {
    vec![
        DownloadItem {
            id: "linux-mint".to_string(),
            name: "Linux Mint 22.1 (Cinnamon 64-bit)".to_string(),
            category: "linux".to_string(),
            version: "22.1".to_string(),
            filename: "linuxmint-22.1-cinnamon-64bit.iso".to_string(),
            direct_url: "https://mirrors.kernel.org/linuxmint/stable/22.1/linuxmint-22.1-cinnamon-64bit.iso".to_string(),
            size_estimate_mb: 2800,
            sha256: None,
        },
        DownloadItem {
            id: "ubuntu".to_string(),
            name: "Ubuntu 24.04.1 LTS Desktop".to_string(),
            category: "linux".to_string(),
            version: "24.04.1".to_string(),
            filename: "ubuntu-24.04.1-desktop-amd64.iso".to_string(),
            direct_url: "https://releases.ubuntu.com/24.04.1/ubuntu-24.04.1-desktop-amd64.iso".to_string(),
            size_estimate_mb: 5800,
            sha256: None,
        },
        DownloadItem {
            id: "clonezilla".to_string(),
            name: "Clonezilla Live (Disk Imaging & Backup)".to_string(),
            category: "rescue".to_string(),
            version: "3.1.2".to_string(),
            filename: "clonezilla-live-3.1.2-22-amd64.iso".to_string(),
            direct_url: "https://downloads.sourceforge.net/project/clonezilla/clonezilla_live_stable/3.1.2-22/clonezilla-live-3.1.2-22-amd64.iso".to_string(),
            size_estimate_mb: 410,
            sha256: None,
        },
        DownloadItem {
            id: "gparted".to_string(),
            name: "GParted Live (Partition Editor)".to_string(),
            category: "rescue".to_string(),
            version: "1.6.0-3".to_string(),
            filename: "gparted-live-1.6.0-3-amd64.iso".to_string(),
            direct_url: "https://downloads.sourceforge.net/project/gparted/gparted-live-stable/1.6.0-3/gparted-live-1.6.0-3-amd64.iso".to_string(),
            size_estimate_mb: 520,
            sha256: None,
        },
        DownloadItem {
            id: "memtest".to_string(),
            name: "MemTest86+ (Stand-alone Memory Diagnostic)".to_string(),
            category: "rescue".to_string(),
            version: "7.00".to_string(),
            filename: "mt86plus_7.00.bin.iso".to_string(),
            direct_url: "https://www.memtest.org/download/v7.00/mt86plus_7.00.bin.iso".to_string(),
            size_estimate_mb: 15,
            sha256: None,
        },
        DownloadItem {
            id: "alpine".to_string(),
            name: "Alpine Linux 3.20 (Minimal / Fast)".to_string(),
            category: "linux".to_string(),
            version: "3.20.0".to_string(),
            filename: "alpine-standard-3.20.0-x86_64.iso".to_string(),
            direct_url: "https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/x86_64/alpine-standard-3.20.0-x86_64.iso".to_string(),
            size_estimate_mb: 210,
            sha256: None,
        },
    ]
}

/// Download an OS image directly with progress feedback
pub async fn download_os_image<F>(
    item: DownloadItem,
    target_dir: PathBuf,
    mut progress_callback: F,
) -> Result<PathBuf, String>
where
    F: FnMut(DownloadStatus) + Send + 'static,
{
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    let dest_path = target_dir.join(&item.filename);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 BotdigitBootForge/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&item.direct_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to mirror: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Mirror returned error status: {}", response.status()));
    }

    let total_bytes = response.content_length().unwrap_or(item.size_estimate_mb * 1024 * 1024);
    let mut file = File::create(&dest_path).map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start_time = Instant::now();
    let mut last_update = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if last_update.elapsed().as_millis() > 200 || downloaded == total_bytes {
            let elapsed_sec = start_time.elapsed().as_secs_f64().max(0.001);
            let speed_mbps = (downloaded as f64 / (1024.0 * 1024.0)) / elapsed_sec;
            let remaining_bytes = total_bytes.saturating_sub(downloaded);
            let eta = if speed_mbps > 0.0 {
                (remaining_bytes as f64 / (speed_mbps * 1024.0 * 1024.0)) as u64
            } else {
                0
            };
            let percentage = if total_bytes > 0 {
                (downloaded as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            progress_callback(DownloadStatus {
                is_downloading: true,
                is_completed: false,
                is_error: false,
                item_id: item.id.clone(),
                file_name: item.filename.clone(),
                saved_path: dest_path.to_string_lossy().to_string(),
                bytes_downloaded: downloaded,
                total_bytes,
                percentage,
                speed_mbps,
                eta_seconds: eta,
                message: format!("Downloading {} ({:.1} MB/s)", item.name, speed_mbps),
            });
            last_update = Instant::now();
        }
    }

    progress_callback(DownloadStatus {
        is_downloading: false,
        is_completed: true,
        is_error: false,
        item_id: item.id.clone(),
        file_name: item.filename.clone(),
        saved_path: dest_path.to_string_lossy().to_string(),
        bytes_downloaded: total_bytes,
        total_bytes,
        percentage: 100.0,
        speed_mbps: 0.0,
        eta_seconds: 0,
        message: format!("✓ {} downloaded successfully!", item.name),
    });

    Ok(dest_path)
}
