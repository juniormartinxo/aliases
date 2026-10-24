//! `aliases`: one program behind the alias manager for Bash, Zsh and PowerShell.

mod alias_file;
mod commands;
mod doctor;
mod init;
mod install;
mod list;
mod paths;
mod prompt;
mod ui;

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    #[value(alias = "powershell")]
    Pwsh,
}

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Pwsh => "pwsh",
        }
    }
}

#[derive(Parser)]
#[command(
    name = "aliases",
    version,
    about = "Manage shell aliases kept in ~/.config/aliases/aliases.d",
    after_help = "Shell integration: run `aliases setup`, or add `eval \"$(aliases init zsh)\"` to your rc file.\n\
                  It also defines the alias-create, alias-list, ... and alias-reload commands."
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create an alias; without arguments, asks for each field
    #[command(
        visible_alias = "add",
        after_help = "In a terminal, the fields not given are asked for, with validation as you type\n\
            and the existing files suggested (Tab completes).\n\n\
            Examples:\n  \
            aliases create\n  \
            aliases create -f git\n  \
            aliases create gs 'git status' git\n  \
            aliases create pfle \"& 'C:\\Program Files\\Notepad++\\notepad++.exe' '$PROFILE'\" system\n  \
            aliases create syncdocs \"& 'C:\\tools\\sync-docs.ps1'\" scripts"
    )]
    Create {
        /// Alias name
        name: Option<String>,
        /// Command the alias runs
        command: Option<String>,
        /// File to write to, with or without .alias [default: local]
        file: Option<String>,
        /// File to write to, when asking for the name and the command
        #[arg(short, long = "file", value_name = "FILE", conflicts_with = "file")]
        file_option: Option<String>,
    },
    /// Remove an alias
    #[command(
        visible_alias = "rm",
        after_help = "Examples:\n  aliases remove gst\n  aliases remove gst git"
    )]
    Remove {
        /// Alias name
        name: String,
        /// File to remove it from; needed when several files define it
        file: Option<String>,
    },
    /// List aliases grouped by file
    #[command(visible_alias = "ls")]
    List,
    /// Find aliases whose name or command contains a text (case-insensitive)
    Find {
        /// Text to look for
        query: String,
    },
    /// Open the file of an alias, or an alias file, in $EDITOR
    #[command(after_help = "Examples:\n  aliases edit git\n  aliases edit gst")]
    Edit {
        /// Alias name or file name
        target: String,
        /// Print the file path instead of opening the editor
        #[arg(long)]
        path: bool,
    },
    /// Enable a disabled alias file
    #[command(after_help = "Examples:\n  aliases enable git\n  aliases enable 10-git")]
    Enable {
        /// File name, without .alias
        name: String,
    },
    /// Disable an alias file without deleting it
    #[command(after_help = "Examples:\n  aliases disable git\n  aliases disable 10-git")]
    Disable {
        /// File name, without .alias
        name: String,
    },
    /// Check the configuration, the alias files and the shell integration
    Doctor {
        /// Shell whose integration to check [default: detected]
        #[arg(long, value_enum)]
        shell: Option<Shell>,
        /// Startup file to check (required for PowerShell: --rc $PROFILE)
        #[arg(long)]
        rc: Option<PathBuf>,
    },
    /// Print the code that loads the aliases into a shell
    #[command(after_help = "Examples:\n  \
        eval \"$(aliases init zsh)\"\n  \
        Invoke-Expression (& aliases init pwsh | Out-String)")]
    Init {
        #[arg(value_enum)]
        shell: Shell,
        /// Only the alias definitions (used by alias-reload)
        #[arg(long, hide = true)]
        aliases_only: bool,
    },
    /// Add the integration block to the shell startup file
    Setup {
        /// Shell to integrate with [default: detected]
        #[arg(long, value_enum)]
        shell: Option<Shell>,
        /// Startup file to change (required for PowerShell: --rc $PROFILE)
        #[arg(long)]
        rc: Option<PathBuf>,
    },
    /// Remove the integration block, and optionally the configuration
    Uninstall {
        #[arg(long, value_enum)]
        shell: Option<Shell>,
        #[arg(long)]
        rc: Option<PathBuf>,
        /// Also delete the configuration directory, without asking
        #[arg(long)]
        remove_config: bool,
    },
    /// Print a completion script for the aliases command
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn run(command: Cmd) -> ui::Result {
    match command {
        Cmd::Create {
            name,
            command,
            file,
            file_option,
        } => {
            let file = file.or(file_option);
            match (name, command) {
                (Some(name), Some(command)) => commands::create(&name, &command, file.as_deref()),
                (name, command) if prompt::is_interactive() => {
                    match prompt::ask_create(name, command, file)? {
                        Some(new) => commands::create(&new.name, &new.command, Some(&new.file)),
                        None => {
                            eprintln!("Cancelled.");
                            std::process::exit(130);
                        }
                    }
                }
                _ => fail!(
                    "Missing the alias name or the command. Pass them as arguments, \
                     or run in a terminal to be asked for them."
                ),
            }
        }
        Cmd::Remove { name, file } => commands::remove(&name, file.as_deref()),
        Cmd::List => commands::list(),
        Cmd::Find { query } => commands::find(&query),
        Cmd::Edit { target, path } => commands::edit(&target, path),
        Cmd::Enable { name } => commands::enable(&name),
        Cmd::Disable { name } => commands::disable(&name),
        Cmd::Doctor { shell, rc } => doctor::run(shell, rc),
        Cmd::Init {
            shell,
            aliases_only,
        } => {
            let files = alias_file::load_all(&paths::aliases_dir())?;
            let unsupported: usize = files.iter().map(|f| f.unsupported().count()).sum();
            if unsupported > 0 {
                ui::warn(&format!(
                    "aliases: {unsupported} unsupported line(s) ignored; run 'aliases doctor' for details"
                ));
            }
            let bin = std::env::current_exe()?;
            print!("{}", init::render(shell, &bin, &files, aliases_only));
            Ok(())
        }
        Cmd::Setup { shell, rc } => install::setup(shell, rc),
        Cmd::Uninstall {
            shell,
            rc,
            remove_config,
        } => install::uninstall(shell, rc, remove_config),
        Cmd::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "aliases", &mut io::stdout());
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            ui::error(&err.0);
            ExitCode::FAILURE
        }
    }
}
