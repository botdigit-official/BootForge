//! Real-Time Live Virtual Machine (QEMU) Boot Tester for Botdigit BootForge.

use std::process::Command;

/// Launch a live real-time QEMU testing window booting the target USB drive directly
pub fn launch_live_vm_test(disk_id: &str, is_uefi: bool) -> Result<String, String> {
    let clean_id = disk_id.trim_start_matches("/dev/");
    let raw_dev = format!("/dev/{}", clean_id);

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(["-m", "2048"]);
    cmd.args(["-smp", "2"]);
    cmd.args(["-vga", "std"]);
    cmd.args(["-name", "⚡ Botdigit BootForge Live Test VM"]);
    cmd.args(["-drive", &format!("file={},format=raw,cache=none", raw_dev)]);

    if is_uefi {
        // Try common OVMF / EDK2 locations
        let ovmf_paths = [
            "/opt/homebrew/share/qemu/edk2-x86_64-code.fd",
            "/usr/local/share/qemu/edk2-x86_64-code.fd",
            "/opt/homebrew/Cellar/qemu/*/share/qemu/edk2-x86_64-code.fd",
        ];
        for p in &ovmf_paths {
            if std::path::Path::new(p).exists() {
                cmd.args(["-bios", p]);
                break;
            }
        }
    }

    match cmd.spawn() {
        Ok(_) => Ok(format!("Live VM window launched testing /dev/{} (Legacy BIOS / UEFI)", clean_id)),
        Err(e) => Err(format!("Could not start QEMU: {}. Make sure qemu-system-x86_64 is installed.", e)),
    }
}
