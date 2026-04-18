use super::super::*;
use super::common::*;

#[test]
fn test_report_days_ago_format() {
    setup();
    let stale = vec![StaleProject {
        path: "/fakehome/old-proj".to_string(),
        last_activity_secs: 45 * 86400, // 45 days
        history_size: 100,
        session_size: 200,
    }];
    let report = make_report_full(&[], vec![], stale, vec![], 1);
    let output = build_report_string(&report, false);

    assert!(output.contains("45 days ago"));
}
