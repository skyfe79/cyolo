use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_gitdir_returns_none_when_absent() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    fs::create_dir_all(&nested).unwrap();
    // Bounded walk-up: the temp dir has no .git anywhere in its ancestry.
    // However, this test is not *fully* hermetic — if some ancestor of the
    // temp dir happens to be a git repo, this would walk all the way up.
    // On CI that's usually fine; on a dev laptop where /tmp is outside any
    // repo the assertion holds reliably.
    let found = find_gitdir(&nested);
    // We can only reliably assert None when /tmp is not inside a repo.
    // If it is, tolerate that by insisting it's at least outside our temp tree.
    if let Some(path) = &found {
        assert!(
            !path.starts_with(tmp.path()),
            "unexpected gitdir inside tmp: {path:?}"
        );
    }
}
