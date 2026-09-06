//! Windows WIM and BCD Engine module for BootForge.
//! Handles Windows setup bypasses (TPM 2.0 / Secure Boot / RAM / MSA checks),
//! unattend.xml generation, and WIMBOOT boot entries.

/// Configuration for automated Windows Setup & bypasses.
#[derive(Debug, Clone)]
pub struct WindowsBypassOptions {
    pub bypass_tpm_secureboot: bool,
    pub bypass_ram_check: bool,
    pub bypass_cpu_check: bool,
    pub bypass_online_account: bool,
    pub auto_create_admin: bool,
    pub admin_username: String,
}

impl Default for WindowsBypassOptions {
    fn default() -> Self {
        Self {
            bypass_tpm_secureboot: true,
            bypass_ram_check: true,
            bypass_cpu_check: true,
            bypass_online_account: true,
            auto_create_admin: false,
            admin_username: "User".to_string(),
        }
    }
}

/// Generate an unattend.xml or registry injection script for Windows 11 setup bypass.
pub fn generate_windows_11_bypass_reg(options: &WindowsBypassOptions) -> String {
    let mut reg = String::new();
    reg.push_str("Windows Registry Editor Version 5.00\r\n\r\n");
    reg.push_str("[HKEY_LOCAL_MACHINE\\SYSTEM\\Setup\\LabConfig]\r\n");

    if options.bypass_tpm_secureboot {
        reg.push_str("\"BypassTPMCheck\"=dword:00000001\r\n");
        reg.push_str("\"BypassSecureBootCheck\"=dword:00000001\r\n");
    }
    if options.bypass_ram_check {
        reg.push_str("\"BypassRAMCheck\"=dword:00000001\r\n");
    }
    if options.bypass_cpu_check {
        reg.push_str("\"BypassCPUCheck\"=dword:00000001\r\n");
        reg.push_str("\"BypassStorageCheck\"=dword:00000001\r\n");
    }
    if options.bypass_online_account {
        reg.push_str("\"BypassNRO\"=dword:00000001\r\n");
    }

    reg
}

/// Generate an unattended XML answer file for automated local Windows setup.
pub fn generate_autounattend_xml(options: &WindowsBypassOptions) -> String {
    let username = &options.admin_username;
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<unattend xmlns="urn:schemas-microsoft-com:unattend">
    <settings pass="windowsPE">
        <component name="Microsoft-Windows-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
            <RunSynchronous>
                <RunSynchronousCommand wcm:action="add">
                    <Order>1</Order>
                    <Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassTPMCheck /t REG_DWORD /d 1 /f</Path>
                </RunSynchronousCommand>
                <RunSynchronousCommand wcm:action="add">
                    <Order>2</Order>
                    <Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassSecureBootCheck /t REG_DWORD /d 1 /f</Path>
                </RunSynchronousCommand>
                <RunSynchronousCommand wcm:action="add">
                    <Order>3</Order>
                    <Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassRAMCheck /t REG_DWORD /d 1 /f</Path>
                </RunSynchronousCommand>
                <RunSynchronousCommand wcm:action="add">
                    <Order>4</Order>
                    <Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassCPUCheck /t REG_DWORD /d 1 /f</Path>
                </RunSynchronousCommand>
            </RunSynchronous>
        </component>
    </settings>
    <settings pass="oobeSystem">
        <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
            <UserAccounts>
                <LocalAccounts>
                    <LocalAccount wcm:action="add">
                        <Description>Local Administrator</Description>
                        <DisplayName>{username}</DisplayName>
                        <Group>Administrators</Group>
                        <Name>{username}</Name>
                    </LocalAccount>
                </LocalAccounts>
            </UserAccounts>
            <OOBE>
                <HideEULAPage>true</HideEULAPage>
                <HideOnlineAccountScreens>true</HideOnlineAccountScreens>
                <HideWirelessSetupInOOBE>true</HideWirelessSetupInOOBE>
                <ProtectYourPC>3</ProtectYourPC>
            </OOBE>
        </component>
    </settings>
</unattend>
"#
    )
}

/// Split WIM strategy plan for FAT32 filesystems.
#[derive(Debug, Clone)]
pub struct SplitWimPlan {
    pub source_wim_bytes: u64,
    pub chunk_size_mb: u64,
    pub total_chunks: usize,
    pub base_filename: String,
}

/// Calculate Split WIM (.swm) chunks for an install.wim file exceeding 4GB.
pub fn calculate_swm_chunks(wim_size_bytes: u64, max_chunk_mb: u64) -> SplitWimPlan {
    let chunk_size_bytes = max_chunk_mb * 1024 * 1024;
    let total_chunks = if chunk_size_bytes > 0 {
        ((wim_size_bytes + chunk_size_bytes - 1) / chunk_size_bytes) as usize
    } else {
        1
    };

    SplitWimPlan {
        source_wim_bytes: wim_size_bytes,
        chunk_size_mb: max_chunk_mb,
        total_chunks: total_chunks.max(1),
        base_filename: "install.swm".to_string(),
    }
}
