#[cfg(test)]
mod tests {
    use bootforge_core::types::*;
    use bootforge_engines::config_gen::*;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_generate_grub_entry_ubuntu() {
        let iso = IsoAnalysis {
            id: "ubuntu-2404".to_string(),
            file_path: PathBuf::from("/path/to/ubuntu-24.04-desktop-amd64.iso"),
            file_name: "ubuntu-24.04-desktop-amd64.iso".to_string(),
            file_size: 5_700_000_000,
            format: ImageFormat::Iso9660,
            hash: HashResult {
                sha256: "fakehash".into(),
                sha512: None,
                md5: None,
                status: VerificationStatus::Verified,
            },
            os_info: OsInfo {
                family: OsFamily::Linux,
                distribution: Some("Ubuntu".to_string()),
                version: Some("24.04".to_string()),
                codename: None,
                image_type: ImageType::Live,
                display_name: "Ubuntu 24.04 LTS".to_string(),
            },
            boot_info: BootInfo {
                uefi_capable: true,
                bios_capable: true,
                secure_boot: SecureBootStatus::Supported,
                efi_architectures: vec!["x86_64".into()],
            },
            architecture: Architecture::X86_64,
            compatibility: CompatibilityScore {
                score: 100,
                level: CompatLevel::Excellent,
                details: vec![],
            },
            quirks: vec![],
            boot_strategy: BootStrategy::GrubLoopback,
            supports_persistence: true,
            analyzed_at: Utc::now(),
            warnings: vec![],
        };

        let entry = generate_grub_entry(&iso);
        assert!(entry.contains("Ubuntu 24.04 LTS"));
        assert!(entry.contains("/images/linux/ubuntu-24.04-desktop-amd64.iso"));
        assert!(entry.contains("casper/vmlinuz"));
    }

    #[test]
    fn test_generate_grub_cfg_complete() {
        let iso = IsoAnalysis {
            id: "win11".to_string(),
            file_path: PathBuf::from("/path/to/Win11_24H2_x64.iso"),
            file_name: "Win11_24H2_x64.iso".to_string(),
            file_size: 6_200_000_000,
            format: ImageFormat::Iso9660,
            hash: HashResult {
                sha256: "fakehash".into(),
                sha512: None,
                md5: None,
                status: VerificationStatus::Verified,
            },
            os_info: OsInfo {
                family: OsFamily::Windows,
                distribution: Some("Windows".to_string()),
                version: Some("11".to_string()),
                codename: None,
                image_type: ImageType::Installer,
                display_name: "Windows 11".to_string(),
            },
            boot_info: BootInfo {
                uefi_capable: true,
                bios_capable: false,
                secure_boot: SecureBootStatus::Supported,
                efi_architectures: vec!["x86_64".into()],
            },
            architecture: Architecture::X86_64,
            compatibility: CompatibilityScore {
                score: 95,
                level: CompatLevel::Excellent,
                details: vec![],
            },
            quirks: vec![],
            boot_strategy: BootStrategy::WindowsWim,
            supports_persistence: false,
            analyzed_at: Utc::now(),
            warnings: vec![],
        };

        let cfg = generate_grub_cfg(&[iso], 10);
        assert!(cfg.contains("set timeout=10"));
        assert!(cfg.contains("search --no-floppy --set=root --label BOOTFORGE"));
        assert!(cfg.contains("Windows 11"));
        assert!(cfg.contains("chainloader (loop)/efi/boot/bootx64.efi"));
        assert!(cfg.contains("Reboot System"));
    }
}
