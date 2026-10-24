#!/bin/bash
# share/init.sh: compatibility shim for startup files still holding the old
# `source .../share/init.sh` block. Run ./setup.sh to switch to the new integration
# (`eval "$(aliases init <shell>)"`); this file will be removed in a future version.

_aliases_manager_shim() {
    local shell_name=bash
    [ -n "${ZSH_VERSION-}" ] && shell_name=zsh

    # shellcheck disable=SC2296 # zsh fallback for the script path
    local script_path="${BASH_SOURCE[0]:-${(%):-%x}}"
    local repo_root bin
    repo_root="$(cd "$(dirname "$script_path")/.." && pwd)"
    bin="$(command -v aliases 2>/dev/null)"
    [ -x "$bin" ] || bin="$repo_root/target/release/aliases"

    if [ ! -x "$bin" ]; then
        echo "aliases-manager: the aliases program is not installed. Run $repo_root/setup.sh" >&2
        return 1
    fi
    eval "$("$bin" init "$shell_name")"
}

_aliases_manager_shim
unset -f _aliases_manager_shim
