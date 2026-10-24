# Architecture Decisions Log

## ADR-001: Adoption of XDG Standard
**Date**: 2026-01-02
**Context**: The previous version stored configuration in `~/aliases`. This cluttered the home directory and didn't follow modern standards.
**Decision**: Adopt the XDG Base Directory specification.
**Consequences**: 
- Configuration moves to `~/.config/aliases` (or `$XDG_CONFIG_HOME/aliases`).
- The repository structure is separated from the user configuration.

## ADR-002: Idempotent Installation via Shell Markers
**Date**: 2026-01-02
**Context**: The previous installer blindly appended to RC files, causing duplication.
**Decision**: Use distinct start/end markers (`# >>> aliases-manager >>>`) in RC files.
**Consequences**:
- `setup.sh` can be run multiple times safely.
- `uninstall.sh` can cleanly remove the block.

## ADR-003: Shell-Agnostic Implementation
**Status**: Superseded by ADR-005.
**Date**: 2026-01-02
**Context**: Support for both Bash and Zsh is required without external dependencies like Node or Python.
**Decision**: Use pure POSIX sh/bash. Avoid shell-specific extensions where possible or check for specific shell versions (e.g., `nullglob` handling).
**Consequences**:
- Logic in `utils.sh` and `init.sh` must handle subtle differences between shells.

## ADR-004: Centralized Logic in `share/init.sh`
**Status**: Superseded by ADR-005. The single-line RC block remains; it now evaluates `aliases init <shell>`.
**Date**: 2026-01-02
**Context**: Logic to load aliases needs to be available to the shell.
**Decision**: Place loading logic in `share/init.sh` and source ONLY this file in RC.
**Consequences**:
- Keeps RC file clean (one line).
- Allows updating loading logic without asking users to update their RC files.

## ADR-005: A Single Rust Program Behind Every Shell
**Date**: 2026-10-24
**Context**: Every command existed twice: as Bash scripts in `bin/` and as functions in the 490-line `share/init.ps1` (`alias-edit` three times). Each feature had to be written for each shell, and the `.alias` files were read with `grep` patterns that differed between them (regex characters in names, extra spaces, duplicates across files).
**Decision**: Replace the scripts with one program, `aliases`, written in Rust, with no runtime dependencies. It parses the `.alias` files and prints the code each shell evaluates (`eval "$(aliases init zsh)"`, `Invoke-Expression (& aliases init pwsh | Out-String)`), including the `alias-*` wrappers. Only what must change the current shell stays in shell code: defining the aliases and `alias-reload`.
**Consequences**:
- One implementation for Bash, Zsh and PowerShell; the file format and the config directory are unchanged.
- Windows and WSL install separate builds of the same code, each with its own config by default (`ALIASES_DIR` can point both to one folder).
- Installing needs Rust (`cargo`) until prebuilt binaries are published.
- Only `alias` definitions are loaded; other shell code in `.alias` files is reported and ignored.
- `share/init.sh` and `share/init.ps1` remain as compatibility shims for the old RC block, to be removed later.
