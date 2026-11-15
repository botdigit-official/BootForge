#!/bin/bash
# Re-install the Legacy BIOS boot code on an existing BootForge drive.
# Fixes "blinking cursor" on old PCs (ThinkPad R400/T420, Core 2 era, any BIOS/CSM machine).
# Only sector 0 boot code and the 1 MB gap before partition 1 are rewritten; files are kept.
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "================================================================"
echo " BootForge — Fix Legacy BIOS boot (old PCs)"
echo "================================================================"
echo ""

HELPER=""
for c in "$DIR/dist/BootForge.app/Contents/MacOS/write_mbr_core" "$DIR/target/release/write_mbr_core" "$DIR/target/debug/write_mbr_core"; do
    if [ -x "$c" ]; then HELPER="$c"; break; fi
done
if [ -z "$HELPER" ]; then
    echo "Building the disk helper..."
    cargo build --release --bin write_mbr_core
    HELPER="$DIR/target/release/write_mbr_core"
fi

echo "External drives:"
diskutil list external physical
echo ""
read -r -p "Disk to fix (e.g. disk6): " DISK
DISK="${DISK#/dev/}"
if [[ ! "$DISK" =~ ^disk[0-9]+$ ]]; then
    echo "Not a whole-disk name: '$DISK'. Nothing was written."
    exit 1
fi

# The helper refuses internal drives, the macOS disk, and non-BootForge drives.
sudo "$HELPER" "/dev/$DISK"

echo ""
echo "Done. Boot the old PC from the plain USB entry (not 'UEFI:')."
read -n 1 -s -r -p "Press any key to close..."
