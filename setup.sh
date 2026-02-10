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

# 3. Ensure ~/.local/bin is in PATH
if [ "$NEED_SUDO" = false ]; then
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            export PATH="$INSTALL_DIR:$PATH"
            # Offer to persist to shell profile
            printf "    Add $INSTALL_DIR to PATH in shell config? [Y/n] "
            read -r reply
            if [ -z "$reply" ] || [ "$reply" = "y" ] || [ "$reply" = "Y" ]; then
                for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
                    if [ -f "$rc" ] && ! grep -q "$INSTALL_DIR" "$rc" 2>/dev/null; then
                        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$rc"
                        echo "    Updated $(basename "$rc")"
                    fi
                done
            fi
            ;;
    esac
fi

echo "==> Done! Launching ktop..."
echo "    Next time just run: ktop (open a new terminal if not found)"
exec "$INSTALL_PATH" "$@"
