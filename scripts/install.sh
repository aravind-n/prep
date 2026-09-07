#!/bin/bash

set -euo pipefail

REPO="aravind-n/prep"
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

echo "Installing the latest $BIN_NAME release for $PLATFORM..."

# GitHub redirects this stable URL to the matching asset in the latest release.
URL="https://github.com/$REPO/releases/latest/download/${BIN_NAME}-${PLATFORM}"

TMP="$(mktemp -d)"
curl -fsSL "$URL" -o "$TMP/$BIN_NAME"
chmod +x "$TMP/$BIN_NAME"

INSTALL_DIR="/usr/local/bin"

# Not present on a stock macOS install, so create it before the move.
if [ ! -d "$INSTALL_DIR" ]; then
  echo "Creating $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR" 2>/dev/null || sudo install -d -m 755 "$INSTALL_DIR"
fi

# Verify install dir access
if [ ! -w "$INSTALL_DIR" ]; then
  echo "Unable to write to $INSTALL_DIR. Attempting with sudo"
  sudo mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "$BIN_NAME installed to $INSTALL_DIR"
