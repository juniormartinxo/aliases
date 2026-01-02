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
**Date**: 2026-01-02
**Context**: Support for both Bash and Zsh is required without external dependencies like Node or Python.
**Decision**: Use pure POSIX sh/bash. Avoid shell-specific extensions where possible or check for specific shell versions (e.g., `nullglob` handling).
**Consequences**:
- Logic in `utils.sh` and `init.sh` must handle subtle differences between shells.

## ADR-004: Centralized Logic in `share/init.sh`
**Date**: 2026-01-02
**Context**: Logic to load aliases needs to be available to the shell.
**Decision**: Place loading logic in `share/init.sh` and source ONLY this file in RC.
**Consequences**:
- Keeps RC file clean (one line).
- Allows updating loading logic without asking users to update their RC files.
