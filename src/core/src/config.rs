//! Application configuration management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application data directory.
    pub data_dir: PathBuf,
    /// Path to compatibility database.
    pub compat_db_path: PathBuf,
    /// Download directory for ISOs.
    pub download_dir: PathBuf,
    /// Whether to check for compatibility DB updates on launch.
    pub auto_update_compat_db: bool,
    /// Whether to auto-verify ISO hashes.
    pub auto_verify_hashes: bool,
    /// Default partition scheme.
    pub default_partition_scheme: String,
    /// Default data filesystem.
    pub default_filesystem: String,
    /// Default persistence size in bytes (0 = disabled).
    pub default_persistence_size: u64,
    /// QEMU path (for VM testing).
    pub qemu_path: Option<PathBuf>,
    /// OVMF firmware path (for UEFI VM testing).
    pub ovmf_path: Option<PathBuf>,
    /// Logging level.
    pub log_level: String,
    /// Whether opt-in telemetry is enabled.
    pub telemetry_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BootForge");

        Self {
            compat_db_path: data_dir.join("compat-db.toml"),
            download_dir: data_dir.join("downloads"),
            data_dir,
            auto_update_compat_db: true,
            auto_verify_hashes: true,
            default_partition_scheme: "gpt_uefi_plus_bios".to_string(),
            default_filesystem: "exfat".to_string(),
            default_persistence_size: 0,
            qemu_path: None,
            ovmf_path: None,
            log_level: "info".to_string(),
            telemetry_enabled: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from disk, or create default.
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BootForge")
            .join("config.toml");

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        tracing::warn!("Failed to parse config, using defaults: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config file, using defaults: {}", e);
                }
            }
        }

        Self::default()
    }

    /// Save current configuration to disk.
    pub fn save(&self) -> crate::Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BootForge");

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let contents = toml::to_string_pretty(self)
            .map_err(|e| crate::BootForgeError::ConfigError {
                detail: e.to_string(),
            })?;

        std::fs::write(&config_path, contents)?;
        Ok(())
    }
}
