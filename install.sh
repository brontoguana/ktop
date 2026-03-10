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
    x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
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
echo "Run 'ktop' to start."
