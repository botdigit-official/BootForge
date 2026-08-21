# BootForge — Compatibility Database Specification

## Overview

The Compatibility Database is a curated knowledge base of tested ISO images, their known quirks, workarounds, and recommended boot strategies. It ensures BootForge can handle the wide variety of operating systems and utilities users may throw at it.

---

## Database Architecture

```
┌────────────────────────────────────────────────────┐
│               Compatibility Database                │
│                                                    │
│  ┌──────────────────────────────────────────────┐  │
│  │  Built-in Database (TOML)                    │  │
│  │  Ships with BootForge, updated per release   │  │
│  │  Contains tested ISOs with known configs     │  │
│  └──────────────────────┬───────────────────────┘  │
│                         │                          │
│  ┌──────────────────────┴───────────────────────┐  │
│  │  Online Updates (Optional)                   │  │
│  │  Fetched from bootforge.dev/compat           │  │
│  │  Community-contributed entries                │  │
│  └──────────────────────┬───────────────────────┘  │
│                         │                          │
│  ┌──────────────────────┴───────────────────────┐  │
│  │  User Overrides (Local)                      │  │
│  │  Custom entries, user-tested configurations  │  │
│  └──────────────────────────────────────────────┘  │
│                                                    │
│  Priority: User Override > Online > Built-in       │
└────────────────────────────────────────────────────┘
```

---

## Entry Schema

```toml
# Each ISO entry is identified by a unique key
[windows-11-x64-24h2]

# Basic Information
name = "Windows 11 24H2"
family = "windows"                    # windows | linux | bsd | rescue | utility | firmware
distribution = "Windows"
version = "24H2"
codename = ""
release_date = "2024-10-01"

# Architecture
architectures = ["x86_64"]            # x86 | x86_64 | arm64 | ia32 | multi
primary_architecture = "x86_64"

# Image Type
image_type = "installer"              # installer | live | rescue | pe | netinstall | firmware | utility

# Boot Compatibility
boot_methods = ["uefi"]               # uefi | legacy | both
secure_boot = "required"              # required | supported | not_supported | varies
uefi_boot_file = "efi/boot/bootx64.efi"

# BootForge Configuration
boot_strategy = "windows_wim"         # standard | grub_loopback | windows_wim | wimboot | memdisk | chainload
grub_template = "windows"             # Template name for GRUB entry generation
persistence = false
persistence_method = ""

# Compatibility
compatibility_level = "excellent"     # excellent | good | fair | poor | untested | incompatible
tested = true
last_tested_date = "2026-07-15"
tested_bootforge_version = "1.0.0"

# Quirks
quirks = []                           # List of quirk IDs
notes = "Requires UEFI. install.wim may exceed 4GB, use exFAT data partition."

# Detection Signatures
[windows-11-x64-24h2.detection]
volume_id_pattern = ".*"
file_markers = ["sources/install.wim", "sources/boot.wim", "boot/bcd"]
version_file = "sources/install.wim"  # Parse WIM XML metadata
version_regex = "Windows 11"

# Known Hashes (optional, for verification)
[windows-11-x64-24h2.hashes]
# SHA256 hashes of known official ISO files
# Multiple versions/builds may exist
known_hashes = []
hash_url = ""
```

---

## GRUB Templates

### Template: `standard_linux`
```grub
menuentry "${NAME}" --class ${CLASS} --class linux {
    set iso_path="${ISO_PATH}"
    loopback loop $iso_path
    linux (loop)${VMLINUZ_PATH} ${KERNEL_PARAMS}
    initrd (loop)${INITRD_PATH}
}
```

### Template: `windows`
```grub
menuentry "${NAME}" --class windows {
    set iso_path="${ISO_PATH}"
    loopback loop $iso_path
    chainloader (loop)/efi/boot/bootx64.efi
}
```

### Template: `wimboot`
```grub
menuentry "${NAME}" --class windows --class rescue {
    linux16 /bootforge/wimboot
    initrd16 \
        newc:bootmgr:(loop)/bootmgr \
        newc:bcd:(loop)/boot/bcd \
        newc:boot.sdi:(loop)/boot/boot.sdi \
        newc:boot.wim:(loop)/sources/boot.wim
}
```

### Template: `persistence_ubuntu`
```grub
menuentry "${NAME} (Persistent)" --class ${CLASS} --class linux {
    set iso_path="${ISO_PATH}"
    loopback loop $iso_path
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$iso_path persistent quiet splash ---
    initrd (loop)/casper/initrd
}
```

---

## Known Quirks Registry

| Quirk ID | Description | Affected ISOs | Auto-Fix |
|----------|-------------|---------------|----------|
| `wim_over_4gb` | install.wim exceeds FAT32 4GB limit | Windows 11 24H2+ | ✅ Use exFAT partition |
| `wimboot_required` | Requires WIMBOOT mode | Hiren's PE, some WinPE ISOs | ✅ Use WIMBOOT template |
| `no_isolinux` | Missing ISOLINUX, needs GRUB loopback | Arch, some minimal ISOs | ✅ Use GRUB loopback |
| `legacy_only` | Only boots in Legacy/BIOS mode | Very old ISOs | ⚠️ Warn user |
| `secure_boot_fail` | Fails with Secure Boot enabled | Unsigned ISOs | ⚠️ Warn user |
| `custom_kernel_params` | Needs special kernel parameters | Specific distro versions | ✅ Apply from DB |
| `ntfs_required` | Requires NTFS for Windows boot | Some Windows ISOs | ⚠️ Create NTFS partition |
| `initrd_path_differs` | Initrd at non-standard path | Some Ubuntu flavors | ✅ Use detected path |
| `dracut_vs_initramfs` | Dracut-based initrd (Fedora/RHEL) | Fedora, RHEL, CentOS | ✅ Use Dracut params |
| `casper_vs_live` | Different live system (casper vs live-boot) | Ubuntu vs Debian | ✅ Auto-detect |
| `efi_only` | No Legacy BIOS support | Modern minimal ISOs | ✅ Skip BIOS config |
| `iso_hybrid_missing` | Not a hybrid ISO, can't dd | Older ISOs | ⚠️ Use GRUB loopback only |

---

## V1 Built-in Database Entries

### Windows Family

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `windows-11-x64` | 11 (23H2, 24H2) | x64 | UEFI | ✅ Tested |
| `windows-10-x64` | 10 (22H2) | x64 | UEFI+Legacy | ✅ Tested |
| `windows-11-arm64` | 11 ARM | ARM64 | UEFI | 🔜 V1.5 |
| `windows-pe` | PE (various) | x64 | UEFI | ✅ Tested |

### Ubuntu Family

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `ubuntu-24.04` | 24.04 LTS | x64 | UEFI+Legacy | ✅ Tested |
| `ubuntu-24.10` | 24.10 | x64 | UEFI+Legacy | ✅ Tested |
| `ubuntu-26.04` | 26.04 LTS | x64 | UEFI+Legacy | ✅ Tested |
| `kubuntu-24.04` | 24.04 LTS | x64 | UEFI+Legacy | ✅ Tested |
| `xubuntu-24.04` | 24.04 LTS | x64 | UEFI+Legacy | ✅ Tested |
| `ubuntu-server-24.04` | 24.04 LTS | x64 | UEFI+Legacy | ✅ Tested |

### Debian Family

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `debian-12` | 12 (Bookworm) | x64 | UEFI+Legacy | ✅ Tested |
| `debian-13` | 13 (Trixie) | x64 | UEFI+Legacy | ✅ Tested |
| `linux-mint-22` | 22 | x64 | UEFI+Legacy | ✅ Tested |
| `kali-2024` | 2024.x | x64 | UEFI+Legacy | ✅ Tested |

### RPM Family

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `fedora-40` | 40 | x64 | UEFI+Legacy | ✅ Tested |
| `fedora-41` | 41 | x64 | UEFI+Legacy | ✅ Tested |

### Independent

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `arch-latest` | Rolling | x64 | UEFI+Legacy | ✅ Tested |
| `alpine-3.20` | 3.20 | x64 | UEFI+Legacy | ✅ Tested |
| `manjaro-latest` | Rolling | x64 | UEFI+Legacy | ✅ Tested |

### Rescue & Utilities

| Entry | Version | Arch | Boot | Status |
|-------|---------|------|------|--------|
| `hirens-pe` | Latest | x64 | UEFI | ✅ Tested |
| `clonezilla-live` | 3.x | x64 | UEFI+Legacy | ✅ Tested |
| `gparted-live` | 1.x | x64 | UEFI+Legacy | ✅ Tested |
| `systemrescue` | 11.x | x64 | UEFI+Legacy | ✅ Tested |
| `memtest86` | Latest | x64 | UEFI+Legacy | ✅ Tested |

---

## Matching Algorithm

```rust
pub fn match_iso_to_database(analysis: &IsoAnalysis, db: &CompatibilityDb) -> Option<CompatEntry> {
    // Priority 1: Exact hash match
    if let Some(entry) = db.find_by_hash(&analysis.hash.sha256) {
        return Some(entry);
    }
    
    // Priority 2: OS + Version + Architecture match
    if let Some(entry) = db.find_by_os_version_arch(
        &analysis.os_info.distribution,
        &analysis.os_info.version,
        &analysis.architecture,
    ) {
        return Some(entry);
    }
    
    // Priority 3: OS Family + Architecture match (generic entry)
    if let Some(entry) = db.find_by_family_arch(
        &analysis.os_info.family,
        &analysis.architecture,
    ) {
        return Some(entry);
    }
    
    // Priority 4: Boot method heuristic (no DB match)
    None // Caller should use heuristic-based configuration
}
```

---

## Community Contributions

### Submission Format

Users can contribute tested ISO configurations:

```toml
# community-submission.toml
[submission]
iso_name = "Pop!_OS 24.04"
iso_sha256 = "abc123..."
tester = "username"
bootforge_version = "1.2.0"
test_date = "2026-08-15"
hardware = "Dell XPS 15, UEFI, Secure Boot disabled"
result = "success"
notes = "Boots perfectly with standard GRUB loopback"

[config]
family = "linux"
distribution = "Pop!_OS"
version = "24.04"
architectures = ["x86_64"]
boot_methods = ["uefi", "legacy"]
boot_strategy = "grub_loopback"
secure_boot = "not_supported"
persistence = true
persistence_method = "casper"
compatibility_level = "excellent"
quirks = []
```

### Contribution Workflow

1. User tests an ISO that's not in the database
2. BootForge prompts: "This ISO booted successfully. Would you like to contribute this to the community database?"
3. User submits (anonymized) test result
4. Maintainers review and merge into official database
5. Next update includes the new entry

---

## Update Mechanism

```
BootForge App
      │
      ▼
Check for DB updates (on launch, max 1x/day)
      │
      ▼
Download compat-db-{version}.toml.gz (signed)
      │
      ▼
Verify GPG signature
      │
      ▼
Merge with local database
      │
      ▼
User overrides preserved (highest priority)
```

- Updates are **optional** and **non-blocking**
- App works fully offline with built-in database
- Updates only add/modify entries, never remove user overrides
