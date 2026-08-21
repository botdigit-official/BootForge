# BootForge — Data Flow Diagrams

## Overview

This document describes the complete data flow through BootForge for all major user operations.

---

## 1. Application Launch Flow

```
App Start
    │
    ▼
┌─────────────────┐
│ Initialize Core  │
│                 │
│ • Load config   │
│ • Init logging  │
│ • Load compat DB│
│ • Check updates │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Start Watchers   │
│                 │
│ • USB monitor   │
│ • File watcher  │
│ • Event loop    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Detect Devices   │
│                 │
│ • Enumerate USB │
│ • Filter system │
│ • Read health   │
│ • Get partitions│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Render UI        │
│                 │
│ • Show devices  │
│ • Show status   │
│ • Ready for user│
└─────────────────┘
```

---

## 2. ISO Import Flow

```
User drops ISO file(s)
         │
         ▼
┌─────────────────────┐
│ File Validation      │
│                     │
│ • File exists?      │
│ • File readable?    │
│ • Size > 0?         │
│ • Supported format? │
│ • Not duplicate?    │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │ Invalid   │ Valid
     │           │
     ▼           ▼
  Show Error   ┌─────────────────────┐
               │ Start Analysis       │
               │ (Background Thread)  │
               │                     │
               │ UI shows: "Analyzing │
               │ Ubuntu.iso..."       │
               └──────────┬──────────┘
                          │
              ┌───────────┼───────────┐
              │           │           │
              ▼           ▼           ▼
        ┌──────────┐┌──────────┐┌──────────┐
        │ Hash     ││ Format   ││ Boot     │
        │ Compute  ││ Detect   ││ Analyze  │
        │          ││          ││          │
        │ SHA256   ││ ISO9660  ││ El Torito│
        │ progress ││ UDF      ││ EFI scan │
        │ bar      ││ WIM      ││ MBR/GRUB │
        └────┬─────┘└────┬─────┘└────┬─────┘
             │           │           │
             └───────────┼───────────┘
                         │
                         ▼
               ┌─────────────────────┐
               │ OS Identification    │
               │                     │
               │ • File markers      │
               │ • Version files     │
               │ • Volume ID         │
               │ • Boot config       │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Architecture Detect  │
               │                     │
               │ • EFI binary PE hdr │
               │ • Kernel binary     │
               │ • Filename patterns │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Compatibility Check  │
               │                     │
               │ • Database lookup   │
               │ • Quirk detection   │
               │ • Score calculation │
               │ • Strategy select  │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Update UI           │
               │                     │
               │ • Show ISO card     │
               │ • Show score        │
               │ • Show boot info    │
               │ • Enable actions    │
               └─────────────────────┘
```

---

## 3. Create Boot Drive Flow

```
User clicks "CREATE BOOT DRIVE"
         │
         ▼
┌─────────────────────┐
│ Pre-flight Checks    │
│                     │
│ • Drive selected?   │
│ • ISOs added?       │
│ • Enough space?     │
│ • Not system drive? │
│ • Drive writable?   │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │ Fail      │ Pass
     │           │
     ▼           ▼
  Show Error   ┌─────────────────────┐
               │ Confirmation Dialog  │
               │                     │
               │ "This will erase:   │
               │  SanDisk 256GB      │
               │  /dev/disk2         │
               │                     │
               │  ⚠ ALL DATA WILL   │
               │  BE PERMANENTLY     │
               │  DELETED            │
               │                     │
               │ [Cancel] [ERASE &   │
               │          CREATE]"   │
               └──────────┬──────────┘
                          │
                    User confirms
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 1: UNMOUNT     │
               │                     │
               │ • Unmount all vols  │
               │ • Release handles   │
               │ • Verify unmounted  │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 2: PARTITION   │  Progress: 5%
               │                     │
               │ • Create GPT table  │
               │ • Create EFI part   │
               │   (FAT32, 512MB)    │
               │ • Create BIOS part  │
               │   (1MB, if needed)  │
               │ • Create data part  │
               │   (exFAT, remaining)│
               │ • Create persist    │
               │   (ext4, if needed) │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 3: FORMAT      │  Progress: 10%
               │                     │
               │ • Format EFI (FAT32)│
               │ • Format data       │
               │   (exFAT)           │
               │ • Format persistence│
               │   (ext4, if needed) │
               │ • Set labels        │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 4: BOOTLOADER  │  Progress: 15%
               │                     │
               │ • Install GRUB2 EFI │
               │ • Install GRUB2     │
               │   BIOS (if needed)  │
               │ • Install Shim      │
               │   (Secure Boot)     │
               │ • Copy GRUB modules │
               │ • Copy theme files  │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 5: COPY ISOs   │  Progress: 20-90%
               │                     │
               │ For each ISO:       │
               │ • Create category   │
               │   directory         │
               │ • Copy ISO file     │
               │   (with progress)   │
               │ • Verify copy       │
               │   (hash check)      │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 6: CONFIGURE   │  Progress: 90%
               │                     │
               │ • Generate grub.cfg │
               │ • Generate entries  │
               │ • Write boot menu   │
               │ • Configure persist │
               │ • Write metadata    │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 7: VERIFY      │  Progress: 95%
               │                     │
               │ • Verify bootloader │
               │ • Verify ISO files  │
               │ • Verify config     │
               │ • Test GRUB parse   │
               │ • Sync filesystem   │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │ Phase 8: COMPLETE    │  Progress: 100%
               │                     │
               │ • Sync & flush      │
               │ • Safely eject      │
               │ • Show success      │
               │ • Show boot guide   │
               └─────────────────────┘
```

---

## 4. OS Download Flow

```
User opens "Download OS"
         │
         ▼
┌─────────────────────┐
│ Load Download Catalog│
│                     │
│ • Built-in catalog  │
│ • Online updates    │
│ • Show categories   │
│ • Show popular      │
└──────────┬──────────┘
           │
      User selects OS
           │
           ▼
┌─────────────────────┐
│ Show Download Options│
│                     │
│ • Version           │
│ • Architecture      │
│ • Edition           │
│ • Size              │
│ • Source URL         │
└──────────┬──────────┘
           │
      User confirms
           │
           ▼
┌─────────────────────┐
│ Download ISO         │
│                     │
│ • Download from     │
│   official source   │
│ • Progress bar      │
│ • Speed display     │
│ • Pause/Resume      │
│ • ETA               │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Verify Download      │
│                     │
│ • Download hash file│
│ • Compare SHA256    │
│ • Show result       │
│ • GPG verify if     │
│   available         │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │ Mismatch  │ Match
     │           │
     ▼           ▼
  Warn user   ┌─────────────────────┐
  (re-download │ Auto-add to ISO list │
  or proceed)  │                     │
               │ • Run ISO analysis  │
               │ • Add to UI         │
               │ • Ready for use     │
               └─────────────────────┘
```

---

## 5. VM Test Flow

```
User clicks "Test in VM"
         │
         ▼
┌─────────────────────┐
│ Check QEMU           │
│                     │
│ • QEMU installed?   │
│ • OVMF available?   │
│ • Enough resources? │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │ Not found │ Found
     │           │
     ▼           ▼
  Offer to    ┌─────────────────────┐
  install     │ Configure VM         │
              │                     │
              │ • Select firmware:  │
              │   BIOS / UEFI /     │
              │   Secure Boot       │
              │ • Memory: 2-4GB     │
              │ • CPU: 2 cores      │
              └──────────┬──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │ Launch QEMU          │
              │                     │
              │ qemu-system-x86_64  │
              │  -m 2048            │
              │  -smp 2             │
              │  -bios OVMF.fd      │
              │  -cdrom image.iso   │
              │  -boot d            │
              │  -display gtk       │
              └──────────┬──────────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
        Boot OK    Boot Slow    Boot Fail
              │          │          │
              ▼          ▼          ▼
         ┌────────┐ ┌────────┐ ┌────────┐
         │ Report │ │ Report │ │ Report │
         │SUCCESS │ │WARNING │ │ FAIL   │
         │        │ │timeout │ │ error  │
         │ time,  │ │        │ │ log,   │
         │ screen │ │ try    │ │ suggest│
         │ shot   │ │ other  │ │ fix    │
         └────────┘ │ mode   │ └────────┘
                    └────────┘
```

---

## 6. Device Hot-Plug Flow

```
USB Device Plugged In
         │
         ▼
┌─────────────────────┐
│ Platform Event       │
│                     │
│ macOS: DiskArb      │
│ Windows: WMI        │
│ Linux: udev         │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Filter Event         │
│                     │
│ • Is it storage?    │
│ • Is it USB?        │
│ • Is it removable?  │
│ • NOT system drive? │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │ Ignore    │ Process
     │(system/   │
     │ non-USB)  │
     ▼           ▼
   (nothing)  ┌─────────────────────┐
              │ Read Device Info     │
              │                     │
              │ • Vendor, model     │
              │ • Capacity          │
              │ • Interface/speed   │
              │ • Health            │
              │ • Partitions        │
              └──────────┬──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │ Check Existing BF    │
              │                     │
              │ • Has BootForge     │
              │   partition?        │
              │ • Has bootforge/    │
              │   config.json?      │
              │ • Read existing     │
              │   configuration     │
              └──────────┬──────────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
         New Drive   BootForge   Other Data
              │      Drive        │
              ▼          │          ▼
         Show as      Show as     Show as
         "Available"  "Managed"   "Has Data"
         with format  with ISO    with warning
         options      list


USB Device Unplugged
         │
         ▼
┌─────────────────────┐
│ Remove from UI       │
│                     │
│ • If writing:       │
│   ABORT + ERROR     │
│ • If idle:          │
│   Clean removal     │
│ • Update device list│
└─────────────────────┘
```

---

## 7. Error Recovery Flows

### Write Failure Recovery

```
Write Error Detected
         │
         ▼
┌─────────────────────┐
│ Classify Error       │
│                     │
│ • I/O error → retry │
│ • Drive removed →   │
│   abort              │
│ • Space full →      │
│   show space error   │
│ • Permission →      │
│   re-request auth    │
└──────────┬──────────┘
           │
     ┌─────┼─────┐
     │     │     │
  Retry  Abort  Re-auth
     │     │     │
     ▼     ▼     ▼
  Resume  Show  Elevate
  from    error privileges
  last    + clean then
  known   up     resume
  good
  point
```

### Corrupted ISO Recovery

```
Hash Mismatch Detected
         │
         ▼
┌─────────────────────┐
│ Show Warning         │
│                     │
│ "ISO hash does not  │
│  match expected     │
│  value.             │
│                     │
│  Expected: abc123...│
│  Got:      def456...│
│                     │
│  This file may be   │
│  corrupted."        │
│                     │
│ [Re-download]       │
│ [Use Anyway (⚠)]   │
│ [Remove]            │
└─────────────────────┘
```
