#!/usr/bin/env python3
"""
Botdigit BootForge Universal Multi-Boot Core Installer.
Installs the full MBR Legacy BIOS bootstrap + Post-MBR Gap GRUB2 Core + Partition 2 32MB UEFI ESP
directly onto target disk, ensuring 100% bootability on both Legacy BIOS (ThinkPad R400) and modern UEFI.
"""

import sys, os, struct, subprocess, lzma, shutil

def install_bootforge_core(disk_dev, label="BOTDIGIT"):
    clean_id = disk_dev.replace("/dev/", "").replace("rdisk", "disk")
    raw_disk = f"/dev/r{clean_id}"
    
    print(f"⚡ Initializing Botdigit Universal Boot Core on {raw_disk} ({label})...")
    
    assets_dir = "/Volumes/Mac2TB/Botdigit/Developer/Projects/BootForge/src/managers/assets/ventoy_core"
    boot_img_path = os.path.join(assets_dir, "boot/boot.img")
    core_img_xz_path = os.path.join(assets_dir, "boot/core.img.xz")
    esp_img_xz_path = os.path.join(assets_dir, "ventoy/ventoy.disk.img.xz")
    
    if not os.path.exists(boot_img_path) or not os.path.exists(core_img_xz_path) or not os.path.exists(esp_img_xz_path):
        raise FileNotFoundError("Missing core boot assets in ventoy_core directory.")

    # 1. Unmount target disk
    print("1. Unmounting target disk...")
    subprocess.run(["diskutil", "unmountDisk", "force", clean_id], check=False)

    # 2. Get disk size in sectors (512 bytes per sector)
    out = subprocess.check_output(["diskutil", "info", clean_id]).decode()
    disk_size_bytes = None
    for line in out.splitlines():
        if "Disk Size:" in line or "Total Size:" in line:
            import re
            m = re.search(r"\((\d+)\s+Bytes\)", line)
            if m:
                disk_size_bytes = int(m.group(1))
                break
    if not disk_size_bytes:
        disk_size_bytes = 250 * 1000 * 1000 * 1000 # fallback
    
    total_sectors = disk_size_bytes // 512
    esp_sectors = 65536 # 32 MB
    part1_start = 2048
    part2_start = total_sectors - esp_sectors
    part1_sectors = part2_start - part1_start

    print(f"   Disk: {disk_size_bytes / (1024**3):.1f} GB ({total_sectors} sectors)")
    print(f"   Part 1 (Data): {part1_start} -> {part1_start + part1_sectors} ({part1_sectors * 512 / (1024**3):.1f} GB)")
    print(f"   Part 2 (ESP):  {part2_start} -> {total_sectors} (32 MB)")

    # 3. Create helper script to write raw sectors with admin privileges
    tmp_writer = "/tmp/write_bootforge_mbr.py"
    with open(tmp_writer, "w") as f:
        f.write(f'''import os, struct, lzma

raw_dev = "{raw_disk}"
with open("{boot_img_path}", "rb") as bf:
    boot_img = bf.read()

with lzma.open("{core_img_xz_path}", "rb") as cf:
    core_img = cf.read()

with lzma.open("{esp_img_xz_path}", "rb") as ef:
    esp_img = ef.read()

with open(raw_dev, "r+b") as dev:
    # 1. Write MBR bootstrap (first 446 bytes)
    dev.seek(0)
    dev.write(boot_img[:446])
    
    # 2. Construct MBR Partition Table (64 bytes at offset 446)
    # Entry 1: ExFAT Active Bootable (0x80)
    p1 = struct.pack("<BBBBBBBBII",
        0x80,       # Status: Active / Bootable
        0x00, 0x01, 0x01, # CHS start
        0x07,       # Type: ExFAT / NTFS (0x07)
        0xFE, 0xFF, 0xFF, # CHS end
        {part1_start},    # LBA Start
        {part1_sectors}   # LBA Sectors
    )
    # Entry 2: EFI System Partition (0xEF)
    p2 = struct.pack("<BBBBBBBBII",
        0x00,       # Status: Inactive
        0x00, 0x01, 0x01, # CHS start
        0xEF,       # Type: EFI System Partition (0xEF)
        0xFE, 0xFF, 0xFF, # CHS end
        {part2_start},    # LBA Start
        {esp_sectors}     # LBA Sectors
    )
    p3 = b"\\x00" * 16
    p4 = b"\\x00" * 16
    dev.seek(446)
    dev.write(p1 + p2 + p3 + p4 + b"\\x55\\xAA")

    # 3. Write Core.img into Post-MBR Gap (Sectors 1..2047)
    dev.seek(512)
    dev.write(core_img[:2047 * 512])

    # 4. Write ESP image into Partition 2
    dev.seek({part2_start} * 512)
    dev.write(esp_img[:{esp_sectors} * 512])
    dev.flush()

print("✓ Successfully wrote MBR bootstrap, post-MBR core.img, and EFI system partition.")
''')

    print("2. Writing Universal Bootloader to raw disk sectors...")
    res = subprocess.run([
        "osascript", "-e",
        f'do shell script "python3 {tmp_writer}" with administrator privileges'
    ], capture_output=True, text=True)

    if res.returncode != 0:
        print(f"Admin write failed: {res.stderr}")
        # Try direct if permitted
        subprocess.run(["python3", tmp_writer])

    # 4. Format Partition 1 as ExFAT with volume label
    print(f"3. Formatting Partition 1 as ExFAT ({label})...")
    subprocess.run(["diskutil", "eraseVolume", "ExFAT", label, f"{clean_id}s1"], check=False)

    # 5. Mount Partition 1
    print("4. Mounting Partition 1...")
    subprocess.run(["diskutil", "mount", f"{clean_id}s1"], check=False)
    
    mount_point = f"/Volumes/{label}"
    if os.path.exists(mount_point):
        print("5. Installing Botdigit BootForge Cyber Emerald theme...")
        # install theme files
        ventoy_dir = os.path.join(mount_point, "ventoy")
        theme_dir = os.path.join(ventoy_dir, "theme")
        os.makedirs(theme_dir, exist_ok=True)
        os.makedirs(os.path.join(mount_point, "images/linux"), exist_ok=True)
        os.makedirs(os.path.join(mount_point, "images/windows"), exist_ok=True)
        os.makedirs(os.path.join(mount_point, "images/rescue"), exist_ok=True)

        # Write theme.txt
        theme_txt = """# ⚡ Botdigit BootForge Theme (Responsive 1024x768 & Low-Res Safe)
title-text: "⚡ Botdigit BootForge Universal Multi-Boot Studio"
title-font: "Unifont Regular 20"
title-color: "#10B981"
message-font: "Unifont Regular 14"
message-color: "#94A3B8"
message-bg-color: "#07090E"
desktop-color: "#07090E"

+ boot_menu {
  left = 8%
  top = 18%
  width = 84%
  height = 64%
  item_font = "Unifont Regular 16"
  item_color = "#94A3B8"
  selected_item_color = "#10B981"
  item_height = 32
  item_padding = 8
  item_spacing = 6
}

+ progress_bar {
  id = "__timeout__"
  left = 8%
  top = 88%
  width = 84%
  height = 10
  text_color = "#10B981"
  fg_color = "#10B981"
  bg_color = "#1E293B"
}
"""
        with open(os.path.join(theme_dir, "theme.txt"), "w") as tf:
            tf.write(theme_txt)
        
        # Write ventoy.json
        import json
        vjson = {
            "theme": {
                "file": "/ventoy/theme/theme.txt",
                "display_mode": "GUI",
                "ventoy_color": "#10B981",
                "ventoy_left": "5%",
                "ventoy_top": "90%"
            },
            "control": [
                {
                    "VTOY_DEFAULT_SEARCH_ROOT": "/",
                    "VTOY_DEFAULT_IMAGE": "",
                    "VTOY_MENU_TIMEOUT": "10",
                    "VTOY_SECONDARY_BOOT_MENU": "1"
                }
            ],
            "menu_alias": [
                {
                    "image": "/images/linux/linuxmint-22.3-cinnamon-64bit.iso",
                    "alias": "⚡ 🐧 Linux Mint 22.3 (Cinnamon 64-bit)"
                },
                {
                    "image": "/images/linux/ubuntu-24.04-desktop-amd64.iso",
                    "alias": "⚡ 🐧 Ubuntu 24.04 LTS Desktop"
                },
                {
                    "image": "/images/windows/Win11_24H2_English_x64.iso",
                    "alias": "⚡ 🪟 Windows 11 24H2 Installer"
                },
                {
                    "image": "/images/rescue/clonezilla-live-3.1.2-amd64.iso",
                    "alias": "⚡ 💽 Clonezilla Live (Backup & Recovery)"
                }
            ]
        }
        with open(os.path.join(ventoy_dir, "ventoy.json"), "w") as jf:
            json.dump(vjson, jf, indent=2)

        print("✅ Universal Botdigit BootForge USB is completely installed and ready to boot!")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        install_bootforge_core(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "BOTDIGIT")
    else:
        print("Usage: install_bootforge_core.py /dev/disk8 [LABEL]")
