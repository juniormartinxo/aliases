# Aliases Manager

A robust, shell-agnostic CLI tool to manage your shell aliases, following XDG standards.

## Features

- **Standardized Storage**: Stores aliases in `~/.config/aliases/aliases.d/`.
- **Modular**: Organize aliases into separate files (e.g., `10-git.alias`, `20-docker.alias`).
- **Idempotent Setup**: Safe to run `setup.sh` multiple times.
- **Diagnostics**: Built-in `alias-doctor` command.
- **Searchable**: Find aliases easily with `alias-find`.

## Installation

```bash
git clone https://github.com/yourusername/aliases.git
cd aliases
./setup.sh
```

Restart your terminal or run `source ~/.zshrc` (or `~/.bashrc`) to apply.

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `alias-create <name> <cmd> [file]` | Create a new alias. Defaults to `local.alias`. |
| `alias-remove <name>` | Remove an alias. |
| `alias-list` | List all managed aliases. |
| `alias-find <query>` | Search for aliases. |
| `alias-edit <name>` | Edit the file containing the alias (requires `$EDITOR`). |
| `alias-reload` | Reload aliases immediately. |
| `alias-enable <name>` | Enable a disabled alias file. |
| `alias-disable <name>` | Disable an alias file. |
| `alias-doctor` | Verify installation health. |

### Examples

```bash
# Create a new alias in the 'git' category
alias-create gs 'git status' git

# List all aliases
alias-list

# Search for docker related aliases
alias-find docker
```

## Structure

```
~/.config/aliases/
  aliases.d/
    00-core.alias    # Core aliases
    local.alias      # Local/Custom aliases
    ...
```

Files are loaded in alphabetical order. `local.alias` is always loaded last.

## Development

- **Tests**: Run `bats tests/`
- **Linting**: Run `shellcheck bin/*`

## Uninstallation

Run `./uninstall.sh` to remove the shell integration. You will be prompted if you want to remove the configuration directory.
