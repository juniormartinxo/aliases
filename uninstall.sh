#!/bin/bash
# uninstall.sh

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$REPO_ROOT/lib/utils.sh"

print_header() {
    echo -e "${RED}🗑️  Aliases Manager Uninstall${NC}"
    echo "=============================================="
}

main() {
    print_header

    # 1. Remove Shell Integration
    local shell_config
    shell_config="$(detect_shell_config)"
    
    if [[ -n "$shell_config" ]]; then
        print_info "Removing configuration from $shell_config..."
        
        local start_marker="# >>> aliases-manager >>>"
        local end_marker="# <<< aliases-manager <<<"
        
        if grep -qF "$start_marker" "$shell_config"; then
            local temp_rc
            temp_rc=$(mktemp)
            local in_block=0
            
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
            print_success "Removed block from $shell_config"
        else
            print_warning "No configuration block found in $shell_config"
        fi
    fi

    # 2. Remove Config Directory (Optional)
    echo -e "${YELLOW}Do you want to remove the configuration directory ($ALIASES_ROOT)? [y/N]${NC}"
    read -r response
    if [[ "$response" =~ ^[yY]$ ]]; then
        rm -rf "$ALIASES_ROOT"
        print_success "Removed $ALIASES_ROOT"
    else
        print_info "Configuration directory kept at $ALIASES_ROOT"
    fi

    print_success "Uninstallation complete."
}

main
