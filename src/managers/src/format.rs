//! Drive creation: writes the standard BootForge multi-boot layout and its configuration.
//!
//! How a BootForge drive boots (see [`crate::boot_layout`] for the on-disk layout):
//!
//! * The boot loader on the 32 MiB ESP (bundled `ventoy_core`, signed for Secure Boot)
//!   scans the data partition at boot time and lists **every** ISO/WIM/VHD/IMG/EFI file
//!   it finds, on Legacy BIOS and on UEFI. Nothing has to be registered per image.
//! * BootForge adds its theme, `ventoy.json` options and the F6 "BootForge Tools" page
//!   (`/ventoy/ventoy_grub.cfg`) on the data partition. It never edits the ESP's own
//!   `grub.cfg` — replacing it is what breaks image scanning.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// F6 tools page shown inside the boot menu.
pub const BOOTFORGE_TOOLS_CFG: &str = include_str!("../assets/bootforge/ventoy_grub.cfg");
pub static UNIFONT_PF2: &[u8] = include_bytes!("../assets/fonts/unicode.pf2");
pub static ASCII_PF2: &[u8] = include_bytes!("../assets/fonts/ascii.pf2");
pub static UNIFONT_NAMED_PF2: &[u8] = include_bytes!("../assets/fonts/unifont_named.pf2");

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormatOptions {
    pub disk_id: String,
    pub mode: String,             // "fresh_multiboot", "add_isos", "raw_flash"
    pub partition_scheme: String, // always the standard MBR layout; kept for API compatibility
    pub filesystem: String,       // always exFAT; kept for API compatibility
    pub volume_label: String,
    pub theme_name: String,
}

/// Normalise a volume label to something exFAT and every firmware accepts:
/// up to 11 characters, A–Z 0–9 `_` `-`.
pub fn sanitize_volume_label(label: &str) -> String {
    let clean: String = label
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(11)
        .collect();
    if clean.is_empty() { "BOOTFORGE".to_string() } else { clean }
}

/// Install BootForge's theme, boot options and F6 tools page onto a mounted data partition.
///
/// Safe to run again on an existing drive (used by the repair function): image files are
/// never touched.
pub fn install_botdigit_boot_theme(mount_path: &Path, title: &str, _label: &str) -> std::io::Result<()> {
    let ventoy_dir = mount_path.join("ventoy");
    let ventoy_theme_dir = ventoy_dir.join("theme");
    let vt_fonts = ventoy_theme_dir.join("fonts");
    fs::create_dir_all(&vt_fonts)?;
    fs::write(vt_fonts.join("unicode.pf2"), UNIFONT_PF2)?;
    fs::write(vt_fonts.join("unifont_named.pf2"), UNIFONT_NAMED_PF2)?;
    fs::write(vt_fonts.join("ascii.pf2"), ASCII_PF2)?;

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

    // Keep any options the user added by hand; only (re)set the keys BootForge owns.
    let json_path = ventoy_dir.join("ventoy.json");
    let mut cfg: serde_json::Value = fs::read_to_string(&json_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .filter(|v: &serde_json::Value| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    cfg["theme"] = serde_json::json!({
        "file": "/ventoy/theme/theme.txt",
        // GUI on capable firmware; the loader falls back to text mode on its own.
        "display_mode": "GUI",
        "gfxmode": "1024x768",
        "ventoy_color": "#10B981",
        "ventoy_left": "5%",
        "ventoy_top": "92%"
    });
    if cfg.get("control").is_none() {
        cfg["control"] = serde_json::json!([
            // Lets Windows 11 Setup run on older PCs (no TPM 2.0 / Secure Boot / 4 GB RAM checks).
            { "VTOY_WIN11_BYPASS_CHECK": "1" },
            // Shows the Normal / grub2 / memdisk choice for images that need a different method.
            { "VTOY_SECONDARY_BOOT_MENU": "1" }
        ]);
    }
    fs::write(&json_path, serde_json::to_string_pretty(&cfg).unwrap_or_default())?;

    fs::write(ventoy_dir.join("ventoy_grub.cfg"), BOOTFORGE_TOOLS_CFG)?;

    // Suggested folders (images are found anywhere on the partition).
    for sub in ["windows", "linux", "macos-tools", "rescue", "other"] {
        fs::create_dir_all(mount_path.join("images").join(sub))?;
    }
    fs::create_dir_all(mount_path.join("bootforge").join("tools"))?;
    fs::write(mount_path.join("README-BOOTFORGE.txt"), DRIVE_README.replace("{title}", title))?;
    Ok(())
}

const DRIVE_README: &str = "{title}\r
\r
HOW TO USE THIS DRIVE\r
1. Copy .iso / .wim / .vhd / .vhdx / .img files anywhere on this drive\r
   (the images\\ folders are only suggestions). No other setup is needed.\r
2. Plug the drive into the computer you want to start and power it on while\r
   pressing its boot-menu key (F12 Dell/Lenovo, F9 HP, F8 ASUS, F11 MSI,\r
   Esc/F12 Acer, Option/Alt on an Intel Mac).\r
3. Choose the USB entry. Pick the image in the menu.\r
   - UEFI entry   = modern PCs (2012+).  Legacy/USB-HDD entry = older PCs.\r
   - Press F6 for BootForge Tools (system info, boot installed OS, memtest).\r
4. First boot with Secure Boot ON: a blue 'Verification failed' screen appears\r
   once. Choose Enroll key from disk > VTOYEFI > ENROLL_THIS_KEY_IN_MOKMANAGER.cer\r
   > Continue > Yes > Reboot. Or switch Secure Boot off in firmware setup.\r
\r
Do not delete the 'ventoy' folder or the small VTOYEFI partition.\r
";

// ── Assets ────────────────────────────────────────────────────────

/// Resolve the BootForge boot assets directory across various environments.
pub fn resolve_bootforge_assets_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BOOTFORGE_ASSETS_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }

    let is_assets = |p: &Path| p.join("ventoy_core/ventoy/ventoy.disk.img.xz").exists();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidates = [
                parent.join("assets"),
                parent.join("../Resources/assets"),
                parent.join("../../assets"),
                parent.join("../../src/managers/assets"),
                parent.join("../../../src/managers/assets"),
            ];
            for c in &candidates {
                if is_assets(c) {
                    return c.clone();
                }
            }
        }
    }

    // Compile-time location of this crate (works for `cargo run` / dev builds).
    let crate_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    if is_assets(&crate_assets) {
        return crate_assets;
    }

    PathBuf::from("src/managers/assets")
}

/// Decompress an `.xz` file into memory using whatever decoder the system has.
///
/// Tries `xz` (Homebrew / system) first, then Python's built-in `lzma` module (present on
/// macOS with the Command Line Tools and on every Linux). Absolute paths are tried as
/// well because privileged helpers run with a minimal `PATH`.
pub fn decompress_xz_file(path: &Path) -> Result<Vec<u8>, String> {
    let xz_candidates = ["xz", "/opt/homebrew/bin/xz", "/usr/local/bin/xz", "/usr/bin/xz"];
    for xz in xz_candidates {
        if let Ok(out) = Command::new(xz).args(["-dc"]).arg(path).stderr(Stdio::null()).output() {
            if out.status.success() && !out.stdout.is_empty() {
                return Ok(out.stdout);
            }
        }
    }
    for py in ["python3", "/usr/bin/python3", "/opt/homebrew/bin/python3"] {
        let script = "import lzma,sys;sys.stdout.buffer.write(lzma.open(sys.argv[1]).read())";
        if let Ok(out) = Command::new(py).args(["-I", "-c", script]).arg(path).stderr(Stdio::null()).output() {
            if out.status.success() && !out.stdout.is_empty() {
                return Ok(out.stdout);
            }
        }
    }
    Err(format!(
        "Could not decompress {}. Install xz (`brew install xz`) or the Xcode Command Line Tools.",
        path.display()
    ))
}

/// Load the three boot payloads from the assets directory.
pub fn load_boot_payloads(assets: &Path) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
    let core = assets.join("ventoy_core");
    let boot = fs::read(core.join("boot/boot.img"))
        .map_err(|e| format!("Missing boot/boot.img in {}: {e}", core.display()))?;
    let core_img = decompress_xz_file(&core.join("boot/core.img.xz"))?;
    let esp = decompress_xz_file(&core.join("ventoy/ventoy.disk.img.xz"))?;
    Ok((boot, core_img, esp))
}

// ── macOS ─────────────────────────────────────────────────────────

/// Query physical disk size in bytes via `diskutil info` on macOS.
#[cfg(target_os = "macos")]
pub fn query_disk_size_bytes_macos(clean_id: &str) -> Result<u64, String> {
    let output = Command::new("diskutil")
        .args(["info", clean_id])
        .output()
        .map_err(|e| format!("Failed to query diskutil info: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("Disk Size:") || line.contains("Total Size:") {
            if let (Some(start), Some(end)) = (line.find('('), line.find(" Bytes)")) {
                if let Ok(bytes) = line[start + 1..end].trim().replace(',', "").parse::<u64>() {
                    return Ok(bytes);
                }
            }
        }
    }
    Err(format!("Could not determine disk capacity for /dev/{}", clean_id))
}

#[cfg(target_os = "macos")]
fn diskutil(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("diskutil").args(args).output()
}

/// **Privileged step** (needs root): erase `disk_id` and write the complete standard layout,
/// then create the exFAT data partition and mount it. Used by the `write_mbr_core --full` helper.
#[cfg(target_os = "macos")]
pub fn write_standard_drive_macos(disk_id: &str, label: &str) -> Result<String, String> {
    use crate::boot_layout::*;
    use std::fs::OpenOptions;

    let clean_id = disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");
    crate::device::ensure_safe_write_target(&clean_id)?;
    let label = sanitize_volume_label(label);

    let (boot_img, core_img, esp_img) = load_boot_payloads(&resolve_bootforge_assets_dir())?;
    let size = query_disk_size_bytes_macos(&clean_id)?;
    let layout = plan_standard_layout(size / SECTOR_SIZE, 0)?;

    let unmount = diskutil(&["unmountDisk", "force", &clean_id]).map_err(|e| e.to_string())?;
    if !unmount.status.success() {
        return Err(format!(
            "Could not unmount /dev/{clean_id}: {}. Close any app using the drive and retry.",
            String::from_utf8_lossy(&unmount.stderr).trim()
        ));
    }

    let raw = format!("/dev/r{clean_id}");
    {
        let mut dev = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&raw)
            .map_err(|e| format!("Cannot open {raw} for writing ({e}). Administrator rights are required."))?;
        let payloads = BootPayloads { boot_img: &boot_img, core_img: &core_img, esp_img: &esp_img };
        write_standard_layout(&mut dev, &layout, &payloads, new_disk_uuid())
            .map_err(|e| format!("Writing boot structures to {raw} failed: {e}"))?;
        let problems = verify_standard_layout(&mut dev, &layout).map_err(|e| e.to_string())?;
        if !problems.is_empty() {
            return Err(format!("Verification after write failed: {}", problems.join("; ")));
        }
        dev.flush().ok();
    } // closing the whole-disk node makes macOS re-read the new partition table

    // Wait for the kernel to publish the new partition 1, then format it.
    let part1 = format!("{clean_id}s1");
    let expected = layout.part1_sectors * SECTOR_SIZE;
    let mut ready = false;
    for _ in 0..40 {
        let _ = diskutil(&["unmountDisk", "force", &clean_id]);
        if let Ok(out) = diskutil(&["info", &part1]) {
            let t = String::from_utf8_lossy(&out.stdout);
            if t.contains(&format!("({expected} Bytes)")) {
                ready = true;
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    if !ready {
        return Err(format!(
            "Boot structures were written, but macOS did not re-read the partition table of /dev/{clean_id}. \
             Unplug and re-plug the drive, then run Format again."
        ));
    }

    let newfs = Command::new("/sbin/newfs_exfat")
        .args(["-v", &label, &format!("/dev/r{part1}")])
        .output()
        .map_err(|e| format!("newfs_exfat failed to start: {e}"))?;
    if !newfs.status.success() {
        return Err(format!("Creating the exFAT data partition failed: {}", String::from_utf8_lossy(&newfs.stderr)));
    }
    let _ = diskutil(&["mountDisk", &clean_id]);

    Ok(format!(
        "/dev/{clean_id}: MBR boot code + BIOS stage 2, partition 1 '{label}' exFAT ({:.1} GB), partition 2 VTOYEFI 32 MiB ESP",
        expected as f64 / 1e9
    ))
}

/// Path of the privileged helper binary (`write_mbr_core`), next to the running executable.
pub fn resolve_disk_helper_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["write_mbr_core", "../Resources/write_mbr_core"] {
                let c = dir.join(name);
                if c.exists() {
                    return c;
                }
            }
        }
    }
    PathBuf::from("write_mbr_core")
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Run a shell command as root through the standard macOS administrator password dialog.
#[cfg(target_os = "macos")]
pub fn run_as_admin_macos(shell_cmd: &str, prompt: &str) -> Result<String, String> {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "do shell script \"{}\" with prompt \"{}\" with administrator privileges",
        esc(shell_cmd),
        esc(prompt)
    );
    let out = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Could not request administrator rights: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        if err.contains("-128") {
            Err("Cancelled: the administrator password was not entered.".to_string())
        } else {
            Err(err.trim().to_string())
        }
    }
}

#[cfg(target_os = "macos")]
fn running_as_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

/// Create a BootForge drive on macOS: erase, write the standard layout (Legacy BIOS + UEFI +
/// Secure Boot), format the data partition and install BootForge's configuration.
///
/// Asks for the administrator password once through the native macOS dialog.
#[cfg(target_os = "macos")]
pub fn format_disk_macos(disk_id: &str, _scheme: &str, _filesystem: &str, label: &str) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");
    crate::device::ensure_safe_write_target(&clean_id)?;
    let label = sanitize_volume_label(label);

    let summary = if running_as_root() {
        write_standard_drive_macos(&clean_id, &label)?
    } else {
        let helper = resolve_disk_helper_path();
        if !helper.exists() {
            return Err(format!(
                "Disk helper not found at {}. Build it with: cargo build --release --bin write_mbr_core",
                helper.display()
            ));
        }
        let mut cmd = format!(
            "{} --full {} {}",
            shell_quote(&helper.to_string_lossy()),
            shell_quote(&format!("/dev/{clean_id}")),
            shell_quote(&label)
        );
        // Forward explicit overrides to the root helper (its environment is reset).
        for var in ["BOOTFORGE_ASSETS_DIR", "BOOTFORGE_ALLOW_DISK_IMAGES"] {
            if let Ok(v) = std::env::var(var) {
                cmd = format!("{var}={} {cmd}", shell_quote(&v));
            }
        }
        run_as_admin_macos(&cmd, &format!("BootForge needs to erase /dev/{clean_id} and write boot code."))?
    };

    let mount = PathBuf::from(format!("/Volumes/{label}"));
    for _ in 0..20 {
        if mount.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    if !mount.exists() {
        let _ = diskutil(&["mountDisk", &clean_id]);
    }
    if !mount.exists() {
        return Err(format!("Drive was written but /Volumes/{label} did not mount. Re-plug the drive."));
    }
    install_botdigit_boot_theme(&mount, "Botdigit BootForge", &label)
        .map_err(|e| format!("Drive was written but configuration could not be installed: {e}"))?;

    Ok(format!("{}\nReady: copy image files to /Volumes/{label}/", summary.trim()))
}

#[cfg(not(target_os = "macos"))]
pub fn format_disk_macos(disk_id: &str, _scheme: &str, _filesystem: &str, _label: &str) -> Result<String, String> {
    Err(format!(
        "Creating drives is implemented for macOS in this build. On Linux, run the bundled \
         ventoy_core/Ventoy2Disk.sh -i {disk_id}, then copy BootForge's ventoy/ folder onto the drive."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_firmware_safe() {
        assert_eq!(sanitize_volume_label("My Boot Drive!"), "MYBOOTDRIVE");
        assert_eq!(sanitize_volume_label(""), "BOOTFORGE");
        assert_eq!(sanitize_volume_label("botdigit"), "BOTDIGIT");
        assert_eq!(sanitize_volume_label("a-very-long-label-name").len(), 11);
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("/Volumes/My Disk/it's"), "'/Volumes/My Disk/it'\\''s'");
    }

    #[test]
    fn theme_install_preserves_user_options_and_images() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("ventoy")).unwrap();
        fs::write(root.join("ventoy/ventoy.json"), r#"{"menu_alias":[{"image":"/a.iso","alias":"A"}]}"#).unwrap();
        fs::create_dir_all(root.join("images/linux")).unwrap();
        fs::write(root.join("images/linux/a.iso"), b"iso").unwrap();

        install_botdigit_boot_theme(root, "Test", "T").unwrap();

        let cfg: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("ventoy/ventoy.json")).unwrap()).unwrap();
        assert_eq!(cfg["menu_alias"][0]["alias"], "A");
        assert_eq!(cfg["theme"]["file"], "/ventoy/theme/theme.txt");
        assert!(root.join("ventoy/ventoy_grub.cfg").exists());
        assert_eq!(fs::read(root.join("images/linux/a.iso")).unwrap(), b"iso");
        // Must never write a boot loader config that would shadow the scanner.
        assert!(!root.join("EFI").exists());
    }

    #[test]
    fn bundled_payloads_have_expected_sizes() {
        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let Ok((boot, core, esp)) = load_boot_payloads(&assets) else {
            eprintln!("skipping: no xz decoder available");
            return;
        };
        assert_eq!(boot.len(), 512);
        assert!(!core.is_empty() && core.len() <= crate::boot_layout::CORE_MAX_BYTES);
        assert_eq!(esp.len() as u64, crate::boot_layout::ESP_SECTORS * 512);
    }
}
