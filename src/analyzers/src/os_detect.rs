//! Operating system identification heuristics and signatures.

use bootforge_core::types::{ImageType, OsFamily, OsInfo};

/// Identify operating system from volume identifier, file names, or string markers.
pub fn identify_os_from_label(label: &str) -> OsInfo {
    let lower = label.to_lowercase();

    // 1. Windows Installers & WinPE
    if lower.contains("windows 11") || lower.contains("win11") || lower.contains("win_11") {
        return OsInfo {
            family: OsFamily::Windows,
            distribution: Some("Windows".to_string()),
            version: Some("11".to_string()),
            codename: Some("Sun Valley".to_string()),
            image_type: ImageType::Installer,
            display_name: "Windows 11".to_string(),
        };
    }
    if lower.contains("windows 10") || lower.contains("win10") || lower.contains("win_10") {
        return OsInfo {
            family: OsFamily::Windows,
            distribution: Some("Windows".to_string()),
            version: Some("10".to_string()),
            codename: None,
            image_type: ImageType::Installer,
            display_name: "Windows 10".to_string(),
        };
    }
    if lower.contains("windows server") || lower.contains("winsrv") || lower.contains("srv2022") || lower.contains("srv2025") {
        let version = if lower.contains("2025") {
            "2025"
        } else if lower.contains("2022") {
            "2022"
        } else if lower.contains("2019") {
            "2019"
        } else {
            "Server"
        };
        return OsInfo {
            family: OsFamily::Windows,
            distribution: Some("Windows Server".to_string()),
            version: Some(version.to_string()),
            codename: None,
            image_type: ImageType::Installer,
            display_name: format!("Windows Server {}", version),
        };
    }
    if lower.contains("windows") || lower.contains("winpe") || lower.contains("hbcd") || lower.contains("hiren") {
        let is_pe = lower.contains("pe") || lower.contains("hbcd") || lower.contains("hiren");
        return OsInfo {
            family: if is_pe { OsFamily::Recovery } else { OsFamily::Windows },
            distribution: Some(if is_pe { "Hiren's BootCD PE".to_string() } else { "Windows".to_string() }),
            version: None,
            codename: None,
            image_type: if is_pe { ImageType::WindowsPe } else { ImageType::Installer },
            display_name: if is_pe { "Hiren's BootCD PE (WinPE)".to_string() } else { "Windows".to_string() },
        };
    }

    // 2. Linux Distributions
    if lower.contains("ubuntu") {
        let version = extract_version(&lower, "ubuntu");
        let display = match &version {
            Some(v) => format!("Ubuntu {} LTS", v),
            None => "Ubuntu Linux".to_string(),
        };
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Ubuntu".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: display,
        };
    }
    if lower.contains("debian") {
        let version = extract_version(&lower, "debian");
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Debian".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Debian GNU/Linux".to_string(),
        };
    }
    if lower.contains("mint") || lower.contains("linuxmint") {
        let version = extract_version(&lower, "mint");
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Linux Mint".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Linux Mint".to_string(),
        };
    }
    if lower.contains("fedora") {
        let version = extract_version(&lower, "fedora");
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Fedora".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Fedora Workstation".to_string(),
        };
    }
    if lower.contains("arch") || lower.contains("archlinux") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Arch Linux".to_string()),
            version: Some("Rolling".to_string()),
            codename: None,
            image_type: ImageType::Live,
            display_name: "Arch Linux".to_string(),
        };
    }
    if lower.contains("manjaro") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Manjaro".to_string()),
            version: extract_version(&lower, "manjaro"),
            codename: None,
            image_type: ImageType::Live,
            display_name: "Manjaro Linux".to_string(),
        };
    }
    if lower.contains("pop-os") || lower.contains("pop_os") || lower.contains("popos") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Pop!_OS".to_string()),
            version: extract_version(&lower, "pop"),
            codename: None,
            image_type: ImageType::Live,
            display_name: "System76 Pop!_OS".to_string(),
        };
    }
    if lower.contains("alpine") {
        let version = extract_version(&lower, "alpine");
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Alpine Linux".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Alpine Linux".to_string(),
        };
    }
    if lower.contains("kali") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Kali Linux".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Kali Linux".to_string(),
        };
    }
    if lower.contains("tails") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Tails".to_string()),
            version: extract_version(&lower, "tails"),
            codename: None,
            image_type: ImageType::Live,
            display_name: "Tails (Privacy & Anonymity)".to_string(),
        };
    }
    if lower.contains("opensuse") || lower.contains("suse") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("openSUSE".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Live,
            display_name: "openSUSE Linux".to_string(),
        };
    }
    if lower.contains("rocky") || lower.contains("almalinux") || lower.contains("rhel") || lower.contains("centos") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Enterprise Linux".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Installer,
            display_name: "Enterprise Linux".to_string(),
        };
    }
    if lower.contains("proxmox") || lower.contains("pve") {
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Proxmox VE".to_string()),
            version: extract_version(&lower, "pve"),
            codename: None,
            image_type: ImageType::Installer,
            display_name: "Proxmox Virtual Environment".to_string(),
        };
    }
    if lower.contains("truenas") || lower.contains("freenas") {
        return OsInfo {
            family: OsFamily::Bsd,
            distribution: Some("TrueNAS".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Installer,
            display_name: "TrueNAS Storage OS".to_string(),
        };
    }
    if lower.contains("freebsd") || lower.contains("openbsd") || lower.contains("netbsd") {
        return OsInfo {
            family: OsFamily::Bsd,
            distribution: Some("BSD".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Installer,
            display_name: "BSD Operating System".to_string(),
        };
    }

    // 3. Rescue & Diagnostic Tools
    if lower.contains("clonezilla") {
        return OsInfo {
            family: OsFamily::Recovery,
            distribution: Some("Clonezilla".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Rescue,
            display_name: "Clonezilla".to_string(),
        };
    }
    if lower.contains("gparted") {
        return OsInfo {
            family: OsFamily::Recovery,
            distribution: Some("GParted".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Utility,
            display_name: "GParted Live".to_string(),
        };
    }
    if lower.contains("memtest") {
        return OsInfo {
            family: OsFamily::Utility,
            distribution: Some("MemTest86".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Utility,
            display_name: "MemTest86+".to_string(),
        };
    }
    if lower.contains("systemrescue") || lower.contains("sysrescue") {
        return OsInfo {
            family: OsFamily::Recovery,
            distribution: Some("SystemRescue".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Rescue,
            display_name: "SystemRescue Toolkit".to_string(),
        };
    }
    if lower.contains("dban") || lower.contains("shredos") {
        return OsInfo {
            family: OsFamily::Utility,
            distribution: Some("Disk Eraser".to_string()),
            version: None,
            codename: None,
            image_type: ImageType::Utility,
            display_name: "Drive Eraser & Sanitize".to_string(),
        };
    }

    // Unknown fallback
    OsInfo {
        family: OsFamily::Unknown,
        distribution: None,
        version: None,
        codename: None,
        image_type: ImageType::Unknown,
        display_name: label.to_string(),
    }
}

fn extract_version(input: &str, _prefix: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+(\.\d+)+)").ok()?;
    if let Some(caps) = re.captures(input) {
        caps.get(1).map(|m| m.as_str().to_string())
    } else {
        None
    }
}
