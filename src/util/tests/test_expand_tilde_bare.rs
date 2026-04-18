use super::super::*;
use std::path::PathBuf;

/// Bare `~` expands to the resolved home directory, never staying literal.
#[test]
fn test_expand_tilde_bare() {
    let expanded = expand_tilde("~");
    assert_ne!(expanded, PathBuf::from("~"));
}
