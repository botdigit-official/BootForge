#[cfg(test)]
mod tests {
    use bootforge_managers::partition::*;
    use bootforge_managers::repair::diagnose_host_operating_systems;

    #[test]
    fn test_partition_boundaries_maximum_compatibility() {
        let layout = PlannedPartitionLayout::default();
        assert_eq!(layout.bios_boot_size_mb, 2);
        assert_eq!(layout.efi_size_mb, 512);

        // Calculate for a 64 GB USB drive
        let capacity = 64_000_000_000u64;
        let map = calculate_partition_boundaries(capacity, &layout);

        // 2 MiB = 4096 sectors of 512 bytes
        assert_eq!(map.bios_boot_sectors, 4096);
        // 512 MiB = 1,048,576 sectors of 512 bytes
        assert_eq!(map.efi_sectors, 1_048_576);
        assert!(map.data_sectors > 100_000_000);
        assert_eq!(map.total_sectors, capacity / 512);
    }

    #[test]
    fn test_os_doctor_diagnosis_healthy() {
        let diagnosis = diagnose_host_operating_systems();
        assert!(!diagnosis.detected_operating_systems.is_empty());
        assert!(diagnosis.issues_found.is_empty());
        assert!(diagnosis.can_auto_repair);
    }

    // ── Write-target safety guard ──
    use bootforge_managers::device::protection_reason_from_info;

    const USB_STICK: &str = "   Device Identifier:         disk6
   Whole:                     Yes
   Protocol:                  USB
   Device Location:           External
   Removable Media:           Removable
   Read-Only Media:           No
";

    #[test]
    fn guard_allows_external_usb_stick() {
        assert_eq!(protection_reason_from_info("disk6", USB_STICK, &[], false), None);
        // Disk numbers mean nothing on their own: a USB stick can be disk2.
        assert_eq!(protection_reason_from_info("disk2", USB_STICK, &[], false), None);
    }

    #[test]
    fn guard_blocks_internal_boot_and_self_disks() {
        let internal = "   Whole: Yes\n   Protocol: Apple Fabric\n   Internal: Yes\n   Device Location: Internal\n";
        assert!(protection_reason_from_info("disk0", internal, &[], false).is_some());
        // An external SSD that holds macOS or BootForge itself (e.g. a Mac2TB work drive).
        let ext_ssd = "   Whole: Yes\n   Protocol: USB\n   Device Location: External\n";
        assert!(protection_reason_from_info("disk4", ext_ssd, &["disk4".to_string()], false).is_some());
        assert!(protection_reason_from_info("disk6s1", "   Whole: No\n   Protocol: USB\n", &[], false).is_some());
        assert!(protection_reason_from_info("disk9", "", &[], false).is_some());
    }

    #[test]
    fn guard_blocks_disk_images_unless_allowed() {
        let image = "   Whole: Yes\n   Protocol: Disk Image\n";
        assert!(protection_reason_from_info("disk7", image, &[], false).is_some());
        assert_eq!(protection_reason_from_info("disk7", image, &[], true), None);
    }

    #[test]
    fn guard_blocks_read_only_media() {
        let ro = "   Whole: Yes\n   Protocol: USB\n   Device Location: External\n   Read-Only Media: Yes\n";
        assert!(protection_reason_from_info("disk6", ro, &[], false).is_some());
    }
}
