#[cfg(test)]
mod tests {
    use bootforge_analyzers::arch_detect::*;
    use bootforge_analyzers::compat::*;
    use bootforge_analyzers::format_detect::*;
    use bootforge_analyzers::hash::*;
    use bootforge_analyzers::os_detect::*;
    use bootforge_core::types::*;

    #[test]
    fn test_format_detection_wim_header() {
        let mut header = vec![0u8; 512];
        header[0..8].copy_from_slice(b"MSWIM\x00\x00\x00");
        let format = detect_format_from_bytes(&header, 512, None);
        assert_eq!(format, ImageFormat::Wim);
    }

    #[test]
    fn test_format_detection_iso_header() {
        let mut header = vec![0u8; 0x9000];
        header[0x8001..0x8006].copy_from_slice(b"CD001");
        let format = detect_format_from_bytes(&header, 0x9000, None);
        assert_eq!(format, ImageFormat::Iso9660);
    }

    #[test]
    fn test_format_detection_vhdx_header() {
        let mut header = vec![0u8; 512];
        header[0..8].copy_from_slice(b"vhdxfile");
        let format = detect_format_from_bytes(&header, 512, None);
        assert_eq!(format, ImageFormat::Vhdx);
    }

    #[test]
    fn test_os_identification_windows_11() {
        let os = identify_os_from_label("Win11_24H2_English_x64.iso");
        assert_eq!(os.family, OsFamily::Windows);
        assert_eq!(os.version.as_deref(), Some("11"));
        assert_eq!(os.display_name, "Windows 11");
    }

    #[test]
    fn test_os_identification_ubuntu() {
        let os = identify_os_from_label("ubuntu-24.04-desktop-amd64.iso");
        assert_eq!(os.family, OsFamily::Linux);
        assert_eq!(os.distribution.as_deref(), Some("Ubuntu"));
        assert_eq!(os.version.as_deref(), Some("24.04"));
    }

    #[test]
    fn test_os_identification_clonezilla() {
        let os = identify_os_from_label("clonezilla-live-3.1.1-amd64.iso");
        assert_eq!(os.family, OsFamily::Recovery);
        assert_eq!(os.display_name, "Clonezilla");
    }

    #[test]
    fn test_arch_detection() {
        assert_eq!(detect_architecture_from_str("ubuntu-24.04-desktop-amd64.iso"), Architecture::X86_64);
        assert_eq!(detect_architecture_from_str("alpine-standard-3.20.0-aarch64.iso"), Architecture::Arm64);
        assert_eq!(detect_architecture_from_str("debian-12.5.0-i386-netinst.iso"), Architecture::X86);
    }

    #[test]
    fn test_hash_calculation_and_verification() {
        let data = b"BootForge universal boot media test string";
        let hashes = compute_hashes_from_bytes(data);
        assert!(!hashes.sha256.is_empty());
        assert!(hashes.sha512.is_some());
        assert!(hashes.md5.is_some());

        assert_eq!(
            verify_hash(&hashes.sha256, &hashes.sha256),
            VerificationStatus::Verified
        );
        assert_eq!(
            verify_hash(&hashes.sha256, "invalid_hash_value"),
            VerificationStatus::Mismatch
        );
    }

    #[test]
    fn test_evaluate_compatibility_score() {
        let os_family = OsFamily::Linux;
        let boot_info = BootInfo {
            uefi_capable: true,
            bios_capable: true,
            secure_boot: SecureBootStatus::Supported,
            efi_architectures: vec!["x86_64".into()],
        };
        let hash = HashResult {
            sha256: "abc".into(),
            sha512: None,
            md5: None,
            status: VerificationStatus::Verified,
        };

        let (score, strategy) = evaluate_compatibility(&os_family, &boot_info, &hash, 0);
        assert_eq!(score.score, 100);
        assert_eq!(score.level, CompatLevel::Excellent);
        assert_eq!(strategy, BootStrategy::GrubLoopback);
    }

    #[test]
    fn test_boot_capabilities_detection() {
        use bootforge_analyzers::boot_detect::detect_boot_capabilities;
        let temp_dir = tempfile::tempdir().unwrap();
        let dummy_iso = temp_dir.path().join("ubuntu-24.04-desktop-amd64.iso");
        std::fs::write(&dummy_iso, b"dummy content with EFI/BOOT/BOOTX64.EFI and shimx64.efi").unwrap();

        let boot_info = detect_boot_capabilities(&dummy_iso, "ubuntu-24.04-desktop-amd64.iso");
        assert!(boot_info.uefi_capable);
        assert!(boot_info.bios_capable);
        assert_eq!(boot_info.secure_boot, SecureBootStatus::Supported);
    }
}
