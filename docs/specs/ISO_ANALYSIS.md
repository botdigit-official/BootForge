# BootForge — ISO Analysis Specification

## Overview

The ISO Analyzer is responsible for examining disk image files to extract metadata, verify integrity, identify the operating system, determine boot compatibility, and assess overall readiness for inclusion on a multi-boot drive.

---

## Analysis Pipeline

```
File Input
   │
   ▼
┌──────────────────┐
│ 1. FILE VALIDATION│
│                   │
│ • File exists     │
│ • Readable        │
│ • Size > 0        │
│ • Extension check │
│ • Magic bytes     │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 2. HASH VERIFY   │
│                   │
│ • SHA256          │
│ • SHA512          │
│ • MD5 (legacy)    │
│ • Online lookup   │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 3. FORMAT DETECT  │
│                   │
│ • ISO 9660        │
│ • UDF             │
│ • Hybrid ISO      │
│ • WIM             │
│ • Raw IMG         │
│ • VHD/VHDX        │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 4. BOOT ANALYSIS  │
│                   │
│ • El Torito       │
│ • EFI directory   │
│ • MBR bootloader  │
│ • GRUB files      │
│ • Syslinux/Isolinux│
│ • Boot catalog    │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 5. OS IDENTIFY    │
│                   │
│ • File patterns   │
│ • Version files   │
│ • Package lists   │
│ • Boot config     │
│ • Metadata files  │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 6. ARCH DETECT    │
│                   │
│ • EFI binary arch │
│ • Kernel binary   │
│ • Package arch    │
│ • Multi-arch      │
└────────┬──────────┘
         │
         ▼
┌──────────────────┐
│ 7. COMPAT CHECK   │
│                   │
│ • Database lookup │
│ • Quirk detection │
│ • Strategy select │
│ • Score assign    │
└────────┬──────────┘
         │
         ▼
   IsoAnalysis Result
```

---

## Stage 1: File Validation

### Magic Bytes Detection

| Format | Magic Bytes | Offset | Description |
|--------|-------------|--------|-------------|
| ISO 9660 | `CD001` | 0x8001, 0x8801, or 0x9001 | Primary Volume Descriptor |
| UDF | `BEA01` + `NSR02`/`NSR03` | 0x8001+ | UDF descriptor |
| WIM | `MSWIM\0\0\0` | 0x00 | Windows Imaging Format |
| VHD | `conectix` | 0x00 | Virtual Hard Disk footer |
| VHDX | `vhdxfile` | 0x00 | VHDX file type identifier |
| Raw IMG | (no magic) | — | Detected by partition table |
| EFI | `MZ` | 0x00 | PE executable (check PE header) |

### File Extension Mapping

```
.iso  → ISO 9660 / UDF / Hybrid
.wim  → Windows Imaging Format
.esd  → Windows ESD (encrypted WIM)
.img  → Raw disk image or ISO
.vhd  → Virtual Hard Disk
.vhdx → Virtual Hard Disk v2
.efi  → EFI Application
```

---

## Stage 2: Hash Verification

### Hash Computation

```rust
pub struct HashResult {
    pub sha256: String,
    pub sha512: Option<String>,
    pub md5: Option<String>,
    pub verified: VerificationStatus,
    pub source: Option<HashSource>,
}

pub enum VerificationStatus {
    Verified,           // Hash matches known good value
    Downloaded,         // Hash downloaded but user needs to confirm
    Unverified,         // No known hash to compare against
    Mismatch,           // Hash does NOT match — file may be corrupted
}

pub enum HashSource {
    BuiltInDatabase,    // From BootForge compatibility database
    OfficialHashFile,   // Downloaded from OS vendor's hash file
    UserProvided,       // User manually provided hash
}
```

### Official Hash Sources

| OS | Hash URL Pattern | Format |
|----|-----------------|--------|
| Ubuntu | `releases.ubuntu.com/{version}/SHA256SUMS` | `sha256  filename` |
| Debian | `cdimage.debian.org/debian-cd/{version}/SHA256SUMS` | `sha256  filename` |
| Fedora | `download.fedoraproject.org/.../CHECKSUM` | `SHA256 (filename) = hash` |
| Arch | `archlinux.org/download/` (page scrape) | SHA256 on download page |
| Alpine | `alpinelinux.org/downloads/` | SHA256 on download page |
| Linux Mint | `linuxmint.com/verify.php` | SHA256SUMS file |
| Windows | N/A (no public hash file) | Use internal database |

---

## Stage 3: Format Detection

### ISO 9660 Analysis

```rust
pub struct Iso9660Info {
    pub system_id: String,          // e.g., "LINUX", "WIN32"
    pub volume_id: String,          // e.g., "Ubuntu 24.04 LTS amd64"
    pub volume_set_id: String,
    pub publisher_id: String,
    pub data_preparer_id: String,
    pub application_id: String,
    pub volume_creation_date: DateTime,
    pub volume_size: u64,
    pub logical_block_size: u32,    // Usually 2048
    pub joliet_supported: bool,
    pub rock_ridge: bool,
    pub udf_bridge: bool,
}
```

### Hybrid ISO Detection

A hybrid ISO can be written directly to USB (dd-able) because it has both:
- An ISO 9660 filesystem (for CD/DVD)
- A valid MBR partition table (for USB boot)

Detection:
```
1. Read first 512 bytes (MBR)
2. Check for valid MBR signature (0x55AA at offset 510)
3. Check for at least one valid partition entry
4. Verify ISO 9660 descriptor is also present
5. If both → Hybrid ISO
```

---

## Stage 4: Boot Structure Analysis

### El Torito Boot Catalog

The El Torito specification defines how CDs/DVDs boot. Located via the Boot Record Volume Descriptor.

```rust
pub struct ElToritoInfo {
    pub boot_catalog_sector: u32,
    pub entries: Vec<ElToritoEntry>,
}

pub struct ElToritoEntry {
    pub platform: ElToritoPlatform,
    pub boot_media: ElToritoMedia,
    pub load_segment: u16,
    pub sector_count: u16,
    pub load_lba: u32,
}

pub enum ElToritoPlatform {
    X86,        // 0x00 — BIOS boot
    PowerPC,    // 0x01
    Mac,        // 0x02
    Efi,        // 0xEF — EFI boot
}

pub enum ElToritoMedia {
    NoEmulation,    // Most common for modern ISOs
    Floppy12,
    Floppy144,
    Floppy288,
    HardDisk,
}
```

### EFI Directory Scan

```
EFI/
├── BOOT/
│   ├── BOOTx64.EFI      → x86_64 UEFI bootable
│   ├── BOOTia32.EFI     → IA32 UEFI bootable
│   ├── BOOTaa64.EFI     → ARM64 UEFI bootable
│   ├── grubx64.efi      → GRUB2 x64
│   ├── shimx64.efi      → Secure Boot shim
│   └── mmx64.efi        → MOK Manager
├── fedora/               → Fedora-specific EFI files
├── ubuntu/               → Ubuntu-specific EFI files
└── Microsoft/
    └── Boot/
        ├── bootmgfw.efi  → Windows Boot Manager
        └── BCD           → Boot Configuration Data
```

### Boot Capability Summary

```rust
pub struct BootCapability {
    pub bios_bootable: bool,
    pub uefi_x64: bool,
    pub uefi_ia32: bool,
    pub uefi_aa64: bool,
    pub secure_boot: SecureBootCapability,
    pub el_torito: Option<ElToritoInfo>,
    pub isolinux: bool,
    pub syslinux: bool,
    pub grub2: bool,
    pub windows_bootmgr: bool,
}
```

---

## Stage 5: OS Identification

### Detection Heuristics

#### Windows Detection

| Check | Files/Patterns | Indicates |
|-------|---------------|-----------|
| WIM presence | `sources/install.wim` or `sources/install.esd` | Windows installer |
| Boot WIM | `sources/boot.wim` | Windows PE boot |
| Autorun | `autorun.inf` contains `setup.exe` | Windows installer ISO |
| BCD | `boot/bcd` | Windows boot config |
| Version | Parse `sources/install.wim` XML metadata | Exact Windows version |

**Windows Version Detection** (from install.wim XML):
```xml
<IMAGE>
  <NAME>Windows 11 Pro</NAME>
  <DESCRIPTION>Windows 11 Pro</DESCRIPTION>
  <WINDOWS>
    <ARCH>9</ARCH>  <!-- 0=x86, 9=x64, 12=ARM64 -->
    <PRODUCTNAME>Windows 11 Pro</PRODUCTNAME>
    <EDITIONID>Professional</EDITIONID>
    <INSTALLATIONTYPE>Client</INSTALLATIONTYPE>
    <SERVICEPACKBUILD>2</SERVICEPACKBUILD>
    <VERSION>
      <MAJOR>10</MAJOR>
      <MINOR>0</MINOR>
      <BUILD>26100</BUILD>
      <SPBUILD>1742</SPBUILD>
    </VERSION>
  </WINDOWS>
</IMAGE>
```

#### Linux Detection

| Distribution | Detection Files | Version Source |
|-------------|----------------|----------------|
| Ubuntu | `.disk/info`, `casper/`, `ubuntu` in volume ID | `.disk/info` → "Ubuntu 24.04 LTS amd64" |
| Debian | `.disk/info`, `live/`, `install/` | `.disk/info` → "Debian GNU/Linux 12.5" |
| Fedora | `.discinfo`, `LiveOS/`, `images/` | `.discinfo` first line = timestamp |
| Linux Mint | `.disk/info` contains "Linux Mint" | `.disk/info` → "Linux Mint 22" |
| Arch | `arch/` directory, `pkglist.x86_64.txt` | `arch/version` file |
| Alpine | `apks/` directory | `alpine-release` or volume ID |
| Kali | `.disk/info` contains "Kali" | `.disk/info` → "Kali Linux 2024.1" |
| openSUSE | `boot/x86_64/` directory | Volume ID contains "openSUSE" |
| Manjaro | `manjaro/` directory | `manjaro/version` |

#### Rescue/Utility Detection

| Tool | Detection | Notes |
|------|-----------|-------|
| Hiren's Boot PE | `HBCD/` directory, WinPE markers | Windows PE based |
| Clonezilla | `live/` + `filesystem.squashfs` + Clonezilla in volume ID | Debian-based live |
| GParted | `live/` + `filesystem.squashfs` + GParted in volume ID | Debian-based live |
| SystemRescue | `sysrescue/` directory | Arch-based |
| MemTest86 | `memtest` binary, specific volume ID | Standalone |
| DBAN | `dban/` directory | Autonuke disk eraser |

---

## Stage 6: Architecture Detection

### Detection Methods

**Method 1: EFI Binary Analysis**
```
Read PE header of EFI/BOOT/BOOT*.EFI
  → Machine field:
    0x014c = x86 (IA32)
    0x8664 = x86_64 (AMD64)
    0xAA64 = ARM64 (AArch64)
```

**Method 2: Linux Kernel Binary**
```
Read vmlinuz header
  → Linux kernel header at offset 0x202
  → Check setup header for architecture markers
```

**Method 3: Volume ID / Filename**
```
"amd64" in name → x86_64
"x86_64" in name → x86_64
"arm64" in name → ARM64
"aarch64" in name → ARM64
"i386" in name → x86
"i686" in name → x86
```

**Method 4: Package List Analysis**
```
pkglist.x86_64.txt → x86_64
Packages.amd64 → x86_64
```

### Multi-Architecture ISOs

Some ISOs support multiple architectures:
```rust
pub enum Architecture {
    X86,
    X86_64,
    Arm64,
    Ia32,
    Multi(Vec<Architecture>),  // e.g., Ubuntu supports x64 + ARM64
    Unknown,
}
```

---

## Stage 7: Compatibility Assessment

### Scoring Algorithm

```rust
pub fn calculate_compatibility(analysis: &IsoAnalysis) -> CompatibilityScore {
    let mut score = 100;
    let mut flags = vec![];

    // Deductions
    if !analysis.boot_info.uefi_capable && !analysis.boot_info.bios_capable {
        score -= 50;
        flags.push("No bootloader detected");
    }
    
    if analysis.os_info.family == OsFamily::Unknown {
        score -= 20;
        flags.push("Unknown operating system");
    }
    
    if analysis.hash.verified == VerificationStatus::Mismatch {
        score = 0;
        flags.push("HASH MISMATCH — file may be corrupted");
    }
    
    if !analysis.quirks.is_empty() {
        score -= 5 * analysis.quirks.len() as i32;
        flags.push("Known compatibility quirks");
    }

    // Bonuses
    if analysis.boot_info.uefi_capable && analysis.boot_info.bios_capable {
        score += 5;
        flags.push("Dual boot capable (UEFI + BIOS)");
    }
    
    if analysis.boot_info.secure_boot == SecureBootStatus::Supported {
        score += 5;
    }

    CompatibilityScore {
        score: score.clamp(0, 100),
        level: match score {
            90..=100 => CompatLevel::Excellent,
            70..=89  => CompatLevel::Good,
            50..=69  => CompatLevel::Fair,
            1..=49   => CompatLevel::Poor,
            _        => CompatLevel::Incompatible,
        },
        flags,
    }
}
```

### Display Format

```
Compatibility: ████████████████████ 95/100 — EXCELLENT

✓ ISO integrity verified (SHA256)
✓ UEFI bootloader detected (x86_64)
✓ BIOS bootloader detected
✓ Secure Boot files present
✓ Linux installer detected (Ubuntu 24.04 LTS)
✓ Persistence supported
✓ Tested in BootForge compatibility database

Boot Methods: UEFI ✓  BIOS ✓  Secure Boot ✓
```

---

## Output Structure

```rust
pub struct IsoAnalysis {
    // File info
    pub file_path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
    pub format: ImageFormat,
    
    // Verification
    pub hash: HashResult,
    
    // OS identification
    pub os_info: OsInfo,
    
    // Boot capability
    pub boot_info: BootCapability,
    
    // Architecture
    pub architecture: Architecture,
    
    // Compatibility
    pub compatibility: CompatibilityScore,
    pub quirks: Vec<KnownQuirk>,
    pub recommended_strategy: BootStrategy,
    
    // Timing
    pub analysis_duration: Duration,
    
    // Warnings
    pub warnings: Vec<AnalysisWarning>,
}

pub enum AnalysisWarning {
    LargeFile { size: u64 },               // > 4GB, FAT32 limitation
    NoSecureBoot,                           // May not boot on locked systems
    UntestedIso,                            // Not in compatibility DB
    PossibleCorruption { detail: String },  // Suspicious file structure
    DeprecatedFormat { format: String },    // Old/uncommon format
}
```
