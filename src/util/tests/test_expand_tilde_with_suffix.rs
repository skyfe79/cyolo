use super::super::*;

/// `~/foo/bar` joins onto the resolved home directory and sheds the tilde.
#[test]
fn test_expand_tilde_with_suffix() {
    let expanded = expand_tilde("~/foo/bar");
    let s = expanded.to_string_lossy();
    assert!(s.ends_with("/foo/bar"), "got {s}");
    assert!(!s.starts_with("~"), "got {s}");
}
