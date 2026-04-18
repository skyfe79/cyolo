use super::super::*;
use super::common::*;

#[test]
fn test_remove_session_folders_empty_list() {
    let (removed, freed) = remove_session_folders(&[]).unwrap();
    assert_eq!(removed, 0);
    assert_eq!(freed, 0);
}
