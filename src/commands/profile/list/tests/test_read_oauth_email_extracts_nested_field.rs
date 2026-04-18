use super::super::*;

/// Happy path: well-formed `.claude.json` with an `oauthAccount.emailAddress`
/// string yields that email.
#[test]
fn test_read_oauth_email_extracts_nested_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".claude.json"),
        r#"{"oauthAccount":{"emailAddress":"test@example.com","accountUuid":"u"}}"#,
    )
    .unwrap();
    assert_eq!(
        read_oauth_email(dir.path()),
        Some("test@example.com".to_string())
    );
}
