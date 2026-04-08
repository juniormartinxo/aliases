# Changelog

## 2026-04-08

### Added

- PowerShell-focused installation and usage guidance in the English and Portuguese READMEs.
- PowerShell examples in `alias-create --help`, including executables with spaces in the path and `.ps1` scripts.
- `alias-edit` support in the PowerShell runtime.

### Fixed

- PowerShell aliases are now created in the global function scope, so they remain available after `alias-reload`.
- PowerShell argument forwarding no longer injects an empty argument when a command is called without parameters.
- Aliases that start with Windows executable paths containing spaces now run correctly.
- `setup.ps1` now ensures the PowerShell profile directory exists before writing the integration block.
- `setup.ps1` now replaces the existing `aliases-manager` block in `$PROFILE` instead of duplicating it.
- PowerShell alias lookup and removal now use the correct regex matching behavior for `.alias` files.
