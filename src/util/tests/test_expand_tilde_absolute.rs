use super::super::*;
use std::path::PathBuf;

/// Absolute paths pass through unchanged — no tilde, no expansion.
#[test]
fn test_expand_tilde_absolute() {
    assert_eq!(expand_tilde("/absolute/path"), PathBuf::from("/absolute/path"));
}
