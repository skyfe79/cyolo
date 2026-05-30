use super::super::*;

/// The `versions` object keys come back newest-first regardless of their order
/// in the registry document.
#[test]
fn test_parse_npm_versions_extracts_sorted() {
    let body = r#"{
        "versions": {
            "2.1.9": {},
            "2.1.100": {},
            "2.1.10": {}
        },
        "dist-tags": { "latest": "2.1.100" }
    }"#;
    let (versions, _latest) = parse_npm_versions(body).unwrap();
    assert_eq!(
        versions,
        vec![
            "2.1.100".to_string(),
            "2.1.10".to_string(),
            "2.1.9".to_string()
        ]
    );
}
