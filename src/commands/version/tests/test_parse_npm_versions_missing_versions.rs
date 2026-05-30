use super::super::*;
use crate::error::CyoloError;

/// Valid JSON that lacks a `versions` object (e.g. an auth/error payload) is a
/// fetch failure, not an empty success.
#[test]
fn test_parse_npm_versions_missing_versions() {
    match parse_npm_versions(r#"{"error":"not found"}"#) {
        Err(CyoloError::RemoteFetchFailed { .. }) => {}
        other => panic!("expected RemoteFetchFailed, got {other:?}"),
    }
}
