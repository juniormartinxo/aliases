#!/bin/bash
# share/init.sh
# Check if we are running in an interactive shell
case "$-" in
    *i*) ;;
    *) return ;;
esac

# Function to load aliases from a directory
_aliases_manager_load() {
    local config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/aliases"
    local aliases_d="$config_dir/aliases.d"

    # Export the CLI binary path if not already in PATH
    # This assumes init.sh is located at <repo>/share/init.sh
    # and binaries are at <repo>/bin
    local current_script_path="${BASH_SOURCE[0]:-${(%):-%x}}"
    local repo_root
    repo_root="$(cd "$(dirname "$current_script_path")/.." && pwd)"
    
    # Add bin to PATH if it's not there
    case ":$PATH:" in
        *":$repo_root/bin:"*) ;;
        *) export PATH="$repo_root/bin:$PATH" ;;
    esac

    # Ensure config directory exists
    if [[ ! -d "$aliases_d" ]]; then
       # Silently return or maybe warn? 
       # For a CLI tool, we might expect the setup to have run.
       return
    fi

    # Load aliases in lexical order
    # Using a glob that works in both bash and zsh
    if [ -n "$ZSH_VERSION" ]; then
        setopt local_options nullglob
        for file in "$aliases_d"/*.alias; do
            [[ -f "$file" ]] && source "$file"
        done
    else
        shopt -s nullglob
        for file in "$aliases_d"/*.alias; do
            [[ -f "$file" ]] && source "$file"
        done
        shopt -u nullglob
    fi

    # Load local.alias last and specifically
    if [[ -f "$aliases_d/local.alias" ]]; then
        source "$aliases_d/local.alias"
    fi
}

alias-reload() {
    # Resolve repo root based on this script location (works in bash and zsh)
    local current_script_path="${BASH_SOURCE[0]:-${(%):-%x}}"
    local repo_root
    repo_root="$(cd "$(dirname "$current_script_path")/.." && pwd)"

    if [[ -f "$repo_root/share/init.sh" ]]; then
        source "$repo_root/share/init.sh"
    else
        echo "aliases-manager: init script not found at $repo_root/share/init.sh" >&2
        return 1
    fi
    echo "Aliases reloaded!"
}

alias-edit() {
    local name="$1"
    local config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/aliases"
    local aliases_d="$config_dir/aliases.d"
    
    if [[ -z "$name" ]]; then
        echo "Usage: alias-edit <name>"
        return 1
    fi
    
    # Try to find the file
    local target_file=""
    if [[ -f "$aliases_d/$name.alias" ]]; then
        target_file="$aliases_d/$name.alias"
    elif [[ -f "$aliases_d/$name" ]]; then
        target_file="$aliases_d/$name"
    else
        # Search for defined alias to find which file it belongs to?
        # That's hard without parsing.
        # Fallback to opening the main dir or finding a file with that name
        # Let's assume the user passes the alias name or file basename.
        # For now simple file match.
        
        # Check if we can find a file that contains this alias definition?
        # Too heavy for a shell function? Maybe.
        # Let's just try to open "$aliases_d/$name.alias" or create it if user wants?
        # Requirement: "abre no $EDITOR".
        
        # If not found, look for partial match in filenames?
        local matches=("$aliases_d"/*"$name"*.alias)
        if [[ ${#matches[@]} -eq 1 ]]; then
            target_file="${matches[0]}"
        elif [[ ${#matches[@]} -gt 1 ]]; then
            echo "Multiple files match '$name':"
            printf '%s\n' "${matches[@]##*/}"
            return 1
        fi
    fi
    
    if [[ -n "$target_file" ]]; then
        ${EDITOR:-vi} "$target_file"
        alias-reload
    else
        echo "Alias file not found for '$name'."
        echo "Available files:"
        ls "$aliases_d"
    fi
}

_aliases_manager_load
unset -f _aliases_manager_load
