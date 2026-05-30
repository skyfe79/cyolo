use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `cyolo update <version>` is intercepted for local version switching, and
/// `update --help`/`-h` is intercepted for cyolo's own help — both with the
/// leading `update` token stripped before reaching the verb.
#[test]
fn test_classify_update_with_version() {
    assert_eq!(
        classify(&args(&["update", "2.1.156"])),
        Command::Update(args(&["2.1.156"]))
    );
    assert_eq!(
        classify(&args(&["update", "--help"])),
        Command::Update(args(&["--help"]))
    );
    assert_eq!(
        classify(&args(&["update", "-h"])),
        Command::Update(args(&["-h"]))
    );
}
