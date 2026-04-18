use super::super::*;
use super::common::*;

#[test]
fn test_remove_cache_contents_no_caches() {
    let (removed, freed) = remove_cache_contents(&[]).unwrap();
    assert_eq!(removed, 0);
    assert_eq!(freed, 0);
}
