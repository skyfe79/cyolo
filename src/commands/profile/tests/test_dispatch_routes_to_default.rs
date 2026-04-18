use super::super::*;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `default` with no args prints the current default and returns Ok. We
/// use the Ok/Err split to prove that clap routed into `default_cmd::run`
/// rather than the unknown-subcommand fall-through.
#[test]
fn test_dispatch_routes_to_default() {
    owo_colors::set_override(false);
    let result = dispatch(&to_owned(&["default"]));
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}
