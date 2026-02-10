#!/usr/bin/env bash
# ktop installer — installs ktop as a command
# Usage:
#   ./setup.sh              Install to ~/.local/bin  (no sudo needed)
#   ./setup.sh --system     Install to /usr/local/bin (needs sudo)
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
VENV_DIR="$REPO_DIR/.venv"

# Determine install location
if [ "${1:-}" = "--system" ]; then
    INSTALL_DIR="/usr/local/bin"
    NEED_SUDO=true
else
    INSTALL_DIR="$HOME/.local/bin"
    NEED_SUDO=false
    mkdir -p "$INSTALL_DIR"
fi

INSTALL_PATH="$INSTALL_DIR/ktop"

echo "==> Installing ktop from $REPO_DIR"
echo "    Target: $INSTALL_PATH"

# 1. Create/update virtual environment
if [ ! -d "$VENV_DIR" ]; then
    echo "    Creating virtual environment..."
    if ! python3 -m venv "$VENV_DIR" 2>/dev/null; then
        echo "    python3-venv not found, installing it..."
        if command -v apt-get &>/dev/null; then
            sudo apt-get install -y python3-venv
        elif command -v dnf &>/dev/null; then
            sudo dnf install -y python3-libs
        elif command -v pacman &>/dev/null; then
            sudo pacman -S --noconfirm python
        else
            echo "ERROR: python3-venv is required. Install it with your package manager."
            exit 1
        fi
        python3 -m venv "$VENV_DIR"
    fi
fi

echo "    Installing dependencies..."
"$VENV_DIR/bin/pip" install --quiet --upgrade pip
"$VENV_DIR/bin/pip" install --quiet -r "$REPO_DIR/requirements.txt"

# 2. Write wrapper script
WRAPPER=$(mktemp)
cat > "$WRAPPER" <<EOF
#!/usr/bin/env bash
exec "$VENV_DIR/bin/python" "$REPO_DIR/ktop.py" "\$@"
EOF

if [ "$NEED_SUDO" = true ]; then
    sudo install -m 755 "$WRAPPER" "$INSTALL_PATH"
else
    install -m 755 "$WRAPPER" "$INSTALL_PATH"
fi
rm -f "$WRAPPER"

echo "==> Done! Run 'ktop' from anywhere."
echo "    Options: ktop -r 2  (2-second refresh)"

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "    NOTE: Add $INSTALL_DIR to your PATH if it's not already there." ;;
esac
