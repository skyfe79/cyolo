use super::super::*;
use super::common::*;

#[test]
fn test_report_with_stale_projects() {
    setup();
    let stale = vec![
        StaleProject {
            path: "/fakehome/work/old-client".to_string(),
            last_activity_secs: 120 * 86400, // 120 days
            history_size: 340_000,
            session_size: 10_000,
        },
        StaleProject {
            path: "/fakehome/tmp/experiment".to_string(),
            last_activity_secs: 91 * 86400, // 91 days
            history_size: 580_000,
            session_size: 5_000,
        },
    ];
    let report = make_report_full(&[], vec![], stale, vec![], 3);
    let output = build_report_string(&report, false);

    assert!(output.contains("stale projects (2):"));
    assert!(output.contains("120 days ago"));
    assert!(output.contains("91 days ago"));
    assert!(output.contains("/fakehome/work/old-client"));
    assert!(output.contains("/fakehome/tmp/experiment"));
    assert!(output.contains("(history clearable)"));
}
