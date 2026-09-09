#!/bin/bash
set -e

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "================================================================"
echo " ⚡ BOTDIGIT BOOTFORGE — LEGACY BIOS MBR BOOTSTRAP WRITER"
echo " (Enables boot on ThinkPad R400/T420 & older Legacy BIOS PCs)"
echo "================================================================"
echo ""

# Find the target USB drive
TARGET_DISK=$(diskutil list | grep -B 2 "BOTDIGIT" | grep "/dev/disk" | awk '{print $1}' | head -n 1)

if [ -z "$TARGET_DISK" ]; then
    TARGET_DISK="/dev/disk6"
fi

CLEAN_ID=$(echo "$TARGET_DISK" | sed 's|/dev/||' | sed 's|rdisk|disk|')
RAW_DEV="/dev/r${CLEAN_ID}"

echo "Target USB Drive detected: $TARGET_DISK ($RAW_DEV)"
echo "This will install GRUB stage 1 (boot.img) to Sector 0 and stage 2 (core.img) to sectors 1-2047."
echo ""
echo "Please enter your macOS password when prompted below:"
echo ""

# Assets paths
ASSETS="$DIR/src/managers/assets/ventoy_core/boot"
BOOT_IMG="$ASSETS/boot.img"
CORE_XZ="$ASSETS/core.img.xz"

if [ ! -f "$BOOT_IMG" ] || [ ! -f "$CORE_XZ" ]; then
    echo "❌ Error: Could not find boot.img or core.img.xz in $ASSETS"
    exit 1
fi

# 1. Unmount target disk
echo "1. Unmounting $CLEAN_ID..."
diskutil unmountDisk force "$CLEAN_ID"

# 2. Write Sector 0 MBR Bootstrap (first 446 bytes, preserving partition table)
echo "2. Writing Sector 0 MBR bootstrap code (446 bytes)..."
sudo dd if="$BOOT_IMG" of="$RAW_DEV" bs=1 count=446 conv=notrunc status=none

# 3. Write Sectors 1..2047 post-MBR gap (GRUB core.img)
echo "3. Writing Sectors 1..2047 Post-MBR GRUB2 Core..."
xzcat "$CORE_XZ" | sudo dd of="$RAW_DEV" bs=512 seek=1 count=2047 conv=notrunc status=none

# 4. Sync and remount
echo "4. Remounting volumes..."
diskutil mountDisk "$CLEAN_ID" || true

echo ""
echo "================================================================"
echo " ✅ SUCCESS: Legacy BIOS bootstrap is now active on $TARGET_DISK!"
echo " Your ThinkPad R400 / Legacy BIOS PC will now boot without"
echo " any blinking cursor!"
echo "================================================================"
echo ""
echo "Press any key to exit..."
read -n 1 -s
