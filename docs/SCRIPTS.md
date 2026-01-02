# Scripts Documentation

This document describes in detail the responsibility of each script in the repository.

## Root

### `setup.sh`
**Purpose**: Perform initial installation and environment configuration.
- Creates directory structure in `~/.config/aliases`.
- Detects current shell (Bash/Zsh).
- Injects initialization block (source) into shell's `.rc` file.
- Uses markers (`# >>> aliases-manager >>>`) to ensure idempotency (can be run multiple times without duplicating code).

### `uninstall.sh`
**Purpose**: Remove the tool from the system.
- Cleans up the `.rc` file by removing the initialization block.
- Prompts user to remove configuration files.

## `share/`

### `share/init.sh`
**Purpose**: Script loaded at every new shell session.
- Adds `bin/` to `$PATH`.
- Loads all `.alias` files from `~/.config/aliases/aliases.d/` in alphabetical order.
- Defines shell functions that need to modify current environment (`alias-cd`, `alias-reload`, `alias-edit`).

## `lib/`

### `lib/utils.sh`
**Purpose**: Shared library of functions and constants.
- Defines output colors (red, green, yellow).
- Defines default paths (`XDG_CONFIG_HOME`).
- Contains helper functions like `print_success`, `print_error`, `ensure_dir`.

## `bin/` (Executables)

These scripts are added to PATH and can be executed as commands.

### `alias-create`
**Usage**: `alias-create <name> <command> [file]`
- Creates a new alias.
- Automatically escapes special characters.
- Writes to specified file (or `local.alias` if omitted).
- Checks for duplicates before writing.

### `alias-remove`
**Usage**: `alias-remove <name>`
- Removes an existing alias.
- Generates automatic backup of file before editing (`.bak`).
- Uses temporary files to ensure atomic writes.

### `alias-list`
**Usage**: `alias-list`
- Lists all configured aliases.
- Shows: Alias name, source file, and command executed.

### `alias-find`
**Usage**: `alias-find <query>`
- Text search across all alias files.
- Returns file and line where term was found.

### `alias-doctor`
**Usage**: `alias-doctor`
- Diagnostic script.
- Verifies:
    - Directory structure.
    - Correct Path.
    - Integration in `.rc` file.
    - Execution permissions.

### `alias-enable` / `alias-disable`
**Usage**: `alias-enable <name>` / `alias-disable <name>`
- Manages alias groups by renaming files.
- `disable`: Adds `.disabled` extension to `.alias` file.
- `enable`: Removes `.disabled` extension.
