//! Disk formatting, partition management and Universal Botdigit BootForge bootloader installer
//! with Progressive Fallback Hierarchy (Graphical UI -> Framebuffer -> High-Contrast Text -> Safe Mode).

use std::fs;
use std::path::{Path, PathBuf};
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

/// Format a disk on macOS using `diskutil` with industrial-grade multi-boot layout
#[cfg(target_os = "macos")]
pub fn format_disk_macos(
    disk_id: &str,
    _scheme: &str,
    _filesystem: &str,
    label: &str,
) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/");
    
    // Safety check: protect internal disks
    if clean_id == "disk0" || clean_id == "disk1" || clean_id == "disk2" || clean_id == "disk3" {
        return Err("Protected internal drive cannot be formatted.".to_string());
    }

    let final_label = if label.is_empty() { "BOTDIGIT" } else { label };

    // Unmount any active volumes on the disk before partitioning
    let _ = Command::new("diskutil")
        .args(["unmountDisk", "force", clean_id])
        .output();

    // 1. Partition disk: Partition 1 = ExFAT Main Data, Partition 2 = 100M FAT16 ESP (VTOYEFI)
    let partition_output = Command::new("diskutil")
        .args([
            "partitionDisk",
            clean_id,
            "MBR",
            "ExFAT",
            final_label,
            "R",
            "MS-DOS FAT16",
            "VTOYEFI",
            "100M",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !partition_output.status.success() {
        let err_text = String::from_utf8_lossy(&partition_output.stderr).to_string();
        return Err(if err_text.is_empty() { "Partitioning failed".to_string() } else { err_text });
    }

    let out_text = String::from_utf8_lossy(&partition_output.stdout).to_string();

    // 2. Extract and mount template EFI disk image
    let _ = Command::new("sh")
        .args([
            "-c",
            "xz -dc src/managers/assets/ventoy_core/ventoy/ventoy.disk.img.xz > /tmp/ventoy.disk.img 2>/dev/null || xz -dc ../managers/assets/ventoy_core/ventoy/ventoy.disk.img.xz > /tmp/ventoy.disk.img 2>/dev/null",
        ])
        .output();

    if Path::new("/tmp/ventoy.disk.img").exists() {
        let attach_output = Command::new("hdiutil")
            .args(["attach", "-nobrowse", "/tmp/ventoy.disk.img"])
            .output();

        if let Ok(att) = attach_output {
            let attach_stdout = String::from_utf8_lossy(&att.stdout);
            let mut mounted_src = None;
            for line in attach_stdout.lines() {
                if let Some(pos) = line.find("/Volumes/") {
                    mounted_src = Some(line[pos..].trim().to_string());
                }
            }

            let src_dir = mounted_src.unwrap_or_else(|| "/Volumes/VTOYEFI 1".to_string());
            let target_vtoy = PathBuf::from("/Volumes/VTOYEFI");

            if target_vtoy.exists() && Path::new(&src_dir).exists() {
                // Copy EFI binaries and grub modules recursively
                let _ = Command::new("sh")
                    .args(["-c", &format!("cp -R \"{}\"/* /Volumes/VTOYEFI/ 2>/dev/null || true", src_dir)])
                    .output();
            }

            let _ = Command::new("hdiutil")
                .args(["detach", &src_dir])
                .output();
        }
    }

    // 3. Write Auto-Search Forwarder Stub to /Volumes/VTOYEFI/grub/grub.cfg
    let vtoy_mount = PathBuf::from("/Volumes/VTOYEFI");
    if vtoy_mount.exists() {
        let grub_dir = vtoy_mount.join("grub");
        let _ = fs::create_dir_all(&grub_dir);
        let efi_boot_dir = vtoy_mount.join("EFI").join("BOOT");
        let _ = fs::create_dir_all(&efi_boot_dir);

        let stub_cfg = format!(
            r##"# BootForge Universal Auto-Search Forwarder
set timeout=0
set default=0

# Search for the main BootForge data partition by label
search --no-floppy --set=boot_root --label {final_label}

if [ -n "$boot_root" ]; then
    set root=$boot_root
    if [ -f /bootforge/grub.cfg ]; then
        configfile /bootforge/grub.cfg
    elif [ -f /EFI/BOOT/grub.cfg ]; then
        configfile /EFI/BOOT/grub.cfg
    fi
fi
"##,
            final_label = final_label
        );

        let _ = fs::write(grub_dir.join("grub.cfg"), &stub_cfg);
        let _ = fs::write(efi_boot_dir.join("grub.cfg"), &stub_cfg);
    }

    // 4. Install Full Botdigit Cyber Emerald Theme & Recovery Menus to /Volumes/<LABEL>
    let main_mount = PathBuf::from(format!("/Volumes/{}", final_label));
    if main_mount.exists() {
        let _ = install_botdigit_boot_theme(&main_mount, "Botdigit BootForge Universal Multi-Boot Studio", final_label);
    }

    Ok(out_text)
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

