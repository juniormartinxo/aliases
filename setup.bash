#!/usr/bin/env bash
# setup.bash

set -euo pipefail

# Get the absolute path of the repository
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
source "$REPO_ROOT/lib/utils.sh"

print_header() {
    echo -e "${BLUE}🚀 Aliases Manager Setup${NC}"
    echo "=============================================="
}

main() {
    print_header

    # 1. Setup XDG Config Directory
    print_info "Setting up configuration directory..."
    ensure_dir "$ALIASES_DIR"

    # Copy default aliases if directory is empty
    if [[ -z "$(ls -A "$ALIASES_DIR" 2>/dev/null)" ]]; then
        print_info "Populating with default templates..."
        # Create a dummy core alias if none exists, just to show it works
        echo "# Core aliases" > "$ALIASES_DIR/00-core.alias"
        echo "alias ll='ls -la'" >> "$ALIASES_DIR/00-core.alias"
        print_success "Created 00-core.alias"
    else
        print_info "Configuration directory already populated."
    fi

    # 2. Setup Shell Integration
    local shell_config
    shell_config="$(detect_shell_config)"

    if [[ -z "$shell_config" ]]; then
        print_error "Could not detect shell config file. Manual setup required."
        exit 1
    fi

    print_info "Configuring $shell_config..."

    # Prepare the block to insert
    # We use the absolute path to the init script
    local init_script="$REPO_ROOT/share/init.sh"

    local start_marker="# >>> aliases-manager >>>"
    local end_marker="# <<< aliases-manager <<<"
    local block_content="source \"$init_script\""

    # Check if block exists
    if grep -qF "$start_marker" "$shell_config"; then
        print_warning "Aliases Manager block already exists in $shell_config. Updating path if necessary..."

        # We need to replace the block.
        # Strategy: Create a temporary file, strip the old block, append new block.
        # This is safer than trying to edit in place with complex sed.

        local temp_rc
        temp_rc="$(mktemp)"
        local in_block=0
        trap 'rm -f "$temp_rc"' RETURN

        while IFS= read -r line; do
            if [[ "$line" == "$start_marker" ]]; then
                in_block=1
                continue
            fi
            if [[ "$line" == "$end_marker" ]]; then
                in_block=0
                continue
            fi
            if [[ "$in_block" -eq 0 ]]; then
                echo "$line" >> "$temp_rc"
            fi
        done < "$shell_config"

        mv "$temp_rc" "$shell_config"
        print_success "Removed old block."
    fi

    # Ensure the rc file ends with a newline so the block does not stick to the last line
    if [[ -s "$shell_config" && "$(tail -c1 "$shell_config")" != $'\n' ]]; then
        echo "" >> "$shell_config"
    fi

    {
        echo "$start_marker"
        echo "$block_content"
        echo "$end_marker"
    } >> "$shell_config"

    print_success "Added configuration to $shell_config"

    # 3. Make binaries executable
    print_info "Setting permissions..."
    chmod +x "$REPO_ROOT/bin"/* 2>/dev/null || true

    echo ""
    print_success "Setup complete! 🎉"
    print_info "Please run: source $shell_config"
    print_info "Your aliases are stored in: $ALIASES_DIR"
}

main
