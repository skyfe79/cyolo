use super::super::*;
use std::path::Path;

/// Malformed JSON produces `ConfigParseError` (the serde parse failure),
/// NOT `ProfileFileError`. `ProfileFileError` is reserved for the case
/// where JSON is valid but no recognized field is present — see
/// `test_parse_empty_object` / `test_parse_unknown_fields_only`.
#[test]
fn test_parse_malformed_json() {
    let json = b"not json";
    let err = parse(json, Path::new("test.json")).unwrap_err();
    assert!(
        matches!(err, CyoloError::ConfigParseError { .. }),
        "expected ConfigParseError for malformed JSON, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("failed to parse config"), "got: {msg}");
}
