use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_classify_help_variants() {
    assert_eq!(classify(&args(&["help"])), Command::Help);
    assert_eq!(classify(&args(&["--help"])), Command::Help);
    assert_eq!(classify(&args(&["-h"])), Command::Help);
}
