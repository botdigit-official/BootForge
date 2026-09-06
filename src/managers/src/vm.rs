//! Real-Time Live Virtual Machine (QEMU) Boot Tester for Botdigit BootForge.
//!
//! Provides both interactive (GUI window) and headless (serial-only) boot testing
//! with structured telemetry output for automated validation.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Result of an automated headless boot test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadlessBootTestResult {
    pub success: bool,
    pub boot_time_ms: u64,
    pub firmware_mode: String,
    pub error: Option<String>,
    pub serial_output: String,
    pub boot_stage_reached: String,
}

/// Find OVMF/EDK2 UEFI firmware binary on the system.
fn find_ovmf_firmware() -> Option<PathBuf> {
    let ovmf_paths = [
        // macOS Homebrew (Apple Silicon)
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd",
        // macOS Homebrew (Intel)
        "/usr/local/share/qemu/edk2-x86_64-code.fd",
        // Linux common locations
        "/usr/share/OVMF/OVMF_CODE.fd",
        "/usr/share/edk2/ovmf/OVMF_CODE.fd",
        "/usr/share/qemu/OVMF_CODE.fd",
        "/usr/share/edk2/x64/OVMF_CODE.fd",
        // Ubuntu/Debian
        "/usr/share/ovmf/OVMF_CODE.fd",
    ];

    for p in &ovmf_paths {
        if Path::new(p).exists() {
            return Some(PathBuf::from(p));
        }
    }

    // Try glob patterns for versioned paths
    let glob_patterns = [
        "/opt/homebrew/Cellar/qemu/*/share/qemu/edk2-x86_64-code.fd",
        "/usr/local/Cellar/qemu/*/share/qemu/edk2-x86_64-code.fd",
    ];

    for pattern in &glob_patterns {
        if let Ok(entries) = std::fs::read_dir(Path::new(pattern).parent().unwrap_or(Path::new("/"))) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("share/qemu/edk2-x86_64-code.fd");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Launch a live real-time QEMU testing window booting the target USB drive directly.
pub fn launch_live_vm_test(disk_id: &str, is_uefi: bool) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");
    let raw_dev = format!("/dev/r{}", clean_id);

    // 1. Unmount disk so QEMU has full block device read access
    let _ = Command::new("diskutil")
        .args(["unmountDisk", &clean_id])
        .output();

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(["-m", "2048"]);
    cmd.args(["-smp", "2"]);
    cmd.args(["-vga", "std"]);
    cmd.args(["-name", "⚡ Botdigit BootForge Live Test VM"]);
    cmd.args(["-drive", &format!("file={},format=raw,cache=none,readonly=on", raw_dev)]);

    if is_uefi {
        if let Some(ovmf) = find_ovmf_firmware() {
            cmd.args(["-bios", &ovmf.to_string_lossy()]);
        }
    }

    match cmd.spawn() {
        Ok(_) => Ok(format!("Live VM window launched testing /dev/{} (Legacy BIOS / UEFI)", clean_id)),
        Err(e) => Err(format!("Could not start QEMU: {}. Make sure qemu-system-x86_64 is installed.", e)),
    }
}

/// Run a headless boot test with serial console output capture.
/// Returns structured telemetry after the VM boots or times out.
pub fn run_headless_boot_test(
    disk_id: &str,
    is_uefi: bool,
    timeout_secs: u64,
) -> Result<HeadlessBootTestResult, String> {
    let clean_id = disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");
    let raw_dev = format!("/dev/r{}", clean_id);
    let serial_log = PathBuf::from(format!("/tmp/bootforge_serial_{}.log", clean_id));

    // 1. Unmount disk
    let _ = Command::new("diskutil")
        .args(["unmountDisk", &clean_id])
        .output();

    // 2. Build QEMU command for headless test
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(["-m", "2048"]);
    cmd.args(["-smp", "2"]);
    cmd.args(["-nographic"]);  // No GUI - serial only
    cmd.args(["-serial", &format!("file:{}", serial_log.to_string_lossy())]);
    cmd.args(["-monitor", "none"]);  // No monitor
    cmd.args(["-no-reboot"]);  // Exit on reboot instead of looping
    cmd.args(["-drive", &format!("file={},format=raw,cache=none,readonly=on", raw_dev)]);

    if is_uefi {
        if let Some(ovmf) = find_ovmf_firmware() {
            cmd.args(["-bios", &ovmf.to_string_lossy()]);
        }
    }

    // 3. Start the VM and capture timing
    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start QEMU: {}", e))?;

    // 4. Wait for completion or timeout
    let timeout = Duration::from_secs(timeout_secs);
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let elapsed = start.elapsed().as_millis() as u64;
                let serial_output = std::fs::read_to_string(&serial_log)
                    .unwrap_or_else(|_| "(no serial output)".to_string());

                let boot_stage = detect_boot_stage(&serial_output);

                break Ok(HeadlessBootTestResult {
                    success: status.success() && !serial_output.contains("kernel panic"),
                    boot_time_ms: elapsed,
                    firmware_mode: if is_uefi { "UEFI".to_string() } else { "Legacy BIOS".to_string() },
                    error: if !status.success() {
                        Some(format!("QEMU exited with status: {}", status))
                    } else {
                        None
                    },
                    serial_output,
                    boot_stage_reached: boot_stage,
                });
            }
            Ok(None) => {
                // Still running - check timeout
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let serial_output = std::fs::read_to_string(&serial_log)
                        .unwrap_or_else(|_| "(no serial output)".to_string());

                    let boot_stage = detect_boot_stage(&serial_output);
                    break Ok(HeadlessBootTestResult {
                        success: false,
                        boot_time_ms: timeout.as_millis() as u64,
                        firmware_mode: if is_uefi { "UEFI".to_string() } else { "Legacy BIOS".to_string() },
                        error: Some(format!("Boot test timed out after {}s", timeout_secs)),
                        serial_output,
                        boot_stage_reached: boot_stage,
                    });
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                break Err(format!("Error checking QEMU status: {}", e));
            }
        }
    };

    // Cleanup
    let _ = child.kill();

    result
}

/// Detect which boot stage was reached based on serial output patterns.
fn detect_boot_stage(serial_output: &str) -> String {
    let stages = [
        ("[BF07] Interactive UI Ready", "UI ready"),
        ("[BF06] Recovery Daemons", "Recovery userspace"),
        ("[BF05] Initramfs", "Initramfs"),
        ("[BF04] Storage Discovery", "Storage discovery"),
        ("[BF03] Console", "Console init"),
        ("[BF02] Memory", "Memory setup"),
        ("[BF01] Kernel Entry", "Kernel entry"),
    ];

    for (pattern, stage_name) in &stages {
        if serial_output.contains(pattern) {
            return stage_name.to_string();
        }
    }

    // Fallback detection based on common kernel messages
    if serial_output.contains("Linux version") {
        "Kernel loaded".to_string()
    } else if serial_output.contains("GRUB") || serial_output.contains("grub") {
        "GRUB bootloader".to_string()
    } else if serial_output.contains("UEFI") || serial_output.contains("EFI") {
        "UEFI firmware".to_string()
    } else if serial_output.is_empty() {
        "No output (blank screen)".to_string()
    } else {
        "Early boot".to_string()
    }
}
