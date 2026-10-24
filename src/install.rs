//! `aliases setup` / `aliases uninstall`: the integration block in the shell startup
//! file, delimited by markers so it can be replaced and removed safely (ADR-002).

use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::Shell;
use crate::alias_file::{self, EXTENSION, shell_quote};
use crate::fail;
use crate::paths;
use crate::ui::{self, Result};

pub const START_MARKER: &str = "# >>> aliases-manager >>>";
pub const END_MARKER: &str = "# <<< aliases-manager <<<";

/// The shell named by `$SHELL`; PowerShell on Windows; otherwise whichever startup
/// file exists.
pub fn detect_shell() -> Result<Shell> {
    if cfg!(windows) {
        return Ok(Shell::Pwsh);
    }
    let shell = env::var("SHELL").unwrap_or_default();
    if shell.ends_with("/zsh") {
        return Ok(Shell::Zsh);
    }
    if shell.ends_with("/bash") {
        return Ok(Shell::Bash);
    }
    if default_rc(Shell::Zsh)?.exists() {
        return Ok(Shell::Zsh);
    }
    if default_rc(Shell::Bash)?.exists() {
        return Ok(Shell::Bash);
    }
    fail!("Could not detect the shell; pass --shell bash|zsh|pwsh")
}

pub fn default_rc(shell: Shell) -> Result<PathBuf> {
    Ok(match shell {
        Shell::Bash => paths::home().join(".bashrc"),
        Shell::Zsh => env::var_os("ZDOTDIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(paths::home)
            .join(".zshrc"),
        Shell::Pwsh => fail!("Pass the PowerShell profile with --rc $PROFILE"),
    })
}

/// The line that loads the aliases at shell startup.
pub fn integration_line(shell: Shell, bin: &Path) -> String {
    let bin = bin.to_string_lossy();
    match shell {
        Shell::Bash | Shell::Zsh => {
            format!("eval \"$({} init {})\"", shell_quote(&bin), shell.name())
        }
        // The path is ASCII-only once quoted, so the profile's encoding does not matter.
        Shell::Pwsh => format!(
            "Invoke-Expression (& {} init pwsh | Out-String)",
            crate::init::pwsh_string(&bin)
        ),
    }
}

/// Byte ranges of the lines of `text`, line endings included.
fn lines_with_endings(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

/// The lines between the markers, if the block is complete.
pub fn find_block(text: &str) -> Option<Vec<&str>> {
    let lines = lines_with_endings(text);
    let start = lines.iter().position(|l| l.trim() == START_MARKER)?;
    let end = start + lines[start..].iter().position(|l| l.trim() == END_MARKER)?;
    Some(lines[start + 1..end].iter().map(|l| l.trim()).collect())
}

/// `text` without the block. An unterminated block is left alone rather than removing
/// everything after it.
pub fn strip_block(text: &str) -> (String, bool) {
    let lines = lines_with_endings(text);
    let Some(start) = lines.iter().position(|l| l.trim() == START_MARKER) else {
        return (text.to_string(), false);
    };
    let Some(len) = lines[start..].iter().position(|l| l.trim() == END_MARKER) else {
        return (text.to_string(), false);
    };
    let mut kept: Vec<&str> = lines[..start].to_vec();
    kept.extend(&lines[start + len + 1..]);
    (kept.concat(), true)
}

pub fn with_block(text: &str, content: &str) -> String {
    let eol = if text.contains("\r\n") || (text.is_empty() && cfg!(windows)) {
        "\r\n"
    } else {
        "\n"
    };
    let (mut out, _) = strip_block(text);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_str(eol);
    }
    for line in [START_MARKER, content, END_MARKER] {
        out.push_str(line);
        out.push_str(eol);
    }
    out
}

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Reads a startup file, telling whether it had a byte order mark. Windows PowerShell
/// 5.1 only reads a profile as UTF-8 when it has one, so a rewrite keeps it.
fn read_rc(path: &Path) -> Result<(String, bool)> {
    if !path.exists() {
        return Ok((String::new(), false));
    }
    let bytes = fs::read(path)?;
    let bom = bytes.starts_with(UTF8_BOM)
        || bytes.starts_with(&[0xFF, 0xFE])
        || bytes.starts_with(&[0xFE, 0xFF]);
    Ok((alias_file::read_text(path, true)?, bom))
}

fn write_rc(path: &Path, text: &str, bom: bool) -> Result {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = if bom { UTF8_BOM.to_vec() } else { Vec::new() };
    bytes.extend_from_slice(text.as_bytes());
    fs::write(path, bytes)?;
    Ok(())
}

fn resolve(shell: Option<Shell>, rc: Option<PathBuf>) -> Result<(Shell, PathBuf)> {
    let shell = match shell {
        Some(shell) => shell,
        None => detect_shell()?,
    };
    let rc = match rc {
        Some(rc) => rc,
        None => default_rc(shell)?,
    };
    Ok((shell, rc))
}

pub fn setup(shell: Option<Shell>, rc: Option<PathBuf>) -> Result {
    let (shell, rc) = resolve(shell, rc)?;
    let bin = env::current_exe()?;

    let dir = paths::aliases_dir();
    fs::create_dir_all(&dir)?;
    let has_files = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .any(|e| !alias_file::file_name(&e.path()).ends_with(".bak"));
    if has_files {
        ui::info(&format!("Using existing aliases in {}", dir.display()));
    } else {
        let ll = match shell {
            Shell::Pwsh => "Get-ChildItem -Force",
            _ => "ls -la",
        };
        let core = dir.join(format!("00-core{EXTENSION}"));
        fs::write(
            &core,
            format!("# Core aliases\nalias ll={}\n", shell_quote(ll)),
        )?;
        ui::success(&format!("Created {}", core.display()));
    }

    let (text, bom) = read_rc(&rc)?;
    let replaced = find_block(&text).is_some();
    let updated = with_block(&text, &integration_line(shell, &bin));
    write_rc(&rc, &updated, bom || (!rc.exists() && shell == Shell::Pwsh))?;
    ui::success(&format!(
        "{} the aliases-manager block in {}",
        if replaced { "Updated" } else { "Added" },
        rc.display()
    ));

    match shell {
        Shell::Pwsh => ui::info("Run '. $PROFILE' or open a new PowerShell session."),
        _ => ui::info(&format!(
            "Run 'source {}' or open a new terminal.",
            rc.display()
        )),
    }
    Ok(())
}

fn confirm(question: &str) -> bool {
    if !io::stdin().is_terminal() {
        return false;
    }
    print!("{question} [y/N] ");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    let _ = io::stdin().lock().read_line(&mut answer);
    matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn uninstall(shell: Option<Shell>, rc: Option<PathBuf>, remove_config: bool) -> Result {
    let (_, rc) = resolve(shell, rc)?;
    let (text, bom) = read_rc(&rc)?;
    let (stripped, found) = strip_block(&text);
    if found {
        write_rc(&rc, &stripped, bom)?;
        ui::success(&format!(
            "Removed the aliases-manager block from {}",
            rc.display()
        ));
    } else {
        ui::warn(&format!(
            "No aliases-manager block found in {}",
            rc.display()
        ));
    }

    let root = paths::root();
    let remove = remove_config
        || (root.exists()
            && confirm(&format!(
                "Remove the configuration directory ({})?",
                root.display()
            )));
    if remove && root.exists() {
        fs::remove_dir_all(&root)?;
        ui::success(&format!("Removed {}", root.display()));
    } else {
        ui::info(&format!("Configuration kept at {}", root.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_the_block_in_place_of_appending_twice() {
        let text = "a\n# >>> aliases-manager >>>\nsource old\n# <<< aliases-manager <<<\nb\n";
        assert_eq!(find_block(text).unwrap(), vec!["source old"]);
        let updated = with_block(text, "eval new");
        assert_eq!(
            updated,
            "a\nb\n# >>> aliases-manager >>>\neval new\n# <<< aliases-manager <<<\n"
        );
        assert_eq!(with_block(&updated, "eval new"), updated);
    }

    #[test]
    fn keeps_crlf_and_adds_a_missing_newline() {
        assert_eq!(
            with_block("x\r\ny", "z"),
            "x\r\ny\r\n# >>> aliases-manager >>>\r\nz\r\n# <<< aliases-manager <<<\r\n"
        );
    }

    #[test]
    fn leaves_unterminated_blocks_alone() {
        let text = "a\n# >>> aliases-manager >>>\nb\n";
        assert_eq!(strip_block(text), (text.to_string(), false));
        assert!(find_block(text).is_none());
    }
}
