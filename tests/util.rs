use qs::util::{
    build_detach_command, build_tmux_command, check_command, exit_code, shell_escape_single,
    validate_tmux_session,
};

#[test]
fn check_command_returns_false_for_nonexistent() {
    assert!(!check_command("definitely_not_a_real_command"));
}

#[test]
fn check_command_returns_true_for_common_tools() {
    #[cfg(unix)]
    {
        assert!(check_command("ls"));
    }
}

#[test]
fn shell_escape_single_plain() {
    assert_eq!(shell_escape_single("python train.py"), "'python train.py'");
}

#[test]
fn shell_escape_single_embedded_quote() {
    assert_eq!(shell_escape_single("don't"), r"'don'\''t'");
    assert_eq!(shell_escape_single("''"), r"''\'''\'''");
}

#[test]
fn shell_escape_single_double_quote_dollar_backtick() {
    assert_eq!(
        shell_escape_single(r#"echo "$HOME" `id`"#),
        r#"'echo "$HOME" `id`'"#
    );
}

#[test]
fn shell_escape_single_operators_and_newline() {
    assert_eq!(
        shell_escape_single("a && b; c | d\ne"),
        "'a && b; c | d\ne'"
    );
}

#[test]
fn shell_escape_single_empty() {
    assert_eq!(shell_escape_single(""), "''");
}

#[test]
fn build_detach_command_exact() {
    assert_eq!(
        build_detach_command("python train.py"),
        "__qs_log=\"$(mktemp /tmp/qs-detach.XXXXXX)\" || exit 1; \
         nohup sh -c 'python train.py' >\"$__qs_log\" 2>&1 </dev/null & \
         echo \"✓ Detached (pid $!)\"; echo \"  Log: $__qs_log\""
    );
}

#[test]
fn build_detach_command_escapes_quotes() {
    let wrapped = build_detach_command("echo 'hi there'");
    assert!(
        wrapped.contains(r"nohup sh -c 'echo '\''hi there'\'''"),
        "{wrapped}"
    );
}

#[test]
fn build_tmux_command_exact() {
    assert_eq!(
        build_tmux_command("train", "python train.py"),
        "tmux new-session -d -s 'train' 'python train.py' && \
         echo \"✓ tmux session 'train' started (attach: tmux attach -t train)\""
    );
}

#[test]
fn build_tmux_command_escapes_cmd() {
    let wrapped = build_tmux_command("job", "echo '$X'");
    assert!(
        wrapped.starts_with(r"tmux new-session -d -s 'job' 'echo '\''$X'\'''"),
        "{wrapped}"
    );
}

#[test]
fn validate_tmux_session_accepts_safe() {
    for name in ["work", "my-session_1", "A1"] {
        assert!(validate_tmux_session(name).is_ok(), "{name}");
    }
}

#[test]
fn validate_tmux_session_rejects_bad() {
    for name in ["", "bad name", "it's", "a.b", "a:b", "a;b", "-lead"] {
        assert!(validate_tmux_session(name).is_err(), "{name:?}");
    }
}

#[cfg(unix)]
#[test]
fn exit_code_passes_through_codes() {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    assert_eq!(exit_code(ExitStatus::from_raw(0)), 0);
    assert_eq!(exit_code(ExitStatus::from_raw(42 << 8)), 42);
    assert_eq!(exit_code(ExitStatus::from_raw(255 << 8)), 255);
}

#[cfg(unix)]
#[test]
fn exit_code_maps_signals_to_128_plus() {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    assert_eq!(exit_code(ExitStatus::from_raw(9)), 137);
    assert_eq!(exit_code(ExitStatus::from_raw(15)), 143);
}
