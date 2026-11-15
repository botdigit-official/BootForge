//! Operating system identification from volume labels and file names.
//!
//! Matching is token based, so short names do not match inside other words
//! (`arch` no longer matches `aarch64`, `pe` no longer matches `open`).
//!
//! Pattern syntax used in [`RULES`]:
//! * `word`      — a whole token (`ubuntu-24.04` → tokens `ubuntu`, `24`, `04`)
//! * `prefix*`   — a token starting with `prefix`
//! * `two words` — a phrase in the label with separators collapsed to spaces

use bootforge_core::types::{ImageType, OsFamily, OsInfo};

struct Rule {
    patterns: &'static [&'static str],
    family: OsFamily,
    distribution: &'static str,
    image_type: ImageType,
    display: &'static str,
    /// `Some(v)` = fixed version, `None` = extract digits from the label.
    version: Option<&'static str>,
}

const fn rule(
    patterns: &'static [&'static str],
    family: OsFamily,
    distribution: &'static str,
    image_type: ImageType,
    display: &'static str,
    version: Option<&'static str>,
) -> Rule {
    Rule { patterns, family, distribution, image_type, display, version }
}

use ImageType::*;
use OsFamily as F;

/// First match wins, so specific entries come before generic ones.
static RULES: &[Rule] = &[
    // ── Windows PE / rescue (before generic Windows) ──
    rule(&["hbcd*", "hiren*", "hirens*"], F::Recovery, "Hiren's BootCD PE", WindowsPe, "Hiren's BootCD PE (WinPE)", Some("")),
    rule(&["winpe", "strelec*", "sergei strelec"], F::Recovery, "Windows PE", WindowsPe, "Windows PE rescue", Some("")),
    rule(&["medicat*"], F::Recovery, "MediCat", Rescue, "MediCat USB toolkit", Some("")),
    // ── Windows ──
    rule(&["windows server", "win server", "winserver*", "srv2008*", "srv2012*", "srv2016*", "srv2019*", "srv2022*", "srv2025*", "sss*"], F::Windows, "Windows Server", Installer, "Windows Server", None),
    rule(&["win11*", "windows 11", "win 11", "tiny11*", "nano11*"], F::Windows, "Windows", Installer, "Windows 11", Some("11")),
    rule(&["win10*", "windows 10", "win 10", "tiny10*", "nano10*"], F::Windows, "Windows", Installer, "Windows 10", Some("10")),
    rule(&["cccoma*"], F::Windows, "Windows", Installer, "Windows 10/11", Some("10/11")),
    rule(&["win8*", "windows 8", "ir5*", "ir3*"], F::Windows, "Windows", Installer, "Windows 8.1", Some("8.1")),
    rule(&["win7*", "windows 7", "grmc*", "gsp1rm*", "grmsx*", "grmhom*", "grmpro*", "grmult*"], F::Windows, "Windows", Installer, "Windows 7", Some("7")),
    rule(&["vista", "lrmc*", "fb1em*"], F::Windows, "Windows", Installer, "Windows Vista", Some("Vista")),
    rule(&["winxp*", "windows xp", "win xp", "wxp*", "grtmp*", "vrmpfpp*"], F::Windows, "Windows", Installer, "Windows XP", Some("XP")),
    rule(&["*x64fre*", "*x86fre*", "*a64fre*", "windows", "win"], F::Windows, "Windows", Installer, "Windows", Some("")),
    // ── macOS (installers cannot be booted from an ISO menu; see target guidance) ──
    rule(&["tahoe"], F::MacOs, "macOS", Installer, "macOS Tahoe installer", Some("26")),
    rule(&["sequoia"], F::MacOs, "macOS", Installer, "macOS Sequoia installer", Some("15")),
    rule(&["sonoma"], F::MacOs, "macOS", Installer, "macOS Sonoma installer", Some("14")),
    rule(&["ventura"], F::MacOs, "macOS", Installer, "macOS Ventura installer", Some("13")),
    rule(&["monterey"], F::MacOs, "macOS", Installer, "macOS Monterey installer", Some("12")),
    rule(&["big sur", "bigsur"], F::MacOs, "macOS", Installer, "macOS Big Sur installer", Some("11")),
    rule(&["catalina"], F::MacOs, "macOS", Installer, "macOS Catalina installer", Some("10.15")),
    rule(&["mojave"], F::MacOs, "macOS", Installer, "macOS Mojave installer", Some("10.14")),
    rule(&["high sierra", "highsierra"], F::MacOs, "macOS", Installer, "macOS High Sierra installer", Some("10.13")),
    rule(&["el capitan", "elcapitan", "yosemite"], F::MacOs, "macOS", Installer, "OS X installer", Some("10.11")),
    rule(&["macos", "osx", "mac os", "installesd", "basesystem", "install macos"], F::MacOs, "macOS", Installer, "macOS installer", Some("")),
    // ── Linux (lightweight distros first: they are the ones old devices need) ──
    rule(&["lubuntu*"], F::Linux, "Lubuntu", Live, "Lubuntu (lightweight)", None),
    rule(&["xubuntu*"], F::Linux, "Xubuntu", Live, "Xubuntu", None),
    rule(&["kubuntu*"], F::Linux, "Kubuntu", Live, "Kubuntu", None),
    rule(&["ubuntu*"], F::Linux, "Ubuntu", Live, "Ubuntu", None),
    rule(&["linuxmint*", "mint"], F::Linux, "Linux Mint", Live, "Linux Mint", None),
    rule(&["antix*"], F::Linux, "antiX", Live, "antiX (very old PCs)", None),
    rule(&["mx"], F::Linux, "MX Linux", Live, "MX Linux", None),
    rule(&["puppy*", "fossapup*", "bookwormpup*", "bionicpup*"], F::Linux, "Puppy Linux", Live, "Puppy Linux (very old PCs)", None),
    rule(&["tinycore*", "coreplus*"], F::Linux, "Tiny Core", Live, "Tiny Core Linux", None),
    rule(&["q4os*"], F::Linux, "Q4OS", Live, "Q4OS", None),
    rule(&["bodhi*"], F::Linux, "Bodhi Linux", Live, "Bodhi Linux", None),
    rule(&["peppermint*"], F::Linux, "Peppermint OS", Live, "Peppermint OS", None),
    rule(&["debian*"], F::Linux, "Debian", Live, "Debian GNU/Linux", None),
    rule(&["fedora*"], F::Linux, "Fedora", Live, "Fedora", None),
    rule(&["bazzite*"], F::Linux, "Bazzite", Live, "Bazzite", None),
    rule(&["archlinux*", "arch"], F::Linux, "Arch Linux", Live, "Arch Linux", Some("Rolling")),
    rule(&["endeavouros*"], F::Linux, "EndeavourOS", Live, "EndeavourOS", None),
    rule(&["cachyos*"], F::Linux, "CachyOS", Live, "CachyOS", None),
    rule(&["garuda*"], F::Linux, "Garuda Linux", Live, "Garuda Linux", None),
    rule(&["manjaro*"], F::Linux, "Manjaro", Live, "Manjaro Linux", None),
    rule(&["pop os", "popos*", "pop_os"], F::Linux, "Pop!_OS", Live, "Pop!_OS", None),
    rule(&["zorin*"], F::Linux, "Zorin OS", Live, "Zorin OS", None),
    rule(&["elementaryos*", "elementary"], F::Linux, "elementary OS", Live, "elementary OS", None),
    rule(&["kali*"], F::Linux, "Kali Linux", Live, "Kali Linux", None),
    rule(&["parrot*"], F::Linux, "Parrot OS", Live, "Parrot OS", None),
    rule(&["tails*"], F::Linux, "Tails", Live, "Tails (privacy)", None),
    rule(&["opensuse*", "suse*", "sle*"], F::Linux, "openSUSE", Live, "openSUSE", None),
    rule(&["rocky*", "almalinux*", "rhel*", "centos*", "oraclelinux*"], F::Linux, "Enterprise Linux", Installer, "Enterprise Linux (RHEL family)", None),
    rule(&["nixos*"], F::Linux, "NixOS", Live, "NixOS", None),
    rule(&["void*"], F::Linux, "Void Linux", Live, "Void Linux", None),
    rule(&["gentoo*", "install amd64 minimal"], F::Linux, "Gentoo", Live, "Gentoo", None),
    rule(&["slackware*"], F::Linux, "Slackware", Installer, "Slackware", None),
    rule(&["alpine*"], F::Linux, "Alpine Linux", Live, "Alpine Linux", None),
    rule(&["proxmox*", "pve"], F::Linux, "Proxmox VE", Installer, "Proxmox Virtual Environment", None),
    rule(&["truenas scale"], F::Linux, "TrueNAS SCALE", Installer, "TrueNAS SCALE", None),
    rule(&["chromeos*", "chrome os"], F::Linux, "ChromeOS Flex", Installer, "ChromeOS Flex", None),
    rule(&["android x86", "androidx86*", "bliss*", "primeos*"], F::Linux, "Android-x86", Live, "Android for PC", None),
    // ── BSD ──
    rule(&["truenas*", "freenas*"], F::Bsd, "TrueNAS CORE", Installer, "TrueNAS CORE", None),
    rule(&["pfsense*", "opnsense*"], F::Bsd, "Firewall (BSD)", Installer, "pfSense / OPNsense firewall", None),
    rule(&["ghostbsd*"], F::Bsd, "GhostBSD", Live, "GhostBSD", None),
    rule(&["freebsd*", "openbsd*", "netbsd*", "install7*"], F::Bsd, "BSD", Installer, "BSD", None),
    // ── Rescue & utilities ──
    rule(&["clonezilla*"], F::Recovery, "Clonezilla", Rescue, "Clonezilla", None),
    rule(&["rescuezilla*"], F::Recovery, "Rescuezilla", Rescue, "Rescuezilla", None),
    rule(&["redorescue*", "redo"], F::Recovery, "Redo Rescue", Rescue, "Redo Rescue", None),
    rule(&["gparted*"], F::Recovery, "GParted", Utility, "GParted Live", None),
    rule(&["systemrescue*", "sysrescue*", "systemrescuecd*"], F::Recovery, "SystemRescue", Rescue, "SystemRescue", None),
    rule(&["boot repair", "bootrepair*"], F::Recovery, "Boot-Repair-Disk", Rescue, "Boot-Repair-Disk", None),
    rule(&["memtest*", "mt86*"], F::Utility, "MemTest86+", Utility, "MemTest86+", None),
    rule(&["ubcd*", "ultimate boot cd"], F::Utility, "Ultimate Boot CD", Utility, "Ultimate Boot CD", None),
    rule(&["shredos*", "dban*", "nwipe*"], F::Utility, "Disk Eraser", Utility, "Disk eraser (ShredOS/DBAN)", None),
    rule(&["freedos*", "fd14*", "fd13*"], F::Utility, "FreeDOS", Utility, "FreeDOS (BIOS/firmware updates)", None),
    rule(&["ventoy*"], F::Utility, "Ventoy", Utility, "Ventoy tools", None),
    // ── Others ──
    rule(&["reactos*"], F::Unknown, "ReactOS", Installer, "ReactOS", None),
    rule(&["haiku*"], F::Unknown, "Haiku", Live, "Haiku", None),
];

fn tokens(normalized: &str) -> impl Iterator<Item = &str> {
    normalized.split(' ').filter(|t| !t.is_empty())
}

fn pattern_matches(pattern: &str, normalized: &str) -> bool {
    if pattern.contains(' ') || pattern.contains('_') {
        let phrase = pattern.replace('_', " ");
        return format!(" {normalized} ").contains(&format!(" {phrase} "));
    }
    match (pattern.strip_prefix('*'), pattern.strip_suffix('*')) {
        (Some(inner), _) => {
            let inner = inner.trim_end_matches('*');
            tokens(normalized).any(|t| t.contains(inner))
        }
        (None, Some(prefix)) => tokens(normalized).any(|t| t.starts_with(prefix)),
        (None, None) => tokens(normalized).any(|t| t == pattern),
    }
}

/// Lowercase and turn every separator into a single space.
fn normalize(label: &str) -> String {
    let lower: String = label
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    lower.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Identify the operating system from a volume label and/or file name.
pub fn identify_os_from_label(label: &str) -> OsInfo {
    let normalized = normalize(label);

    for r in RULES {
        if r.patterns.iter().any(|p| pattern_matches(p, &normalized)) {
            let version = match r.version {
                Some("") => None,
                Some(v) => Some(v.to_string()),
                None => extract_version(label),
            };
            let display_name = match (&r.family, &version, r.version) {
                (F::Windows, Some(v), None) => format!("{} {}", r.display, v),
                (F::Linux | F::Bsd, Some(v), None) => format!("{} {}", r.display, v),
                _ => r.display.to_string(),
            };
            return OsInfo {
                family: r.family.clone(),
                distribution: Some(r.distribution.to_string()),
                version,
                codename: None,
                image_type: r.image_type.clone(),
                display_name,
            };
        }
    }

    OsInfo {
        family: OsFamily::Unknown,
        distribution: None,
        version: None,
        codename: None,
        image_type: ImageType::Unknown,
        display_name: label.to_string(),
    }
}

fn extract_version(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    // Windows Server years.
    if let Some(m) = regex::Regex::new(r"20(0[38]|1[269]|2[25])").ok()?.find(&lower) {
        if lower.contains("server") || lower.contains("srv") {
            return Some(m.as_str().to_string());
        }
    }
    let re = regex::Regex::new(r"(\d+(\.\d+)+)").ok()?;
    re.captures(&lower).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fam(label: &str) -> (OsFamily, String) {
        let i = identify_os_from_label(label);
        (i.family, i.distribution.unwrap_or_default())
    }

    #[test]
    fn arch_does_not_match_aarch64() {
        assert_eq!(fam("alpine-standard-3.20.0-aarch64.iso").1, "Alpine Linux");
        assert_eq!(fam("nixos-minimal-24.05-aarch64-linux.iso").1, "NixOS");
        assert_eq!(fam("archlinux-2024.11.01-x86_64.iso").1, "Arch Linux");
    }

    #[test]
    fn windows_versions_and_ms_labels() {
        assert_eq!(identify_os_from_label("Win11_24H2_English_x64.iso").display_name, "Windows 11");
        assert_eq!(identify_os_from_label("CCCOMA_X64FRE_EN-US_DV9 Win10_22H2.iso").version.as_deref(), Some("10"));
        assert_eq!(identify_os_from_label("CCCOMA_X64FRE_EN-US_DV9").display_name, "Windows 10/11");
        assert_eq!(identify_os_from_label("GRMCULFRER_EN_DVD").display_name, "Windows 7");
        assert_eq!(identify_os_from_label("en_windows_xp_professional_sp3.iso").display_name, "Windows XP");
        assert_eq!(identify_os_from_label("en_windows_server_2022_x64.iso").display_name, "Windows Server 2022");
        assert_eq!(fam("HBCD_PE_x64.iso").0, OsFamily::Recovery);
    }

    #[test]
    fn short_words_do_not_false_match() {
        // "open" must not become WinPE, "research" must not become Arch.
        assert_eq!(fam("openmediavault_7.iso").0, OsFamily::Unknown);
        assert_eq!(fam("research-notes.iso").0, OsFamily::Unknown);
        assert_eq!(fam("details.img").0, OsFamily::Unknown);
    }

    #[test]
    fn old_device_distros() {
        assert_eq!(fam("antiX-23.1_386-full.iso").1, "antiX");
        assert_eq!(fam("lubuntu-24.04-desktop-amd64.iso").1, "Lubuntu");
        assert_eq!(fam("MX-23.4_386.iso").1, "MX Linux");
        assert_eq!(fam("FD14-LiveCD.iso").1, "FreeDOS");
    }

    #[test]
    fn macos_installers() {
        let i = identify_os_from_label("Install macOS Sonoma.dmg");
        assert_eq!(i.family, OsFamily::MacOs);
        assert_eq!(i.version.as_deref(), Some("14"));
    }
}
