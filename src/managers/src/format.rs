//! Disk formatting, partition management and custom bootloader theme installer.

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

/// Install Botdigit BootForge custom theme and EFI bootloader configuration onto target USB volume
pub fn install_botdigit_boot_theme(mount_path: &Path, title: &str) -> std::io::Result<()> {
    // 1. Create Ventoy customization config if Ventoy or multi-boot is used
    let ventoy_dir = mount_path.join("ventoy");
    let ventoy_theme_dir = ventoy_dir.join("theme");
    fs::create_dir_all(&ventoy_theme_dir)?;

    let ventoy_json = r##"{
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
      "VTOY_SECONDARY_TIMEOUT": "10"
    }
  ]
}
"##;
    fs::write(ventoy_dir.join("ventoy.json"), ventoy_json)?;

    let theme_txt = format!(
        "# Botdigit BootForge Theme\ntitle-text: \"⚡ {}\"\ntitle-font: \"Unifont Regular 20\"\ntitle-color: \"#10B981\"\nmessage-font: \"Unifont Regular 14\"\nmessage-color: \"#94A3B8\"\nmessage-bg-color: \"#07090E\"\n\n+ boot_menu {{\n  left = 15%\n  top = 22%\n  width = 70%\n  height = 58%\n  item_font = \"Unifont Regular 16\"\n  item_color = \"#94A3B8\"\n  selected_item_color = \"#FFFFFF\"\n  item_height = 36\n  item_padding = 10\n  item_spacing = 8\n}}\n",
        title
    );
    fs::write(ventoy_theme_dir.join("theme.txt"), theme_txt)?;

    // 2. Create native BootForge GRUB2 / EFI configuration
    let bootforge_dir = mount_path.join("bootforge");
    let bootforge_theme_dir = bootforge_dir.join("themes").join("default");
    fs::create_dir_all(&bootforge_theme_dir)?;

    let grub_cfg = r##"# ⚡ Botdigit BootForge Universal Multi-Boot Menu
set timeout=10
set default=0
insmod part_gpt
insmod part_msdos
insmod fat
insmod exfat
insmod ntfs
insmod all_video
insmod gfxterm

set menu_color_normal=light-gray/black
set menu_color_highlight=green/dark-gray

menuentry "⚡ Botdigit BootForge — Scan All Bootable ISOs" {
    echo "Scanning drive for installed operating systems..."
}
"##;
    fs::write(bootforge_dir.join("grub.cfg"), grub_cfg)?;
    fs::write(bootforge_theme_dir.join("theme.txt"), format!("title-text: \"⚡ {}\"\n", title))?;

    // 3. Create images directory structure
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

    let scheme_arg = match scheme.to_uppercase().as_str() {
        "MBR" => "MBRFormat",
        _ => "GPTFormat",
    };

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

#[cfg(not(target_os = "macos"))]
pub fn format_disk_macos(
    _disk_id: &str,
    _scheme: &str,
    _filesystem: &str,
    _label: &str,
) -> Result<String, String> {
    Ok("Mock formatted on non-macOS".to_string())
}
