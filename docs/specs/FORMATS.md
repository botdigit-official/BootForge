# BootForge — Supported Formats Specification

## Overview

BootForge supports multiple disk image formats, each with different characteristics, use cases, and handling requirements.

---

## Format Matrix

| Format | Extension | Read | Write | Multi-boot | Boot | V1 | Description |
|--------|-----------|------|-------|-----------|------|-----|-------------|
| ISO 9660 | `.iso` | ✅ | ✅ | ✅ | ✅ | ✅ | Standard optical disc image |
| UDF | `.iso` | ✅ | ✅ | ✅ | ✅ | ✅ | Universal Disk Format (large ISOs) |
| Hybrid ISO | `.iso` | ✅ | ✅ | ✅ | ✅ | ✅ | ISO + MBR (dd-able) |
| WIM | `.wim` | ✅ | ❌ | ✅ | ✅ | ✅ | Windows Imaging Format |
| ESD | `.esd` | ✅ | ❌ | ✅ | ✅ | ✅ | Encrypted/compressed WIM |
| Raw Image | `.img` | ✅ | ✅ | ❌ | ✅ | ✅ | Raw disk image (dd output) |
| VHD | `.vhd` | ✅ | ❌ | ✅ | ⚠️ | 🔜 | Microsoft Virtual Hard Disk |
| VHDX | `.vhdx` | ✅ | ❌ | ✅ | ⚠️ | 🔜 | Virtual Hard Disk v2 |
| VMDK | `.vmdk` | ✅ | ❌ | ❌ | ❌ | 🔜 | VMware Virtual Disk |
| QCOW2 | `.qcow2` | ✅ | ❌ | ❌ | ⚠️ | 🔜 | QEMU Copy On Write |

---

## ISO 9660 (Primary Format)

### Structure

```
ISO 9660 Image
│
├── System Area (32,768 bytes)
│   └── May contain MBR (if hybrid ISO)
│
├── Volume Descriptors (starting at sector 16)
│   ├── Primary Volume Descriptor (PVD)
│   │   ├── System Identifier
│   │   ├── Volume Identifier
│   │   ├── Volume Space Size
│   │   ├── Logical Block Size (2048)
│   │   └── Root Directory Record
│   │
│   ├── Boot Record Volume Descriptor (BRVD)
│   │   └── El Torito Boot Specification
│   │
│   ├── Supplementary Volume Descriptor (Joliet)
│   │   └── Unicode filenames
│   │
│   └── Volume Descriptor Set Terminator
│
├── Path Table
│
├── Directory Records
│   └── File data
│
└── El Torito Boot Catalog (if bootable)
    ├── Validation Entry
    ├── Default Boot Entry (BIOS)
    └── Section Entries (EFI, etc.)
```

### Detection Code

```rust
pub fn detect_iso9660(path: &Path) -> Result<Iso9660Info> {
    let mut file = File::open(path)?;
    
    // Read Primary Volume Descriptor at sector 16
    file.seek(SeekFrom::Start(16 * 2048))?;
    let mut pvd = [0u8; 2048];
    file.read_exact(&mut pvd)?;
    
    // Check magic: CD001
    if &pvd[1..6] != b"CD001" {
        return Err(Error::NotIso9660);
    }
    
    // Parse PVD fields
    let system_id = String::from_utf8_lossy(&pvd[8..40]).trim().to_string();
    let volume_id = String::from_utf8_lossy(&pvd[40..72]).trim().to_string();
    let volume_size = u32::from_le_bytes(pvd[80..84].try_into()?) as u64 * 2048;
    let block_size = u16::from_le_bytes(pvd[128..130].try_into()?);
    
    // Check for Boot Record (El Torito)
    file.seek(SeekFrom::Start(17 * 2048))?;
    let mut brvd = [0u8; 2048];
    file.read_exact(&mut brvd)?;
    
    let el_torito = if &brvd[1..6] == b"CD001" && brvd[0] == 0x00 {
        Some(parse_el_torito(&file, &brvd)?)
    } else {
        None
    };
    
    Ok(Iso9660Info {
        system_id,
        volume_id,
        volume_size,
        block_size,
        el_torito,
        // ... more fields
    })
}
```

---

## Windows Imaging Format (WIM)

### Structure

```
WIM File
│
├── WIM Header (208 bytes)
│   ├── Magic: "MSWIM\0\0\0"
│   ├── Header Size
│   ├── Version (major.minor)
│   ├── Flags (compression, etc.)
│   ├── Chunk Size
│   ├── GUID
│   ├── Part Number
│   ├── Total Parts
│   ├── Image Count
│   ├── Offset Table
│   ├── XML Data offset
│   ├── Boot metadata offset
│   └── Integrity table offset
│
├── XML Metadata
│   └── Image descriptions (OS name, version, arch)
│
├── File Resources
│   └── Compressed file data (LZX, XPRESS, LZMS)
│
├── Metadata Resources
│   └── Directory structure, security descriptors
│
└── Integrity Table
    └── SHA1 hashes of chunks
```

### WIM XML Metadata Parsing

```rust
pub fn parse_wim_xml(path: &Path) -> Result<Vec<WimImage>> {
    // Read WIM header to find XML offset
    let header = read_wim_header(path)?;
    
    // Read XML data
    let xml_data = read_at_offset(path, header.xml_offset, header.xml_size)?;
    
    // Parse XML
    // <WIM>
    //   <IMAGE INDEX="1">
    //     <NAME>Windows 11 Pro</NAME>
    //     <DESCRIPTION>Windows 11 Pro</DESCRIPTION>
    //     <WINDOWS>
    //       <ARCH>9</ARCH>                    <!-- 0=x86, 9=x64, 12=ARM64 -->
    //       <PRODUCTNAME>...</PRODUCTNAME>
    //       <EDITIONID>Professional</EDITIONID>
    //       <VERSION>
    //         <MAJOR>10</MAJOR>
    //         <MINOR>0</MINOR>
    //         <BUILD>26100</BUILD>
    //       </VERSION>
    //     </WINDOWS>
    //   </IMAGE>
    // </WIM>
}
```

### Windows Architecture Codes

| Code | Architecture |
|------|-------------|
| 0 | x86 (32-bit) |
| 5 | ARM (32-bit) |
| 6 | IA64 (Itanium) |
| 9 | x86_64 (AMD64) |
| 12 | ARM64 (AArch64) |

---

## Raw Disk Images (.img)

### Detection

Raw images have no specific magic bytes. Detection is based on:

1. **Partition table at offset 0**:
   - MBR signature `0x55AA` at offset 510
   - GPT signature `EFI PART` at LBA 1 (offset 512)

2. **Filesystem at offset 0** (no partition table):
   - ext4 magic `0xEF53` at offset 1080
   - FAT magic at offset 0x36 or 0x52

### Handling

```rust
pub fn handle_raw_image(path: &Path) -> Result<ImageAction> {
    // Raw images are typically meant to be written directly (dd)
    // NOT suitable for multi-boot loopback mounting
    
    // Check if it's a partition image or full disk image
    let has_partition_table = check_partition_table(path)?;
    
    if has_partition_table {
        // Full disk image — can be written with dd
        Ok(ImageAction::DirectWrite)
    } else {
        // Partition image — needs to be written to a partition
        Ok(ImageAction::PartitionWrite)
    }
}
```

---

## VHD / VHDX (V2)

### VHD Structure

```
VHD File
│
├── Data blocks (dynamic or fixed)
│
└── Footer (512 bytes, at end of file)
    ├── Magic: "conectix"
    ├── Features
    ├── Version
    ├── Data Offset
    ├── Timestamp
    ├── Creator Application
    ├── Creator Version
    ├── Creator Host OS
    ├── Original Size
    ├── Current Size
    ├── Disk Geometry
    ├── Disk Type (fixed, dynamic, differencing)
    ├── Checksum
    └── UUID
```

### VHD Types

| Type | Description | Use in BootForge |
|------|-------------|-----------------|
| Fixed | Pre-allocated, full size on disk | Direct boot possible |
| Dynamic | Sparse, grows as needed | Convert to fixed or loopback |
| Differencing | Changes relative to parent | Not supported in V2 |

---

## Format Detection Pipeline

```rust
pub fn detect_format(path: &Path) -> Result<ImageFormat> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 8192];
    file.read_exact(&mut header)?;
    
    // Check by magic bytes (priority order)
    
    // ISO 9660: "CD001" at offset 0x8001
    if header.len() > 0x8005 && &header[0x8001..0x8006] == b"CD001" {
        return Ok(ImageFormat::Iso9660);
    }
    
    // WIM: "MSWIM\0\0\0" at offset 0
    if &header[0..8] == b"MSWIM\x00\x00\x00" {
        return Ok(ImageFormat::Wim);
    }
    
    // VHD: "conectix" (check footer at end of file)
    let file_size = file.metadata()?.len();
    if file_size >= 512 {
        file.seek(SeekFrom::End(-512))?;
        let mut footer = [0u8; 512];
        file.read_exact(&mut footer)?;
        if &footer[0..8] == b"conectix" {
            return Ok(ImageFormat::Vhd);
        }
    }
    
    // VHDX: "vhdxfile" at offset 0
    if &header[0..8] == b"vhdxfile" {
        return Ok(ImageFormat::Vhdx);
    }
    
    // VMDK: check for descriptor
    if header.starts_with(b"# Disk DescriptorFile") || header.starts_with(b"KDMV") {
        return Ok(ImageFormat::Vmdk);
    }
    
    // QCOW2: magic 0x514649FB at offset 0
    if &header[0..4] == &[0x51, 0x46, 0x49, 0xFB] {
        return Ok(ImageFormat::Qcow2);
    }
    
    // Raw: check for partition table
    if header[510] == 0x55 && header[511] == 0xAA {
        // Check for GPT
        if &header[512..520] == b"EFI PART" {
            return Ok(ImageFormat::RawGpt);
        }
        return Ok(ImageFormat::RawMbr);
    }
    
    // Fallback: try by extension
    match path.extension().and_then(|e| e.to_str()) {
        Some("iso") => Ok(ImageFormat::Iso9660),
        Some("img") => Ok(ImageFormat::RawUnknown),
        Some("wim") => Ok(ImageFormat::Wim),
        Some("esd") => Ok(ImageFormat::Esd),
        _ => Ok(ImageFormat::Unknown),
    }
}
```

---

## FAT32 Limitations & Workarounds

### The 4GB Problem

FAT32 has a maximum file size of 4,294,967,295 bytes (4 GB - 1 byte). Many modern ISOs exceed this:

| ISO | Typical Size | Exceeds 4GB? |
|-----|-------------|--------------|
| Windows 11 24H2 | ~6.2 GB | ✅ Yes |
| Ubuntu 24.04 Desktop | ~5.7 GB | ✅ Yes |
| Fedora 40 Workstation | ~2.2 GB | ❌ No |
| Alpine | ~200 MB | ❌ No |

### Solutions

| Solution | Implementation | Pros | Cons |
|----------|---------------|------|------|
| **exFAT data partition** | Store ISOs on exFAT, keep FAT32 only for EFI | No file size limit | Requires exFAT support in GRUB |
| **NTFS data partition** | Store ISOs on NTFS | No file size limit, Windows native | NTFS driver in GRUB needed |
| **Split WIM** | Split install.wim into < 4GB parts | Works on FAT32 | More complex, Windows-specific |
| **Ext4 data partition** | Store ISOs on ext4 | Full Linux support | Windows can't read without tools |

**BootForge V1 Strategy**: Use **exFAT for the data partition** with FAT32 only for the EFI System Partition. This provides:
- No 4GB file size limit
- Read/write support on macOS, Windows, and Linux
- GRUB exFAT module for boot-time access

---

## Filesystem Comparison for Boot Drives

| Feature | FAT32 | exFAT | NTFS | ext4 |
|---------|-------|-------|------|------|
| Max file size | 4 GB | 16 EB | 16 TB | 16 TB |
| Max volume | 2 TB | 128 PB | 256 TB | 1 EB |
| macOS read | ✅ | ✅ | ✅ | ❌ |
| macOS write | ✅ | ✅ | ❌ | ❌ |
| Windows read | ✅ | ✅ | ✅ | ❌ |
| Windows write | ✅ | ✅ | ✅ | ❌ |
| Linux read | ✅ | ✅ | ✅ | ✅ |
| Linux write | ✅ | ✅ | ✅ | ✅ |
| GRUB support | ✅ | ✅* | ✅* | ✅ |
| EFI boot | ✅ | ❌ | ❌ | ❌ |
| Journaling | ❌ | ❌ | ✅ | ✅ |
| Permissions | ❌ | ❌ | ✅ | ✅ |

*Requires GRUB module
