# Changelog

## 2026-10-24

### Changed

- The tool is now a single Rust program, `aliases`, used by Bash, Zsh and PowerShell (ADR-005). The `alias-*` commands are unchanged; they are now shell functions that call `aliases <subcommand>`, which can also be run directly (`aliases create`, `aliases list`, ...). The `.alias` format and the config directory are the same.
- `setup.sh` / `setup.ps1` build and install the program (requires Rust). The shell block becomes `eval "$(aliases init <shell>)"`. Run the setup again to migrate; until then, the old block keeps working through `share/init.sh` / `share/init.ps1`.
- Commands that change aliases (`alias-create`, `alias-remove`, `alias-edit`, `alias-enable`, `alias-disable`) reload them in Bash/Zsh too, as PowerShell already did.
- `alias-reload` removes aliases that were deleted or disabled, in every shell.
- `alias-remove` without a file refuses to guess when several files define the alias, in Bash/Zsh too.
- `alias-find` searches names and commands, ignoring case, and exits with 1 when nothing matches.
- `alias-list` in PowerShell uses the grouped column layout.
- Lines that are not alias definitions are no longer run; the shell warns at startup and `alias-doctor` lists them.

### Added

- `alias-create` without arguments asks for the name, the command and the file, validating each field as it is typed, suggesting the existing files (Tab completes) and confirming before writing. Given values are not asked for; `-f/--file` picks the file up front. Outside a terminal, missing arguments are an error.
- `alias-doctor` reports unsupported lines, aliases defined in several files, names PowerShell cannot load, the old integration block, and WSL sessions that only see the Windows `aliases.exe`.
- `alias-create` warns when another file already defines the alias and says which one wins.
- `ALIASES_DIR` / `ALIASES_ROOT` to choose the config location (e.g. one folder for Windows and WSL).
- `aliases completions <shell>`.
- `aliases edit --path` prints the file instead of opening the editor.
- CI on Linux, macOS and Windows, including PowerShell 7 and Windows PowerShell 5.1.

### Fixed

- Alias names with regex characters, or definitions with extra spaces (`alias  gs=...`), are now found by `alias-create`, `alias-remove` and `alias-edit`.
- `alias-create` no longer glues the new definition to the last line of a file without a trailing newline, and keeps CRLF files in CRLF.
- `alias-remove` keeps the file's permissions (it used to leave them as 0600).
- Alias files with Windows line endings or a UTF-8/UTF-16 byte order mark load correctly in Bash/Zsh.
- PowerShell aliases with non-ASCII characters load correctly in Windows PowerShell 5.1.
- Alias and file names can no longer point outside the aliases directory.


## 2026-10-23

### Changed

- `alias-list` now groups aliases by source file, with a header per file, and lays them out in up to 3 columns depending on terminal width. Long commands are truncated with `…`, alias names are never cut, and colors are only used when writing to a terminal (respects `NO_COLOR`).

## 2026-04-08

### Added

- PowerShell-focused installation and usage guidance in the English and Portuguese READMEs.
- PowerShell examples in `alias-create --help`, including executables with spaces in the path and `.ps1` scripts.
- `alias-edit` support in the PowerShell runtime.
- `uninstall.ps1` for removing the PowerShell profile integration and optional config directory.
- `-Help` support for PowerShell commands such as `alias-reload`, `alias-remove`, `alias-edit`, `alias-list`, `alias-find`, `alias-enable`, `alias-disable`, and `alias-doctor`.
- `bin/alias-edit` for Bash / Zsh environments without relying only on shell functions.
- Standardized `--help` output for Bash scripts such as `alias-remove`, `alias-enable`, `alias-disable`, and `alias-doctor`.

### Fixed

- PowerShell aliases are now created in the global function scope, so they remain available after `alias-reload`.
- PowerShell argument forwarding no longer injects an empty argument when a command is called without parameters.
- Aliases that start with Windows executable paths containing spaces now run correctly.
- `setup.ps1` now ensures the PowerShell profile directory exists before writing the integration block.
- `setup.ps1` now replaces the existing `aliases-manager` block in `$PROFILE` instead of duplicating it.
- PowerShell alias lookup and removal now use the correct regex matching behavior for `.alias` files.
