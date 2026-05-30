use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// Bare `cyolo update` (and `update` with a non-help flag) still pass through
/// to `claude update` — Claude Code's native auto-updater. Only `update
/// <version>` and `update --help`/`-h` are intercepted; see
/// `test_classify_update_with_version`.
#[test]
fn test_classify_update_is_not_intercepted() {
    assert_eq!(
        classify(&args(&["update"])),
        Command::Claude(args(&["update"]))
    );
    // A non-help flag is claude's to interpret, so it passes through.
    assert_eq!(
        classify(&args(&["update", "--check"])),
        Command::Claude(args(&["update", "--check"]))
    );
}
