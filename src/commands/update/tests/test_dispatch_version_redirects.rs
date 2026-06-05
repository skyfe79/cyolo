use super::super::*;

/// The legacy `cyolo update <version>` form is no longer a switch — it returns
/// a non-zero exit (2) after printing the redirect to `cyolo use`. We assert on
/// the exit code; the redirect text goes to stderr.
#[test]
fn test_dispatch_version_redirects() {
    let args = vec!["2.1.158".to_string()];
    match dispatch(&args) {
        Err(CyoloError::NonZeroExit(2)) => {}
        other => panic!("expected NonZeroExit(2), got {other:?}"),
    }
}
