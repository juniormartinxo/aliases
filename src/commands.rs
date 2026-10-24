//! The commands that read and change the alias files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::alias_file::{
    self, AliasFile, DISABLED_SUFFIX, EXTENSION, append_definition, is_valid_name,
    normalize_file_arg, remove_definitions,
};
use crate::fail;
use crate::list::{self, Group, Style};
use crate::paths;
use crate::ui::{self, GREEN, Result, YELLOW, paint, reload_hint, stdout_color};

fn existing_dir() -> Result<PathBuf> {
    let dir = paths::aliases_dir();
    if !dir.is_dir() {
        fail!("Aliases directory not found: {}", dir.display());
    }
    Ok(dir)
}

fn files_defining(files: Vec<AliasFile>, name: &str) -> Vec<AliasFile> {
    files.into_iter().filter(|f| f.defines(name)).collect()
}

fn names(files: &[AliasFile]) -> String {
    files
        .iter()
        .map(|f| f.file_name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A file or alias name given on the command line; it never contains a path.
fn plain(arg: &str) -> Result<&str> {
    if arg.is_empty() || arg.contains(['/', '\\']) || arg == "." || arg == ".." {
        fail!("Invalid name '{arg}': use a plain file or alias name");
    }
    Ok(arg)
}

pub fn create(name: &str, command: &str, file: Option<&str>) -> Result {
    if !is_valid_name(name) {
        fail!(
            "Invalid alias name '{name}': use letters, digits and _.-:+@%,^~, not starting with '-'"
        );
    }
    if command.trim().is_empty() {
        fail!("The command cannot be empty");
    }
    if command.contains(['\n', '\r']) {
        fail!("The command must fit on one line");
    }
    let file_name = normalize_file_arg(file.unwrap_or("local"))?;
    let dir = paths::aliases_dir();
    fs::create_dir_all(&dir)?;
    let target = dir.join(&file_name);

    if target.exists() && AliasFile::load(&target)?.defines(name) {
        fail!("Alias '{name}' already exists in {file_name}");
    }
    append_definition(&target, name, command)?;
    ui::success(&format!("Alias '{name}' added to {}", target.display()));

    let defining = files_defining(alias_file::load_all(&dir)?, name);
    if defining.len() > 1 {
        let winner = &defining[defining.len() - 1].file_name;
        ui::warn(&format!(
            "'{name}' is also defined in {}; {winner} wins because it loads last",
            names(&defining)
        ));
    }
    reload_hint();
    Ok(())
}

pub fn remove(name: &str, file: Option<&str>) -> Result {
    let dir = existing_dir()?;
    let target = match file {
        Some(file) => {
            let file_name = normalize_file_arg(file)?;
            let target = dir.join(&file_name);
            if !target.is_file() {
                fail!("File {file_name} does not exist.");
            }
            target
        }
        None => {
            let mut defining = files_defining(alias_file::load_all(&dir)?, name);
            match defining.len() {
                0 => fail!("Alias '{name}' not found."),
                1 => defining.remove(0).path,
                _ => fail!(
                    "Alias '{name}' exists in multiple files: {}. Specify the file: alias-remove {name} <file>",
                    names(&defining)
                ),
            }
        }
    };

    let file_name = alias_file::file_name(&target);
    if remove_definitions(&target, name)? == 0 {
        fail!("Alias '{name}' not found in {file_name}.");
    }
    ui::success(&format!(
        "Removed '{name}' from {file_name} (backup: {file_name}.bak)"
    ));
    reload_hint();
    Ok(())
}

fn terminal_width() -> usize {
    if let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() {
        return usize::from(width);
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse().ok())
        .filter(|&c| c > 0)
        .unwrap_or(80)
}

pub fn list() -> Result {
    let dir = existing_dir()?;
    let files = alias_file::load_all(&dir)?;
    let groups: Vec<Group> = files
        .iter()
        .map(|f| Group {
            file_name: &f.file_name,
            aliases: f
                .definitions()
                .map(|d| (d.name.as_str(), d.value.as_str()))
                .collect(),
        })
        .collect();
    let style = Style {
        width: terminal_width(),
        color: stdout_color(),
        unicode: ui::unicode_enabled(),
    };
    match list::render(&groups, &style) {
        Some(text) => print!("{text}"),
        None => ui::info(&format!("No aliases found in {}", dir.display())),
    }
    Ok(())
}

pub fn find(query: &str) -> Result {
    let dir = existing_dir()?;
    let needle = query.to_lowercase();
    let color = stdout_color();
    let mut found = false;
    for file in alias_file::load_all(&dir)? {
        for d in file.definitions() {
            if d.name.to_lowercase().contains(&needle) || d.value.to_lowercase().contains(&needle) {
                found = true;
                println!(
                    "{}: {}  {}",
                    paint(YELLOW, &file.file_name, color),
                    paint(GREEN, &d.name, color),
                    d.value
                );
            }
        }
    }
    if !found {
        fail!("No aliases matching '{query}'");
    }
    Ok(())
}

/// The file named `target` (with or without `.alias`), or the one file defining the
/// alias `target`.
fn resolve_edit_target(dir: &Path, target: &str) -> Result<PathBuf> {
    let target = plain(target)?;
    for candidate in [target.to_string(), format!("{target}{EXTENSION}")] {
        let path = dir.join(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }
    let files = alias_file::load_all(dir)?;
    let available = names(&files);
    let mut defining = files_defining(files, target);
    match defining.len() {
        1 => Ok(defining.remove(0).path),
        0 => fail!("Alias file or alias '{target}' not found. Available files: {available}"),
        _ => fail!(
            "Alias '{target}' exists in multiple files: {}. Specify the file name.",
            names(&defining)
        ),
    }
}

#[cfg(unix)]
fn editor_command(path: &Path) -> Command {
    // Through sh, so EDITOR can carry arguments ("code -w").
    let editor = std::env::var("EDITOR")
        .ok()
        .filter(|e| !e.trim().is_empty());
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("{} \"$1\"", editor.as_deref().unwrap_or("vi")))
        .arg("sh")
        .arg(path);
    command
}

#[cfg(windows)]
fn editor_command(path: &Path) -> Command {
    use std::os::windows::process::CommandExt;

    match std::env::var("EDITOR")
        .ok()
        .filter(|e| !e.trim().is_empty())
    {
        // Through cmd, so EDITOR can carry arguments and name .cmd shims such as code.
        Some(editor) => {
            let mut command = Command::new("cmd");
            command.raw_arg(format!("/S /C \"{editor} \"{}\"\"", path.display()));
            command
        }
        None => {
            let mut command = Command::new("notepad");
            command.arg(path);
            command
        }
    }
}

pub fn edit(target: &str, print_path: bool) -> Result {
    let dir = existing_dir()?;
    let path = resolve_edit_target(&dir, target)?;
    if print_path {
        println!("{}", path.display());
        return Ok(());
    }
    let status = editor_command(&path)
        .status()
        .map_err(|e| ui::Error(format!("Could not start the editor: {e}")))?;
    if !status.success() {
        fail!("The editor exited with {status}");
    }
    reload_hint();
    Ok(())
}

fn rename(dir: &Path, from: &Path, to_name: &str) -> Result {
    let to = dir.join(to_name);
    if to.exists() {
        fail!("{to_name} already exists; remove or rename it first");
    }
    fs::rename(from, to)?;
    Ok(())
}

pub fn disable(name: &str) -> Result {
    let dir = existing_dir()?;
    let name = plain(name)?;
    let Some(file_name) = [format!("{name}{EXTENSION}"), name.to_string()]
        .into_iter()
        .find(|c| c.ends_with(EXTENSION) && dir.join(c).is_file())
    else {
        fail!("File {name}{EXTENSION} not found in {}", dir.display());
    };
    let new_name = format!("{file_name}{DISABLED_SUFFIX}");
    rename(&dir, &dir.join(&file_name), &new_name)?;
    ui::success(&format!("Disabled {name} (renamed to {new_name})"));
    reload_hint();
    Ok(())
}

pub fn enable(name: &str) -> Result {
    let dir = existing_dir()?;
    let name = plain(name)?;
    let Some(file_name) = [
        format!("{name}{EXTENSION}{DISABLED_SUFFIX}"),
        format!("{name}{DISABLED_SUFFIX}"),
    ]
    .into_iter()
    .find(|c| dir.join(c).is_file()) else {
        fail!(
            "File {name}{EXTENSION}{DISABLED_SUFFIX} not found in {}",
            dir.display()
        );
    };
    let new_name = file_name[..file_name.len() - DISABLED_SUFFIX.len()].to_string();
    rename(&dir, &dir.join(&file_name), &new_name)?;
    ui::success(&format!("Enabled {name} (renamed to {new_name})"));
    reload_hint();
    Ok(())
}
