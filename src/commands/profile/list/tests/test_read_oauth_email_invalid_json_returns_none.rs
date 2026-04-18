use super::super::*;

/// Corrupt `.claude.json` (not JSON at all) → silent `None`. The read is
/// best-effort because a transient editor save can produce this shape.
#[test]
fn test_read_oauth_email_invalid_json_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".claude.json"), "not json").unwrap();
    assert_eq!(read_oauth_email(dir.path()), None);
}
