//! Privileged disk helper for BootForge (run as root).
//!
//! ```text
//! write_mbr_core --full /dev/diskN LABEL   Erase and create a complete BootForge drive
//! write_mbr_core /dev/diskN                Re-install the Legacy BIOS boot code only
//!                                          (keeps partitions and files; fixes a blinking
//!                                          cursor on old PCs)
//! ```
//!
//! Both modes refuse internal drives, the boot disk and the disk BootForge runs from.

use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [flag, dev, label] if flag == "--full" => full(dev, label),
        [flag, dev] if flag == "--full" => full(dev, "BOOTFORGE"),
        [dev] if !dev.starts_with('-') => bios_only(dev),
        _ => {
            eprintln!("Usage:\n  write_mbr_core --full /dev/diskN [LABEL]\n  write_mbr_core /dev/diskN");
            exit(2);
        }
    };
    match result {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("BootForge: {e}");
            exit(1);
        }
    }
}

#[cfg(target_os = "macos")]
fn full(dev: &str, label: &str) -> Result<String, String> {
    bootforge_managers::format::write_standard_drive_macos(dev, label)
}

/// Rewrite sector 0 boot code and sectors 1..2047 (core.img) while keeping the
/// partition table, disk UUID and every file.
#[cfg(target_os = "macos")]
fn bios_only(dev: &str) -> Result<String, String> {
    use bootforge_managers::format::{load_boot_payloads, resolve_bootforge_assets_dir};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::process::Command;

    let clean_id = dev.trim_start_matches("/dev/").replace("rdisk", "disk");
    bootforge_managers::device::ensure_safe_write_target(&clean_id)?;
    let (boot_img, core_img, _esp) = load_boot_payloads(&resolve_bootforge_assets_dir())?;

    let _ = Command::new("diskutil").args(["unmountDisk", "force", &clean_id]).output();
    let raw = format!("/dev/r{clean_id}");
    let mut disk = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&raw)
        .map_err(|e| format!("Cannot open {raw}: {e}"))?;

    let mut mbr = [0u8; 512];
    disk.read_exact(&mut mbr).map_err(|e| e.to_string())?;
    if mbr[510] != 0x55 || mbr[511] != 0xAA || mbr[466] != 0xEF {
        return Err(format!(
            "/dev/{clean_id} is not a BootForge drive (no MBR with an EFI partition 2). Use Format instead."
        ));
    }
    let first_part_start = u32::from_le_bytes(mbr[454..458].try_into().unwrap());
    if first_part_start < 2048 {
        return Err("Partition 1 starts before sector 2048; there is no room for the BIOS stage 2.".into());
    }

    // Bytes 0..384 boot code, keep 384..446 (disk UUID + signature) and the table.
    mbr[..384].copy_from_slice(&boot_img[..384]);
    mbr[446] = 0x80; // partition 1 active: some old BIOSes skip disks without an active partition
    let mut core = core_img;
    core.resize(bootforge_managers::boot_layout::CORE_MAX_BYTES, 0);

    let io = |e: std::io::Error| e.to_string();
    disk.seek(SeekFrom::Start(512)).map_err(io)?;
    disk.write_all(&core).map_err(io)?;
    disk.seek(SeekFrom::Start(0)).map_err(io)?;
    disk.write_all(&mbr).map_err(io)?;
    disk.flush().map_err(io)?;
    drop(disk);

    let _ = Command::new("diskutil").args(["mountDisk", &clean_id]).output();
    Ok(format!("Legacy BIOS boot code re-installed on /dev/{clean_id}. Files were not touched."))
}

#[cfg(not(target_os = "macos"))]
fn full(dev: &str, _label: &str) -> Result<String, String> {
    Err(format!("Not implemented on this OS; use ventoy_core/Ventoy2Disk.sh -i {dev}"))
}

#[cfg(not(target_os = "macos"))]
fn bios_only(dev: &str) -> Result<String, String> {
    Err(format!("Not implemented on this OS; use ventoy_core/Ventoy2Disk.sh -u {dev}"))
}
