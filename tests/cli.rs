//! End-to-end tests of the `aliases` program against a throwaway config directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use tempfile::TempDir;

struct Env {
    home: TempDir,
}

impl Env {
    fn new() -> Self {
        let env = Env {
            home: tempfile::tempdir().unwrap(),
        };
        fs::create_dir_all(env.dir()).unwrap();
        env
    }

    fn dir(&self) -> PathBuf {
        self.home
            .path()
            .join(".config")
            .join("aliases")
            .join("aliases.d")
    }

    fn file(&self, name: &str) -> PathBuf {
        self.dir().join(name)
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.file(name)).unwrap()
    }

    fn write(&self, name: &str, content: &str) {
        fs::write(self.file(name), content).unwrap();
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("aliases").unwrap();
        cmd.args(args)
            .env("HOME", self.home.path())
            .env("USERPROFILE", self.home.path())
            .env("XDG_CONFIG_HOME", self.home.path().join(".config"))
            .env("COLUMNS", "100")
            .env("LANG", "C.UTF-8")
            .env_remove("ALIASES_ROOT")
            .env_remove("ALIASES_DIR")
            .env_remove("ALIASES_MANAGER_WRAPPED")
            .env_remove("NO_COLOR")
            .env_remove("LC_ALL")
            .env_remove("LC_CTYPE");
        cmd
    }

    fn ok(&self, args: &[&str]) -> String {
        let output = self.cmd(args).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?} failed: {}",
            text(&output.stderr)
        );
        text(&output.stdout)
    }

    /// Runs a command that must fail with exit code 1; returns its stderr.
    fn fails(&self, args: &[&str]) -> String {
        let output: Output = self.cmd(args).output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "{args:?}: {}",
            text(&output.stdout)
        );
        text(&output.stderr)
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn create_writes_a_new_file_with_a_header() {
    let env = Env::new();
    let out = env.ok(&["create", "foo", "echo bar", "testfile"]);
    assert!(out.contains("Alias 'foo' added"));
    assert!(out.contains("alias-reload"));
    assert_eq!(
        env.read("testfile.alias"),
        "# testfile.alias\nalias foo='echo bar'\n"
    );
}

#[test]
fn create_defaults_to_local_and_escapes_quotes() {
    let env = Env::new();
    env.ok(&["create", "say", "echo 'hi' $HOME"]);
    assert!(
        env.read("local.alias")
            .contains(r"alias say='echo '\''hi'\'' $HOME'")
    );
}

#[test]
fn create_refuses_an_alias_already_in_the_file() {
    let env = Env::new();
    env.ok(&["create", "foo", "echo bar", "testfile"]);
    let err = env.fails(&["create", "foo", "echo baz", "testfile.alias"]);
    assert!(err.contains("already exists in testfile.alias"));
}

#[test]
fn create_warns_when_another_file_defines_the_alias() {
    let env = Env::new();
    env.write("git.alias", "alias gs='git status'\n");
    let output = env
        .cmd(&["create", "gs", "git status -sb"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        text(&output.stderr).contains("also defined in git.alias, local.alias; local.alias wins")
    );
}

#[test]
fn create_keeps_the_last_line_intact_and_crlf_endings() {
    let env = Env::new();
    env.write("a.alias", "alias x='1'");
    env.ok(&["create", "y", "2", "a"]);
    assert_eq!(env.read("a.alias"), "alias x='1'\nalias y='2'\n");

    env.write("b.alias", "alias x='1'\r\n");
    env.ok(&["create", "y", "2", "b"]);
    assert_eq!(env.read("b.alias"), "alias x='1'\r\nalias y='2'\r\n");
}

#[test]
fn create_validates_its_arguments() {
    let env = Env::new();
    assert!(
        env.fails(&["create", "a b", "x"])
            .contains("Invalid alias name")
    );
    assert!(
        env.fails(&["create", "a", "  "])
            .contains("cannot be empty")
    );
    assert!(env.fails(&["create", "a", "x\ny"]).contains("one line"));
    assert!(
        env.fails(&["create", "a", "x", "../evil"])
            .contains("Invalid file name")
    );
    assert!(
        !env.home
            .path()
            .join(".config")
            .join("aliases")
            .join("evil.alias")
            .exists()
    );
}

#[test]
fn remove_deletes_the_alias_and_keeps_a_backup() {
    let env = Env::new();
    env.write(
        "t.alias",
        "# t\nalias foo='a'\n  alias bar='b'\nalias foo='c'\n",
    );
    let out = env.ok(&["remove", "foo"]);
    assert!(out.contains("Removed 'foo' from t.alias"));
    assert_eq!(env.read("t.alias"), "# t\n  alias bar='b'\n");
    assert!(env.read("t.alias.bak").contains("alias foo='a'"));
    // Leading whitespace is fine too.
    env.ok(&["remove", "bar"]);
    assert_eq!(env.read("t.alias"), "# t\n");
}

#[test]
fn remove_asks_for_the_file_when_the_alias_is_in_several() {
    let env = Env::new();
    env.write("a.alias", "alias x='1'\n");
    env.write("b.alias", "alias x='2'\n");
    assert!(
        env.fails(&["remove", "x"])
            .contains("multiple files: a.alias, b.alias")
    );
    env.ok(&["remove", "x", "b"]);
    assert_eq!(env.read("b.alias"), "");
    assert_eq!(env.read("a.alias"), "alias x='1'\n");
}

#[test]
fn remove_reports_missing_aliases_and_files() {
    let env = Env::new();
    env.write("a.alias", "alias x='1'\n");
    assert!(env.fails(&["remove", "nope"]).contains("not found"));
    assert!(
        env.fails(&["remove", "x", "zzz"])
            .contains("zzz.alias does not exist")
    );
    assert!(
        env.fails(&["remove", "nope", "a"])
            .contains("not found in a.alias")
    );
}

#[test]
fn list_groups_aliases_under_their_file() {
    let env = Env::new();
    env.ok(&["create", "gs", "git status", "git"]);
    env.ok(&["create", "dcu", "docker compose up", "docker"]);
    env.ok(&["create", "zz", "last", "local"]);
    let out = env.ok(&["list"]);
    let docker = out.find("docker.alias").unwrap();
    let git = out.find("git.alias").unwrap();
    let local = out.find("local.alias").unwrap();
    assert!(docker < git && git < local, "{out}");
    assert!(out[docker..git].contains("dcu  docker compose up"));
    assert!(out.contains("3 aliases in 3 files"));
    assert!(
        !out.contains('\x1b'),
        "no colors when stdout is not a terminal"
    );
}

#[test]
fn list_with_no_aliases() {
    let env = Env::new();
    assert!(env.ok(&["list"]).contains("No aliases found"));
}

#[test]
fn find_matches_names_and_commands_ignoring_case() {
    let env = Env::new();
    env.write(
        "git.alias",
        "alias gs='git status'\nalias findme='echo found'\n",
    );
    let out = env.ok(&["find", "FINDME"]);
    assert_eq!(out, "git.alias: findme  echo found\n");
    assert!(env.ok(&["find", "status"]).contains("gs  git status"));
    assert!(
        env.fails(&["find", "nothing"])
            .contains("No aliases matching")
    );
}

#[test]
fn edit_resolves_files_and_aliases() {
    let env = Env::new();
    env.write("git.alias", "alias gs='git status'\n");
    env.write("other.alias", "alias dup='1'\n");
    env.write("more.alias", "alias dup='2'\n");
    let git = env.file("git.alias").display().to_string();
    for target in ["git", "git.alias", "gs"] {
        assert_eq!(env.ok(&["edit", "--path", target]).trim(), git);
    }
    assert!(
        env.fails(&["edit", "--path", "dup"])
            .contains("multiple files")
    );
    assert!(
        env.fails(&["edit", "--path", "nope"])
            .contains("Available files")
    );
}

#[cfg(unix)]
#[test]
fn edit_runs_the_editor_with_arguments() {
    let env = Env::new();
    env.write("git.alias", "alias gs='git status'\n");
    let out = env
        .cmd(&["edit", "gs"])
        .env("EDITOR", "sed -i -e s/status/log/")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env.read("git.alias"), "alias gs='git log'\n");
}

#[test]
fn disable_and_enable_rename_the_file() {
    let env = Env::new();
    env.write("git.alias", "alias gs='git status'\n");
    assert!(
        env.ok(&["disable", "git"])
            .contains("renamed to git.alias.disabled")
    );
    assert!(env.file("git.alias.disabled").exists());
    assert!(env.ok(&["list"]).contains("No aliases found"));
    assert!(env.fails(&["disable", "git"]).contains("not found"));
    assert!(env.ok(&["enable", "git"]).contains("renamed to git.alias"));
    assert!(env.file("git.alias").exists());
    assert!(env.fails(&["enable", "git"]).contains("not found"));
}

#[test]
fn wrapped_commands_skip_the_reload_hint() {
    let env = Env::new();
    let out = env
        .cmd(&["create", "a", "b"])
        .env("ALIASES_MANAGER_WRAPPED", "1")
        .output()
        .unwrap();
    assert!(!text(&out.stdout).contains("alias-reload"));
}

#[test]
fn init_posix_defines_aliases_and_wrappers() {
    let env = Env::new();
    env.write("local.alias", "alias gs='local wins'\n");
    env.write(
        "git.alias",
        "alias gs='git status'\nalias say=\"echo \\\"hi\\\"\"\r\nexport X=1\n",
    );
    let output = env.cmd(&["init", "bash"]).output().unwrap();
    let out = text(&output.stdout);
    assert!(out.contains("alias-create() {"));
    assert!(out.contains("init bash --aliases-only"));
    assert!(
        out.contains("alias say=\"echo \\\"hi\\\"\"\n"),
        "raw value kept, CR dropped"
    );
    assert!(out.contains("alias gs='local wins'"));
    assert!(
        !out.contains("git status"),
        "the definition that loads last wins"
    );
    assert!(out.contains("_aliases_manager_loaded='say gs'"));
    assert!(text(&output.stderr).contains("1 unsupported line(s) ignored"));

    let only = env.ok(&["init", "zsh", "--aliases-only"]);
    assert!(!only.contains("alias-create"));
    assert!(only.contains("${=_aliases_manager_loaded}"));
}

#[test]
fn init_pwsh_output_is_ascii() {
    let env = Env::new();
    env.write(
        "a.alias",
        "alias olá='x'\nalias hi='echo olá ''x'''\nalias ok-name='Get-ChildItem'\n",
    );
    let out = env.ok(&["init", "pwsh"]);
    assert!(out.is_ascii());
    assert!(out.contains("function global:alias-create"));
    assert!(out.contains(
        "function global:ok-name { _aliases_manager_invoke_alias_command -Command 'Get-ChildItem' -Arguments $args }"
    ));
    assert!(out.contains("[char]0x00E1"));
    assert!(
        !out.contains("function global:ol"),
        "names PowerShell cannot use are skipped"
    );
    assert!(out.contains("$global:ALIASES_LOADED = @('hi', 'ok-name')"));
}

#[test]
fn setup_is_idempotent_and_uninstall_removes_the_block() {
    let env = Env::new();
    let rc = env.home.path().join(".zshrc");
    fs::write(&rc, "export A=1\n# >>> aliases-manager >>>\nsource \"/old/share/init.sh\"\n# <<< aliases-manager <<<\n").unwrap();

    let out = env.ok(&["setup", "--shell", "zsh"]);
    assert!(out.contains("Updated the aliases-manager block"));
    assert!(
        out.contains("Created"),
        "an empty config gets the default aliases"
    );
    env.ok(&["setup", "--shell", "zsh"]);

    let content = fs::read_to_string(&rc).unwrap();
    assert!(content.starts_with("export A=1\n# >>> aliases-manager >>>\neval \"$('"));
    assert!(content.ends_with(" init zsh)\"\n# <<< aliases-manager <<<\n"));
    assert_eq!(content.matches(">>> aliases-manager").count(), 1);
    assert!(env.read("00-core.alias").contains("alias ll='ls -la'"));

    let doctor = env.ok(&["doctor", "--shell", "zsh"]);
    assert!(doctor.contains("Everything looks healthy"), "{doctor}");

    env.ok(&["uninstall", "--shell", "zsh"]);
    assert_eq!(fs::read_to_string(&rc).unwrap(), "export A=1\n");
    assert!(env.dir().exists(), "config is kept unless asked");
    env.ok(&["uninstall", "--rc", rc.to_str().unwrap(), "--remove-config"]);
    assert!(!env.home.path().join(".config").join("aliases").exists());
}

#[test]
fn setup_for_pwsh_needs_the_profile_path() {
    let env = Env::new();
    assert!(
        env.fails(&["setup", "--shell", "pwsh"])
            .contains("--rc $PROFILE")
    );
    let profile = env.home.path().join("Documents/PowerShell/profile.ps1");
    env.ok(&[
        "setup",
        "--shell",
        "pwsh",
        "--rc",
        profile.to_str().unwrap(),
    ]);
    let bytes = fs::read(&profile).unwrap();
    assert!(
        bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
        "new profiles get a BOM for Windows PowerShell"
    );
    assert!(text(&bytes).contains("Invoke-Expression (& "));
}

#[test]
fn doctor_reports_problems() {
    let env = Env::new();
    env.write("a.alias", "alias x='1'\nalias -g G='| grep'\n");
    env.write("b.alias", "alias x='2'\n");
    let rc = env.home.path().join(".bashrc");
    fs::write(
        &rc,
        "# >>> aliases-manager >>>\nsource \"/repo/share/init.sh\"\n# <<< aliases-manager <<<\n",
    )
    .unwrap();
    let output = env.cmd(&["doctor", "--shell", "bash"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let out = text(&output.stdout);
    assert!(
        out.contains("a.alias:2: unsupported line ignored: alias -g G='| grep'"),
        "{out}"
    );
    assert!(out.contains("'x' is defined in a.alias, b.alias; b.alias wins"));
    assert!(out.contains("old script-based integration"));
}

#[test]
fn honours_aliases_dir_overrides() {
    let env = Env::new();
    let custom: &Path = &env.home.path().join("shared");
    fs::create_dir_all(custom).unwrap();
    env.cmd(&["create", "x", "y", "z"])
        .env("ALIASES_DIR", custom)
        .assert()
        .success();
    assert!(custom.join("z.alias").exists());
    assert!(!env.file("z.alias").exists());
}

#[test]
fn create_without_a_terminal_needs_the_arguments() {
    let env = Env::new();
    let err = env.fails(&["create"]);
    assert!(err.contains("run in a terminal to be asked"), "{err}");
    assert!(
        env.fails(&["create", "gs"])
            .contains("Missing the alias name or the command")
    );
    assert!(!env.file("local.alias").exists());
}

#[test]
fn create_takes_the_file_as_an_option_too() {
    let env = Env::new();
    env.ok(&["create", "gs", "git status", "-f", "git"]);
    assert!(env.read("git.alias").contains("alias gs='git status'"));
    let output = env
        .cmd(&["create", "a", "b", "x", "--file", "y"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(2),
        "positional file and --file conflict"
    );
}
