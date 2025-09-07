use qs::util::check_command;

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
