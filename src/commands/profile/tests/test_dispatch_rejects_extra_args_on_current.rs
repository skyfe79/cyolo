use super::super::*;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `current` accepts no args. Extra positional arg → clap rejects → Err.
/// Replaces the old hand-rolled `test_current_rejects_extra_args`.
#[test]
fn test_dispatch_rejects_extra_args_on_current() {
    owo_colors::set_override(false);
    let result = dispatch(&to_owned(&["current", "unexpected"]));
    assert!(result.is_err(), "expected Err, got {result:?}");
}
