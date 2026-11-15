//! Standard multi-boot disk layout (Ventoy-compatible MBR layout).
//!
//! This is the layout every BootForge drive uses, because it is the one layout
//! that boots on the widest range of real hardware:
//!
//! ```text
//! Sector 0          MBR: GRUB boot.img (446 B) + disk UUID + partition table
//! Sectors 1..2047   GRUB core.img  (Legacy BIOS stage 2 — old PCs, CSM)
//! Partition 1       exFAT data partition, type 0x07, ACTIVE  — your ISO/WIM/VHD/IMG files
//! Partition 2       32 MiB ESP, type 0xEF                   — signed UEFI loaders (x64, IA32, ARM64)
//! ```
//!
//! * Legacy BIOS / CSM PCs boot sector 0 → core.img → GRUB on partition 2.
//! * UEFI PCs (with or without Secure Boot) find `EFI/BOOT/BOOTX64.EFI` on the 0xEF partition.
//! * 32-bit UEFI tablets find `BOOTIA32.EFI`; ARM64 UEFI boards find `BOOTAA64.EFI`.
//!
//! The layout math mirrors `format_ventoy_disk_mbr` in Ventoy's `ventoy_lib.sh`
//! exactly (partition 2 is 65 536 sectors, 4 KiB aligned, directly after partition 1)
//! so the bundled boot loader accepts the device as a standard device.
//!
//! Everything here is pure and works on any `Read + Write + Seek` target, so it is
//! unit-tested against plain files and disk images — no real disk required.

use std::io::{self, Read, Seek, SeekFrom, Write};

pub const SECTOR_SIZE: u64 = 512;
/// First sector of the data partition (1 MiB alignment).
pub const PART1_START: u64 = 2048;
/// Size of the ESP in sectors (32 MiB) — must match `ventoy.disk.img`.
pub const ESP_SECTORS: u64 = 65_536;
/// GRUB core.img lives in sectors 1..2047.
pub const CORE_MAX_BYTES: usize = (PART1_START as usize - 1) * SECTOR_SIZE as usize;
/// Smallest drive we will accept (1 GiB) — anything smaller cannot hold an OS image anyway.
pub const MIN_DISK_BYTES: u64 = 1024 * 1024 * 1024;

/// Planned sector boundaries for the standard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardLayout {
    pub total_sectors: u64,
    pub part1_start: u64,
    pub part1_sectors: u64,
    pub part2_start: u64,
    pub part2_sectors: u64,
}

impl StandardLayout {
    pub fn part1_end(&self) -> u64 {
        self.part1_start + self.part1_sectors - 1
    }
}

/// Compute the standard layout for a disk with `total_sectors` 512-byte sectors.
///
/// `reserve_mb` leaves unallocated space at the end of the disk (0 for none).
pub fn plan_standard_layout(total_sectors: u64, reserve_mb: u64) -> Result<StandardLayout, String> {
    if total_sectors * SECTOR_SIZE < MIN_DISK_BYTES {
        return Err(format!(
            "Drive is too small ({} MB). BootForge needs at least 1 GB.",
            total_sectors * SECTOR_SIZE / 1_000_000
        ));
    }
    if total_sectors > u32::MAX as u64 {
        return Err(
            "Drive is larger than 2 TiB, which the MBR layout cannot address. Use a drive of 2 TB or less."
                .to_string(),
        );
    }

    let reserve = reserve_mb * 2048;
    let mut part1_end = total_sectors
        .checked_sub(reserve + ESP_SECTORS + 1)
        .ok_or("Reserved space is larger than the drive")?;
    let mut part2_start = part1_end + 1;
    let misalign = part2_start % 8;
    if misalign > 0 {
        part1_end -= misalign;
        part2_start = part1_end + 1;
    }
    if part1_end <= PART1_START {
        return Err("Not enough space for the data partition".to_string());
    }

    Ok(StandardLayout {
        total_sectors,
        part1_start: PART1_START,
        part1_sectors: part1_end - PART1_START + 1,
        part2_start,
        part2_sectors: ESP_SECTORS,
    })
}

fn lba_to_chs(lba: u64) -> [u8; 3] {
    // Standard 255 heads × 63 sectors translation; saturate when out of CHS range.
    const HEADS: u64 = 255;
    const SPT: u64 = 63;
    let cylinder = lba / (HEADS * SPT);
    if cylinder > 1023 {
        return [0xFE, 0xFF, 0xFF];
    }
    let head = (lba / SPT) % HEADS;
    let sector = (lba % SPT) + 1;
    [
        head as u8,
        (sector as u8) | (((cylinder >> 2) & 0xC0) as u8),
        (cylinder & 0xFF) as u8,
    ]
}

fn write_entry(sector: &mut [u8], offset: usize, active: bool, part_type: u8, start: u64, count: u64) {
    sector[offset] = if active { 0x80 } else { 0x00 };
    sector[offset + 1..offset + 4].copy_from_slice(&lba_to_chs(start));
    sector[offset + 4] = part_type;
    sector[offset + 5..offset + 8].copy_from_slice(&lba_to_chs(start + count - 1));
    sector[offset + 8..offset + 12].copy_from_slice(&(start as u32).to_le_bytes());
    sector[offset + 12..offset + 16].copy_from_slice(&(count as u32).to_le_bytes());
}

/// Build sector 0: boot code + 16-byte disk UUID (offset 384) + disk signature
/// (offset 440) + two partition entries + 0x55AA.
pub fn build_standard_mbr(boot_img: &[u8], layout: &StandardLayout, disk_uuid: [u8; 16]) -> Vec<u8> {
    let mut sector = vec![0u8; 512];
    let code_len = boot_img.len().min(440);
    sector[..code_len].copy_from_slice(&boot_img[..code_len]);
    sector[384..400].copy_from_slice(&disk_uuid);
    sector[440..444].copy_from_slice(&disk_uuid[12..16]);
    write_entry(&mut sector, 446, true, 0x07, layout.part1_start, layout.part1_sectors);
    write_entry(&mut sector, 462, false, 0xEF, layout.part2_start, layout.part2_sectors);
    sector[510] = 0x55;
    sector[511] = 0xAA;
    sector
}

/// Payloads written to the raw device.
pub struct BootPayloads<'a> {
    /// GRUB i386-pc boot.img (at least 440 bytes).
    pub boot_img: &'a [u8],
    /// GRUB i386-pc core.img (≤ 1 047 552 bytes).
    pub core_img: &'a [u8],
    /// Pre-built 32 MiB FAT ESP image containing the signed UEFI loaders.
    pub esp_img: &'a [u8],
}

/// Write the complete boot structure to `dev` (a raw disk, or a file for tests).
///
/// Order matters: the ESP and core are written first, the partition table last,
/// so an interrupted write never leaves a drive that *looks* valid but is not.
/// The first 1 MiB of partition 1 is zeroed so no stale filesystem is detected
/// before the data partition is formatted.
pub fn write_standard_layout<D: Read + Write + Seek>(
    dev: &mut D,
    layout: &StandardLayout,
    payloads: &BootPayloads<'_>,
    disk_uuid: [u8; 16],
) -> io::Result<()> {
    if payloads.boot_img.len() < 440 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "boot.img is smaller than 440 bytes"));
    }
    if payloads.core_img.is_empty() || payloads.core_img.len() > CORE_MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("core.img must be 1..={} bytes (got {})", CORE_MAX_BYTES, payloads.core_img.len()),
        ));
    }
    if payloads.esp_img.len() as u64 != ESP_SECTORS * SECTOR_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ESP image must be exactly 32 MiB (got {} bytes)", payloads.esp_img.len()),
        ));
    }

    // Raw character devices on macOS require sector-aligned I/O, so every write
    // below is a whole number of 512-byte sectors.
    let zero_mib = vec![0u8; 1024 * 1024];

    // 1. ESP image into partition 2.
    dev.seek(SeekFrom::Start(layout.part2_start * SECTOR_SIZE))?;
    dev.write_all(payloads.esp_img)?;

    // 2. Wipe start of partition 1 (stale exFAT/NTFS/FAT boot sectors).
    dev.seek(SeekFrom::Start(layout.part1_start * SECTOR_SIZE))?;
    dev.write_all(&zero_mib)?;

    // 3. core.img into sectors 1..2047 (padded to whole sectors).
    let mut core = payloads.core_img.to_vec();
    core.resize(CORE_MAX_BYTES, 0);
    dev.seek(SeekFrom::Start(SECTOR_SIZE))?;
    dev.write_all(&core)?;

    // 4. Sector 0 last.
    let mbr = build_standard_mbr(payloads.boot_img, layout, disk_uuid);
    dev.seek(SeekFrom::Start(0))?;
    dev.write_all(&mbr)?;
    dev.flush()?;
    Ok(())
}

/// Problems found when checking a drive against the standard layout.
pub fn verify_standard_layout<D: Read + Seek>(dev: &mut D, layout: &StandardLayout) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    let mut mbr = [0u8; 512];
    dev.seek(SeekFrom::Start(0))?;
    dev.read_exact(&mut mbr)?;

    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        problems.push("Missing 0x55AA boot signature in sector 0".to_string());
    }
    if mbr[446] != 0x80 {
        problems.push("Partition 1 is not marked active (some old BIOSes refuse to boot)".to_string());
    }
    if mbr[450] != 0x07 {
        problems.push(format!("Partition 1 type is 0x{:02X}, expected 0x07 (exFAT/NTFS)", mbr[450]));
    }
    if mbr[466] != 0xEF {
        problems.push(format!("Partition 2 type is 0x{:02X}, expected 0xEF (EFI System)", mbr[466]));
    }
    let p1_start = u32::from_le_bytes(mbr[454..458].try_into().unwrap()) as u64;
    let p1_len = u32::from_le_bytes(mbr[458..462].try_into().unwrap()) as u64;
    let p2_start = u32::from_le_bytes(mbr[470..474].try_into().unwrap()) as u64;
    let p2_len = u32::from_le_bytes(mbr[474..478].try_into().unwrap()) as u64;
    if p1_start != layout.part1_start || p1_len != layout.part1_sectors {
        problems.push("Partition 1 boundaries differ from the standard layout".to_string());
    }
    if p2_start != layout.part2_start || p2_len != ESP_SECTORS {
        problems.push("Partition 2 is not a 32 MiB ESP directly after partition 1".to_string());
    }

    let mut esp_boot = [0u8; 512];
    dev.seek(SeekFrom::Start(p2_start * SECTOR_SIZE))?;
    dev.read_exact(&mut esp_boot)?;
    if esp_boot[510] != 0x55 || esp_boot[511] != 0xAA {
        problems.push("ESP does not contain a FAT filesystem".to_string());
    }

    let mut core_head = [0u8; 512];
    dev.seek(SeekFrom::Start(SECTOR_SIZE))?;
    dev.read_exact(&mut core_head)?;
    if core_head.iter().all(|&b| b == 0) {
        problems.push("Legacy BIOS stage 2 (core.img) is missing — old PCs will show a blinking cursor".to_string());
    }
    Ok(problems)
}

/// Fresh random 16-byte disk UUID.
pub fn new_disk_uuid() -> [u8; 16] {
    *uuid::Uuid::new_v4().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const GIB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn layout_matches_ventoy_rules() {
        let total = 16 * GIB / SECTOR_SIZE;
        let l = plan_standard_layout(total, 0).unwrap();
        assert_eq!(l.part1_start, 2048);
        assert_eq!(l.part2_sectors, 65_536);
        assert_eq!(l.part2_start, l.part1_end() + 1);
        assert_eq!(l.part2_start % 8, 0);
        assert!(l.part2_start + l.part2_sectors <= total);
    }

    #[test]
    fn layout_aligns_odd_sized_disks() {
        // Real flash drives rarely have a round sector count.
        for total in [30_031_872u64, 30_031_873, 60_437_499, 7_864_319] {
            let l = plan_standard_layout(total, 0).unwrap();
            assert_eq!(l.part2_start % 8, 0, "total={total}");
            assert_eq!(l.part2_start, l.part1_end() + 1);
            assert!(l.part2_start + ESP_SECTORS <= total);
        }
    }

    #[test]
    fn layout_rejects_tiny_and_huge_disks() {
        assert!(plan_standard_layout(512 * 1024 * 1024 / SECTOR_SIZE, 0).is_err());
        assert!(plan_standard_layout(5 * 1024 * GIB / SECTOR_SIZE, 0).is_err());
    }

    #[test]
    fn mbr_has_correct_entries() {
        let l = plan_standard_layout(8 * GIB / SECTOR_SIZE, 0).unwrap();
        let uuid = [7u8; 16];
        let mbr = build_standard_mbr(&[0xEB; 512], &l, uuid);
        assert_eq!(mbr[0], 0xEB);
        assert_eq!(&mbr[384..400], &uuid);
        assert_eq!(mbr[446], 0x80);
        assert_eq!(mbr[450], 0x07);
        assert_eq!(mbr[462], 0x00);
        assert_eq!(mbr[466], 0xEF);
        assert_eq!(u32::from_le_bytes(mbr[454..458].try_into().unwrap()), 2048);
        assert_eq!(&mbr[510..], &[0x55, 0xAA]);
    }

    #[test]
    fn write_then_verify_roundtrip() {
        // A small sparse in-memory "disk" of 1 GiB would be too big for a unit test,
        // so plan for 1 GiB but back it with a Vec sized to the layout.
        let total = GIB / SECTOR_SIZE;
        let l = plan_standard_layout(total, 0).unwrap();
        let mut disk = Cursor::new(vec![0u8; (total * SECTOR_SIZE) as usize]);
        let mut esp = vec![0u8; (ESP_SECTORS * SECTOR_SIZE) as usize];
        esp[510] = 0x55;
        esp[511] = 0xAA;
        let payloads = BootPayloads { boot_img: &[0x90; 512], core_img: &[0x11; 4096], esp_img: &esp };

        // Before writing, a blank disk must report problems.
        assert!(!verify_standard_layout(&mut disk, &l).unwrap().is_empty());

        write_standard_layout(&mut disk, &l, &payloads, new_disk_uuid()).unwrap();
        let problems = verify_standard_layout(&mut disk, &l).unwrap();
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn write_rejects_wrong_esp_size() {
        let l = plan_standard_layout(GIB / SECTOR_SIZE, 0).unwrap();
        let mut disk = Cursor::new(Vec::new());
        let payloads = BootPayloads { boot_img: &[0; 512], core_img: &[1; 16], esp_img: &[0; 1024] };
        assert!(write_standard_layout(&mut disk, &l, &payloads, [0; 16]).is_err());
    }
}
