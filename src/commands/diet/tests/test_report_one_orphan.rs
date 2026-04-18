use super::super::*;
use super::common::*;

#[test]
fn test_report_one_orphan() {
    setup();
    let report = make_report(
        &[("/fakehome/projects/deleted", 1024)],
        vec![],
        2,
    );
    let output = build_report_string(&report, false);
    assert!(output.contains("orphaned projects (1):"));
    assert!(output.contains("/fakehome/projects/deleted"));
    assert!(output.contains("1.0 KB"));
    assert!(output.contains("active projects (2):"));
    // Active size = config_file_size(50000) - orphan_size(1024) = 48976
    assert!(output.contains("47.8 KB"));
    assert!(output.contains("(keep)"));
    assert!(output.contains("Run with --apply to proceed."));
}
