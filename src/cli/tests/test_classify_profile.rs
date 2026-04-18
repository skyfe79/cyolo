use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_classify_profile() {
    assert_eq!(
        classify(&args(&["profile", "list"])),
        Command::Profile(args(&["list"]))
    );
}
