use super::super::*;

/// The `dist-tags.latest` value is surfaced so the listing can flag it.
#[test]
fn test_parse_npm_versions_reads_latest() {
    let body = r#"{
        "versions": { "2.1.156": {}, "2.1.158": {} },
        "dist-tags": { "latest": "2.1.158" }
    }"#;
    let (_versions, latest) = parse_npm_versions(body).unwrap();
    assert_eq!(latest, Some("2.1.158".to_string()));
}
