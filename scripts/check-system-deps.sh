#!/usr/bin/env bash
set -euo pipefail

# System dependencies for Debian Bookworm
packages=("libatspi2.0-dev" "libdbus-1-dev" "libglib2.0-dev" "pkg-config")

# Check via dpkg if available (Debian-based)
if command -v dpkg > /dev/null 2>&1; then
    echo "Checking packages via dpkg..."
    for pkg in "${packages[@]}"; do
        if ! dpkg -l "$pkg" > /dev/null 2>&1; then
            echo "Error: Package $pkg is not installed."
            exit 1
        fi
    done
else
    echo "Warning: dpkg not found. Skipping package check."
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

echo "All system dependencies are met."
exit 0
