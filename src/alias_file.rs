//! Parsing and editing of `.alias` files.
//!
//! A file holds one definition per line, in POSIX shell syntax:
//!
//! ```text
//! # comment
//! alias gs='git status'
//! alias say="echo \"hi\""
//! ```
//!
//! Anything else is reported as unsupported and ignored when loading.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fail;
use crate::ui::{Error, Result};

pub const EXTENSION: &str = ".alias";
pub const DISABLED_SUFFIX: &str = ".disabled";
/// Loaded after every other file, so its definitions win.
pub const LOCAL_FILE: &str = "local.alias";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub name: String,
    /// The command after shell quoting is removed: what the alias runs.
    pub value: String,
    /// The value exactly as written, quotes included.
    pub raw_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Blank,
    Comment,
    Alias(Definition),
    Unsupported(String),
}

/// Characters allowed in alias names besides ASCII letters and digits.
const NAME_PUNCTUATION: &str = "_.-:+@%,^~";

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || NAME_PUNCTUATION.contains(c))
}

pub fn parse_line(line: &str) -> Line {
    let line = line.trim_end_matches(['\r', '\n']);
    let text = line.trim();
    if text.is_empty() {
        Line::Blank
    } else if text.starts_with('#') {
        Line::Comment
    } else {
        match parse_alias(text) {
            Some(definition) => Line::Alias(definition),
            None => Line::Unsupported(text.to_string()),
        }
    }
}

fn parse_alias(text: &str) -> Option<Definition> {
    let rest = text.strip_prefix("alias")?;
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let rest = rest.trim_start();
    let (name, after) = rest.split_once('=')?;
    if !is_valid_name(name) {
        return None;
    }
    let (value, used) = parse_word(after)?;
    let tail = after[used..].trim_start();
    let tail = tail.strip_prefix(';').unwrap_or(tail).trim_start();
    if !tail.is_empty() && !tail.starts_with('#') {
        return None;
    }
    Some(Definition {
        name: name.to_string(),
        value,
        raw_value: after[..used].to_string(),
    })
}

/// Reads one shell word, removing quotes the way the shell would.
/// Returns the unquoted text and how many bytes of `input` it spans.
fn parse_word(input: &str) -> Option<(String, usize)> {
    let mut out = String::new();
    let mut chars = input.char_indices().peekable();
    while let Some(&(index, c)) = chars.peek() {
        match c {
            '\'' => {
                chars.next();
                loop {
                    match chars.next()? {
                        (_, '\'') => break,
                        (_, ch) => out.push(ch),
                    }
                }
            }
            '"' => {
                chars.next();
                loop {
                    match chars.next()? {
                        (_, '"') => break,
                        (_, '\\') => match chars.next()? {
                            (_, escaped @ ('$' | '`' | '"' | '\\')) => out.push(escaped),
                            (_, other) => {
                                out.push('\\');
                                out.push(other);
                            }
                        },
                        (_, ch) => out.push(ch),
                    }
                }
            }
            '\\' => {
                chars.next();
                out.push(chars.next()?.1);
            }
            c if c.is_whitespace() || ";&|<>()".contains(c) => return Some((out, index)),
            _ => {
                out.push(c);
                chars.next();
            }
        }
    }
    Some((out, input.len()))
}

/// Single-quotes `value` for a POSIX shell: `it's` becomes `'it'\''s'`.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Decodes a file, accepting the UTF-8 and UTF-16 byte order marks Windows editors write.
/// With `strict`, invalid UTF-8 is an error instead of being replaced, so a rewrite never
/// corrupts the file.
fn decode(bytes: &[u8], strict: bool) -> Option<String> {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return decode(rest, strict);
    }
    let utf16 = |rest: &[u8], from: fn([u8; 2]) -> u16| {
        let units: Vec<u16> = rest.chunks_exact(2).map(|c| from([c[0], c[1]])).collect();
        String::from_utf16(&units).ok()
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(rest, u16::from_le_bytes);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(rest, u16::from_be_bytes);
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => Some(text.to_string()),
        Err(_) if !strict => Some(String::from_utf8_lossy(bytes).into_owned()),
        Err(_) => None,
    }
}

pub fn read_text(path: &Path, strict: bool) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    match decode(&bytes, strict) {
        Some(text) => Ok(text),
        None => fail!("{} is not valid UTF-8 text", path.display()),
    }
}

pub struct AliasFile {
    pub path: PathBuf,
    pub file_name: String,
    pub lines: Vec<Line>,
}

impl AliasFile {
    pub fn load(path: &Path) -> Result<Self> {
        let text = read_text(path, false)?;
        Ok(AliasFile {
            path: path.to_path_buf(),
            file_name: file_name(path),
            lines: text.lines().map(parse_line).collect(),
        })
    }

    pub fn definitions(&self) -> impl Iterator<Item = &Definition> {
        self.lines.iter().filter_map(|line| match line {
            Line::Alias(definition) => Some(definition),
            _ => None,
        })
    }

    pub fn defines(&self, name: &str) -> bool {
        self.definitions().any(|d| d.name == name)
    }

    /// Unsupported lines with their 1-based line numbers.
    pub fn unsupported(&self) -> impl Iterator<Item = (usize, &str)> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| match line {
                Line::Unsupported(text) => Some((i + 1, text.as_str())),
                _ => None,
            })
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The `.alias` files of `dir` in load order: alphabetical, with `local.alias` last.
/// A missing directory has no files.
pub fn alias_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => fail!("{}: {e}", dir.display()),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.is_file() && file_name(&path).ends_with(EXTENSION) {
            paths.push(path);
        }
    }
    paths.sort_by_key(|path| {
        let name = file_name(path);
        (name == LOCAL_FILE, name.to_lowercase(), name)
    });
    Ok(paths)
}

pub fn load_all(dir: &Path) -> Result<Vec<AliasFile>> {
    alias_paths(dir)?
        .iter()
        .map(|path| AliasFile::load(path))
        .collect()
}

/// Turns a user-supplied file name (`git`, `git.alias`) into a file name inside the
/// aliases directory, refusing anything that would point elsewhere.
pub fn normalize_file_arg(arg: &str) -> Result<String> {
    if arg.is_empty() || arg.contains(['/', '\\']) || arg == "." || arg == ".." {
        fail!("Invalid file name '{arg}': use a plain name such as 'git' or 'git.alias'");
    }
    if arg.ends_with(EXTENSION) {
        Ok(arg.to_string())
    } else {
        Ok(format!("{arg}{EXTENSION}"))
    }
}

fn line_ending(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

/// Writes through a temporary file and a rename, keeping the original permissions.
fn write_atomic(path: &Path, content: &str) -> Result {
    let tmp = path.with_file_name(format!(".{}.tmp", file_name(path)));
    fs::write(&tmp, content)?;
    if let Ok(metadata) = fs::metadata(path) {
        fs::set_permissions(&tmp, metadata.permissions())?;
    }
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// Appends `alias name='value'`, creating the file with a header comment if needed.
pub fn append_definition(path: &Path, name: &str, value: &str) -> Result {
    let entry = format!("alias {name}={}", shell_quote(value));
    if !path.exists() {
        fs::write(path, format!("# {}\n{entry}\n", file_name(path)))?;
        return Ok(());
    }
    let mut text = read_text(path, true)?;
    let eol = line_ending(&text);
    if !text.is_empty() && !text.ends_with('\n') {
        text.push_str(eol);
    }
    text.push_str(&entry);
    text.push_str(eol);
    write_atomic(path, &text)
}

/// Removes every definition of `name`, after copying the file to `<file>.bak`.
/// Returns how many lines were removed; the file is untouched when there are none.
pub fn remove_definitions(path: &Path, name: &str) -> Result<usize> {
    let text = read_text(path, true)?;
    let mut removed = 0;
    let kept: String = text
        .split_inclusive('\n')
        .filter(|line| match parse_line(line) {
            Line::Alias(definition) if definition.name == name => {
                removed += 1;
                false
            }
            _ => true,
        })
        .collect();
    if removed > 0 {
        let backup = path.with_file_name(format!("{}.bak", file_name(path)));
        fs::copy(path, backup)?;
        write_atomic(path, &kept)?;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(line: &str) -> Definition {
        match parse_line(line) {
            Line::Alias(definition) => definition,
            other => panic!("{line:?} parsed as {other:?}"),
        }
    }

    #[test]
    fn parses_single_quoted_values() {
        let d = alias("alias gs='git status'");
        assert_eq!(d.name, "gs");
        assert_eq!(d.value, "git status");
        assert_eq!(d.raw_value, "'git status'");
    }

    #[test]
    fn parses_escaped_single_quotes() {
        assert_eq!(alias(r"alias say='echo '\''hi'\'''").value, "echo 'hi'");
    }

    #[test]
    fn parses_double_quoted_values_keeping_variables() {
        let d = alias(r#"alias home="cd $HOME && echo \"ok\"""#);
        assert_eq!(d.value, r#"cd $HOME && echo "ok""#);
        assert_eq!(d.raw_value, r#""cd $HOME && echo \"ok\"""#);
    }

    #[test]
    fn parses_unquoted_values_and_trailing_comments() {
        assert_eq!(alias("alias ll=ls # list").value, "ls");
        assert_eq!(alias("  alias  ll=ls\\ -la;").value, "ls -la");
        assert_eq!(alias("alias empty=").value, "");
    }

    #[test]
    fn ignores_line_endings() {
        assert_eq!(alias("alias gs='git status'\r\n").value, "git status");
    }

    #[test]
    fn classifies_other_lines() {
        assert_eq!(parse_line("   "), Line::Blank);
        assert_eq!(parse_line("# git"), Line::Comment);
        for line in [
            "export FOO=1",
            "alias -g G='| grep'",
            "alias gs",
            "alias gs='unterminated",
            "alias a='x'; alias b='y'",
            "alias $(rm)='x'",
            "aliasgs='x'",
        ] {
            assert!(
                matches!(parse_line(line), Line::Unsupported(_)),
                "{line:?} should be unsupported"
            );
        }
    }

    #[test]
    fn quoting_round_trips() {
        for value in ["git status", "echo 'hi'", "it's", "a\"b$c", ""] {
            let line = format!("alias x={}", shell_quote(value));
            assert_eq!(alias(&line).value, value);
        }
    }

    #[test]
    fn decodes_byte_order_marks() {
        let utf8 = [&[0xEF, 0xBB, 0xBF][..], b"alias a=b"].concat();
        assert_eq!(decode(&utf8, true).unwrap(), "alias a=b");
        let mut utf16 = vec![0xFF, 0xFE];
        for unit in "alias a=b".encode_utf16() {
            utf16.extend(unit.to_le_bytes());
        }
        assert_eq!(decode(&utf16, true).unwrap(), "alias a=b");
        assert!(decode(&[0xE3, b'a'], true).is_none());
        assert!(decode(&[0xE3, b'a'], false).is_some());
    }

    #[test]
    fn validates_names() {
        for name in ["gs", "docker-up", "..", "g.s", "k8s_ctx", "x+"] {
            assert!(is_valid_name(name), "{name}");
        }
        for name in ["", "-x", "a b", "a/b", "a$b", "a'b", "a=b"] {
            assert!(!is_valid_name(name), "{name}");
        }
    }

    #[test]
    fn normalizes_file_arguments() {
        assert_eq!(normalize_file_arg("git").unwrap(), "git.alias");
        assert_eq!(normalize_file_arg("git.alias").unwrap(), "git.alias");
        assert!(normalize_file_arg("../x").is_err());
        assert!(normalize_file_arg("a\\b").is_err());
    }
}
