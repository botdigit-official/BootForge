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
}
