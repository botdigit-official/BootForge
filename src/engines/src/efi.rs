//! EFI & UEFI Bootloader Engine module for BootForge.
//! Handles EFI binary layouts, Shim loader chaining, and NVRAM boot descriptors.

use std::path::{Path, PathBuf};

/// EFI boot binary layout structure.
#[derive(Debug, Clone)]
pub struct EfiLayout {
    pub esp_mount_path: PathBuf,
    pub boot_binary_x64: Option<PathBuf>,
    pub boot_binary_aa64: Option<PathBuf>,
    pub shim_binary: Option<PathBuf>,
    pub grub_binary: Option<PathBuf>,
    pub mok_manager: Option<PathBuf>,
}

impl EfiLayout {
    /// Create layout given an ESP mount point.
    pub fn new(esp_mount: impl AsRef<Path>) -> Self {
        let root = esp_mount.as_ref().to_path_buf();
        let efi_boot = root.join("EFI").join("BOOT");
        Self {
            esp_mount_path: root,
            boot_binary_x64: Some(efi_boot.join("BOOTX64.EFI")),
            boot_binary_aa64: Some(efi_boot.join("BOOTAA64.EFI")),
            shim_binary: Some(efi_boot.join("shimx64.efi")),
            grub_binary: Some(efi_boot.join("grubx64.efi")),
            mok_manager: Some(efi_boot.join("mmx64.efi")),
        }
    }

    /// Generate directory structure required on the EFI system partition.
    pub fn create_efi_skeleton(&self) -> std::io::Result<()> {
        let efi_boot = self.esp_mount_path.join("EFI").join("BOOT");
        std::fs::create_dir_all(&efi_boot)?;
        let bootforge_dir = self.esp_mount_path.join("bootforge");
        std::fs::create_dir_all(&bootforge_dir)?;
        Ok(())
    }
}
