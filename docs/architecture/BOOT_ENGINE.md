# BootForge — Boot Engine Deep Dive

## Overview

The Boot Engine is the heart of BootForge. It manages the complex task of creating universal boot media that works across different firmware types, architectures, and operating systems.

---

## Boot Process Understanding

### How Computers Boot

```
Power On
   │
   ├─── BIOS/Legacy Path ─────────────────────┐
   │    │                                       │
   │    ▼                                       │
   │   MBR (512 bytes)                          │
   │    │                                       │
   │    ▼                                       │
   │   Bootloader Stage 1 (GRUB core)          │
   │    │                                       │
   │    ▼                                       │
   │   Bootloader Stage 2 (GRUB modules)       │
   │    │                                       │
   │    ▼                                       │
   │   Boot Menu ──── Select OS ──── Boot OS   │
   │                                            │
   ├─── UEFI Path ────────────────────────────┐│
   │    │                                      ││
   │    ▼                                      ││
   │   EFI System Partition                    ││
   │    │                                      ││
   │    ▼                                      ││
   │   EFI Application (BOOTx64.EFI)          ││
   │    │                                      ││
   │    ├─── Secure Boot? ──── Shim ──┐        ││
   │    │                             │        ││
   │    ▼                             ▼        ││
   │   GRUB2 EFI                  GRUB2 EFI   ││
   │    │                                      ││
   │    ▼                                      ││
   │   Boot Menu ──── Select OS ──── Boot OS   ││
   │                                            │
   └────────────────────────────────────────────┘
```

### What BootForge Must Handle

| Boot Scenario | Our Solution |
|---------------|-------------|
| UEFI x64 | Install GRUB2 EFI (grubx64.efi) in ESP |
| UEFI ARM64 | Install GRUB2 EFI (grubaa64.efi) in ESP |
| Legacy BIOS | Install GRUB2 to MBR + embed in BIOS boot partition |
| Secure Boot | Use Shim → GRUB2 chain with signed modules |
| Hybrid | GPT with both EFI partition + BIOS boot partition |

---

## GRUB2 Configuration

### Master Configuration (grub.cfg)

```grub
# BootForge Master GRUB Configuration
# Auto-generated — do not edit manually

set default=0
set timeout=10
set gfxmode=auto
set gfxpayload=keep

# Load modules
insmod all_video
insmod gfxterm
insmod png
insmod part_gpt
insmod part_msdos
insmod fat
insmod exfat
insmod ntfs
insmod ext2
insmod iso9660
insmod loopback
insmod search
insmod search_fs_uuid

# Set theme
set theme=/bootforge/themes/default/theme.txt

# Detect BootForge partition
search --no-floppy --set=root --label "BOOTFORGE"

# Include generated entries
source /bootforge/grub/entries.cfg
```

### Entry Templates

**Linux ISO (GRUB Loopback)**:
```grub
menuentry "Ubuntu 24.04 LTS" --class ubuntu --class linux {
    set iso_path="/images/linux/ubuntu-24.04-desktop-amd64.iso"
    loopback loop $iso_path
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$iso_path quiet splash ---
    initrd (loop)/casper/initrd
}
```

**Linux with Persistence**:
```grub
menuentry "Ubuntu 24.04 LTS (Persistent)" --class ubuntu --class linux {
    set iso_path="/images/linux/ubuntu-24.04-desktop-amd64.iso"
    loopback loop $iso_path
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$iso_path persistent quiet splash ---
    initrd (loop)/casper/initrd
}
```

**Windows ISO**:
```grub
menuentry "Windows 11" --class windows {
    set iso_path="/images/windows/Win11_24H2_x64.iso"
    loopback loop $iso_path
    
    # Method 1: Direct chainload (most compatible)
    chainloader (loop)/efi/boot/bootx64.efi
    
    # Method 2: WIMBOOT (fallback)
    # linux16 /bootforge/wimboot
    # initrd16 \
    #   newc:bcd:(loop)/boot/bcd \
    #   newc:boot.sdi:(loop)/boot/boot.sdi \
    #   newc:boot.wim:(loop)/sources/boot.wim
}
```

**Rescue/Utility ISO**:
```grub
menuentry "Clonezilla" --class clonezilla --class rescue {
    set iso_path="/images/rescue/clonezilla-live-3.1.1-amd64.iso"
    loopback loop $iso_path
    linux (loop)/live/vmlinuz boot=live union=overlay username=user config \
        components noswap edd=on nomodeset locales= keyboard-layouts= \
        ocs_live_run="ocs-live-general" ocs_live_extra_param="" \
        ocs_live_batch=no vga=788 ip= net.ifnames=0 \
        nosplash findiso=$iso_path
    initrd (loop)/live/initrd.img
}
```

---

## Boot Method Matrix

| OS | UEFI | Legacy BIOS | Secure Boot | Method | Notes |
|----|------|-------------|-------------|--------|-------|
| Windows 11 | ✅ | ✅ | ✅ | WIM/Chainload | Requires BCD + WIM extraction for Legacy |
| Windows 10 | ✅ | ✅ | ✅ | WIM/Chainload | Similar to Win11 |
| Ubuntu | ✅ | ✅ | ✅ | GRUB Loopback | Most compatible |
| Debian | ✅ | ✅ | ✅ | GRUB Loopback | Standard Linux boot |
| Fedora | ✅ | ✅ | ✅ | GRUB Loopback | Dracut-based initrd |
| Linux Mint | ✅ | ✅ | ⚠️ | GRUB Loopback | Secure Boot varies by version |
| Arch | ✅ | ✅ | ❌ | GRUB Loopback | No Secure Boot by default |
| Alpine | ✅ | ✅ | ❌ | GRUB Loopback | Minimal ISO |
| Kali | ✅ | ✅ | ⚠️ | GRUB Loopback | Secure Boot varies |
| Hiren's PE | ✅ | ❌ | ❌ | WIMBOOT | Windows PE based |
| Clonezilla | ✅ | ✅ | ❌ | GRUB Loopback | Debian-based live |
| GParted | ✅ | ✅ | ❌ | GRUB Loopback | Debian-based live |
| SystemRescue | ✅ | ✅ | ❌ | GRUB Loopback | Arch-based |
| MemTest86 | ✅ | ✅ | ✅ | Direct EFI/MEMDISK | Standalone utility |

---

## Windows Boot Handling

Windows ISOs require special handling because they don't follow standard Linux boot conventions.

### Windows ISO Structure

```
Windows ISO
├── boot/
│   ├── bcd            # Boot Configuration Data
│   ├── boot.sdi       # System Deployment Image
│   └── fonts/
├── efi/
│   └── boot/
│       └── bootx64.efi
├── sources/
│   ├── boot.wim       # Windows PE boot image (~500MB)
│   └── install.wim    # Windows installation image (~4-6GB)
└── autorun.inf
```

### Windows Boot Methods

**Method 1: EFI Chainload (Preferred for UEFI)**
```
GRUB2 → Chainload bootx64.efi from ISO → Windows PE → Installer
```

**Method 2: WIMBOOT (Most Compatible)**
```
GRUB2 → wimboot → Load BCD + boot.sdi + boot.wim → Windows PE → Installer
```

**Method 3: Direct WIM (Fallback)**
```
Extract boot.wim → Mount as RAM disk → Boot Windows PE
```

### Windows-Specific Challenges

| Challenge | Solution |
|-----------|----------|
| install.wim > 4GB (can't fit on FAT32 EFI) | Split WIM or use exFAT for data partition |
| BCD must be patched | Generate new BCD entries pointing to ISO location |
| WinPE needs drivers | Include VirtIO / USB3 drivers in boot.wim if needed |
| NTFS dependency | Use exFAT (natively supported) for image storage |

---

## Secure Boot Handling

### Secure Boot Chain

```
Firmware → Shim (signed by Microsoft) → GRUB2 (signed by distro) → Kernel (signed)
```

### Implementation

```
EFI/BOOT/
├── BOOTx64.EFI          # Shim (Microsoft-signed)
├── grubx64.efi           # GRUB2 (signed with our or distro key)
├── mmx64.efi             # MOK Manager (for custom keys)
└── shimx64.efi           # Shim (alternate name)
```

**Shim Sources**:
- Use pre-built shim from a signed distro (e.g., Ubuntu's shim)
- Shim validates GRUB2, GRUB2 validates kernels
- Users can enroll custom MOK (Machine Owner Key) for unsigned kernels

### Secure Boot Compatibility Matrix

| Scenario | Secure Boot | Solution |
|----------|-------------|----------|
| Windows 11 | ✅ Required | Chainload Windows bootloader (already signed) |
| Ubuntu/Fedora | ✅ Supported | Use distro's signed shim + GRUB |
| Arch/Kali | ❌ Unsigned | Disable Secure Boot or enroll MOK |
| Rescue ISOs | ❌ Usually unsigned | Disable Secure Boot or use MOK |

---

## Persistence Engine

### Linux Persistence

Persistence allows saving changes across reboots when booting from a live USB.

**Supported Methods**:

| Distro | Method | File/Partition |
|--------|--------|----------------|
| Ubuntu | casper-rw | File or partition labeled `casper-rw` |
| Debian | persistence.conf | File or partition with `persistence.conf` |
| Fedora | LiveOS overlay | Separate overlay partition |
| Linux Mint | casper-rw | Same as Ubuntu (Ubuntu-based) |
| Kali | persistence.conf | Same as Debian (Debian-based) |

**Implementation**:

```
Persistence Partition (ext4, label: persistence)
├── persistence.conf    # Contains "/ union"
└── (saved data)
```

Or file-based:

```
BootForge Data Partition
└── persistence/
    └── ubuntu/
        └── casper-rw    # Pre-created ext4 file (user-configured size)
```

---

## Boot Menu Theming

### Default Theme

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│                      B O O T F O R G E                   │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │                                                    │  │
│  │  ▶  Windows 11                           [UEFI]   │  │
│  │     Ubuntu 24.04 LTS                     [UEFI]   │  │
│  │     Ubuntu 24.04 LTS (Persistent)        [UEFI]   │  │
│  │     Fedora 40 Workstation                [UEFI]   │  │
│  │     Linux Mint 22                        [UEFI]   │  │
│  │     ─────────────────────────────────             │  │
│  │     Clonezilla                           [BIOS]   │  │
│  │     SystemRescue                         [UEFI]   │  │
│  │     MemTest86                            [UEFI]   │  │
│  │     ─────────────────────────────────             │  │
│  │     Boot from local disk                          │  │
│  │     UEFI Firmware Settings                        │  │
│  │                                                    │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  Use ↑↓ to select, Enter to boot, 'e' to edit           │
│  Created with BootForge — bootforge.dev                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### GRUB Theme Structure

```
bootforge/themes/default/
├── theme.txt
├── background.png
├── icons/
│   ├── windows.png
│   ├── ubuntu.png
│   ├── debian.png
│   ├── fedora.png
│   ├── linux.png
│   ├── rescue.png
│   ├── utility.png
│   └── settings.png
├── fonts/
│   ├── inter-regular-16.pf2
│   └── inter-bold-24.pf2
└── select_c.png        # Selection highlight
```

---

## Error Handling in Boot Engine

### Boot Failures

| Error | Detection | Recovery |
|-------|-----------|----------|
| ISO not found on drive | GRUB can't locate file | Show error with path, suggest re-creating |
| Wrong architecture | x86 ISO on ARM hardware | Show architecture mismatch message |
| Secure Boot rejection | Shim/GRUB signature invalid | Offer to disable Secure Boot in firmware |
| Missing kernel | vmlinuz not at expected path | Try alternative paths, check ISO integrity |
| initrd failure | Can't load initial ramdisk | Verify ISO, try alternative boot method |
| Windows BCD corrupt | WinPE fails to start | Regenerate BCD, try WIMBOOT fallback |

### Fallback Chain

```
Primary Method (auto-detected)
    │
    ├─ Failed → Fallback Method 1
    │              │
    │              ├─ Failed → Fallback Method 2
    │              │              │
    │              │              └─ Failed → Show diagnostic info
    │              │
    │              └─ Success → Boot
    │
    └─ Success → Boot
```

For example, Windows boot fallback:
1. **Try**: EFI chainload from ISO
2. **Fallback 1**: WIMBOOT with extracted BCD
3. **Fallback 2**: Manual BCD with direct WIM load
4. **Last resort**: Show diagnostic, suggest alternative boot method
