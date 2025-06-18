#!/bin/bash

set -e

TEMP_DIR=$(mktemp -d)
THEMES_DIR="assets/themes"

echo "Downloading latest iTerm2-Color-Schemes..."
curl -L "https://github.com/mbadolato/iTerm2-Color-Schemes/archive/refs/heads/master.tar.gz" | tar -xz -C "$TEMP_DIR"

echo "Copying themes..."
if [ -d "$TEMP_DIR/iTerm2-Color-Schemes-master/ghostty" ]; then
    cp -n "$TEMP_DIR/iTerm2-Color-Schemes-master/ghostty"/* "$THEMES_DIR/" || true
else
    echo "Error: ghostty themes directory not found."
    exit 1
fi

rm -rf "$TEMP_DIR"

echo "Done!" 
