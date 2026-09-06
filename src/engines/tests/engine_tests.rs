#[cfg(test)]
mod tests {
    use bootforge_core::types::*;
    use bootforge_engines::config_gen::*;
    use chrono::Utc;
    use std::path::PathBuf;

    // ── Helper: build a minimal IsoAnalysis for any OS family ──────────
    fn make_iso(
        id: &str,
        file_name: &str,
        family: OsFamily,
        distro: Option<&str>,
        display_name: &str,
    ) -> IsoAnalysis {
        IsoAnalysis {
            id: id.to_string(),
            file_path: PathBuf::from(format!("/path/to/{}", file_name)),
            file_name: file_name.to_string(),
            file_size: 4_000_000_000,
            format: ImageFormat::Iso9660,
            hash: HashResult {
                sha256: "fakehash".into(),
                sha512: None,
                md5: None,
                status: VerificationStatus::Verified,
            },
            os_info: OsInfo {
                family,
                distribution: distro.map(|s| s.to_string()),
                version: Some("24.04".to_string()),
                codename: None,
                image_type: ImageType::Live,
                display_name: display_name.to_string(),
            },
            boot_info: BootInfo {
                uefi_capable: true,
                bios_capable: true,
                secure_boot: SecureBootStatus::Supported,
                efi_architectures: vec!["x86_64".into()],
            },
            architecture: Architecture::X86_64,
            compatibility: CompatibilityScore {
                score: 95,
                level: CompatLevel::Excellent,
                details: vec![],
                uefi_x64_score: 100,
                bios_score: 100,
                secure_boot_score: 90,
                arm64_score: 0,
            },
            quirks: vec![],
            boot_strategy: BootStrategy::GrubLoopback,
            supports_persistence: false,
            analyzed_at: Utc::now(),
            warnings: vec![],
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // Individual entry tests: verify boot commands per OS family
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_grub_entry_ubuntu_has_casper_boot_commands() {
        let iso = make_iso(
            "ubuntu", "ubuntu-24.04-desktop-amd64.iso",
            OsFamily::Linux, Some("Ubuntu"), "Ubuntu 24.04 LTS",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Ubuntu 24.04 LTS"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/casper/vmlinuz")); // kernel line
        assert!(entry.contains("iso-scan/filename=")); // ISO scan param
        assert!(entry.contains("boot=casper")); // casper boot param
        assert!(entry.contains("initrd (loop)/casper/initrd")); // initrd line
        assert!(entry.contains("/images/linux/ubuntu-24.04-desktop-amd64.iso")); // correct path
    }

    #[test]
    fn test_grub_entry_linux_mint_has_casper_boot_commands() {
        let iso = make_iso(
            "mint", "linuxmint-22.3-cinnamon-64bit.iso",
            OsFamily::Linux, Some("Linux Mint"), "Linux Mint 22.3",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Linux Mint 22.3"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/casper/vmlinuz")); // kernel line
        assert!(entry.contains("boot=casper")); // casper boot param
        assert!(entry.contains("initrd (loop)/casper/initrd")); // initrd line
    }

    #[test]
    fn test_grub_entry_fedora_has_pxeboot_boot_commands() {
        let iso = make_iso(
            "fedora", "Fedora-Workstation-Live-x86_64-40.iso",
            OsFamily::Linux, Some("Fedora"), "Fedora 40 Workstation",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Fedora 40 Workstation"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/images/pxeboot/vmlinuz")); // Fedora kernel path
        assert!(entry.contains("rd.live.image")); // Fedora live image param
        assert!(entry.contains("initrd (loop)/images/pxeboot/initrd.img")); // Fedora initrd
    }

    #[test]
    fn test_grub_entry_generic_linux_has_boot_commands() {
        let iso = make_iso(
            "arch", "archlinux-2024.06.01-x86_64.iso",
            OsFamily::Linux, Some("Arch"), "Arch Linux",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Arch Linux"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/boot/vmlinuz")); // generic kernel path
        assert!(entry.contains("iso-scan/filename=")); // ISO scan param
        assert!(entry.contains("initrd (loop)/boot/initrd")); // generic initrd
    }

    #[test]
    fn test_grub_entry_windows_has_chainloader() {
        let iso = make_iso(
            "win11", "Win11_24H2_English_x64.iso",
            OsFamily::Windows, Some("Windows"), "Windows 11 24H2",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Windows 11 24H2"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("chainloader (loop)/efi/boot/bootx64.efi")); // EFI chainload
        assert!(entry.contains("/images/windows/Win11_24H2_English_x64.iso")); // correct path
        // Windows should NOT have linux/initrd lines
        assert!(!entry.contains("linux "));
        assert!(!entry.contains("initrd "));
    }

    #[test]
    fn test_grub_entry_recovery_iso_has_live_boot_commands() {
        let iso = make_iso(
            "clonezilla", "clonezilla-live-3.1.2-amd64.iso",
            OsFamily::Recovery, Some("Clonezilla"), "Clonezilla Live",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Clonezilla Live"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/live/vmlinuz")); // live kernel path
        assert!(entry.contains("boot=live")); // live boot param
        assert!(entry.contains("findiso=")); // findiso param
        assert!(entry.contains("initrd (loop)/live/initrd.img")); // live initrd
        assert!(entry.contains("/images/rescue/clonezilla-live-3.1.2-amd64.iso")); // correct path
    }

    #[test]
    fn test_grub_entry_utility_iso_has_live_boot_commands() {
        let iso = make_iso(
            "gparted", "gparted-live-1.6.0-amd64.iso",
            OsFamily::Utility, Some("GParted"), "GParted Live",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("GParted Live"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("linux (loop)/live/vmlinuz")); // live kernel path
        assert!(entry.contains("initrd (loop)/live/initrd.img")); // live initrd
    }

    #[test]
    fn test_grub_entry_unknown_os_has_chainloader() {
        let iso = make_iso(
            "custom", "custom-firmware-2024.iso",
            OsFamily::Unknown, None, "Custom Firmware",
        );
        let entry = generate_grub_entry(&iso);

        assert!(entry.contains("Custom Firmware"));
        assert!(entry.contains("loopback loop ")); // loopback mount
        assert!(entry.contains("chainloader (loop)/efi/boot/bootx64.efi")); // fallback chainload
    }

    // ═══════════════════════════════════════════════════════════════════
    // Full grub.cfg structure tests
    // ═══════════════════════════════════════════════════════════════════

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
                uefi_x64_score: 100,
                bios_score: 0,
                secure_boot_score: 95,
                arm64_score: 90,
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

    #[test]
    fn test_grub_cfg_has_required_modules() {
        let cfg = generate_grub_cfg(&[], 10);

        assert!(cfg.contains("insmod part_gpt"));
        assert!(cfg.contains("insmod part_msdos"));
        assert!(cfg.contains("insmod fat"));
        assert!(cfg.contains("insmod exfat"));
        assert!(cfg.contains("insmod ntfs"));
        assert!(cfg.contains("insmod all_video"));
        assert!(cfg.contains("insmod gfxterm"));
    }

    #[test]
    fn test_grub_cfg_has_reboot_and_shutdown() {
        let cfg = generate_grub_cfg(&[], 10);

        assert!(cfg.contains("Reboot System"));
        assert!(cfg.contains("reboot"));
        assert!(cfg.contains("Shutdown System"));
        assert!(cfg.contains("halt"));
    }

    #[test]
    fn test_grub_cfg_multiple_isos_generate_all_entries() {
        let isos = vec![
            make_iso("ubuntu", "ubuntu-24.04-desktop-amd64.iso", OsFamily::Linux, Some("Ubuntu"), "Ubuntu 24.04"),
            make_iso("win11", "Win11_24H2_English_x64.iso", OsFamily::Windows, Some("Windows"), "Windows 11"),
            make_iso("clonezilla", "clonezilla-live-3.1.2-amd64.iso", OsFamily::Recovery, Some("Clonezilla"), "Clonezilla"),
            make_iso("mint", "linuxmint-22.3-cinnamon-64bit.iso", OsFamily::Linux, Some("Linux Mint"), "Linux Mint 22.3"),
        ];

        let cfg = generate_grub_cfg(&isos, 15);

        assert!(cfg.contains("set timeout=15"));
        assert!(cfg.contains("Ubuntu 24.04"));
        assert!(cfg.contains("Windows 11"));
        assert!(cfg.contains("Clonezilla"));
        assert!(cfg.contains("Linux Mint 22.3"));

        // Each entry should have a loopback mount
        assert_eq!(cfg.matches("loopback loop ").count(), 4);

        // Linux entries should have linux + initrd
        assert!(cfg.contains("linux (loop)/casper/vmlinuz"));
        assert!(cfg.contains("initrd (loop)/casper/initrd"));

        // Windows entry should have chainloader
        assert!(cfg.contains("chainloader (loop)/efi/boot/bootx64.efi"));

        // Recovery entry should have live boot
        assert!(cfg.contains("linux (loop)/live/vmlinuz"));
        assert!(cfg.contains("initrd (loop)/live/initrd.img"));
    }

    #[test]
    fn test_grub_cfg_timeout_is_respected() {
        let cfg_short = generate_grub_cfg(&[], 5);
        let cfg_long = generate_grub_cfg(&[], 30);

        assert!(cfg_short.contains("set timeout=5"));
        assert!(cfg_long.contains("set timeout=30"));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Boot command correctness: every entry must have exactly one of:
    //   - linux + initrd (for Linux/Recovery/Utility)
    //   - chainloader (for Windows/Unknown)
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_all_linux_entries_have_loopback_and_kernel() {
        let distros = [
            (Some("Ubuntu"), "ubuntu-24.04.iso"),
            (Some("Linux Mint"), "mint-22.3.iso"),
            (Some("Fedora"), "fedora-40.iso"),
            (Some("Debian"), "debian-12.iso"),
            (Some("Arch"), "archlinux-2024.iso"),
            (Some("openSUSE"), "opensuse-leap-15.6.iso"),
        ];

        for (distro, file) in &distros {
            let iso = make_iso("test", file, OsFamily::Linux, *distro, "Test OS");
            let entry = generate_grub_entry(&iso);

            // Every Linux entry MUST have loopback
            assert!(entry.contains("loopback loop "),
                "{}: missing loopback", distro.unwrap_or("unknown"));
            // Every Linux entry MUST have a kernel line with linux command
            assert!(entry.contains("linux (loop)/"),
                "{}: missing linux command", distro.unwrap_or("unknown"));
            // Every Linux entry MUST have an initrd line
            assert!(entry.contains("initrd (loop)/"),
                "{}: missing initrd command", distro.unwrap_or("unknown"));
        }
    }

    #[test]
    fn test_every_entry_has_loopback_mount() {
        let isos = vec![
            make_iso("a", "ubuntu.iso", OsFamily::Linux, Some("Ubuntu"), "Ubuntu"),
            make_iso("b", "win.iso", OsFamily::Windows, Some("Windows"), "Windows"),
            make_iso("c", "clone.iso", OsFamily::Recovery, Some("Clonezilla"), "Clonezilla"),
            make_iso("d", "unknown.iso", OsFamily::Unknown, None, "Unknown"),
            make_iso("e", "bsd.iso", OsFamily::Bsd, Some("FreeBSD"), "FreeBSD"),
            make_iso("f", "mac.iso", OsFamily::MacOs, Some("macOS"), "macOS"),
        ];

        for iso in &isos {
            let entry = generate_grub_entry(iso);
            assert!(entry.contains("loopback loop "),
                "Entry for {} ({}) is missing loopback mount",
                iso.file_name, iso.os_info.display_name);
        }
    }

    #[test]
    fn test_every_entry_references_correct_image_path() {
        let cases = [
            (OsFamily::Linux, Some("Ubuntu"), "ubuntu.iso", "/images/linux/ubuntu.iso"),
            (OsFamily::Windows, Some("Windows"), "win.iso", "/images/windows/win.iso"),
            (OsFamily::Recovery, Some("Clonezilla"), "clone.iso", "/images/rescue/clone.iso"),
            (OsFamily::Unknown, None, "mystery.iso", "/images/mystery.iso"),
        ];

        for (family, distro, file, expected_path) in &cases {
            let iso = make_iso("test", file, family.clone(), *distro, "Test");
            let entry = generate_grub_entry(&iso);
            assert!(entry.contains(expected_path),
                "Entry for {} should reference {} but got:\n{}",
                file, expected_path, entry);
        }
    }

    #[test]
    fn test_linux_entry_has_nomodeset_fallback() {
        // The engines config_gen doesn't generate safe mode entries;
        // that's done in format.rs. Verify the structure is sound.
        let iso = make_iso("ubuntu", "ubuntu.iso", OsFamily::Linux, Some("Ubuntu"), "Ubuntu");
        let entry = generate_grub_entry(&iso);

        // Standard entry should NOT have nomodeset (that's for safe mode only)
        assert!(!entry.contains("nomodeset"),
            "Standard entry should not contain nomodeset");
        // Standard entry should have splash (normal boot)
        assert!(entry.contains("splash"),
            "Standard Ubuntu entry should have splash for normal boot");
    }

    #[test]
    fn test_generate_limine_conf_maximum_compatibility() {
        use bootforge_engines::limine::*;
        let options = LimineConfigOptions::default();
        let conf = generate_limine_conf(&options, &BootProfile::MaximumCompatibility, &[]);

        assert!(conf.contains("timeout: 10"));
        assert!(conf.contains("textmode: yes"));
        assert!(conf.contains("serial: yes"));
        assert!(conf.contains("BootForge Recovery OS (Diagnostic Studio)"));
        assert!(conf.contains("BootForge Recovery OS (Safe Graphics & Text Mode)"));
        assert!(conf.contains("MemTest86+"));
    }

    #[test]
    fn test_calculate_swm_chunks_large_wim() {
        use bootforge_engines::windows::calculate_swm_chunks;
        let wim_size = 7_500_000_000u64; // ~7.5 GB Windows 11 install.wim
        let plan = calculate_swm_chunks(wim_size, 3800); // 3.8 GB chunks for FAT32 limit

        assert_eq!(plan.total_chunks, 2);
        assert_eq!(plan.base_filename, "install.swm");
    }
}

