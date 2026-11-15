# BootForge User Guide

One USB drive that starts Windows, Linux, BSD and rescue tools on old and new computers.

- [1. What you need](#1-what-you-need)
- [2. Make the drive](#2-make-the-drive)
- [3. Add operating systems](#3-add-operating-systems)
- [4. Check where an image will boot](#4-check-where-an-image-will-boot)
- [5. Start a computer from the drive](#5-start-a-computer-from-the-drive)
- [6. Old computers](#6-old-computers)
- [7. Macs](#7-macs)
- [8. Troubleshooting](#8-troubleshooting)
- [9. How the drive is built](#9-how-the-drive-is-built)

---

## 1. What you need

| | |
|---|---|
| A USB drive or external SSD | 8 GB minimum, 32–128 GB recommended, **2 TB maximum**. Everything on it is erased once. |
| A Mac to create the drive | macOS 11 or newer (Intel or Apple Silicon). Linux/Windows: see [9](#9-how-the-drive-is-built). |
| Image files | `.iso`, `.wim`, `.vhd`, `.vhdx`, `.img`, `.efi` — official downloads from the OS vendor. |

> BootForge refuses to write to internal disks, to the disk macOS runs from and to the disk BootForge itself runs from — even if that disk is external (for example the drive holding your projects).

## 2. Make the drive

**App:** open BootForge → pick the USB drive on the left → *New drive (erase & make bootable)* → **Make multi-boot drive** → type the disk name to confirm → enter your Mac password in the macOS dialog.

**Terminal:**

```bash
bootforge devices
```

```bash
bootforge format disk6 --label BOOTFORGE
```

You type the disk name once more to confirm. It takes under a minute. Result:

| Partition | Size | Contents |
|---|---|---|
| `BOOTFORGE` (exFAT) | rest of the drive | your image files, BootForge settings |
| `VTOYEFI` (FAT) | 32 MiB | boot loaders for UEFI x64, 32-bit UEFI, ARM64 and Secure Boot |
| (hidden, before partition 1) | 1 MiB | Legacy BIOS boot code for old PCs |

## 3. Add operating systems

Copy image files onto the `BOOTFORGE` drive in Finder. That is all: the boot menu lists every image it finds, in any folder. The `images/windows`, `images/linux`, `images/rescue` folders are only for tidiness.

- Files larger than 4 GB are fine (exFAT).
- Copy, don't rename the extension. A file that is still copying or 0 bytes will not boot.
- Eject the drive before unplugging it.

Good images for old computers: **antiX**, **Puppy Linux**, **MX Linux**, **Lubuntu**, **Linux Mint XFCE**, **Q4OS**, **Debian** (32-bit `i386` builds exist for antiX, Puppy, MX and Debian).

## 4. Check where an image will boot

```bash
bootforge analyze ~/Downloads/linuxmint-22.3-cinnamon-64bit.iso
```

prints a one-line verdict for each kind of computer:

```
 ❌ Very old PC (32-bit CPU, BIOS)       This is a 64-bit image; the CPU is 32-bit
 ✅ Older PC (64-bit CPU, BIOS / CSM)    Boots through the Legacy BIOS boot code
 ✅ UEFI PC (Secure Boot off)            Boots in UEFI mode
 🟡 Modern PC (UEFI, Secure Boot on)     Works with Secure Boot after a one-time key enrollment
 ✅ Intel Mac (no T2)                    Boots in EFI mode from the Option-key menu
 ❌ Apple Silicon Mac (M1 and later)     Apple Silicon cannot boot x86 operating systems
```

Step-by-step instructions for one kind of computer:

```bash
bootforge guide ~/Downloads/linuxmint-22.3-cinnamon-64bit.iso --device old64
```

Device names: `old32`, `old64`, `uefi`, `secureboot`, `uefi32`, `intelmac`, `t2`, `applesilicon`, `arm64`. `bootforge targets` lists them with example models. The app shows the same information under *Will it boot?* after you add an image.

## 5. Start a computer from the drive

1. Plug in the drive and switch the computer on while pressing its **boot-menu key**:

   | Brand | Key |
   |---|---|
   | Dell, Lenovo ThinkPad, Acer, Toshiba, Gigabyte | F12 |
   | HP | F9 (or Esc, then F9) |
   | ASUS | F8 or Esc |
   | MSI, ASRock | F11 |
   | Lenovo IdeaPad | the small Novo button / pinhole |
   | Intel Mac | hold Option (⌥) |

2. Pick the USB drive. If it is listed twice, `UEFI: …` is for modern mode and the plain entry is Legacy BIOS mode.
3. Choose an image in the BootForge menu. Press **F6** for *BootForge Tools* (system info, start the Windows/Linux already installed on the computer, memory test, firmware settings).

**Secure Boot (first time only per computer):** a blue *Verification failed* screen appears. Choose *OK → Enroll key from disk → VTOYEFI → ENROLL_THIS_KEY_IN_MOKMANAGER.cer → Continue → Yes → Reboot*. Alternatively turn Secure Boot off in the firmware settings.

**Windows 11 on unsupported PCs:** BootForge turns on the setup bypass for TPM 2.0, Secure Boot, CPU and 4 GB RAM checks. Windows installs, but Microsoft does not support such machines and may withhold updates.

## 6. Old computers

| Computer | What works | What to do |
|---|---|---|
| Pentium 4 / Pentium M / Atom N270 netbook (32-bit CPU) | 32-bit images only | Use `i386`/`i686` builds; 64-bit images cannot run |
| Core 2 / 1st–3rd gen Core (ThinkPad T400/R400/X200/T420…) | Linux, Windows 7–11, rescue tools | Choose the plain (non-UEFI) USB entry |
| 2012–2019 PCs | Everything | `UEFI:` entry; Legacy for Windows XP/Vista/7 |
| 2020+ PCs without Legacy/CSM | UEFI images only | Windows 7 and older will not start |

**Blinking cursor and nothing else** on an old PC: in BootForge choose *Fix Legacy BIOS boot* (or `write_mbr_core /dev/diskN` as admin). It rewrites only the BIOS boot code; files are kept.

**Image starts but screen goes black:** pick the image again and choose *Boot in grub2 mode* in the second menu; in the distribution's own menu press `e`, add `nomodeset` to the line starting with `linux`, then press F10.

**Windows XP setup cannot find a disk:** set SATA mode to *IDE / Compatible* in the BIOS.

## 7. Macs

| Mac | Windows / Linux from the BootForge drive | macOS installer |
|---|---|---|
| Intel Mac 2008–2017 | Yes. Hold ⌥, choose **EFI Boot**. Install Boot Camp drivers in Windows afterwards. | Separate drive (below) |
| Intel Mac with T2 (2018–2020) | Yes, after *Recovery (⌘R) → Startup Security Utility → No Security + Allow booting from external media*. Linux needs a T2 kernel for keyboard/trackpad. | Separate drive |
| Apple Silicon (M1+) | **No.** These Macs cannot boot x86 systems or generic ARM ISOs. Use UTM/Parallels, or Asahi Linux. | Separate drive |

### macOS installer drive

A macOS installer cannot be started from the multi-boot menu; it needs its own drive, made with Apple's `createinstallmedia`:

```bash
softwareupdate --fetch-full-installer --full-installer-version 15.1
```

```bash
bootforge macos-installer --app "/Applications/Install macOS Sequoia.app" --disk disk7
```

Boot it with ⌥ (Intel) or by holding the power button (Apple Silicon). The installer only runs on Macs that version of macOS supports.

## 8. Troubleshooting

| Symptom | Fix |
|---|---|
| Drive not in the boot menu at all | Try another USB port (USB 2.0 ports on old PCs), enable *USB boot* in firmware, disable *Fast Boot*. |
| Blinking cursor (old PC) | *Fix Legacy BIOS boot* (section 6). |
| `Verification failed` / security violation | Enroll the key once (section 5) or turn Secure Boot off. |
| Image missing from the menu | Re-copy it (a 0-byte or half-copied file is skipped) and make sure it ends in `.iso`/`.wim`/`.vhd`/`.vhdx`/`.img`/`.efi`. |
| "No bootfile found for UEFI" | The image is BIOS-only: boot the computer in Legacy/CSM mode. |
| Windows setup asks for a driver | Use the USB 2.0 port, or choose *WIMBOOT* mode in the secondary menu. |
| Want to start over | Format again. Images are erased. |

## 9. How the drive is built

```text
Sector 0          MBR boot code + partition table (partition 1 active)
Sectors 1–2047    GRUB stage 2 for Legacy BIOS
Partition 1       exFAT "BOOTFORGE"  — images, /ventoy/ventoy.json, theme, F6 tools page
Partition 2       32 MiB FAT "VTOYEFI" — BOOTX64.EFI, BOOTIA32.EFI, BOOTAA64.EFI, shim + MokManager
```

This is the standard layout of the bundled [Ventoy](https://www.ventoy.net) boot loader (GPL-3.0), which does the image booting. BootForge adds the safe writer, device guidance, theme, tools page and app. Because the layout is standard:

- On **Linux**, `src/managers/assets/ventoy_core/Ventoy2Disk.sh -i /dev/sdX` creates an equivalent drive; then copy BootForge's `ventoy/` folder onto it.
- On **Windows**, Ventoy2Disk.exe from ventoy.net does the same.

MBR is used instead of GPT because it is the only layout old BIOS PCs, UEFI PCs and Macs all accept; it limits drives to 2 TB.
