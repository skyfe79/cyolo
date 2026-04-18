use super::super::*;
use std::path::PathBuf;

/// Relative paths pass through unchanged — the leading tilde is the only
/// trigger for expansion.
#[test]
fn test_expand_tilde_relative() {
    assert_eq!(expand_tilde("relative/path"), PathBuf::from("relative/path"));
}
