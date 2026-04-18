use super::super::*;
use super::common::*;

#[test]
fn test_analyze_detects_orphans() {
    let dir = TempDir::new().unwrap();
    let content = r#"{"projects": {
        "/nonexistent/path/alpha": {"name": "alpha"},
        "/nonexistent/path/beta": {"name": "beta"}
    }}"#;
    let path = write_claude_json(&dir, content);

    let result = analyze_claude_json(&path).unwrap();

    assert_eq!(result.orphaned_projects.len(), 2);
    assert_eq!(result.active_count, 0);
    let orphan_paths: Vec<&str> = result.orphaned_projects.iter().map(|o| o.path.as_str()).collect();
    assert!(orphan_paths.contains(&"/nonexistent/path/alpha"));
    assert!(orphan_paths.contains(&"/nonexistent/path/beta"));
    // Each entry should have non-zero size
    for orphan in &result.orphaned_projects {
        assert!(orphan.entry_size > 0);
    }
}
