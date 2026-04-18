use super::super::*;

/// Calling `run` with neither a name nor `--all` must fall out as a usage
/// error. Clap prevents "both" (conflicts_with); cyolo enforces "at
/// least one" at the `run` level.
#[test]
fn test_sync_mcp_missing_target_arg_is_usage_error() {
    let result = run(Args {
        name: None,
        all: false,
    });
    assert!(result.is_err(), "expected Err for neither-arg, got {result:?}");
}
