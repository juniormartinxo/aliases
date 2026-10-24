#!/usr/bin/env bash
# setup.sh: builds the aliases program, installs it and adds the shell integration.
# Usage: ./setup.sh [--shell bash|zsh] [--rc FILE]
# The install folder defaults to ~/.local/bin; set ALIASES_BIN_DIR to change it.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
BIN_DIR="${ALIASES_BIN_DIR:-$HOME/.local/bin}"

echo "🚀 Aliases Manager Setup"
echo "=============================================="

if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ cargo not found. Install Rust from https://rustup.rs and run setup.sh again." >&2
    exit 1
fi

echo "ℹ️  Building the aliases program..."
cargo build --release --locked --manifest-path "$REPO_ROOT/Cargo.toml"

mkdir -p "$BIN_DIR"
install -m 755 "$REPO_ROOT/target/release/aliases" "$BIN_DIR/aliases"
echo "✅ Installed $BIN_DIR/aliases"

"$BIN_DIR/aliases" setup "$@"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "⚠️  $BIN_DIR is not in PATH. The alias-* commands work anyway; add it to PATH to run 'aliases' directly." ;;
esac
