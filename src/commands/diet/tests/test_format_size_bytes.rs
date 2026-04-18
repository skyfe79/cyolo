use super::super::*;
use super::common::*;

#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(512), "512 B");
}
