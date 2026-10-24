#!/usr/bin/env bash
# uninstall.sh: removes the shell integration and the installed program.
# Usage: ./uninstall.sh [--shell bash|zsh] [--rc FILE] [--remove-config]

set -euo pipefail

BIN_DIR="${ALIASES_BIN_DIR:-$HOME/.local/bin}"
BIN="$BIN_DIR/aliases"

echo "🗑️  Aliases Manager Uninstall"
echo "=============================================="

if [[ ! -x "$BIN" ]]; then
    echo "❌ $BIN not found. Set ALIASES_BIN_DIR to the folder setup.sh installed to." >&2
    exit 1
fi

"$BIN" uninstall "$@"
rm -f "$BIN"
echo "✅ Removed $BIN"
