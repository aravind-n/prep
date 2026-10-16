#!/bin/sh

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

INSTALL_URL="https://aravind-n.github.io/prep/install.sh"

confirm_and_run() {
    PREP_CMD="$1"
    "$PREP_CMD" plan
    printf "\nDo you want to proceed with execution? [y/N] "
    read -r response || response=""
    case "$response" in
        [yY][eE][sS]|[yY])
            "$PREP_CMD" run
            ;;
        *)
            echo "Execution aborted."
            exit 0
            ;;
    esac
}

if [ -f "./prep" ]; then
    chmod +x "./prep"
    PREP_CMD="./prep"
elif command -v prep >/dev/null 2>&1; then
    PREP_CMD="prep"
else
    echo "prep is not installed on this system. Installing to current directory..."
    curl --proto '=https' --tlsv1.2 -fsSL "$INSTALL_URL" | bash -s -- --location .
    PREP_CMD="./prep"
fi

confirm_and_run "$PREP_CMD"
