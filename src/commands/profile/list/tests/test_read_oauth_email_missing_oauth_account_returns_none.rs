use super::super::*;

/// Parseable JSON that simply has no `oauthAccount` key → `None`, no panic.
#[test]
fn test_read_oauth_email_missing_oauth_account_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".claude.json"), r#"{"userID":"abc"}"#).unwrap();
    assert_eq!(read_oauth_email(dir.path()), None);
}
