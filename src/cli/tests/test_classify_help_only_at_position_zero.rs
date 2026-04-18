use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// A later `--help` is passthrough (it's an argument to claude, not a
/// request for cyolo help). The classify function only intercepts at
/// position zero.
#[test]
fn test_classify_help_only_at_position_zero() {
    assert_eq!(
        classify(&args(&["-p", "--help"])),
        Command::Claude(args(&["-p", "--help"]))
    );
}
