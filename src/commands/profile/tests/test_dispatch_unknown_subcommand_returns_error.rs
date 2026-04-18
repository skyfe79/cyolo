use super::super::*;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// Unknown subcommand must not silently route — clap surfaces a parse
/// error and `dispatch` turns it into `NonZeroExit` so the shell sees
/// a non-zero exit code.
#[test]
fn test_dispatch_unknown_subcommand_returns_error() {
    owo_colors::set_override(false);
    let result = dispatch(&to_owned(&["__no_such_subcommand__"]));
    assert!(result.is_err(), "expected Err, got {result:?}");
}
