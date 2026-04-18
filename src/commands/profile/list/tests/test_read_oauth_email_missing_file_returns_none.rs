use super::super::*;

/// A profile that has never logged in has no `.claude.json` on disk →
/// silent `None`, not an error.
#[test]
fn test_read_oauth_email_missing_file_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read_oauth_email(dir.path()), None);
}
