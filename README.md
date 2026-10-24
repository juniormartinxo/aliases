# Aliases Manager

A shell-agnostic tool to manage your aliases, following XDG standards. A single program (`aliases`, written in Rust) serves Bash, Zsh and PowerShell, on Linux, macOS, Windows and WSL.

## Features

- **Standardized Storage**: Stores aliases in `~/.config/aliases/aliases.d/`.
- **Modular**: Organize aliases into separate files (e.g., `10-git.alias`, `20-docker.alias`).
- **One implementation for every shell**: the same program reads the files and generates the code for Bash, Zsh and PowerShell.
- **Idempotent Setup**: Safe to run the setup multiple times.
- **Diagnostics**: Built-in `alias-doctor` command.
- **Searchable**: Find aliases easily with `alias-find`.
- **Completions**: `aliases completions <shell>` prints a completion script.

## Installation

Building requires [Rust](https://rustup.rs) (`cargo`).

### Bash / Zsh (Linux, macOS, WSL)

```bash
git clone https://github.com/yourusername/aliases.git
cd aliases
./setup.sh
```

This builds the program, installs it to `~/.local/bin/aliases` (set `ALIASES_BIN_DIR` to change it) and adds this block to `~/.zshrc` or `~/.bashrc`:

```bash
# >>> aliases-manager >>>
eval "$('/home/you/.local/bin/aliases' init zsh)"
# <<< aliases-manager <<<
```

Restart your terminal or run `source ~/.zshrc` (or `~/.bashrc`) to apply.

### PowerShell (Windows)

```powershell
git clone https://github.com/yourusername/aliases.git
cd aliases
.\setup.ps1
. $PROFILE
```

This builds `aliases.exe`, installs it to `%LOCALAPPDATA%\Programs\aliases` (added to your user `PATH`) and adds the integration block to `$PROFILE`. It works in PowerShell 7 and Windows PowerShell 5.1.
Managed aliases are exposed as PowerShell functions. Use `Get-Command <name>` to inspect them. `Get-Alias` only shows native PowerShell aliases.

Running the setup again replaces the integration block instead of duplicating it.

### Windows and WSL on the same machine

Install each side separately: `.\setup.ps1` in PowerShell and `./setup.sh` inside WSL. They are two builds of the same code (`aliases.exe` and a Linux `aliases`), and each keeps its own aliases: `%USERPROFILE%\.config\aliases` on Windows and `~/.config/aliases` in WSL. This is usually what you want, because commands like `& 'C:\Program Files\...'` only make sense in PowerShell.

To share one set of aliases, point `ALIASES_DIR` at the other side's folder (e.g. `export ALIASES_DIR=/mnt/c/Users/you/.config/aliases/aliases.d` in WSL). Files with Windows line endings are read correctly; reading from `/mnt/c` is slower, though.

`alias-doctor` warns when WSL only sees the Windows `aliases.exe` in `PATH` (WSL appends the Windows `PATH` by default).

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `alias-create <name> <cmd> [file]` | Create a new alias. Defaults to `local.alias`. |
| `alias-remove <name> [file]` | Remove an alias. |
| `alias-list` | List all managed aliases. |
| `alias-find <query>` | Search for aliases by name or command. |
| `alias-edit <name>` | Edit the file containing the alias (uses `$EDITOR`). |
| `alias-reload` | Reload aliases immediately. |
| `alias-enable <name>` | Enable a disabled alias file. |
| `alias-disable <name>` | Disable an alias file. |
| `alias-doctor` | Verify installation health. |

The `alias-*` commands are shell functions defined by the integration. Commands that change aliases reload them in the current shell automatically. Each one calls a subcommand of the `aliases` program, which can also be used directly:

```bash
aliases create gs 'git status' git   # also: aliases add
aliases remove gs                    # also: aliases rm
aliases list                         # also: aliases ls
aliases find docker
aliases edit git --path              # print the file instead of opening it
aliases doctor
aliases --help
```

### Examples

```bash
# Create a new alias in the 'git' category
alias-create gs 'git status' git

# List all aliases
alias-list

# Search for docker-related aliases
alias-find docker

# Show help for a command
alias-edit --help
```

```powershell
# Create a PowerShell alias (it is loaded right away)
alias-create gst "git status -sb" git

# Use the alias normally
gst

# Inspect a managed alias in PowerShell
Get-Command gst

# Use explicit PowerShell syntax for executables with spaces in the path
alias-create pfle "& 'C:\Program Files\Notepad++\notepad++.exe' '$PROFILE'" system
pfle

# Run a PowerShell script
alias-create syncdocs "& 'C:\tools\sync-docs.ps1'" scripts
syncdocs

# Run a PowerShell script with fixed arguments
alias-create deploy-api "& 'D:\work\deploy-api.ps1' -Environment dev -Verbose" scripts
deploy-api
```

### Completions

```bash
aliases completions zsh > ~/.zfunc/_aliases     # with fpath+=~/.zfunc before compinit
aliases completions bash > ~/.local/share/bash-completion/completions/aliases
```

```powershell
aliases completions powershell | Out-String | Invoke-Expression
```

### PowerShell Notes

- Managed aliases are PowerShell functions, not `AliasInfo` entries.
- Use `Get-Command <name>` to verify that an alias is loaded.
- Use `-Help` (or `--help`) with PowerShell commands such as `alias-create -Help`.
- Alias names PowerShell cannot use as function names (e.g. `..`) are skipped there; `alias-doctor` lists them.
- If you change alias files outside the `alias-*` commands, run `alias-reload`.
- For commands whose executable path contains spaces, use the `&` operator: `& 'C:\Path With Spaces\tool.exe'`.
- For `.ps1` files, prefer explicit invocation: `& 'C:\path\script.ps1'`.
- Definitions use the same `.alias` format:

```text
alias gst='git status -sb'
alias pfle='& ''C:\Program Files\Notepad++\notepad++.exe'' ''C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1'''
```

### PowerShell Troubleshooting

- `Get-Alias myalias` does not find the alias:
  Use `Get-Command myalias`. This project registers managed aliases as functions.
- `myalias` is not recognized:
  Run `. $PROFILE` or `alias-reload` in the current PowerShell session.
- An executable with spaces in the path fails to open:
  Define the alias with `&` and quote the executable path.
- A `.ps1` alias fails to start:
  Define it explicitly as `& 'C:\path\script.ps1'`. If script execution is blocked, adjust the PowerShell execution policy to match your environment requirements.
- You changed the installed integration and still see old behavior:
  Run `.\setup.ps1` again, then `. $PROFILE`.

## Alias files

```
~/.config/aliases/
  aliases.d/
    00-core.alias    # Core aliases
    local.alias      # Local/custom aliases
    ...
```

Files are loaded in alphabetical order. `local.alias` is always loaded last, so its definitions win. Each line is a comment, a blank line or a definition in POSIX shell syntax:

```text
# git
alias gs='git status'
alias say="echo \"hi\""
```

Other lines (functions, `export`, `alias -g`) are not loaded; the shell prints a warning at startup and `alias-doctor` lists them.

The location follows `$ALIASES_DIR`, then `$ALIASES_ROOT/aliases.d`, then `$XDG_CONFIG_HOME/aliases/aliases.d`, then `~/.config/aliases/aliases.d`.

## Development

- **Tests**: `cargo test` (program), `bats tests/` (Bash integration, after `cargo build`), `pwsh -File tests/smoke.ps1 -Binary target/debug/aliases.exe` (PowerShell integration)
- **Linting**: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `shellcheck setup.sh uninstall.sh share/init.sh`
- **Architecture**: see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DECISIONS.md](docs/DECISIONS.md)

## Uninstallation

### Bash / Zsh

Run `./uninstall.sh` to remove the shell integration and the program. You will be asked whether to remove the configuration directory (`--remove-config` skips the question).

### PowerShell

```powershell
.\uninstall.ps1
```

This removes the `aliases-manager` block from `$PROFILE`, `aliases.exe` and its `PATH` entry, and asks whether the configuration directory should also be removed.

To remove the configuration directory without an extra prompt:

```powershell
.\uninstall.ps1 -RemoveConfig
```
