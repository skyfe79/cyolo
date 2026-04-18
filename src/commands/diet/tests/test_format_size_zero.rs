use super::super::*;
use super::common::*;

#[test]
fn test_format_size_zero() {
    assert_eq!(format_size(0), "0 B");
}
