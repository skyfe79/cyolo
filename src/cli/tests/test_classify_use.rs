use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `cyolo use <...>` is cyolo's version-switch verb. Everything after the `use`
/// token (version + flags such as `--yes`) is forwarded to the verb with the
/// leading token stripped.
#[test]
fn test_classify_use() {
    assert_eq!(
        classify(&args(&["use", "2.1.158"])),
        Command::Use(args(&["2.1.158"]))
    );
    assert_eq!(
        classify(&args(&["use", "2.1.158", "--yes"])),
        Command::Use(args(&["2.1.158", "--yes"]))
    );
    assert_eq!(classify(&args(&["use"])), Command::Use(args(&[])));
    assert_eq!(
        classify(&args(&["use", "--help"])),
        Command::Use(args(&["--help"]))
    );
}
