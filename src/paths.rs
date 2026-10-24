//! Where the configuration lives, following the XDG Base Directory spec (ADR-001).

use std::env;
use std::path::PathBuf;

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

pub fn home() -> PathBuf {
    env::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// `$ALIASES_ROOT`, or `$XDG_CONFIG_HOME/aliases`, or `~/.config/aliases`.
pub fn root() -> PathBuf {
    env_path("ALIASES_ROOT").unwrap_or_else(|| {
        env_path("XDG_CONFIG_HOME")
            .unwrap_or_else(|| home().join(".config"))
            .join("aliases")
    })
}

/// `$ALIASES_DIR`, or `<root>/aliases.d`.
pub fn aliases_dir() -> PathBuf {
    env_path("ALIASES_DIR").unwrap_or_else(|| root().join("aliases.d"))
}
