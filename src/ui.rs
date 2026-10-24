//! Console output: status messages, colors and the shared error type.

use std::fmt;
use std::io::{self, IsTerminal};

pub const RED: &str = "\x1b[0;31m";
pub const GREEN: &str = "\x1b[0;32m";
pub const YELLOW: &str = "\x1b[1;33m";
pub const BLUE: &str = "\x1b[0;34m";
pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

/// Set by the shell wrappers (`alias-create`, ...), which reload aliases themselves.
pub const WRAPPED_ENV: &str = "ALIASES_MANAGER_WRAPPED";

/// An error that ends the command with exit code 1 and a message on stderr.
#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T = ()> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(err.to_string())
    }
}

/// Returns early with an [`Error`] built from a format string.
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {
        return Err($crate::ui::Error(format!($($arg)*)))
    };
}

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
}

pub fn stdout_color() -> bool {
    !no_color() && io::stdout().is_terminal()
}

fn stderr_color() -> bool {
    !no_color() && io::stderr().is_terminal()
}

pub fn paint(color: &str, text: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub fn success(msg: &str) {
    println!("{}", paint(GREEN, &format!("✅ {msg}"), stdout_color()));
}

pub fn info(msg: &str) {
    println!("{}", paint(BLUE, &format!("ℹ️  {msg}"), stdout_color()));
}

pub fn warn(msg: &str) {
    eprintln!("{}", paint(YELLOW, &format!("⚠️  {msg}"), stderr_color()));
}

pub fn error(msg: &str) {
    eprintln!("{}", paint(RED, &format!("❌ {msg}"), stderr_color()));
}

/// Tells the user to reload, unless a shell wrapper is about to do it.
pub fn reload_hint() {
    if std::env::var_os(WRAPPED_ENV).is_none() {
        info("Run 'alias-reload' to apply changes.");
    }
}

/// Whether box-drawing characters and the ellipsis can be printed.
pub fn unicode_enabled() -> bool {
    if cfg!(windows) {
        return true;
    }
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_ascii_lowercase();
    locale.contains("utf-8") || locale.contains("utf8")
}
