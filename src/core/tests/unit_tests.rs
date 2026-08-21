#[cfg(test)]
mod tests {
    use bootforge_core::types::*;
    use bootforge_core::error::BootForgeError;
    use bootforge_core::config::AppConfig;
    use std::path::PathBuf;

    #[test]
    fn test_usb_interface_display() {
        assert_eq!(UsbInterface::Usb20.to_string(), "USB 2.0");
        assert_eq!(UsbInterface::Usb30.to_string(), "USB 3.0");
        assert_eq!(UsbInterface::Usb31Gen2.to_string(), "USB 3.1 Gen 2");
        assert_eq!(UsbInterface::Usb32Gen2x2.to_string(), "USB 3.2 Gen 2x2");
        assert_eq!(UsbInterface::Usb40.to_string(), "USB 4.0");
        assert_eq!(UsbInterface::Thunderbolt3.to_string(), "Thunderbolt 3");
        assert_eq!(UsbInterface::Thunderbolt4.to_string(), "Thunderbolt 4");
        assert_eq!(UsbInterface::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::UsbFlashDrive.to_string(), "USB Flash Drive");
        assert_eq!(DeviceType::PortableSsd.to_string(), "Portable SSD");
        assert_eq!(DeviceType::ExternalHdd.to_string(), "External HDD");
        assert_eq!(DeviceType::SdCard.to_string(), "SD Card");
        assert_eq!(DeviceType::NvmeEnclosure.to_string(), "NVMe Enclosure");
        assert_eq!(DeviceType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_image_format_display() {
        assert_eq!(ImageFormat::Iso9660.to_string(), "ISO 9660");
        assert_eq!(ImageFormat::Udf.to_string(), "UDF");
        assert_eq!(ImageFormat::HybridIso.to_string(), "Hybrid ISO");
        assert_eq!(ImageFormat::Wim.to_string(), "WIM");
        assert_eq!(ImageFormat::Esd.to_string(), "ESD");
        assert_eq!(ImageFormat::RawImg.to_string(), "Raw Image");
        assert_eq!(ImageFormat::Vhd.to_string(), "VHD");
        assert_eq!(ImageFormat::Vhdx.to_string(), "VHDX");
        assert_eq!(ImageFormat::Vmdk.to_string(), "VMDK");
        assert_eq!(ImageFormat::Qcow2.to_string(), "QCOW2");
        assert_eq!(ImageFormat::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_architecture_display() {
        assert_eq!(Architecture::X86.to_string(), "x86");
        assert_eq!(Architecture::X86_64.to_string(), "x86_64");
        assert_eq!(Architecture::Arm64.to_string(), "ARM64");
        assert_eq!(Architecture::Ia32.to_string(), "IA-32");
        assert_eq!(
            Architecture::Multi(vec!["x86_64".into(), "arm64".into()]).to_string(),
            "Multi (x86_64, arm64)"
        );
        assert_eq!(Architecture::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_compat_level_display() {
        assert_eq!(CompatLevel::Excellent.to_string(), "EXCELLENT");
        assert_eq!(CompatLevel::Good.to_string(), "GOOD");
        assert_eq!(CompatLevel::Fair.to_string(), "FAIR");
        assert_eq!(CompatLevel::Poor.to_string(), "POOR");
        assert_eq!(CompatLevel::Untested.to_string(), "UNTESTED");
        assert_eq!(CompatLevel::Incompatible.to_string(), "INCOMPATIBLE");
    }

    #[test]
    fn test_boot_category_display() {
        assert_eq!(BootCategory::Windows.to_string(), "Windows");
        assert_eq!(BootCategory::Linux.to_string(), "Linux");
        assert_eq!(BootCategory::Bsd.to_string(), "BSD");
        assert_eq!(BootCategory::Recovery.to_string(), "Recovery");
        assert_eq!(BootCategory::Utility.to_string(), "Utilities");
        assert_eq!(BootCategory::Firmware.to_string(), "Firmware");
        assert_eq!(BootCategory::Custom.to_string(), "Custom");
    }

    #[test]
    fn test_write_phase_display() {
        assert_eq!(WritePhase::Unmounting.to_string(), "Unmounting volumes");
        assert_eq!(WritePhase::Partitioning.to_string(), "Creating partitions");
        assert_eq!(WritePhase::Formatting.to_string(), "Formatting");
        assert_eq!(WritePhase::InstallingBootloader.to_string(), "Installing bootloader");
        assert_eq!(WritePhase::CopyingImages.to_string(), "Copying images");
        assert_eq!(WritePhase::WritingConfig.to_string(), "Writing configuration");
        assert_eq!(WritePhase::Verifying.to_string(), "Verifying");
        assert_eq!(WritePhase::Complete.to_string(), "Complete");
        assert_eq!(WritePhase::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_app_config_defaults_and_serialization() {
        let config = AppConfig::default();
        assert_eq!(config.default_partition_scheme, "gpt_uefi_plus_bios");
        assert_eq!(config.default_filesystem, "exfat");
        assert_eq!(config.default_persistence_size, 0);
        assert!(config.auto_update_compat_db);
        assert!(config.auto_verify_hashes);

        let json = serde_json::to_string(&config).expect("Serialize config to JSON");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Deserialize config from JSON");
        assert_eq!(deserialized.default_partition_scheme, config.default_partition_scheme);
        assert_eq!(deserialized.default_filesystem, config.default_filesystem);
    }

    #[test]
    fn test_storage_device_serialization() {
        let device = StorageDevice {
            id: "sandisk-extreme-256".to_string(),
            vendor: "SanDisk".to_string(),
            model: "Extreme Pro".to_string(),
            serial: "1234567890".to_string(),
            firmware_version: Some("1.0".to_string()),
            capacity: 256_000_000_000,
            available: 238_000_000_000,
            interface: UsbInterface::Usb32Gen2x2,
            device_type: DeviceType::PortableSsd,
            removable: true,
            read_only: false,
            health: Some(DeviceHealth {
                status: HealthStatus::Excellent,
                wear_percentage: Some(2),
                power_on_hours: Some(120),
                error_count: 0,
                temperature_celsius: Some(38.5),
            }),
            partitions: vec![
                Partition {
                    number: 1,
                    label: Some("EFI".to_string()),
                    filesystem: Some("FAT32".to_string()),
                    size: 512 * 1024 * 1024,
                    used: Some(50 * 1024 * 1024),
                    mount_point: None,
                    uuid: Some("EFI-UUID-1".to_string()),
                },
                Partition {
                    number: 2,
                    label: Some("BOOTFORGE".to_string()),
                    filesystem: Some("exFAT".to_string()),
                    size: 240 * 1000 * 1000 * 1000,
                    used: Some(10 * 1000 * 1000 * 1000),
                    mount_point: Some(PathBuf::from("/Volumes/BOOTFORGE")),
                    uuid: Some("BF-UUID-2".to_string()),
                },
            ],
            is_system_drive: false,
            platform_path: "/dev/disk2".to_string(),
        };

        let json = serde_json::to_string(&device).expect("Serialize storage device");
        let deserialized: StorageDevice = serde_json::from_str(&json).expect("Deserialize storage device");
        assert_eq!(deserialized.id, "sandisk-extreme-256");
        assert_eq!(deserialized.vendor, "SanDisk");
        assert_eq!(deserialized.partitions.len(), 2);
    }

    #[test]
    fn test_error_formatting() {
        let err = BootForgeError::DeviceTooSmall {
            required: 16_000_000_000,
            available: 8_000_000_000,
        };
        assert_eq!(
            err.to_string(),
            "Not enough space on device. Required: 16000000000 bytes, Available: 8000000000 bytes"
        );

        let err_sys = BootForgeError::SystemDriveDetected {
            device_path: "/dev/disk0".to_string(),
        };
        assert!(err_sys.to_string().contains("SAFETY: Refused to operate on system drive"));
    }
}
