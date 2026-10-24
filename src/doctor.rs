//! `aliases doctor`: checks the configuration, the alias files and the shell integration.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::Shell;
use crate::alias_file::{self, AliasFile};
use crate::init::is_pwsh_name;
use crate::install;
use crate::paths;
use crate::ui::{self, BLUE, GREEN, RED, Result, YELLOW, paint, stdout_color};

enum Status {
    Ok,
    Warn,
    Fail,
}

struct Report {
    failed: bool,
}

impl Report {
    fn check(&mut self, label: &str, status: Status, details: &[String]) {
        let color = stdout_color();
        let status = match status {
            Status::Ok => paint(GREEN, "✅ OK", color),
            Status::Warn => paint(YELLOW, "⚠️  WARN", color),
            Status::Fail => {
                self.failed = true;
                paint(RED, "❌ FAIL", color)
            }
        };
        println!("{label:<49} {status}");
        for detail in details {
            println!("    {detail}");
        }
    }
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(program))
        .find(|path| path.is_file())
}

fn check_files(report: &mut Report, dir: &Path, shell: Option<Shell>) -> Result {
    let files = alias_file::load_all(dir)?;
    let mut details = Vec::new();

    for file in &files {
        for (number, text) in file.unsupported() {
            details.push(format!(
                "{}:{number}: unsupported line ignored: {text}",
                file.file_name
            ));
        }
    }

    let mut defined_in: HashMap<&str, Vec<&str>> = HashMap::new();
    for file in &files {
        for definition in file.definitions() {
            let places = defined_in.entry(definition.name.as_str()).or_default();
            if !places.contains(&file.file_name.as_str()) {
                places.push(&file.file_name);
            }
        }
    }
    let mut repeated: Vec<_> = defined_in.iter().filter(|(_, f)| f.len() > 1).collect();
    repeated.sort();
    for (name, places) in repeated {
        details.push(format!(
            "'{name}' is defined in {}; {} wins because it loads last",
            places.join(", "),
            places.last().unwrap_or(&"")
        ));
    }

    if shell == Some(Shell::Pwsh) {
        let mut skipped: Vec<_> = defined_in.keys().filter(|n| !is_pwsh_name(n)).collect();
        skipped.sort();
        for name in skipped {
            details.push(format!(
                "'{name}' is not a valid PowerShell function name and is skipped"
            ));
        }
    }

    let count: usize = files
        .iter()
        .map(|f: &AliasFile| f.definitions().count())
        .sum();
    let label = format!("Alias files ({count} aliases in {} files)", files.len());
    let status = if details.is_empty() {
        Status::Ok
    } else {
        Status::Warn
    };
    report.check(&label, status, &details);
    Ok(())
}

fn check_integration(report: &mut Report, shell: Option<Shell>, rc: Option<PathBuf>) {
    let rc = match (shell, rc) {
        (_, Some(rc)) => Ok(rc),
        (Some(shell), None) => install::default_rc(shell),
        (None, None) => install::detect_shell().and_then(install::default_rc),
    };
    let rc = match rc {
        Ok(rc) => rc,
        Err(err) => return report.check("Shell integration", Status::Warn, &[err.0]),
    };
    let label = format!("Shell integration ({})", rc.display());
    let text = alias_file::read_text(&rc, false).unwrap_or_default();
    match install::find_block(&text) {
        Some(block)
            if block
                .iter()
                .any(|l| l.contains("share/init") || l.contains("init.ps1")) =>
        {
            report.check(
                &label,
                Status::Fail,
                &["Uses the old script-based integration; run setup again".to_string()],
            )
        }
        Some(block) if block.iter().any(|l| l.contains(" init ")) => {
            report.check(&label, Status::Ok, &[])
        }
        _ => report.check(
            &label,
            Status::Fail,
            &["No aliases-manager block found; run setup".to_string()],
        ),
    }
}

fn check_path(report: &mut Report) {
    let program = if cfg!(windows) {
        "aliases.exe"
    } else {
        "aliases"
    };
    if let Some(path) = find_in_path(program) {
        return report.check(
            &format!("Program in PATH ({})", path.display()),
            Status::Ok,
            &[],
        );
    }
    let mut details =
        vec!["The alias-* commands still work; add the program's folder to PATH to run 'aliases' directly".to_string()];
    if cfg!(unix) && find_in_path("aliases.exe").is_some() {
        details.insert(
            0,
            "Only the Windows build (aliases.exe) is in PATH; install the Linux build inside WSL"
                .to_string(),
        );
    }
    report.check("Program in PATH", Status::Warn, &details);
}

pub fn run(shell: Option<Shell>, rc: Option<PathBuf>) -> Result {
    println!(
        "{}",
        paint(BLUE, "🚑 Aliases Manager Doctor", stdout_color())
    );
    println!("==============================================");
    let mut report = Report { failed: false };

    let dir = paths::aliases_dir();
    let label = format!("Config directory ({})", dir.display());
    if dir.is_dir() {
        report.check(&label, Status::Ok, &[]);
        check_files(&mut report, &dir, shell)?;
    } else {
        report.check(
            &label,
            Status::Fail,
            &["Run setup to create it".to_string()],
        );
    }
    check_integration(&mut report, shell, rc);
    check_path(&mut report);

    println!("==============================================");
    if report.failed {
        crate::fail!("Some checks failed.");
    }
    ui::success("Everything looks healthy! ✨");
    Ok(())
}
