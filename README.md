# Aliases Manager

A robust, shell-agnostic CLI tool to manage your shell aliases, following XDG standards.

## Features

- **Standardized Storage**: Stores aliases in `~/.config/aliases/aliases.d/`.
- **Modular**: Organize aliases into separate files (e.g., `10-git.alias`, `20-docker.alias`).
- **Idempotent Setup**: Safe to run `setup.sh` multiple times.
- **PowerShell Support**: Works on Windows PowerShell with the same alias file format.
- **Diagnostics**: Built-in `alias-doctor` command.
- **Searchable**: Find aliases easily with `alias-find`.

## Installation

### Bash / Zsh

```bash
git clone https://github.com/yourusername/aliases.git
cd aliases
./setup.sh
```

Restart your terminal or run `source ~/.zshrc` (or `~/.bashrc`) to apply.

### PowerShell (Windows)

```powershell
git clone https://github.com/yourusername/aliases.git
cd aliases
.\setup.ps1
. $PROFILE
```

This loads the alias manager into your PowerShell profile and makes commands such as `alias-create`, `alias-list`, `alias-edit`, and `alias-reload` available in future sessions.
Managed aliases are exposed as PowerShell functions. Use `Get-Command <name>` to inspect them. `Get-Alias` only shows native PowerShell aliases.

If you run `.\setup.ps1` again, the integration block in `$PROFILE` is replaced instead of duplicated.

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

```powershell
# Create a PowerShell-friendly alias and reload it
alias-create gst "git status -sb" git
alias-reload

# Use the alias with extra arguments
gst

# Inspect a managed alias in PowerShell
Get-Command gst

# Use explicit PowerShell syntax for executables with spaces in the path
alias-create pfle "& 'C:\Program Files\Notepad++\notepad++.exe' '$PROFILE'" system
pfle

# Run a PowerShell script file
alias-create syncdocs "& 'C:\tools\sync-docs.ps1'" scripts
syncdocs

# Run a PowerShell script file with fixed arguments
alias-create deploy-api "& 'D:\work\deploy-api.ps1' -Environment dev -Verbose" scripts
deploy-api
```

### PowerShell Notes

- Managed aliases are PowerShell functions, not `AliasInfo` entries.
- Use `Get-Command <name>` to verify whether an alias is loaded.
- Use `alias-reload` after creating, editing, enabling, or disabling aliases.
- Use `. $PROFILE` after changing the installed integration or opening a new session.
- For commands whose executable path contains spaces, use the PowerShell call operator: `& 'C:\Path With Spaces\tool.exe'`.
- For `.ps1` files, prefer explicit invocation: `& 'C:\path\script.ps1'`.
- Alias definitions still use the shared `.alias` file format:

```text
alias gst='git status -sb'
alias pfle='& ''C:\Program Files\Notepad++\notepad++.exe'' ''C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1'''
```

### Troubleshooting (PowerShell)

- `Get-Alias mycmd` does not find the alias:
  Use `Get-Command mycmd`. This project registers managed aliases as functions.
- `mycmd` is not recognized:
  Run `. $PROFILE` or `alias-reload` in the current PowerShell session.
- An executable path with spaces fails:
  Define the alias with `&` and quote the executable path.
- A `.ps1` script alias fails to start:
  Define it explicitly as `& 'C:\path\script.ps1'`. If your environment restricts script execution, adjust your PowerShell execution policy according to your system requirements.
- You changed the installed integration and still see old behavior:
  Run `.\setup.ps1` again, then `. $PROFILE`.

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
