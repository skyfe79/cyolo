use super::super::*;

/// Not on disk → download then activate, regardless of what's currently active.
#[test]
fn test_plan_download_when_missing() {
    let installed = vec!["2.1.157".to_string()];
    assert_eq!(
        plan("2.1.158", &installed, Some("2.1.157")),
        Plan::DownloadThenActivate
    );
    // Empty install set still routes to download.
    assert_eq!(
        plan("2.1.158", &[], None),
        Plan::DownloadThenActivate
    );
}
