#!/bin/bash

# lib/utils.sh

# Colors
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export NC='\033[0m' # No Color

# Symbols
export SYMBOL_SUCCESS="✅"
export SYMBOL_INFO="ℹ️ "
export SYMBOL_WARNING="⚠️ "
export SYMBOL_ERROR="❌"

# Paths
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
export ALIASES_ROOT="${ALIASES_ROOT:-$XDG_CONFIG_HOME/aliases}"
export ALIASES_DIR="$ALIASES_ROOT/aliases.d"
export ALIASES_BACKUP_DIR="$ALIASES_ROOT/backups"

print_success() {
    echo -e "${GREEN}${SYMBOL_SUCCESS} $1${NC}"
}

print_info() {
    echo -e "${BLUE}${SYMBOL_INFO} $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}${SYMBOL_WARNING} $1${NC}" >&2
}

print_error() {
    echo -e "${RED}${SYMBOL_ERROR} $1${NC}" >&2
}

# Ensure directory exists
ensure_dir() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir" || {
            print_error "Failed to create directory: $dir"
            return 1
        }
    fi
}

# Detect Shell
detect_shell_config() {
    case "$SHELL" in
        */zsh)
            echo "$HOME/.zshrc"
            ;;
        */bash)
            echo "$HOME/.bashrc"
            ;;
        *)
            # Fallback based on file existence
            if [[ -f "$HOME/.zshrc" ]]; then
                echo "$HOME/.zshrc"
            elif [[ -f "$HOME/.bashrc" ]]; then
                echo "$HOME/.bashrc"
            else
                print_error "Could not detect valid shell config (zshrc/bashrc)"
                return 1
            fi
            ;;
    esac
}
