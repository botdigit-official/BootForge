#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "=========================================================="
echo "          ⚡ BootForge — One Drive. Every OS.            "
echo "=========================================================="
echo "Launching BootForge Desktop UI in your browser..."

if [ -f "./dist/BootForge.app/Contents/MacOS/BootForge" ]; then
    exec "./dist/BootForge.app/Contents/MacOS/BootForge" ui --port 4242
elif [ -f "./target/release/bootforge" ]; then
    exec "./target/release/bootforge" ui --port 4242
else
    exec cargo run --release -- ui --port 4242
fi
