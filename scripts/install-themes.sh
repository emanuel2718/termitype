#!/usr/bin/env bash
set -euo pipefail

THEME_REPO="https://github.com/mbadolato/iTerm2-Color-Schemes"
COMMIT_SHA="0e23daf59234fc892cba949562d7bf69204594bb"
ARCHIVE_URL="$THEME_REPO/archive/$COMMIT_SHA.tar.gz"
ARCHIVE_NAME="iTerm2-Color-Schemes-$COMMIT_SHA"
TARGET_DIR="assets/themes"

echo "Fetching color schemes from repository..."
echo "Repository: $THEME_REPO"
echo "Commit SHA: $COMMIT_SHA"
curl -sL "$ARCHIVE_URL" | tar -xzf -

echo "Installing themes to '$TARGET_DIR'..."
mkdir -p "$TARGET_DIR"
mv "$ARCHIVE_NAME/ghostty/"* "$TARGET_DIR/"

echo "Removing temporary files..."
rm -rf "$ARCHIVE_NAME"

echo "[OK] Installation complete. Themes are now available in '$TARGET_DIR'."
