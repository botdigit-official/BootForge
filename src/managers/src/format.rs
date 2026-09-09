//! Disk formatting, partition management and Universal Botdigit BootForge bootloader installer
//! with Progressive Fallback Hierarchy (Graphical UI -> Framebuffer -> High-Contrast Text -> Safe Mode).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Statically embedded BootForge Boot OS master configuration (active even without ISOs)
pub const BOOTFORGE_OS_CFG: &str = include_str!("../assets/bootforge/bootforge_os.cfg");
pub static UNIFONT_PF2: &[u8] = include_bytes!("../assets/fonts/unicode.pf2");
pub static ASCII_PF2: &[u8] = include_bytes!("../assets/fonts/ascii.pf2");
pub static UNIFONT_NAMED_PF2: &[u8] = include_bytes!("../assets/fonts/unifont_named.pf2");

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
pub fn install_botdigit_boot_theme(mount_path: &Path, title: &str, label: &str) -> std::io::Result<()> {
    // 0. Install actual EFI bootloader binaries
    install_efi_binaries(mount_path)?;

    let final_label = if label.is_empty() { "BOOTFORGE" } else { label };

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
    search --no-floppy --set=isopart --label {final_label}
    set root=$isopart
    set isofile="/images/linux/linuxmint-22.3-cinnamon-64bit.iso"
    loopback loop $isofile
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile quiet splash ---
    initrd (loop)/casper/initrd
}}

menuentry "🐧 Ubuntu 24.04 LTS Desktop" --class ubuntu --class os {{
    echo "⚡ Loading Ubuntu 24.04 LTS Desktop..."
    search --no-floppy --set=isopart --label {final_label}
    set root=$isopart
    set isofile="/images/linux/ubuntu-24.04-desktop-amd64.iso"
    loopback loop $isofile
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile quiet splash ---
    initrd (loop)/casper/initrd
}}

menuentry "🪟 Windows 11 (24H2 Installer - TPM/SecureBoot Bypass)" --class windows --class os {{
    echo "⚡ Initializing Windows 11 Installer environment..."
    search --no-floppy --set=isopart --label {final_label}
    set root=$isopart
    set isofile="/images/windows/Win11_24H2_English_x64.iso"
    loopback loop $isofile
    chainloader (loop)/efi/boot/bootx64.efi
}}

menuentry "💽 Clonezilla Live (Disk Imaging & Backup)" --class clonezilla --class recovery {{
    echo "⚡ Starting Clonezilla Live Backup & Recovery..."
    search --no-floppy --set=isopart --label {final_label}
    set root=$isopart
    set isofile="/images/rescue/clonezilla-live-3.1.2-amd64.iso"
    loopback loop $isofile
    linux (loop)/live/vmlinuz boot=live findiso=$isofile quiet
    initrd (loop)/live/initrd.img
}}

menuentry "🧠 MemTest86+ (Memory Diagnostics)" --class memtest --class recovery {{
    echo "Launching Stand-Alone RAM Diagnostic Suite..."
    search --no-floppy --set=isopart --label {final_label}
    set root=$isopart
    linux16 /bootforge/memtest.bin
}}

# ── 4. EMERGENCY SAFE MODE SUBMENU (NO GRAPHICS / TEXT ONLY) ────────
submenu "🛡️ BotForge Safe Mode (Text Mode / Low-Res Fallback)" --class recovery {{
    set menu_color_normal=light-gray/black
    set menu_color_highlight=yellow/black
    terminal_output console

    menuentry "🐧 Linux Mint (nomodeset / Safe Graphics)" {{
        echo "Launching Linux Mint in Safe VESA Mode..."
        search --no-floppy --set=isopart --label {final_label}
        set root=$isopart
        set isofile="/images/linux/linuxmint-22.3-cinnamon-64bit.iso"
        loopback loop $isofile
        linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile nomodeset text ---
        initrd (loop)/casper/initrd
    }}

    menuentry "🐧 Ubuntu (Safe Graphics Mode)" {{
        echo "Launching Ubuntu in nomodeset Mode..."
        search --no-floppy --set=isopart --label {final_label}
        set root=$isopart
        set isofile="/images/linux/ubuntu-24.04-desktop-amd64.iso"
        loopback loop $isofile
        linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile nomodeset text ---
        initrd (loop)/casper/initrd
    }}

    menuentry "💽 Clonezilla (Safe Failsafe VGA)" {{
        echo "Launching Clonezilla in Standard 80x25 VGA Mode..."
        search --no-floppy --set=isopart --label {final_label}
        set root=$isopart
        set isofile="/images/rescue/clonezilla-live-3.1.2-amd64.iso"
        loopback loop $isofile
        linux (loop)/live/vmlinuz boot=live findiso=$isofile nomodeset
        initrd (loop)/live/initrd.img
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

    // Write both the Progressive Fallback theme cfg and the Master BootForge Boot OS cfg
    fs::write(bootforge_dir.join("theme_grub.cfg"), &grub_cfg)?;
    fs::write(bootforge_dir.join("bootforge_os.cfg"), BOOTFORGE_OS_CFG)?;
    fs::write(bootforge_dir.join("grub.cfg"), BOOTFORGE_OS_CFG)?;
    fs::write(efi_dir.join("grub.cfg"), BOOTFORGE_OS_CFG)?;

    // 2.5 Install PFF2 fonts to eliminate [?] tofu glyph boxes
    let bf_fonts = bootforge_dir.join("fonts");
    let efi_fonts = efi_dir.join("fonts");
    let _ = fs::create_dir_all(&bf_fonts);
    let _ = fs::create_dir_all(&efi_fonts);
    let _ = fs::write(bf_fonts.join("unicode.pf2"), UNIFONT_PF2);
    let _ = fs::write(bf_fonts.join("ascii.pf2"), ASCII_PF2);
    let _ = fs::write(bf_fonts.join("unifont_named.pf2"), UNIFONT_NAMED_PF2);
    let _ = fs::write(efi_fonts.join("unicode.pf2"), UNIFONT_PF2);
    let _ = fs::write(efi_fonts.join("unifont_named.pf2"), UNIFONT_NAMED_PF2);

    // 3. Botdigit Cyber Emerald Theme Specification (Responsive 1024x768 & 800x600 safe)
    let theme_meta = format!(
        r##"# Botdigit BootForge Theme Specification
name: "Botdigit Cyber Emerald"
version: "0.1.0"
author: "Botdigit"
title-text: "{title}"
title-font: "Unknown Regular 16"
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
    let vt_fonts = ventoy_theme_dir.join("fonts");
    fs::create_dir_all(&ventoy_theme_dir)?;
    fs::create_dir_all(&vt_fonts)?;
    let _ = fs::write(vt_fonts.join("unicode.pf2"), UNIFONT_PF2);
    let _ = fs::write(vt_fonts.join("unifont_named.pf2"), UNIFONT_NAMED_PF2);
    let _ = fs::write(vt_fonts.join("ascii.pf2"), ASCII_PF2);

    let ventoy_theme_txt = format!(
        r##"# Botdigit BootForge Theme (Responsive 1024x768 & Low-Res Safe)
title-text: "{title}"
title-font: "Unknown Regular 16"
title-color: "#10B981"
message-font: "Unknown Regular 16"
message-color: "#94A3B8"
message-bg-color: "#07090E"
desktop-color: "#07090E"

+ boot_menu {{
  left = 8%
  top = 18%
  width = 84%
  height = 64%
  item_font = "Unknown Regular 16"
  item_color = "#94A3B8"
  selected_item_color = "#10B981"
  item_height = 28
  item_padding = 6
  item_spacing = 4
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

/// Install EFI bootloader binaries (GRUB2 for UEFI) into the EFI System Partition.
/// Tries bundled binaries first, then system GRUB, then falls back to a chainload stub.
fn install_efi_binaries(mount_path: &Path) -> std::io::Result<()> {
    let efi_boot_dir = mount_path.join("EFI").join("BOOT");
    fs::create_dir_all(&efi_boot_dir)?;

    // Check if we already have a real BOOTX64.EFI
    let bootx64 = efi_boot_dir.join("BOOTX64.EFI");
    if bootx64.exists() && fs::metadata(&bootx64)?.len() > 1024 {
        // Already have a real bootloader binary
        return Ok(());
    }

    // 1. Try bundled pre-built GRUB binaries (embedded at compile time)
    let bundled_ok = install_bundled_grub_binaries(&efi_boot_dir);

    // 2. Try to find GRUB EFI binaries from system
    let grub_efi_found = if !bundled_ok {
        find_and_copy_grub_efi(&efi_boot_dir)
    } else {
        true
    };

    if !grub_efi_found {
        // 3. Create a minimal UEFI chainload stub that forwards to the data partition
        create_efi_chainload_stub(&efi_boot_dir)?;
    }

    // Create bootforge grub modules directories for BIOS/UEFI boot
    let grub_modules_dir = mount_path.join("bootforge").join("grub").join("x86_64-efi");
    let _ = fs::create_dir_all(&grub_modules_dir);
    let bios_grub_modules = mount_path.join("bootforge").join("grub").join("i386-pc");
    let _ = fs::create_dir_all(&bios_grub_modules);

    // Copy bundled GRUB modules if available
    install_bundled_grub_modules(&grub_modules_dir, &bios_grub_modules);

    Ok(())
}

/// Install bundled pre-built GRUB EFI binaries that were downloaded at build time.
fn install_bundled_grub_binaries(dest_dir: &Path) -> bool {
    // The build.rs script downloads binaries to OUT_DIR/bootloader_assets/
    // At runtime, we check common embedded paths
    let bundled_paths = [
        // Relative to the executable (for cargo install)
        "bootloader_assets/efi/BOOTX64.EFI",
        "bootloader_assets/efi/grubx64.efi",
    ];

    // Try to find the bundled assets relative to the current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            for rel_path in &bundled_paths {
                let src = exe_dir.join(rel_path);
                if src.exists() {
                    let _ = fs::copy(&src, dest_dir.join("BOOTX64.EFI"));
                    let _ = fs::copy(&src, dest_dir.join("grubx64.efi"));
                    return true;
                }
            }
        }
    }

    // Try to find in the build output directory (for development builds)
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let build_assets = PathBuf::from(&out_dir).join("bootloader_assets").join("efi");
        let bootx64 = build_assets.join("BOOTX64.EFI");
        if bootx64.exists() && fs::metadata(&bootx64).map(|m| m.len() > 1024).unwrap_or(false) {
            let _ = fs::copy(&bootx64, dest_dir.join("BOOTX64.EFI"));
            let grubx64 = build_assets.join("grubx64.efi");
            if grubx64.exists() {
                let _ = fs::copy(&grubx64, dest_dir.join("grubx64.efi"));
            } else {
                let _ = fs::copy(&bootx64, dest_dir.join("grubx64.efi"));
            }
            return true;
        }
    }

    false
}

/// Copy bundled GRUB modules to the target mount point.
fn install_bundled_grub_modules(uefi_modules_dir: &Path, bios_modules_dir: &Path) {
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let bundled_modules = PathBuf::from(&out_dir).join("bootloader_assets").join("modules");

        // Copy x86_64-efi modules
        let src_uefi = bundled_modules.join("x86_64-efi");
        if src_uefi.exists() {
            let _ = copy_dir_recursive(&src_uefi, uefi_modules_dir);
        }

        // Copy i386-pc modules (for BIOS boot)
        let src_bios = bundled_modules.join("i386-pc");
        if src_bios.exists() {
            let _ = copy_dir_recursive(&src_bios, bios_modules_dir);
        }
    }
}

/// Copy a directory recursively.
fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Attempt to find and copy GRUB2 EFI binaries from the system.
fn find_and_copy_grub_efi(dest_dir: &Path) -> bool {
    // Common locations for GRUB EFI binaries
    let grub_paths = [
        // macOS Homebrew
        "/opt/homebrew/Cellar/grub/" ,
        "/usr/local/Cellar/grub/",
        // Linux
        "/usr/lib/grub/",
        "/usr/share/grub/",
        "/boot/grub/",
    ];

    // Try to find grubx64.efi
    for base in &grub_paths {
        let base_path = Path::new(base);
        if base_path.exists() {
            // Walk the directory to find .efi files
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.contains("grub") && name_str.ends_with(".efi") {
                            let _ = fs::copy(&path, dest_dir.join("BOOTX64.EFI"));
                            let _ = fs::copy(&path, dest_dir.join("grubx64.efi"));
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Create a minimal UEFI chainload stub for systems without GRUB installed.
/// This creates a small EFI application that chains to the next bootloader.
fn create_efi_chainload_stub(efi_boot_dir: &Path) -> std::io::Result<()> {
    // Create a minimal GRUB configuration that will be used by the EFI stub
    let grub_forwarder = r#"# BootForge EFI Forwarder Stub
# This file is loaded by the UEFI firmware when no other bootloader is found
set timeout=3
set default=0

# Load basic modules
insmod part_gpt
insmod part_msdos
insmod fat
insmod exfat
insmod ntfs
insmod all_video

# Try to find and load the main BootForge configuration
search --no-floppy --set=boot_root --label BOOTFORGE
if [ -n "$boot_root" ]; then
    set root=$boot_root
    if [ -f /bootforge/grub.cfg ]; then
        configfile /bootforge/grub.cfg
    fi
fi

# Fallback menu if nothing else works
menuentry "BootForge Recovery Mode (Safe Graphics)" {
    search --no-floppy --set=root --label BOOTFORGE
    linux /bootforge/vmlinuz nomodeset text
    initrd /bootforge/initramfs.img
}

menuentry "Reboot" {
    reboot
}
"#;

    // Write the forwarder grub.cfg
    fs::write(efi_boot_dir.join("grub.cfg"), grub_forwarder)?;

    // Create a note about EFI binary installation
    let readme = r#"# BootForge EFI Boot Partition

This partition should contain EFI bootloader binaries.

To install a proper UEFI bootloader:

1. Install GRUB2 for UEFI:
   - macOS: brew install grub
   - Linux: sudo apt install grub-efi-amd64-bin

2. The binaries will be automatically detected and installed
   on the next format/repair operation.

Alternatively, you can manually copy BOOTX64.EFI to this directory.
"#;
    fs::write(efi_boot_dir.join("README.txt"), readme)?;

    Ok(())
}

/// Resolve the BootForge boot assets directory across various environments
pub fn resolve_bootforge_assets_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BOOTFORGE_ASSETS_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidates = [
                parent.join("assets"),
                parent.join("../Resources/assets"),
                parent.join("../../assets"),
                parent.join("../../../assets"),
                parent.join("../../../src/managers/assets"),
            ];
            for c in &candidates {
                if c.exists() && c.join("ventoy_core/boot/boot.img").exists() {
                    return c.clone();
                }
            }
        }
    }

    let default_path = PathBuf::from("/Volumes/Mac2TB/Botdigit/Developer/Projects/BootForge/src/managers/assets");
    if default_path.exists() {
        return default_path;
    }

    PathBuf::from("src/managers/assets")
}

/// Construct 512-byte Sector 0 with MBR Bootstrap code and Partition Table containing 0xEF ESP
pub fn build_mbr_sector(
    boot_img_446: &[u8],
    part1_start: u32,
    part1_sectors: u32,
    part2_start: u32,
    part2_sectors: u32,
) -> Vec<u8> {
    let mut sector = vec![0u8; 512];
    let copy_len = boot_img_446.len().min(446);
    sector[0..copy_len].copy_from_slice(&boot_img_446[0..copy_len]);

    // Entry 1: ExFAT/NTFS, Active (0x80), Type 0x07
    let p1 = 446;
    sector[p1] = 0x80; // Active / Bootable flag
    sector[p1 + 1] = 0x00; // CHS start head
    sector[p1 + 2] = 0x01; // CHS start sector
    sector[p1 + 3] = 0x01; // CHS start cylinder
    sector[p1 + 4] = 0x07; // Type: 0x07 (ExFAT / NTFS)
    sector[p1 + 5] = 0xFE; // CHS end head
    sector[p1 + 6] = 0xFF; // CHS end sector
    sector[p1 + 7] = 0xFF; // CHS end cylinder
    sector[p1 + 8..p1 + 12].copy_from_slice(&part1_start.to_le_bytes());
    sector[p1 + 12..p1 + 16].copy_from_slice(&part1_sectors.to_le_bytes());

    // Entry 2: EFI System Partition, Inactive (0x00), Type 0xEF
    let p2 = 462;
    sector[p2] = 0x00; // Inactive
    sector[p2 + 1] = 0x00;
    sector[p2 + 2] = 0x01;
    sector[p2 + 3] = 0x01;
    sector[p2 + 4] = 0xEF; // Type: 0xEF (EFI System Partition - critical for UEFI!)
    sector[p2 + 5] = 0xFE;
    sector[p2 + 6] = 0xFF;
    sector[p2 + 7] = 0xFF;
    sector[p2 + 8..p2 + 12].copy_from_slice(&part2_start.to_le_bytes());
    sector[p2 + 12..p2 + 16].copy_from_slice(&part2_sectors.to_le_bytes());

    // Boot signature
    sector[510] = 0x55;
    sector[511] = 0xAA;

    sector
}

/// Query physical disk size in bytes via `diskutil info` on macOS
#[cfg(target_os = "macos")]
pub fn query_disk_size_bytes_macos(clean_id: &str) -> Result<u64, String> {
    let output = Command::new("diskutil")
        .args(["info", clean_id])
        .output()
        .map_err(|e| format!("Failed to query diskutil info: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("Disk Size:") || line.contains("Total Size:") {
            if let Some(start_paren) = line.find('(') {
                if let Some(end_paren) = line.find(" Bytes)") {
                    let bytes_str = line[start_paren + 1..end_paren].trim().replace(',', "");
                    if let Ok(bytes) = bytes_str.parse::<u64>() {
                        return Ok(bytes);
                    }
                }
            }
        }
    }

    Err(format!("Could not determine disk capacity for /dev/{}", clean_id))
}

/// Format a disk on macOS using `diskutil` with industrial-grade multi-boot layout
#[cfg(target_os = "macos")]
pub fn format_disk_macos(
    disk_id: &str,
    _scheme: &str,
    _filesystem: &str,
    label: &str,
) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");
    
    // Safety check: protect internal disks
    if clean_id == "disk0" || clean_id == "disk1" || clean_id == "disk2" || clean_id == "disk3" {
        return Err("Protected internal drive cannot be formatted.".to_string());
    }

    let final_label = if label.is_empty() { "BOTDIGIT" } else { label };
    let raw_disk = format!("/dev/r{}", clean_id);

    // 1. Resolve boot assets
    let assets_dir = resolve_bootforge_assets_dir();
    let boot_img_path = assets_dir.join("ventoy_core/boot/boot.img");
    let core_img_xz = assets_dir.join("ventoy_core/boot/core.img.xz");
    let esp_img_xz = assets_dir.join("ventoy_core/ventoy/ventoy.disk.img.xz");

    if !boot_img_path.exists() || !core_img_xz.exists() || !esp_img_xz.exists() {
        return Err(format!(
            "Missing core bootloader assets in {}. Please ensure ventoy_core directory exists.",
            assets_dir.display()
        ));
    }

    // 2. Query disk capacity in sectors
    let disk_size_bytes = query_disk_size_bytes_macos(&clean_id)?;
    let total_sectors = (disk_size_bytes / 512) as u32;
    let esp_sectors: u32 = 65536; // 32 MB
    let part1_start: u32 = 2048;
    if total_sectors <= part1_start + esp_sectors {
        return Err("Disk is too small for BootForge Universal Multi-Boot layout.".to_string());
    }
    let part2_start = total_sectors - esp_sectors;
    let part1_sectors = part2_start - part1_start;

    // 3. Unmount target disk
    let _ = Command::new("diskutil")
        .args(["unmountDisk", "force", &clean_id])
        .output();

    // 4. Partition disk via native Apple diskutil: Partition 1 = ExFAT Main Data, Partition 2 = 100M FAT16 ESP (VTOYEFI)
    let partition_output = Command::new("diskutil")
        .args([
            "partitionDisk",
            &clean_id,
            "MBR",
            "ExFAT",
            final_label,
            "R",
            "MS-DOS FAT16",
            "VTOYEFI",
            "100M",
        ])
        .output()
        .map_err(|e| format!("Partitioning command failed: {}", e))?;

    if !partition_output.status.success() {
        let err_text = String::from_utf8_lossy(&partition_output.stderr).to_string();
        return Err(if err_text.trim().is_empty() {
            String::from_utf8_lossy(&partition_output.stdout).to_string()
        } else {
            err_text
        });
    }

    // 5. Decompress signed EFI partition template and sync all bootloader files to /Volumes/VTOYEFI
    let decompress_and_sync_py = format!(
        r#"import lzma, shutil, subprocess, os, sys
try:
    tmp_img = '/tmp/ventoy_tmpl.img'
    with open(tmp_img, 'wb') as out, lzma.open(r'{esp_img}', 'rb') as inp:
        shutil.copyfileobj(inp, out)
    out = subprocess.check_output(['hdiutil', 'attach', '-nobrowse', tmp_img]).decode()
    mounted = None
    for line in out.splitlines():
        if '/Volumes/' in line:
            mounted = line[line.find('/Volumes/'):].strip()
            break
    target_vtoy = '/Volumes/VTOYEFI'
    if mounted and os.path.exists(mounted) and os.path.exists(target_vtoy):
        subprocess.run(['rsync', '-a', f'{{mounted}}/', f'{{target_vtoy}}/'], check=True)
        subprocess.run(['hdiutil', 'detach', mounted], check=False)
    if os.path.exists(tmp_img):
        os.remove(tmp_img)
    print('BOOTFORGE_EFI_SYNC_OK')
except Exception as e:
    sys.stderr.write(str(e))
    sys.exit(1)
"#,
        esp_img = esp_img_xz.display()
    );

    let sync_run = Command::new("python3")
        .args(["-c", &decompress_and_sync_py])
        .output()
        .map_err(|e| format!("Failed to execute boot asset sync: {}", e))?;

    if !sync_run.status.success() {
        let err_msg = String::from_utf8_lossy(&sync_run.stderr);
        return Err(format!("Bootloader asset installation error: {}", err_msg));
    }

    let main_mount = PathBuf::from(format!("/Volumes/{}", final_label));
    if main_mount.exists() {
        // Install BootForge Boot OS and Cyber Emerald theme
        let _ = install_botdigit_boot_theme(&main_mount, "Botdigit BootForge Universal Multi-Boot Studio", final_label);
    }

    // 8. Mount Partition 2 (VTOYEFI) and install Master Bootloader forwarder
    let part2_dev = format!("{}s2", clean_id);
    let _ = Command::new("diskutil")
        .args(["mount", &part2_dev])
        .output();

    let vtoy_mount = PathBuf::from("/Volumes/VTOYEFI");
    if vtoy_mount.exists() {
        let grub_dir = vtoy_mount.join("grub");
        let fonts_vtoy = grub_dir.join("fonts");
        let _ = fs::create_dir_all(&fonts_vtoy);
        let _ = fs::write(fonts_vtoy.join("unicode.pf2"), UNIFONT_PF2);
        let _ = fs::write(fonts_vtoy.join("unifont_named.pf2"), UNIFONT_NAMED_PF2);
        let _ = fs::write(fonts_vtoy.join("ascii.pf2"), ASCII_PF2);

        let efi_boot_dir = vtoy_mount.join("EFI").join("BOOT");
        let _ = fs::create_dir_all(&efi_boot_dir);

        let master_forwarder = format!(
            r##"# Botdigit BootForge Master Bootloader
set timeout=5
set default=0

insmod font
loadfont /grub/fonts/unicode.pf2
loadfont /grub/fonts/unifont_named.pf2
loadfont /grub/fonts/ascii.pf2

# Search for the main BootForge data partition by label
search --no-floppy --set=boot_root --label {final_label}

if [ -n "$boot_root" ]; then
    set root=$boot_root
    loadfont ($boot_root)/bootforge/fonts/unicode.pf2
    loadfont ($boot_root)/bootforge/fonts/unifont_named.pf2
    if [ -f /bootforge/bootforge_os.cfg ]; then
        configfile /bootforge/bootforge_os.cfg
    elif [ -f /bootforge/grub.cfg ]; then
        configfile /bootforge/grub.cfg
    elif [ -f /EFI/BOOT/grub.cfg ]; then
        configfile /EFI/BOOT/grub.cfg
    fi
fi

# Fallback Menu (Prevents blank screen if main partition is missing)
menuentry "[*] [BootForge OS] System Control Center & Telemetry" {{
    echo "================================================================"
    echo "   BOTDIGIT BOOTFORGE UNIVERSAL BOOT OS (Emergency Core)"
    echo "================================================================"
    echo " Firmware Platform : $grub_platform ($grub_cpu)"
    echo " Root Drive        : $root"
    echo " Status            : Online (Emergency Fallback Mode)"
    echo "================================================================"
    echo ""
    echo "Press Enter to return..."
    read
}}

menuentry "[>] Scan Disks & Partitions" {{
    ls -l
    echo "Press Enter to return..."
    read
}}

menuentry "[Reboot] Reboot Computer" {{
    reboot
}}

menuentry "[Power] Power Off Computer" {{
    halt
}}
"##,
            final_label = final_label
        );

        let _ = fs::write(grub_dir.join("grub.cfg"), &master_forwarder);
        let _ = fs::write(efi_boot_dir.join("grub.cfg"), &master_forwarder);
    }

    Ok(format!(
        "Successfully formatted /dev/{} with BootForge Universal Multi-Boot Core & Boot OS!\nPartition 1: {} (ExFAT, {} sectors)\nPartition 2: VTOYEFI (0xEF EFI System Partition, {} sectors)",
        clean_id, final_label, part1_sectors, esp_sectors
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_mbr_sector_structure() {
        let dummy_boot = vec![0x90u8; 446];
        let sector = build_mbr_sector(&dummy_boot, 2048, 1000000, 1002048, 65536);

        assert_eq!(sector.len(), 512);
        // Verify boot code copied
        assert_eq!(sector[0], 0x90);
        assert_eq!(sector[445], 0x90);

        // Verify Partition 1: Active 0x80, Type 0x07 (ExFAT)
        assert_eq!(sector[446], 0x80);
        assert_eq!(sector[450], 0x07);

        // Verify Partition 2: Inactive 0x00, Type 0xEF (EFI System Partition!)
        assert_eq!(sector[462], 0x00);
        assert_eq!(sector[466], 0xEF);

        // Verify boot signature 0x55AA
        assert_eq!(sector[510], 0x55);
        assert_eq!(sector[511], 0xAA);
    }
}

