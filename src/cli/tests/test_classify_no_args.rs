use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// Bare `cyolo` launches the interactive claude session; no help or update
/// short-circuit should intervene.
#[test]
fn test_classify_no_args() {
    assert_eq!(classify(&args(&[])), Command::Claude(vec![]));
}
