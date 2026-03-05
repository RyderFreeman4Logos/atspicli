#!/usr/bin/env bash
set -euo pipefail

if ! command -v pkg-config > /dev/null 2>&1; then
    echo "Error: pkg-config is required."
    exit 1
fi

# Check via pkg-config
echo "Checking libraries via pkg-config..."
libraries=("atspi-2" "dbus-1" "glib-2.0")
for lib in "${libraries[@]}"; do
    if ! pkg-config --exists "$lib"; then
        echo "Error: Library $lib not found via pkg-config."
        exit 1
    fi
done

echo "All system dependencies are met for CLI development."
echo "If this fails on your distro, install AT-SPI, DBus, and GLib development packages."
exit 0
