//! Operating system identification heuristics.

use bootforge_core::types::{ImageType, OsFamily, OsInfo};

/// Identify operating system from volume identifier, file names, or string markers.
pub fn identify_os_from_label(label: &str) -> OsInfo {
    let lower = label.to_lowercase();

    // 1. Windows
    if lower.contains("windows 11") || lower.contains("win11") || lower.contains("win_11") {
        return OsInfo {
            family: OsFamily::Windows,
            distribution: Some("Windows".to_string()),
            version: Some("11".to_string()),
            codename: None,
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
    if lower.contains("windows") || lower.contains("winpe") || lower.contains("hbcd") {
        return OsInfo {
            family: OsFamily::Windows,
            distribution: Some("Windows".to_string()),
            version: None,
            codename: None,
            image_type: if lower.contains("pe") || lower.contains("hbcd") {
                ImageType::WindowsPe
            } else {
                ImageType::Installer
            },
            display_name: "Windows".to_string(),
        };
    }

    // 2. Linux Distributions
    if lower.contains("ubuntu") {
        let version = extract_version(&lower, "ubuntu");
        return OsInfo {
            family: OsFamily::Linux,
            distribution: Some("Ubuntu".to_string()),
            version,
            codename: None,
            image_type: ImageType::Live,
            display_name: "Ubuntu Linux".to_string(),
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
