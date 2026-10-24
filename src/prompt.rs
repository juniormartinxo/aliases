//! Interactive fields for `aliases create`, used when the name or the command is missing
//! and the session is a terminal. Each field is validated as it is typed, with the same
//! rules the command-line arguments follow.

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::validator::Validation;
use inquire::{Confirm, CustomUserError, InquireError, Text};

use crate::alias_file::{self, EXTENSION, normalize_file_arg, shell_quote};
use crate::commands::{DEFAULT_FILE, check_command, check_name, check_not_in_file};
use crate::paths;
use crate::ui::{self, BLUE, DIM, Error, Result, paint};

pub struct NewAlias {
    pub name: String,
    pub command: String,
    pub file: String,
}

/// Prompts need a keyboard to read from and a terminal to draw on (they draw on stderr).
pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stderr().is_terminal()
}

fn theme() -> RenderConfig<'static> {
    let (prompt, answered, pointer) = if ui::unicode_enabled() {
        ("◆", "◇", "›")
    } else {
        ("?", ">", ">")
    };
    if ui::no_color() {
        return RenderConfig::empty()
            .with_prompt_prefix(Styled::new(prompt))
            .with_answered_prompt_prefix(Styled::new(answered))
            .with_highlighted_option_prefix(Styled::new(pointer));
    }
    let grey = StyleSheet::new().with_fg(Color::DarkGrey);
    RenderConfig::default_colored()
        .with_prompt_prefix(Styled::new(prompt).with_fg(Color::LightCyan))
        .with_answered_prompt_prefix(Styled::new(answered).with_fg(Color::LightGreen))
        .with_highlighted_option_prefix(Styled::new(pointer).with_fg(Color::LightCyan))
        .with_help_message(grey)
        .with_default_value(grey)
        .with_answer(
            StyleSheet::new()
                .with_fg(Color::LightGreen)
                .with_attr(Attributes::BOLD),
        )
        .with_canceled_prompt_indicator(Styled::new("cancelled").with_fg(Color::DarkGrey))
}

/// Turns one of our checks into the answer a field validator gives.
fn validation(check: Result) -> std::result::Result<Validation, CustomUserError> {
    Ok(match check {
        Ok(()) => Validation::Valid,
        Err(err) => Validation::Invalid(err.0.into()),
    })
}

/// Suggests the existing alias files, by any part of their name.
#[derive(Clone)]
struct FileNames(Vec<String>);

impl FileNames {
    fn load(dir: &Path) -> Result<Self> {
        let mut names: Vec<String> = alias_file::alias_paths(dir)?
            .iter()
            .map(|p| {
                let name = alias_file::file_name(p);
                name[..name.len() - EXTENSION.len()].to_string()
            })
            .collect();
        if !names.iter().any(|n| n == DEFAULT_FILE) {
            names.push(DEFAULT_FILE.to_string());
        }
        Ok(FileNames(names))
    }

    fn matching(&self, input: &str) -> Vec<String> {
        let needle = input.trim().to_lowercase();
        self.0
            .iter()
            .filter(|name| name.to_lowercase().contains(&needle))
            .cloned()
            .collect()
    }
}

impl Autocomplete for FileNames {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, CustomUserError> {
        Ok(self.matching(input))
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted: Option<String>,
    ) -> std::result::Result<Replacement, CustomUserError> {
        Ok(highlighted.or_else(|| self.matching(input).into_iter().next()))
    }
}

/// `Ok(None)` when the user pressed Esc or Ctrl+C.
fn answer<T>(result: std::result::Result<T, InquireError>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => Ok(None),
        Err(err) => Err(Error(err.to_string())),
    }
}

/// Asks for the fields not given on the command line, then for confirmation.
/// Returns `None` when the user cancels.
pub fn ask_create(
    name: Option<String>,
    command: Option<String>,
    file: Option<String>,
) -> Result<Option<NewAlias>> {
    inquire::set_global_render_config(theme());
    let dir: PathBuf = paths::aliases_dir();
    let file = file.map(|f| normalize_file_arg(&f)).transpose()?;

    eprintln!(
        "{}  {}",
        paint(BLUE, "Create an alias", !ui::no_color()),
        paint(DIM, "Enter confirms · Esc cancels", !ui::no_color())
    );

    let name = match name {
        Some(name) => name,
        None => {
            let (dir, file) = (dir.clone(), file.clone());
            let field = Text::new("Alias name")
                .with_placeholder("gs")
                .with_help_message("What you will type: letters, digits and _.-:+@%,^~")
                .with_validator(move |input: &str| {
                    let name = input.trim();
                    validation(check_name(name).and_then(|()| match &file {
                        Some(file) => check_not_in_file(&dir, file, name),
                        None => Ok(()),
                    }))
                });
            match answer(field.prompt())? {
                Some(name) => name.trim().to_string(),
                None => return Ok(None),
            }
        }
    };

    let command = match command {
        Some(command) => command,
        None => {
            let field = Text::new("Command")
                .with_placeholder("git status")
                .with_help_message("What the alias runs")
                .with_validator(|input: &str| validation(check_command(input)));
            match answer(field.prompt())? {
                Some(command) => command.trim().to_string(),
                None => return Ok(None),
            }
        }
    };

    let file = match file {
        Some(file) => file,
        None => {
            let (dir_for_check, name_for_check) = (dir.clone(), name.clone());
            let field =
                Text::new("File")
                    .with_default(DEFAULT_FILE)
                    .with_autocomplete(FileNames::load(&dir)?)
                    .with_help_message("Tab completes an existing file · a new name creates it")
                    .with_validator(move |input: &str| {
                        let input = input.trim();
                        if input.is_empty() {
                            return validation(Ok(()));
                        }
                        validation(normalize_file_arg(input).and_then(|file| {
                            check_not_in_file(&dir_for_check, &file, &name_for_check)
                        }))
                    });
            match answer(field.prompt())? {
                Some(file) if file.trim().is_empty() => normalize_file_arg(DEFAULT_FILE)?,
                Some(file) => normalize_file_arg(file.trim())?,
                None => return Ok(None),
            }
        }
    };

    let line = format!("alias {name}={}", shell_quote(&command));
    let question = format!("Save to {file}?");
    let confirm = Confirm::new(&question)
        .with_default(true)
        .with_help_message(&line);
    match answer(confirm.prompt())? {
        Some(true) => Ok(Some(NewAlias {
            name,
            command,
            file,
        })),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_files_by_any_part_of_the_name() {
        let files = FileNames(vec![
            "00-core".into(),
            "docker".into(),
            "git".into(),
            "local".into(),
        ]);
        assert_eq!(files.matching(""), files.0);
        assert_eq!(files.matching("OC"), vec!["docker", "local"]);
        assert_eq!(files.matching(" gi "), vec!["git"]);
    }

    #[test]
    fn always_offers_the_default_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("git.alias"), "").unwrap();
        std::fs::write(dir.path().join("off.alias.disabled"), "").unwrap();
        assert_eq!(FileNames::load(dir.path()).unwrap().0, vec!["git", "local"]);
    }
}
