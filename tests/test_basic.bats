#!/usr/bin/env bats
# Shell integration: what `eval "$(aliases init bash)"` gives an interactive shell.
# Needs a built program: cargo build (or set ALIASES_BIN).

setup() {
    REPO_ROOT="$(cd "$(dirname "$BATS_TEST_DIRNAME")" && pwd)"
    BIN="${ALIASES_BIN:-$REPO_ROOT/target/debug/aliases}"
    [ -x "$BIN" ] || skip "build the program first (cargo build)"

    TEST_DIR="$(mktemp -d)"
    export HOME="$TEST_DIR"
    export XDG_CONFIG_HOME="$TEST_DIR/.config"
    unset ALIASES_ROOT ALIASES_DIR
    ALIASES_DIR="$XDG_CONFIG_HOME/aliases/aliases.d"
    mkdir -p "$ALIASES_DIR"
}

teardown() {
    rm -rf "$TEST_DIR"
}

# Runs commands in a shell that loaded the integration.
in_shell() {
    run bash -c "eval \"\$('$BIN' init bash)\"; $1"
}

@test "init: loads the aliases of every file, local.alias last" {
    echo "alias gs='git status'" > "$ALIASES_DIR/git.alias"
    echo "alias gs='git status -sb'" > "$ALIASES_DIR/local.alias"
    in_shell 'alias gs'
    [ "$status" -eq 0 ]
    [ "$output" = "alias gs='git status -sb'" ]
}

@test "alias-create: defines the alias in the current shell" {
    in_shell "alias-create foo 'echo bar' testfile >/dev/null && alias foo"
    [ "$status" -eq 0 ]
    [ "$output" = "alias foo='echo bar'" ]
    grep -q "alias foo='echo bar'" "$ALIASES_DIR/testfile.alias"
}

@test "alias-create: fails if the alias exists" {
    echo "alias foo='echo bar'" > "$ALIASES_DIR/testfile.alias"
    in_shell "alias-create foo 'echo baz' testfile"
    [ "$status" -eq 1 ]
}

@test "alias-remove: removes the alias from the current shell" {
    echo "alias foo='echo bar'" > "$ALIASES_DIR/testfile.alias"
    in_shell 'alias-remove foo >/dev/null; alias foo'
    [ "$status" -eq 1 ]
}

@test "alias-disable / alias-enable: unload and reload a file" {
    echo "alias foo='echo bar'" > "$ALIASES_DIR/testfile.alias"
    in_shell 'alias-disable testfile >/dev/null; alias foo'
    [ "$status" -eq 1 ]
    in_shell 'alias-enable testfile >/dev/null; alias foo'
    [ "$status" -eq 0 ]
}

@test "alias-reload: picks up changes made outside the shell" {
    in_shell "echo \"alias late='echo late'\" > '$ALIASES_DIR/late.alias'; alias-reload; alias late"
    [ "$status" -eq 0 ]
    [[ "$output" == *"Aliases reloaded!"* ]]
    [[ "$output" == *"alias late='echo late'"* ]]
}

@test "alias-list and alias-find: work through the wrappers" {
    echo "alias findme='echo found'" > "$ALIASES_DIR/testfile.alias"
    in_shell 'alias-list'
    [[ "$output" == *"testfile.alias"* ]]
    [[ "$output" == *"1 alias in 1 file"* ]]
    in_shell 'alias-find findme'
    [ "$status" -eq 0 ]
    [[ "$output" == *"findme"* ]]
}

@test "aliases expand when used" {
    echo "alias greet='echo hello from alias'" > "$ALIASES_DIR/x.alias"
    run bash -ic "eval \"\$('$BIN' init bash)\"
greet"
    [[ "$output" == *"hello from alias"* ]]
}

@test "share/init.sh: the old integration still loads the aliases" {
    echo "alias foo='echo bar'" > "$ALIASES_DIR/testfile.alias"
    mkdir -p "$TEST_DIR/bin"
    ln -s "$BIN" "$TEST_DIR/bin/aliases"
    run bash -c "PATH='$TEST_DIR/bin':\$PATH; source '$REPO_ROOT/share/init.sh'; alias foo"
    [ "$status" -eq 0 ]
    [ "$output" = "alias foo='echo bar'" ]
}
