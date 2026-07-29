#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const QS_BIN: &str = env!("CARGO_BIN_EXE_qs");

fn write_config(home: &Path) {
    let config_dir = home.join(".config").join("qs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "default = \"test\"\n\n\
         [profiles.test]\n\
         host = \"stub.invalid\"\n\
         user = \"tester\"\n\
         port = 22\n",
    )
    .unwrap();
}

fn write_ssh_stub(dir: &Path, body: &str) {
    let stub = dir.join("ssh");
    fs::write(&stub, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
}

fn run_qs(home: &Path, path_dir: &Path, args: &[&str]) -> Output {
    Command::new(QS_BIN)
        .args(args)
        .env("HOME", home)
        .env("PATH", path_dir)
        .output()
        .unwrap()
}

#[test]
fn exec_propagates_remote_exit_code() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let stubs = TempDir::new().unwrap();
    write_ssh_stub(stubs.path(), "exit 42");

    let out = run_qs(home.path(), stubs.path(), &["exec", "--", "true"]);
    assert_eq!(out.status.code(), Some(42));
}

#[test]
fn exec_success_exits_zero() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let stubs = TempDir::new().unwrap();
    write_ssh_stub(stubs.path(), "exit 0");

    let out = run_qs(home.path(), stubs.path(), &["exec", "--", "true"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn connect_propagates_exit_code() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let stubs = TempDir::new().unwrap();
    write_ssh_stub(stubs.path(), "exit 7");

    let out = run_qs(home.path(), stubs.path(), &["connect"]);
    assert_eq!(out.status.code(), Some(7));
}

#[test]
fn exec_spawn_failure_exits_one() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let empty = TempDir::new().unwrap();

    let out = run_qs(home.path(), empty.path(), &["exec", "--", "true"]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to run ssh"), "{stderr}");
}

#[test]
fn exec_detach_sends_wrapped_command() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let stubs = TempDir::new().unwrap();
    write_ssh_stub(stubs.path(), r#"printf '%s\n' "$@""#);

    let out = run_qs(
        home.path(),
        stubs.path(),
        &["exec", "-d", "--", "sleep", "5"],
    );
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("nohup sh -c 'sleep 5'"), "{stdout}");
    assert!(stdout.contains("mktemp /tmp/qs-detach.XXXXXX"), "{stdout}");
    assert!(stdout.contains("</dev/null"), "{stdout}");
}

#[test]
fn exec_tmux_sends_wrapped_command() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let stubs = TempDir::new().unwrap();
    write_ssh_stub(stubs.path(), r#"printf '%s\n' "$@""#);

    let out = run_qs(
        home.path(),
        stubs.path(),
        &["exec", "--tmux", "job", "--", "sleep", "5"],
    );
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("tmux new-session -d -s 'job' 'sleep 5'"),
        "{stdout}"
    );
}

#[test]
fn exec_tmux_invalid_session_rejected_before_spawn() {
    let home = TempDir::new().unwrap();
    write_config(home.path());
    let empty = TempDir::new().unwrap();

    let out = run_qs(
        home.path(),
        empty.path(),
        &["exec", "--tmux", "bad name", "--", "true"],
    );
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("tmux session name"), "{stderr}");
}
