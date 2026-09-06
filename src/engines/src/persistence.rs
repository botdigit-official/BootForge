//! Linux Persistence overlay engine for BootForge.
//! Manages persistent storage files (casper-rw, persistence.dat, overlay-size allocation)
//! for Ubuntu, Debian, Kali, Tails, and Linux Mint live USB sessions.

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

/// Persistence configuration options.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    pub size_mb: u64,
    pub label: String, // "casper-rw" or "persistence"
    pub filesystem: String, // "ext4"
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            size_mb: 4096, // 4 GB default
            label: "casper-rw".to_string(),
            filesystem: "ext4".to_string(),
        }
    }
}

/// Create a sparse persistence disk image file with a specified size.
pub fn create_persistence_file(path: &Path, config: &PersistenceConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    let target_bytes = config.size_mb * 1024 * 1024;

    if target_bytes > 0 {
        // Set sparse file size by seeking and writing final byte
        file.seek(SeekFrom::Start(target_bytes - 1))?;
        file.write_all(&[0u8])?;
    }

    Ok(())
}
