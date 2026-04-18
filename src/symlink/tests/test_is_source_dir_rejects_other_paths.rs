use super::super::*;

#[test]
fn test_is_source_dir_rejects_other_paths() {
    assert!(!is_source_dir(std::path::Path::new("/tmp/some-profile")));
}
