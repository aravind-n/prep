#!/bin/bash

set -euo pipefail

REPO="aravind-n/prep"
BIN_NAME="prep"

INSTALL_DIR="$HOME/.local/bin"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --location)
      if [[ $# -lt 2 || -z "${2:-}" ]]; then
        echo "Error: --location requires a non-empty path argument." >&2
        exit 1
      fi
      INSTALL_DIR="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--location <path>]"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--location <path>]" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$INSTALL_DIR" ]]; then
  echo "Error: Install location cannot be empty." >&2
  exit 1
fi

# Expand leading tilde if present
INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"

# Determine OS / Arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux*)
    if [[ "$ARCH" == "x86_64" ]]; then
      PLATFORM="linux-x86_64"
    else
      echo "Unsupported architecture for $OS: $ARCH"; exit 1
    fi
    ;;
  darwin*)
    if [[ "$ARCH" == "arm64" ]]; then
      PLATFORM="macos-arm64"
    else
      PLATFORM="macos-x86_64"
    fi
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

echo "Installing the latest $BIN_NAME release for $PLATFORM..."

# GitHub redirects this stable URL to the matching asset in the latest release.
URL="https://github.com/$REPO/releases/latest/download/${BIN_NAME}-${PLATFORM}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/$BIN_NAME"
chmod +x "$TMP/$BIN_NAME"

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
  echo "Creating $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR" 2>/dev/null || sudo mkdir -p "$INSTALL_DIR"
fi

# Verify install dir access
if [ ! -w "$INSTALL_DIR" ]; then
  echo "Unable to write to $INSTALL_DIR. Attempting with sudo"
  sudo mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "$BIN_NAME installed to $INSTALL_DIR"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "You can add it to your shell configuration (e.g., ~/.bashrc or ~/.zshrc):"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    ;;
esac
