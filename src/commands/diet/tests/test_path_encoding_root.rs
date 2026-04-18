use super::super::*;
use super::common::*;

#[test]
fn test_path_encoding_root() {
    assert_eq!(project_path_to_session_dir("/"), "-");
}
