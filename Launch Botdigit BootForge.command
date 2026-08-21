#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "=========================================================="
echo "      ⚡ Botdigit BootForge — Universal Boot Media Studio   "
echo "=========================================================="
echo "Launching Botdigit BootForge Native macOS App..."

if [ -d "./dist/Botdigit BootForge.app" ]; then
    open "./dist/Botdigit BootForge.app"
elif [ -f "./target/release/bootforge" ]; then
    "./target/release/bootforge" ui --port 4242
else
    cargo run --release -- ui --port 4242
fi
