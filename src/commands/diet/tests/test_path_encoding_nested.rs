use super::super::*;
use super::common::*;

#[test]
fn test_path_encoding_nested() {
    assert_eq!(
        project_path_to_session_dir("/a/b/c/d"),
        "-a-b-c-d"
    );
}
