use super::super::*;
use super::common::*;

#[test]
fn test_report_with_cache_dirs() {
    setup();
    let caches = vec![
        CacheDir {
            name: "statsig".to_string(),
            path: PathBuf::from("/fakehome/.claude/statsig"),
            size: 12_582_912, // ~12 MB
        },
        CacheDir {
            name: "file-history".to_string(),
            path: PathBuf::from("/fakehome/.claude/file-history"),
            size: 33_554_432, // ~32 MB
        },
    ];
    let report = make_report_full(&[], vec![], vec![], caches, 3);
    let output = build_report_string(&report, false);

    assert!(output.contains("clearable cache dirs (2):"));
    assert!(output.contains("statsig/"));
    assert!(output.contains("file-history/"));
    assert!(output.contains("(removable)"));
}
