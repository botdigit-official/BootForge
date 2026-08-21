//! USB Micro File Manager module for BotDigit BootForge.

use bootforge_core::types::UsbFileNode;
use std::fs;
use std::path::{Path, PathBuf};

/// Scan the USB drive file tree starting from mount path
pub fn scan_usb_files(mount_path_str: &str) -> Vec<UsbFileNode> {
    let base_path = Path::new(mount_path_str);
    if !base_path.exists() {
        return Vec::new();
    }

    scan_directory_recursive(base_path, base_path)
}

fn scan_directory_recursive(current_dir: &Path, root_dir: &Path) -> Vec<UsbFileNode> {
    let mut nodes = Vec::new();

    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

            // Skip hidden macOS metadata folders
            if file_name.starts_with('.') || file_name.starts_with("._") {
                continue;
            }

            let relative_path = path.strip_prefix(root_dir).unwrap_or(&path).to_string_lossy().to_string();
            let is_dir = path.is_dir();
            let mut size_bytes = 0;
            let mut extension = String::new();
            let mut is_bootable_iso = false;
            let mut category = None;

            if !is_dir {
                if let Ok(meta) = path.metadata() {
                    size_bytes = meta.len();
                }
                if let Some(ext) = path.extension() {
                    extension = ext.to_string_lossy().to_lowercase();
                    if matches!(extension.as_str(), "iso" | "img" | "wim" | "vhd" | "vhdx" | "raw") {
                        is_bootable_iso = true;
                        category = if relative_path.contains("windows") {
                            Some("Windows".to_string())
                        } else if relative_path.contains("rescue") {
                            Some("Recovery".to_string())
                        } else {
                            Some("Linux".to_string())
                        };
                    }
                }
            }

            let size_formatted = format_bytes(size_bytes);

            let children = if is_dir {
                scan_directory_recursive(&path, root_dir)
            } else {
                Vec::new()
            };

            nodes.push(UsbFileNode {
                name: file_name,
                relative_path,
                is_dir,
                size_bytes,
                size_formatted,
                extension,
                is_bootable_iso,
                category,
                sha256_hash: None,
                children,
            });
        }
    }

    nodes.sort_by_key(|n| (!n.is_dir, n.name.clone()));
    nodes
}

/// Delete a file inside the USB drive safely
pub fn delete_usb_file(mount_path_str: &str, relative_path: &str) -> Result<(), String> {
    let full_path = PathBuf::from(mount_path_str).join(relative_path.trim_start_matches('/'));
    
    // Prevent directory traversal escape
    let canonical_root = PathBuf::from(mount_path_str).canonicalize().map_err(|e| e.to_string())?;
    let canonical_target = full_path.canonicalize().map_err(|e| e.to_string())?;

    if !canonical_target.starts_with(&canonical_root) {
        return Err("Access denied: path outside target volume".to_string());
    }

    if canonical_target.is_file() {
        fs::remove_file(&canonical_target).map_err(|e| e.to_string())?;
        Ok(())
    } else if canonical_target.is_dir() {
        fs::remove_dir_all(&canonical_target).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("File not found".to_string())
    }
}

fn format_bytes(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{:.2} GB", gb)
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        if mb >= 1.0 {
            format!("{:.1} MB", mb)
        } else {
            format!("{} KB", bytes / 1024)
        }
    }
}
