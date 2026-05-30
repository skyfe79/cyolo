use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `cyolo version` and its `ls [remote]` forms are handled in-process, with
/// the leading `version` token stripped before reaching the subcommand tree.
#[test]
fn test_classify_version() {
    assert_eq!(classify(&args(&["version"])), Command::Version(args(&[])));
    assert_eq!(
        classify(&args(&["version", "ls"])),
        Command::Version(args(&["ls"]))
    );
    assert_eq!(
        classify(&args(&["version", "ls", "remote"])),
        Command::Version(args(&["ls", "remote"]))
    );
}
