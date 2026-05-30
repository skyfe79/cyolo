use super::super::*;
use crate::error::CyoloError;

/// A non-JSON body (proxy error page, truncated download) surfaces as
/// RemoteFetchFailed rather than panicking.
#[test]
fn test_parse_npm_versions_invalid_json() {
    match parse_npm_versions("<html>502 Bad Gateway</html>") {
        Err(CyoloError::RemoteFetchFailed { .. }) => {}
        other => panic!("expected RemoteFetchFailed, got {other:?}"),
    }
}
