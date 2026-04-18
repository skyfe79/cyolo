use super::super::*;
use super::common::*;

#[test]
fn test_path_encoding() {
    assert_eq!(
        project_path_to_session_dir("/Users/codingmax/Private/labs/test-bot"),
        "-Users-codingmax-Private-labs-test-bot"
    );
}
