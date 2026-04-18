use super::super::*;
use super::common::*;

#[test]
fn test_report_nothing_to_clean_all_empty() {
    setup();
    let report = make_report_full(&[], vec![], vec![], vec![], 5);
    let output = build_report_string(&report, false);

    assert!(output.contains("No orphaned projects found. Nothing to clean up."));
    assert!(!output.contains("├─"));
    assert!(!output.contains("Total reclaimable"));
}
