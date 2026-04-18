use super::super::*;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// Bare `cyolo profile` prints help and returns Ok. This is the clap
/// replacement for the old "dispatch with no args shows help" contract.
#[test]
fn test_dispatch_no_args_shows_help() {
    owo_colors::set_override(false);
    let result = dispatch(&to_owned(&[]));
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}
