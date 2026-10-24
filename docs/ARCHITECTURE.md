# Architecture

The tool is one Rust program, `aliases`, plus thin install scripts. A program cannot change the shell that runs it, so the shell evaluates code the program prints (`eval "$(aliases init zsh)"`), the same approach as `zoxide` and `starship` (ADR-005).

## The program (`src/`)

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Command line (`clap`): subcommands, help texts, exit codes. |
| `alias_file.rs` | The `.alias` format: parses each line (single, double and unquoted values, `'\''` escapes, CRLF, UTF-8/UTF-16 byte order marks), load order, and the edits (append, remove with `.bak` backup, atomic writes that keep permissions). |
| `init.rs` | `aliases init <shell>`: the code each shell evaluates. Bash/Zsh get `alias` lines; PowerShell gets one function per alias, written in plain ASCII so any console encoding reads it. Both also get the `alias-*` wrappers and remember which aliases they loaded, so `alias-reload` removes the ones that are gone. |
| `commands.rs` | `create`, `remove`, `list`, `find`, `edit`, `enable`, `disable`. |
| `list.rs` | The `list` layout: one block per file, up to 3 columns depending on the terminal width, commands truncated with `…`. |
| `install.rs` | `setup` / `uninstall`: the marked block in `.bashrc`, `.zshrc` or `$PROFILE` (ADR-002), shell detection, default `00-core.alias`. |
| `doctor.rs` | `doctor`: config directory, unsupported lines, aliases defined in several files, names PowerShell cannot load, shell integration, program in `PATH` (including the WSL case where only `aliases.exe` is visible). |
| `paths.rs` | Configuration paths (ADR-001). |
| `ui.rs` | Status messages, colors (only on terminals, honoring `NO_COLOR`) and the error type. |

## Shell integration

`aliases init <shell>` prints:

1. The absolute path of the program, so the wrappers work even when it is not in `PATH`.
2. The `alias-*` functions. Those that change files (`alias-create`, `alias-remove`, `alias-edit`, `alias-enable`, `alias-disable`) call the program and then `alias-reload`. They set `ALIASES_MANAGER_WRAPPED=1` so the program does not print the "run alias-reload" hint.
3. The aliases: first the ones from the previous load are removed, then the current ones are defined. `alias-reload` runs `aliases init <shell> --aliases-only`, which prints only this part.

## Root scripts

### `setup.sh` / `setup.ps1`
- Build the program with `cargo build --release`.
- Install it (`~/.local/bin/aliases`, or `%LOCALAPPDATA%\Programs\aliases\aliases.exe`, added to the user `PATH`).
- Run `aliases setup`, which creates the config directory and writes the integration block.

### `uninstall.sh` / `uninstall.ps1`
- Run `aliases uninstall`, which removes the block and optionally the configuration.
- Remove the installed program.

## `share/`

`share/init.sh` and `share/init.ps1` are compatibility shims for startup files that still have the old `source .../share/init.sh` block: they find the program and load its `init` output. Running the setup again replaces the old block. They will be removed in a future version.

## `tests/`

- `cli.rs`: the program end to end, in a temporary config directory (`cargo test`).
- `test_basic.bats`: Bash sessions that load the integration (`bats tests/`).
- `smoke.ps1`: the same for PowerShell 7 and Windows PowerShell 5.1.
