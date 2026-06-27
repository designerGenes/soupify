#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building soupify..."
cargo build --release

echo "Installing binary to ~/.local/bin..."
mkdir -p ~/.local/bin
cp target/release/soupify ~/.local/bin/

# Add ~/.local/bin to PATH in shell rc if not present
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    for rcfile in "$HOME/.zshrc" "$HOME/.bashrc"; do
        if [[ -f "$rcfile" ]] && ! grep -q '.local/bin' "$rcfile" 2>/dev/null; then
            echo '' >> "$rcfile"
            echo '# Added by soupify installer' >> "$rcfile"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rcfile"
            echo "Added ~/.local/bin to PATH in $rcfile"
        fi
    done
fi

echo "Creating config directory and default config..."
cargo run --release -- --version >/dev/null 2>&1 || true

CONFIG_DIR="$HOME/.config/soupify"
mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_DIR/config.yaml" ]]; then
    cargo run --release -- --help >/dev/null 2>&1 || true
fi

echo ""
echo "soupify installed successfully."
echo "  Binary: ~/.local/bin/soupify"
echo "  Config: $CONFIG_DIR/config.yaml"
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "NOTE: ~/.local/bin is not in your PATH."
    echo "  Restart your shell, or run: export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
