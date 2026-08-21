//! Direct Raw MBR + Post-MBR Gap Core.img Writer for macOS / Linux.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: write_mbr_core /dev/diskX");
        std::process::exit(1);
    }

    let dev_name = &args[1];
    let clean_id = dev_name.trim_start_matches("/dev/").replace("rdisk", "disk");
    let raw_path = format!("/dev/r{}", clean_id);

    println!("⚡ Target raw device: {}", raw_path);

    // 1. Force unmount
    println!("1. Force unmounting {}...", clean_id);
    let _ = Command::new("diskutil")
        .args(["unmountDisk", "force", &clean_id])
        .status();

    // 2. Read boot.img and core.img
    let boot_img_path = "/Volumes/Mac2TB/Botdigit/Developer/Projects/BootForge/src/managers/assets/ventoy_core/boot/boot.img";
    let core_img_path = "/tmp/core.img";

    let mut boot_img = Vec::new();
    File::open(boot_img_path)?.read_to_end(&mut boot_img)?;

    let mut core_img = Vec::new();
    File::open(core_img_path)?.read_to_end(&mut core_img)?;

    println!("   boot.img: {} bytes", boot_img.len());
    println!("   core.img: {} bytes", core_img.len());

    // 3. Open raw disk
    println!("2. Writing to {}...", raw_path);
    let mut disk = OpenOptions::new().read(true).write(true).open(&raw_path)?;

    // Read current MBR partition table (bytes 446..512)
    let mut mbr = vec![0u8; 512];
    disk.read_exact(&mut mbr)?;

    // Preserve partition table entries (offset 446 to 510) and signature 0x55AA
    let part_table = mbr[446..512].to_vec();

    // Merge boot.img (first 446 bytes) with preserved partition table
    let mut new_mbr = vec![0u8; 512];
    new_mbr[0..446].copy_from_slice(&boot_img[0..446]);
    new_mbr[446..512].copy_from_slice(&part_table);
    // Ensure Partition 1 is marked ACTIVE (0x80)
    new_mbr[446] = 0x80;

    // Write MBR to Sector 0
    disk.seek(SeekFrom::Start(0))?;
    disk.write_all(&new_mbr)?;
    println!("✓ Sector 0 MBR bootstrap written (Active flag: 0x80)");

    // Write Core.img into Sectors 1..2047 (post-MBR gap, 1048064 bytes)
    let core_len = core_img.len().min(2047 * 512);
    disk.seek(SeekFrom::Start(512))?;
    disk.write_all(&core_img[0..core_len])?;
    disk.flush()?;
    println!("✓ Sectors 1..2047 Post-MBR GRUB2 Core written ({} bytes)", core_len);

    // 4. Re-mount Partition 1
    println!("3. Re-mounting volumes...");
    let _ = Command::new("diskutil").args(["mountDisk", &clean_id]).status();

    println!("✅ MBR + Core.img successfully written! Ready for ThinkPad R400 Legacy BIOS boot.");

    Ok(())
}
