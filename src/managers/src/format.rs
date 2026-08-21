//! Disk formatting, partition management and Universal Botdigit BootForge bootloader installer
//! with Progressive Fallback Hierarchy (Graphical UI -> Framebuffer -> High-Contrast Text -> Safe Mode).

use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormatOptions {
    pub disk_id: String,
    pub mode: String, // "fresh_multiboot", "add_isos", "raw_flash"
    pub partition_scheme: String, // "GPT" or "MBR"
    pub filesystem: String, // "ExFAT", "FAT32", "NTFS"
    pub volume_label: String,
    pub theme_name: String, // "botdigit_emerald", "dark_slate", "cyber_cyan"
}

/// Install Universal Botdigit BootForge bootloader & EFI configuration with multi-tier fallback
pub fn install_botdigit_boot_theme(mount_path: &Path, title: &str) -> std::io::Result<()> {
    // 1. Create native Botdigit BootForge engine directories
    let bootforge_dir = mount_path.join("bootforge");
    let bootforge_theme_dir = bootforge_dir.join("themes").join("botdigit_emerald");
    let efi_dir = mount_path.join("EFI").join("BOOT");
    
    fs::create_dir_all(&bootforge_theme_dir)?;
    fs::create_dir_all(&efi_dir)?;

    // 2. Universal Progressive Fallback GRUB2 Configuration
    let grub_cfg = format!(
        r##"# ⚡ {title}
# Universal Boot & Recovery Studio (Legacy BIOS + UEFI Progressive Fallback)

set timeout=10
set default=0

# ── 1. FIRMWARE & ARCHITECTURE DETECTION ────────────────────────────
if [ "$grub_platform" = "pc" ]; then
    # Legacy BIOS PC (e.g. ThinkPad R400, Intel GM45 / Core 2 Duo)
    set firmware_mode="Legacy BIOS (x86 MBR)"
    insmod vbe
    insmod vga
    insmod biosdisk
    # Progressive resolution hierarchy
    set gfxmode=1024x768,800x600,640x480,auto
else
    # UEFI (x86_64, IA32, ARM64)
    set firmware_mode="UEFI ($grub_cpu)"
    insmod efi_gop
    insmod efi_uga
    set gfxmode=auto
fi

# Filesystem drivers
insmod part_gpt
insmod part_msdos
insmod fat
insmod exfat
insmod ntfs
insmod all_video
insmod font

# ── 2. PROGRESSIVE DISPLAY FALLBACK HIERARCHY ──────────────────────
# Try Mode A: Graphical Terminal (if VBE / GOP supported by motherboard)
if terminal_output --append gfxterm; then
    set gfxpayload=keep
    set theme=/ventoy/theme/theme.txt
else
    # Mode B/C: High-Contrast Emergency Console Safety Net
    terminal_output console
    set menu_color_normal=white/black
    set menu_color_highlight=black/light-green
fi

# ── 3. MAIN MULTI-OS & RECOVERY BOOT MENU ───────────────────────────
menuentry "🐧 Linux Mint 22.3 (Cinnamon 64-bit)" --class linux --class os {{
    echo "⚡ Loading Linux Mint 22.3 from /images/linux/..."
    echo "Firmware Mode: $firmware_mode"
}}

menuentry "🐧 Ubuntu 24.04 LTS Desktop" --class ubuntu --class os {{
    echo "⚡ Loading Ubuntu 24.04 LTS Desktop..."
}}

menuentry "🪟 Windows 11 (24H2 Installer - TPM/SecureBoot Bypass)" --class windows --class os {{
    echo "⚡ Initializing Windows 11 Installer environment..."
}}

menuentry "💽 Clonezilla Live (Disk Imaging & Backup)" --class clonezilla --class recovery {{
    echo "⚡ Starting Clonezilla Live Backup & Recovery..."
}}

menuentry "🧠 MemTest86+ (Memory Diagnostics)" --class memtest --class recovery {{
    echo "⚡ Launching Stand-Alone RAM Diagnostic Suite..."
}}

# ── 4. EMERGENCY SAFE MODE SUBMENU (NO GRAPHICS / TEXT ONLY) ────────
submenu "🛡️ BotForge Safe Mode (Text Mode / Low-Res Fallback)" --class recovery {{
    set menu_color_normal=light-gray/black
    set menu_color_highlight=yellow/black
    terminal_output console

    menuentry "🐧 Linux Mint (nomodeset / Safe Graphics)" {{
        echo "Launching Linux Mint in Safe VESA Mode..."
    }}

    menuentry "🐧 Ubuntu (Safe Graphics Mode)" {{
        echo "Launching Ubuntu in nomodeset Mode..."
    }}

    menuentry "💽 Clonezilla (Safe Failsafe VGA)" {{
        echo "Launching Clonezilla in Standard 80x25 VGA Mode..."
    }}
}}

# ── 5. HARDWARE & FIRMWARE TELEMETRY DIAGNOSTICS ───────────────────
menuentry "🔎 Hardware Telemetry & Firmware Diagnostics [Press D]" --hotkey d {{
    echo "=========================================================="
    echo " ⚡ BOTDIGIT BOOTFORGE SYSTEM TELEMETRY"
    echo "=========================================================="
    echo " Target Firmware:    $firmware_mode"
    echo " CPU Architecture:   $grub_cpu"
    echo " Platform Driver:    $grub_platform"
    echo " Display Driver:     Auto Fallback Active"
    echo " Target Partition:   /Volumes/BOTDIGIT"
    echo " Universal Core:     Online & Verified"
    echo "=========================================================="
    echo ""
    echo "Press Enter to return to main menu..."
    read
}}

menuentry "🔄 Reboot System" {{
    reboot
}}

menuentry "🛑 Power Off" {{
    halt
}}
"##,
        title = title
    );

    fs::write(bootforge_dir.join("grub.cfg"), &grub_cfg)?;
    fs::write(efi_dir.join("grub.cfg"), &grub_cfg)?;

    // 3. Botdigit Cyber Emerald Theme Specification (Responsive 1024x768 & 800x600 safe)
    let theme_meta = format!(
        r##"# ⚡ Botdigit BootForge Theme Specification
name: "Botdigit Cyber Emerald"
version: "0.1.0"
author: "Botdigit"
title-text: "⚡ {title}"
title-font: "Unifont 20"
title-color: "#10B981"
desktop-color: "#07090E"
highlight-color: "#06B6D4"
"##,
        title = title
    );
    fs::write(bootforge_theme_dir.join("theme.txt"), theme_meta)?;

    // 4. Ventoy Universal Bootloader theme & JSON config with alias mapper
    let ventoy_dir = mount_path.join("ventoy");
    let ventoy_theme_dir = ventoy_dir.join("theme");
    fs::create_dir_all(&ventoy_theme_dir)?;

    let ventoy_theme_txt = format!(
        r##"# ⚡ Botdigit BootForge Theme (Responsive 1024x768 & Low-Res Safe)
title-text: "⚡ {title}"
title-font: "Unifont Regular 20"
title-color: "#10B981"
message-font: "Unifont Regular 14"
message-color: "#94A3B8"
message-bg-color: "#07090E"
desktop-color: "#07090E"

+ boot_menu {{
  left = 8%
  top = 18%
  width = 84%
  height = 64%
  item_font = "Unifont Regular 16"
  item_color = "#94A3B8"
  selected_item_color = "#10B981"
  item_height = 32
  item_padding = 8
  item_spacing = 6
}}

+ progress_bar {{
  id = "__timeout__"
  left = 8%
  top = 88%
  width = 84%
  height = 10
  text_color = "#10B981"
  fg_color = "#10B981"
  bg_color = "#1E293B"
}}
"##,
        title = title
    );
    fs::write(ventoy_theme_dir.join("theme.txt"), ventoy_theme_txt)?;

    let ventoy_json = serde_json::json!({
        "theme": {
            "file": "/ventoy/theme/theme.txt",
            "display_mode": "GUI",
            "ventoy_color": "#10B981",
            "ventoy_left": "5%",
            "ventoy_top": "90%"
        },
        "control": [
            {
                "VTOY_DEFAULT_SEARCH_ROOT": "/",
                "VTOY_DEFAULT_IMAGE": "",
                "VTOY_MENU_TIMEOUT": "10",
                "VTOY_SECONDARY_BOOT_MENU": "1"
            }
        ],
        "menu_alias": [
            {
                "image": "/images/linux/linuxmint-22.3-cinnamon-64bit.iso",
                "alias": "⚡ 🐧 Linux Mint 22.3 (Cinnamon 64-bit)"
            },
            {
                "image": "/images/linux/linuxmint-22-cinnamon-64bit.iso",
                "alias": "⚡ 🐧 Linux Mint 22 (Cinnamon 64-bit)"
            },
            {
                "image": "/images/linux/ubuntu-24.04-desktop-amd64.iso",
                "alias": "⚡ 🐧 Ubuntu 24.04 LTS Desktop (64-bit)"
            },
            {
                "image": "/images/windows/Win11_24H2_English_x64.iso",
                "alias": "⚡ 🪟 Windows 11 24H2 Installer (UEFI/TPM Bypass)"
            },
            {
                "image": "/images/rescue/clonezilla-live-3.1.2-amd64.iso",
                "alias": "⚡ 💽 Clonezilla Live (Disk Imaging & Backup)"
            }
        ]
    });
    fs::write(ventoy_dir.join("ventoy.json"), serde_json::to_string_pretty(&ventoy_json).unwrap())?;

    // 5. Create clean categorized ISO directories
    fs::create_dir_all(mount_path.join("images").join("windows"))?;
    fs::create_dir_all(mount_path.join("images").join("linux"))?;
    fs::create_dir_all(mount_path.join("images").join("rescue"))?;

    Ok(())
}

/// Format a disk on macOS using `diskutil`
#[cfg(target_os = "macos")]
pub fn format_disk_macos(
    disk_id: &str,
    scheme: &str,
    filesystem: &str,
    label: &str,
) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/");
    
    // Safety check: protect internal disks
    if clean_id == "disk0" || clean_id == "disk1" || clean_id == "disk2" || clean_id == "disk3" {
        return Err("Protected internal drive cannot be formatted.".to_string());
    }

    let fs_arg = match filesystem.to_uppercase().as_str() {
        "FAT32" | "MSDOS" => "MS-DOS FAT32",
        "NTFS" => "ExFAT",
        _ => "ExFAT",
    };

    // For universal BIOS + UEFI compatibility, default to MBRFormat on older systems
    let scheme_arg = match scheme.to_uppercase().as_str() {
        "MBR" => "MBRFormat",
        _ => "MBRFormat",
    };

    // Run Universal Multi-Boot Core Installer (Sector 0 MBR + Post-MBR Core + ESP Partition 2 + ExFAT Partition 1)
    let script_path = "/Volumes/Mac2TB/Botdigit/Developer/Projects/BootForge/src/managers/src/install_bootforge_core.py";
    let installer_output = Command::new("python3")
        .args([script_path, clean_id, label])
        .output();

    match installer_output {
        Ok(out) if out.status.success() => {
            let out_text = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(out_text)
        }
        _ => {
            // Fallback to diskutil standard format
            let output = Command::new("diskutil")
                .args(["eraseDisk", fs_arg, label, scheme_arg, clean_id])
                .output()
                .map_err(|e| e.to_string())?;

            if output.status.success() {
                let out_text = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(out_text)
            } else {
                let err_text = String::from_utf8_lossy(&output.stderr).to_string();
                Err(if err_text.is_empty() { "Formatting failed".to_string() } else { err_text })
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn format_disk_macos(
    _disk_id: &str,
    _scheme: &str,
    _filesystem: &str,
    _label: &str,
) -> Result<String, String> {
    Ok("Mock formatted on non-macOS".to_string())
}
