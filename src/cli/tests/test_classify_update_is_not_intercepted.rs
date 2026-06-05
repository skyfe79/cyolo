use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `cyolo update` now means "update to the latest build": bare `update` and any
/// flag form (`--check`, `--help`, `-h`) pass through to `claude update` —
/// Claude Code's native auto-updater. Only the leftover positional `update
/// <version>` is intercepted, to redirect to `cyolo use`; see
/// `test_classify_update_with_version`.
#[test]
fn test_classify_update_is_not_intercepted() {
    assert_eq!(
        classify(&args(&["update"])),
        Command::Claude(args(&["update"]))
    );
    // A non-positional flag is claude's to interpret, so it passes through.
    assert_eq!(
        classify(&args(&["update", "--check"])),
        Command::Claude(args(&["update", "--check"]))
    );
    // `--help`/`-h` are no longer cyolo's — `update` is a pass-through verb now.
    assert_eq!(
        classify(&args(&["update", "--help"])),
        Command::Claude(args(&["update", "--help"]))
    );
    assert_eq!(
        classify(&args(&["update", "-h"])),
        Command::Claude(args(&["update", "-h"]))
    );
}
