#!/bin/bash

set -euo pipefail

REPO="aravind/prep"
BIN_NAME="prep"

# Determine OS / Arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux*)   PLATFORM="linux-x86-64" ;;
  darwin*)
    if [[ "$ARCH" == "arm64" ]]; then
      PLATFORM="macos-arm64"
    else
      PLATFORM="macos-x86"
    fi
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Find latest tag (vX.Y.Z)
TAG=$(curl -fsSL "https://gitlab.com/api/v4/projects/$(echo "$REPO" | sed 's|/|%2F|')/releases" \
      | grep -m1 -oE '"tag_name":"v[0-9]+\.[0-9]+\.[0-9]+"' \
      | head -1 | cut -d':' -f2 | tr -d '"')

[ -n "$TAG" ] || { echo "Could not determine latest version"; exit 1; }

echo "Installing $BIN_NAME $TAG for $PLATFORM..."

URL="https://gitlab.com/$REPO/-/releases/$TAG/downloads/binaries/$TAG/${BIN_NAME}-${PLATFORM}"

TMP="$(mktemp -d)"
curl -fsSL "$URL" -o "$TMP/$BIN_NAME"
chmod +x "$TMP/$BIN_NAME"

# Verify install dir access
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  echo "Unable to write to $INSTALL_DIR. Try re-installing with sudo"
fi

mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo "$BIN_NAME installed to $INSTALL_DIR"
