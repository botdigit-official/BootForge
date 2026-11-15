//! Target-device guidance: will this image boot on a given class of computer, and how?
//!
//! The rules are deliberately conservative and explain *why*, so the user knows what to
//! change (firmware setting, image variant, separate drive) instead of guessing.

use bootforge_core::types::{
    Architecture, BootInfo, BootVerdict, DeviceClass, OsFamily, OsInfo, SecureBootStatus, TargetVerdict,
};

/// Static facts about a device class shown to the user.
pub struct DeviceProfile {
    pub class: DeviceClass,
    pub name: &'static str,
    pub examples: &'static str,
    pub how_to_boot: &'static str,
}

pub const PC_BOOT_KEYS: &str = "Boot-menu key: F12 (Dell, Lenovo, Acer, Toshiba), F9 or Esc→F9 (HP), \
F8 or Esc (ASUS), F11 (MSI, ASRock), F12 (Gigabyte), Novo button (Lenovo IdeaPad), F7 (Medion)";

pub static DEVICE_PROFILES: &[DeviceProfile] = &[
    DeviceProfile {
        class: DeviceClass::LegacyBios32,
        name: "Very old PC (32-bit CPU, BIOS)",
        examples: "Pentium 4 / Pentium M laptops, Atom N270/N280 netbooks, 2000–2008",
        how_to_boot: "Press the boot-menu key (often F12 or Esc) and choose USB-HDD.",
    },
    DeviceProfile {
        class: DeviceClass::LegacyBios64,
        name: "Older PC (64-bit CPU, BIOS / CSM)",
        examples: "Core 2 / 1st–3rd gen Core: ThinkPad T400/R400/X200/T420, Dell Latitude E6400, 2006–2012",
        how_to_boot: "Press the boot-menu key and choose the USB drive (not the 'UEFI:' entry).",
    },
    DeviceProfile {
        class: DeviceClass::UefiPc,
        name: "UEFI PC (Secure Boot off)",
        examples: "Most desktops and laptops 2012–2020",
        how_to_boot: "Press the boot-menu key and choose 'UEFI: <your USB drive>'.",
    },
    DeviceProfile {
        class: DeviceClass::UefiSecureBoot,
        name: "Modern PC (UEFI, Secure Boot on)",
        examples: "Windows 11 laptops/desktops 2020+, Surface, no Legacy/CSM option",
        how_to_boot: "Choose 'UEFI: <your USB drive>'. First time only: enroll the BootForge key (see steps).",
    },
    DeviceProfile {
        class: DeviceClass::Uefi32,
        name: "32-bit UEFI tablet / mini PC",
        examples: "Bay Trail / Cherry Trail Atom tablets and stick PCs (Asus T100, Lenovo Miix, 2013–2016)",
        how_to_boot: "Hold Volume-Down or press F12/Esc at power on and pick the USB drive.",
    },
    DeviceProfile {
        class: DeviceClass::IntelMac,
        name: "Intel Mac (no T2)",
        examples: "MacBook / MacBook Pro / iMac / Mac mini 2006–2017",
        how_to_boot: "Hold Option (⌥) right after the chime and choose 'EFI Boot'.",
    },
    DeviceProfile {
        class: DeviceClass::IntelMacT2,
        name: "Intel Mac with T2 chip",
        examples: "MacBook Pro/Air 2018–2020, iMac Pro, Mac mini 2018, Mac Pro 2019",
        how_to_boot: "Allow external boot in Startup Security Utility, then hold Option (⌥) at power on.",
    },
    DeviceProfile {
        class: DeviceClass::AppleSilicon,
        name: "Apple Silicon Mac (M1 and later)",
        examples: "Every Mac sold since late 2020 (M1, M2, M3, M4...)",
        how_to_boot: "Hold the power button until 'Loading startup options', then pick the drive.",
    },
    DeviceProfile {
        class: DeviceClass::Arm64Uefi,
        name: "ARM64 UEFI machine",
        examples: "Snapdragon X Windows laptops, Ampere/Graviton servers, Raspberry Pi 4/5 with UEFI firmware",
        how_to_boot: "Use the firmware boot menu and choose the USB drive.",
    },
];

fn verdict(class: DeviceClass, v: BootVerdict, summary: impl Into<String>, steps: Vec<String>) -> TargetVerdict {
    let name = DEVICE_PROFILES.iter().find(|p| p.class == class).map(|p| p.name).unwrap_or("Device");
    TargetVerdict { device: class, device_name: name.to_string(), verdict: v, summary: summary.into(), steps }
}

fn s(text: &str) -> String {
    text.to_string()
}

fn has_efi(boot: &BootInfo, needle: &str) -> bool {
    boot.efi_architectures.iter().any(|a| a.contains(needle))
}

/// Evaluate an analysed image against every device class.
pub fn evaluate_targets(os: &OsInfo, arch: &Architecture, boot: &BootInfo) -> Vec<TargetVerdict> {
    DEVICE_PROFILES.iter().map(|p| evaluate_one(p.class, os, arch, boot)).collect()
}

fn evaluate_one(class: DeviceClass, os: &OsInfo, arch: &Architecture, boot: &BootInfo) -> TargetVerdict {
    use BootVerdict::*;
    use DeviceClass::*;

    let is_arm = matches!(arch, Architecture::Arm64) || has_efi(boot, "ARM64");
    let is_32 = matches!(arch, Architecture::X86 | Architecture::Ia32);
    let is_x86 = !is_arm;
    let win_ver = os.version.clone().unwrap_or_default();
    let is_win = os.family == OsFamily::Windows;
    let old_windows = is_win && matches!(win_ver.as_str(), "XP" | "Vista" | "7");

    // ── macOS installers: never from the multi-boot menu ──
    if os.family == OsFamily::MacOs {
        let separate = vec![
            s("Use a separate USB drive (16 GB+) for the macOS installer — it cannot be booted from the multi-boot menu."),
            s("On a Mac: run `bootforge macos-installer --app \"/Applications/Install macOS <Name>.app\" --disk diskN`, \
               or Apple's own: sudo \"/Applications/Install macOS <Name>.app/Contents/Resources/createinstallmedia\" --volume /Volumes/<USB>"),
            s("The installer only runs on Mac models that this macOS version supports (check Apple's compatibility list)."),
        ];
        return match class {
            IntelMac => verdict(class, WorksWithSteps, "macOS installers boot from their own drive made with createinstallmedia", {
                let mut v = separate;
                v.push(s("Hold Option (⌥) at power on and choose 'Install macOS'."));
                v
            }),
            IntelMacT2 => verdict(class, WorksWithSteps, "Own installer drive + allow external boot", {
                let mut v = separate;
                v.push(s("Start Recovery (⌘R) → Utilities → Startup Security Utility → 'Allow booting from external media'."));
                v.push(s("Hold Option (⌥) at power on and choose 'Install macOS'."));
                v
            }),
            AppleSilicon => verdict(class, WorksWithSteps, "Own installer drive made with createinstallmedia", {
                let mut v = separate;
                v.push(s("Shut down, hold the power button until startup options appear, choose the installer."));
                v
            }),
            _ => verdict(class, NotSupported, "macOS only installs on Apple hardware", vec![]),
        };
    }

    match class {
        LegacyBios32 => {
            if is_arm {
                verdict(class, NotSupported, "ARM64 image cannot run on an x86 PC", vec![])
            } else if !is_32 {
                verdict(class, NotSupported, "This is a 64-bit image; the CPU is 32-bit", vec![
                    s("Use a 32-bit (i386/i686) edition instead, e.g. antiX, Puppy Linux, MX Linux 32-bit, Debian i386, or Windows 7/XP 32-bit."),
                ])
            } else if !boot.bios_capable {
                verdict(class, NotSupported, "Image has no Legacy BIOS boot loader", vec![])
            } else {
                verdict(class, Works, "32-bit image with BIOS boot loader", vec![s(PC_BOOT_KEYS), s("Choose the USB drive, then the image.")])
            }
        }
        LegacyBios64 => {
            if is_arm {
                verdict(class, NotSupported, "ARM64 image cannot run on an x86 PC", vec![])
            } else if !boot.bios_capable {
                verdict(class, NotSupported, "Image only supports UEFI; this PC boots Legacy BIOS", vec![
                    s("If the PC has a UEFI option in firmware setup (some 2011–2012 models), switch to UEFI and boot the 'UEFI:' entry."),
                ])
            } else {
                let mut steps = vec![s(PC_BOOT_KEYS), s("Choose the plain USB entry (not 'UEFI:').")];
                if is_win && win_ver == "11" {
                    steps.push(s("Windows 11 setup's TPM/Secure Boot/CPU checks are bypassed automatically by BootForge (VTOY_WIN11_BYPASS_CHECK). Microsoft does not support Windows 11 on such hardware; updates may stop."));
                }
                steps.push(s("Blinking cursor instead of a menu? Run 'Fix Legacy BIOS boot' in BootForge (re-writes the BIOS boot code)."));
                verdict(class, if is_win && win_ver == "11" { WorksWithSteps } else { Works }, "Boots through the Legacy BIOS boot code", steps)
            }
        }
        UefiPc | UefiSecureBoot => {
            if is_arm {
                return verdict(class, NotSupported, "ARM64 image cannot run on an x86 PC", vec![]);
            }
            let mut steps = vec![s(PC_BOOT_KEYS), s("Choose 'UEFI: <USB drive>'.")];
            let mut v = Works;
            let mut summary = String::from("Boots in UEFI mode");
            if !boot.uefi_capable || old_windows {
                if class == UefiSecureBoot {
                    return verdict(class, NotSupported, "Image needs Legacy BIOS (CSM), which Secure-Boot-only PCs no longer have", vec![
                        s("Use a newer release of this OS, or a PC with a 'Legacy/CSM' option."),
                    ]);
                }
                v = WorksWithSteps;
                summary = "Needs Legacy/CSM mode".into();
                steps = vec![
                    s("Firmware setup → enable 'CSM' / 'Legacy Support' and disable Secure Boot."),
                    s(PC_BOOT_KEYS),
                    s("Choose the plain USB entry (not 'UEFI:')."),
                ];
                if old_windows {
                    steps.push(s("Windows 7/Vista/XP installers need Legacy mode; XP may also need SATA mode set to IDE/Compatible."));
                }
            } else if is_32 && !has_efi(boot, "IA-32") {
                if class == UefiSecureBoot {
                    return verdict(class, NotSupported, "32-bit image without UEFI boot loader", vec![]);
                }
                v = WorksWithSteps;
                summary = "32-bit image: boot it in Legacy/CSM mode".into();
                steps.insert(0, s("Firmware setup → enable 'CSM' / 'Legacy Support'."));
            }
            if class == UefiSecureBoot && v == Works {
                v = WorksWithSteps;
                summary = "Works with Secure Boot after a one-time key enrollment".into();
                steps.push(s("First boot shows a blue 'Verification failed' screen: OK → Enroll key from disk → VTOYEFI → ENROLL_THIS_KEY_IN_MOKMANAGER.cer → Continue → Yes → Reboot. Done once per PC."));
                if boot.secure_boot != SecureBootStatus::Supported && !is_win {
                    steps.push(s("This image's own kernel is not Secure Boot signed: if it stops after the menu, turn Secure Boot off in firmware setup."));
                }
            }
            verdict(class, v, summary, steps)
        }
        Uefi32 => {
            if is_arm {
                verdict(class, NotSupported, "ARM64 image cannot run on an x86 device", vec![])
            } else if has_efi(boot, "IA-32") {
                verdict(class, Works, "Image ships a 32-bit UEFI loader (BOOTIA32.EFI)", vec![s("Hold Volume-Down / press F12 at power on, choose the USB drive.")])
            } else {
                verdict(class, NotSupported, "Image has no 32-bit UEFI loader (BOOTIA32.EFI); this firmware cannot start it", vec![
                    s("Use a distro that supports 32-bit UEFI on 64-bit CPUs, e.g. Debian 'amd64' netinst/live (multi-arch), or Fedora/Ubuntu respun with bootia32.efi."),
                ])
            }
        }
        IntelMac => {
            if is_arm {
                return verdict(class, NotSupported, "ARM64 image cannot run on an Intel Mac", vec![]);
            }
            if !boot.uefi_capable {
                return verdict(class, NotSupported, "Intel Macs only boot USB drives in EFI mode; this image is BIOS-only", vec![]);
            }
            let mut steps = vec![s("Hold Option (⌥) right after power on and choose 'EFI Boot' (orange USB icon).")];
            if is_win {
                steps.push(s("After installing Windows, run Apple's Boot Camp driver package (Boot Camp Assistant → Action → Download Windows Support Software) for keyboard, Wi-Fi and trackpad."));
            } else {
                steps.push(s("Broadcom Wi-Fi in many Macs needs an extra driver (e.g. broadcom-sta / bcmwl) after installing Linux."));
            }
            steps.push(s("2006–early 2007 Core Duo Macs have 32-bit EFI and cannot start 64-bit EFI images."));
            verdict(class, Works, "Boots in EFI mode from the Option-key menu", steps)
        }
        IntelMacT2 => {
            if is_arm {
                return verdict(class, NotSupported, "ARM64 image cannot run on an Intel Mac", vec![]);
            }
            if !boot.uefi_capable {
                return verdict(class, NotSupported, "T2 Macs only boot USB drives in EFI mode; this image is BIOS-only", vec![]);
            }
            let mut steps = vec![
                s("Restart holding ⌘R → Utilities → Startup Security Utility."),
                s("Set Secure Boot to 'No Security' and External Boot to 'Allow booting from external media'."),
                s("Hold Option (⌥) at power on and choose 'EFI Boot'."),
            ];
            if !is_win {
                steps.push(s("Keyboard, trackpad and Wi-Fi need a T2-patched kernel (see t2linux.org); the stock live image may have no input."));
            }
            verdict(class, WorksWithSteps, "Needs external boot allowed in Startup Security Utility", steps)
        }
        AppleSilicon => {
            let summary = if is_x86 {
                "Apple Silicon cannot boot x86 operating systems"
            } else {
                "Apple Silicon has no UEFI and cannot boot generic ARM64 images from USB"
            };
            verdict(class, NotSupported, summary, vec![
                s("Run the image in a virtual machine instead: UTM (free), Parallels or VMware Fusion."),
                s("For native Linux use Asahi Linux (installed from macOS with its script, not from USB)."),
            ])
        }
        Arm64Uefi => {
            if !is_arm {
                if is_win {
                    return verdict(class, NotSupported, "x86 Windows image; ARM laptops need the Windows ARM64 ISO", vec![]);
                }
                return verdict(class, NotSupported, "x86 image cannot run on an ARM64 machine", vec![s("Download the 'arm64' / 'aarch64' edition of this OS.")]);
            }
            verdict(class, WorksWithSteps, "ARM64 image with UEFI loader", vec![
                s("Disable Secure Boot in firmware if the image's loader is not Microsoft-signed."),
                s("Choose the USB drive in the firmware boot menu."),
                s("Raspberry Pi 4/5: flash the community UEFI firmware (pftf/RPi4 or worproject) to an SD card first."),
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os_detect::identify_os_from_label;

    fn boot(uefi: bool, bios: bool, archs: &[&str]) -> BootInfo {
        BootInfo {
            uefi_capable: uefi,
            bios_capable: bios,
            secure_boot: SecureBootStatus::Supported,
            efi_architectures: archs.iter().map(|a| a.to_string()).collect(),
        }
    }

    fn get(v: &[TargetVerdict], c: DeviceClass) -> BootVerdict {
        v.iter().find(|t| t.device == c).unwrap().verdict
    }

    #[test]
    fn ubuntu_x64_everywhere_except_32bit_and_arm() {
        let os = identify_os_from_label("ubuntu-24.04-desktop-amd64.iso");
        let v = evaluate_targets(&os, &Architecture::X86_64, &boot(true, true, &["x86_64 (UEFI)"]));
        assert_eq!(get(&v, DeviceClass::LegacyBios64), BootVerdict::Works);
        assert_eq!(get(&v, DeviceClass::UefiPc), BootVerdict::Works);
        assert_eq!(get(&v, DeviceClass::UefiSecureBoot), BootVerdict::WorksWithSteps);
        assert_eq!(get(&v, DeviceClass::LegacyBios32), BootVerdict::NotSupported);
        assert_eq!(get(&v, DeviceClass::AppleSilicon), BootVerdict::NotSupported);
        assert_eq!(get(&v, DeviceClass::IntelMac), BootVerdict::Works);
        assert_eq!(v.len(), DEVICE_PROFILES.len());
    }

    #[test]
    fn windows7_needs_csm_and_fails_on_secure_boot_only_pcs() {
        let os = identify_os_from_label("en_windows_7_ultimate_sp1_x64.iso");
        let v = evaluate_targets(&os, &Architecture::X86_64, &boot(true, true, &["x86_64 (UEFI)"]));
        assert_eq!(get(&v, DeviceClass::LegacyBios64), BootVerdict::Works);
        assert_eq!(get(&v, DeviceClass::UefiPc), BootVerdict::WorksWithSteps);
        assert_eq!(get(&v, DeviceClass::UefiSecureBoot), BootVerdict::NotSupported);
    }

    #[test]
    fn i386_image_targets_old_pcs() {
        let os = identify_os_from_label("antiX-23.1_386-full.iso");
        let v = evaluate_targets(&os, &Architecture::X86, &boot(false, true, &[]));
        assert_eq!(get(&v, DeviceClass::LegacyBios32), BootVerdict::Works);
        assert_eq!(get(&v, DeviceClass::IntelMac), BootVerdict::NotSupported);
    }

    #[test]
    fn macos_installer_only_on_macs() {
        let os = identify_os_from_label("Install macOS Sonoma.dmg");
        let v = evaluate_targets(&os, &Architecture::X86_64, &boot(false, false, &[]));
        assert_eq!(get(&v, DeviceClass::AppleSilicon), BootVerdict::WorksWithSteps);
        assert_eq!(get(&v, DeviceClass::UefiPc), BootVerdict::NotSupported);
    }

    #[test]
    fn arm64_image_only_on_arm() {
        let os = identify_os_from_label("debian-12-arm64-netinst.iso");
        let v = evaluate_targets(&os, &Architecture::Arm64, &boot(true, false, &["ARM64 (UEFI)"]));
        assert_eq!(get(&v, DeviceClass::Arm64Uefi), BootVerdict::WorksWithSteps);
        assert_eq!(get(&v, DeviceClass::UefiPc), BootVerdict::NotSupported);
    }
}
