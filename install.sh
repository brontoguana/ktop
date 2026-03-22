#!/usr/bin/env bash
# ktop installer — downloads the latest release binary from GitHub
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/brontoguana/ktop/master/install.sh | bash
set -euo pipefail

REPO="brontoguana/ktop"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="ktop"

echo "ktop installer"
echo ""

# Must be Linux
if [ "$(uname -s)" != "Linux" ]; then
    echo "Error: ktop only supports Linux."
    exit 1
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
    aarch64) TARGET="aarch64-unknown-linux-musl" ;;
    *)
        echo "Error: unsupported architecture: $ARCH"
        echo "ktop supports x86_64 and aarch64."
        exit 1
        ;;
esac

# Get latest release tag from GitHub API
echo "Fetching latest release..."
LATEST=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: could not determine latest release."
    exit 1
fi

ASSET_NAME="ktop-${TARGET}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET_NAME}"

# Check existing version
EXISTING_VERSION=""
if command -v "$BINARY_NAME" &>/dev/null; then
    EXISTING_VERSION=$("$BINARY_NAME" -v 2>/dev/null || true)
    echo "Installed: ${EXISTING_VERSION:-unknown version}"
fi
echo "Latest:    ${LATEST}"

# Download to temp file
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

echo "Downloading ${ASSET_NAME}..."
if ! curl -sSfL -o "$TMPFILE" "$DOWNLOAD_URL"; then
    echo "Error: download failed."
    echo "URL: $DOWNLOAD_URL"
    echo ""
    echo "If this is a new release, binaries may still be building."
    echo "Try again in a few minutes."
    exit 1
fi

chmod +x "$TMPFILE"

# Install — use sudo if we don't have write access
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMPFILE" "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "$TMPFILE" "${INSTALL_DIR}/${BINARY_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
fi

echo ""
echo "ktop ${LATEST} installed to ${INSTALL_DIR}/${BINARY_NAME}"

# Check for stale installs that might shadow the new binary
WHICH_KTOP=$(command -v ktop 2>/dev/null || true)
if [ -n "$WHICH_KTOP" ] && [ "$WHICH_KTOP" != "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    # Check if it's a script (old Python wrapper) rather than an ELF binary
    if file "$WHICH_KTOP" 2>/dev/null | grep -q "text"; then
        echo "WARNING: Found old ktop at $WHICH_KTOP that shadows this install."
        echo "It appears to be a script (likely the old Python version)."
        read -r -p "Remove it? [Y/n] " REPLY </dev/tty 2>/dev/null || REPLY="n"
        REPLY=${REPLY:-Y}
        if [[ "$REPLY" =~ ^[Yy]$ ]]; then
            rm -f "$WHICH_KTOP" 2>/dev/null || sudo rm -f "$WHICH_KTOP"
            echo "Removed $WHICH_KTOP"
            echo "Run 'hash -r' to refresh your shell's path cache, then 'ktop' to start."
            exit 0
        else
            echo "To fix manually: rm $WHICH_KTOP"
        fi
    else
        echo "Note: another ktop exists at $WHICH_KTOP"
        echo "The new install is at ${INSTALL_DIR}/${BINARY_NAME}"
        echo "Make sure ${INSTALL_DIR} comes first in your PATH, or remove the old one."
    fi
fi

echo "Run 'ktop' to start."
