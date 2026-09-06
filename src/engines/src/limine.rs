//! Limine Bootloader Configuration and Deployment Engine for BootForge.
//! Generates universal `limine.conf` configurations supporting BIOS MBR, BIOS GPT,
//! UEFI x86_64, UEFI IA32, and UEFI AArch64 with guaranteed text-mode and serial fallbacks.

use bootforge_core::types::{BootProfile, IsoAnalysis, OsFamily};

/// Configuration generator for the Limine Universal Bootloader.
#[derive(Debug, Clone)]
pub struct LimineConfigOptions {
    pub timeout_seconds: u32,
    pub branding: String,
    pub default_entry: u32,
    pub text_mode_on_bios: bool,
    pub enable_serial: bool,
    pub serial_baud: u32,
    pub safe_smp: bool,
}

impl Default for LimineConfigOptions {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            branding: "Botdigit BootForge Universal Recovery Studio".to_string(),
            default_entry: 1,
            text_mode_on_bios: true,
            enable_serial: true,
            serial_baud: 115200,
            safe_smp: true,
        }
    }
}

/// Generate a complete `limine.conf` file contents.
pub fn generate_limine_conf(
    options: &LimineConfigOptions,
    profile: &BootProfile,
    isos: &[IsoAnalysis],
) -> String {
    let mut out = String::new();

    out.push_str("# ========================================================\n");
    out.push_str("# Botdigit BootForge Universal Bootloader (Limine Engine)\n");
    out.push_str("# ========================================================\n\n");

    out.push_str(&format!("timeout: {}\n", options.timeout_seconds));
    out.push_str(&format!("default_entry: {}\n", options.default_entry));
    out.push_str(&format!("interface_branding: {}\n", options.branding));

    if options.enable_serial {
        out.push_str(&format!("serial: yes\nserial_baud: {}\n", options.serial_baud));
    }

    if options.text_mode_on_bios || *profile == BootProfile::LegacyBiosOnly {
        out.push_str("textmode: yes\n");
    } else {
        out.push_str("graphics: yes\n");
    }

    out.push('\n');

    // ── Primary Entry: BootForge Standalone Recovery OS ─────────────────
    out.push_str("/⚡ BootForge Recovery OS (Diagnostic Studio)\n");
    out.push_str("    protocol: linux\n");
    out.push_str("    kernel_path: boot():/bootforge/vmlinuz\n");
    out.push_str("    initrd_path: boot():/bootforge/initramfs.img\n");
    let smp_flag = if options.safe_smp { "BOOTFORGE_SAFE_SMP=1" } else { "BOOTFORGE_SAFE_SMP=0" };
    out.push_str(&format!(
        "    cmdline: console=tty0 console=ttyS0,{} {} quiet\n\n",
        options.serial_baud, smp_flag
    ));

    // ── Safe Fallback: Text Mode & Minimal Drivers ─────────────────────
    out.push_str("/🛡️ BootForge Recovery OS (Safe Graphics & Text Mode)\n");
    out.push_str("    protocol: linux\n");
    out.push_str("    kernel_path: boot():/bootforge/vmlinuz\n");
    out.push_str("    initrd_path: boot():/bootforge/initramfs.img\n");
    out.push_str(&format!(
        "    cmdline: nomodeset text BOOTFORGE_TEXTMODE=1 BOOTFORGE_SAFE_SMP=0 console=tty0 console=ttyS0,{}\n\n",
        options.serial_baud
    ));

    // ── Hardware Diagnostics & Memory Test ─────────────────────────────
    out.push_str("/🧠 MemTest86+ Memory Diagnostics\n");
    out.push_str("    protocol: linux\n");
    out.push_str("    kernel_path: boot():/bootforge/memtest.bin\n\n");

    // ── User Injected Images (Chainloaded or Loopback) ──────────────────
    if !isos.is_empty() {
        out.push_str("# ── Installed Operating System Images ───────────────────\n\n");
        for iso in isos {
            let label = &iso.os_info.display_name;
            let file_name = &iso.file_name;

            match iso.os_info.family {
                OsFamily::Windows => {
                    out.push_str(&format!("/🪟 {}\n", label));
                    out.push_str("    protocol: efi_chainload\n");
                    out.push_str(&format!("    image_path: boot():/images/windows/{}\n\n", file_name));
                }
                OsFamily::Linux => {
                    out.push_str(&format!("/🐧 {}\n", label));
                    out.push_str("    protocol: linux\n");
                    out.push_str(&format!("    kernel_path: boot():/images/linux/{}\n\n", file_name));
                }
                OsFamily::Recovery | OsFamily::Utility => {
                    out.push_str(&format!("/🛠️ {}\n", label));
                    out.push_str("    protocol: linux\n");
                    out.push_str(&format!("    kernel_path: boot():/images/rescue/{}\n\n", file_name));
                }
                _ => {
                    out.push_str(&format!("/💿 {}\n", label));
                    out.push_str("    protocol: efi_chainload\n");
                    out.push_str(&format!("    image_path: boot():/images/{}\n\n", file_name));
                }
            }
        }
    }

    out.push_str("/🔄 System Reboot\n");
    out.push_str("    protocol: custom\n");
    out.push_str("    cmdline: reboot\n\n");

    out.push_str("/⏻ System Power Off\n");
    out.push_str("    protocol: custom\n");
    out.push_str("    cmdline: poweroff\n");

    out
}
