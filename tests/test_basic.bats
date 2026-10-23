#!/usr/bin/env bats

setup() {
    # Create a temporary directory for config
    export TEST_DIR="$(mktemp -d)"
    export XDG_CONFIG_HOME="$TEST_DIR/.config"
    export HOME="$TEST_DIR"
    
    # Path to repo bins
    REPO_ROOT="$(cd "$(dirname "$BATS_TEST_DIRNAME")" && pwd)"
    export PATH="$REPO_ROOT/bin:$PATH"
    
    # Initialize structure
    mkdir -p "$XDG_CONFIG_HOME/aliases/aliases.d"
    export ALIASES_DIR="$XDG_CONFIG_HOME/aliases/aliases.d"
    
    # Source utils to have env vars if needed
    source "$REPO_ROOT/lib/utils.sh"
}

teardown() {
    rm -rf "$TEST_DIR"
}

@test "alias-create: creates a new alias file" {
    run alias-create foo 'echo bar' testfile
    [ "$status" -eq 0 ]
    [ -f "$ALIASES_DIR/testfile.alias" ]
    grep -q "alias foo='echo bar'" "$ALIASES_DIR/testfile.alias"
}

@test "alias-create: fails if alias exists" {
    alias-create foo 'echo bar' testfile
    run alias-create foo 'echo baz' testfile
    [ "$status" -eq 1 ]
}

@test "alias-list: lists created alias" {
    alias-create foo 'echo bar' testfile
    run alias-list
    [ "$status" -eq 0 ]
    [[ "$output" == *"foo"* ]]
    [[ "$output" == *"echo bar"* ]]
}

@test "alias-list: groups aliases under their source file" {
    alias-create gs 'git status' git
    alias-create dcu 'docker compose up' docker
    run alias-list
    [ "$status" -eq 0 ]
    [[ "$output" == *"docker.alias"* ]]
    [[ "$output" == *"git.alias"* ]]
    # docker.alias block comes first and holds dcu, not gs
    docker_block="${output%%git.alias*}"
    [[ "$docker_block" == *"dcu"* ]]
    [[ "$docker_block" != *"gs "* ]]
    [[ "$output" == *"2 aliases in 2 files"* ]]
}

@test "alias-list: no color codes when output is not a terminal" {
    alias-create foo 'echo bar' testfile
    run alias-list
    [ "$status" -eq 0 ]
    [[ "$output" != *$'\033'* ]]
}

@test "alias-remove: removes alias" {
    alias-create foo 'echo bar' testfile
    run alias-remove foo
    [ "$status" -eq 0 ]
    run grep "alias foo=" "$ALIASES_DIR/testfile.alias"
    [ "$status" -eq 1 ] # grep should fail to find it
}

@test "alias-find: finds alias" {
    alias-create findme 'echo found' testfile
    run alias-find findme
    [ "$status" -eq 0 ]
    [[ "$output" == *"findme"* ]]
}
