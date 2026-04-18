use super::super::*;
use super::common::*;

#[test]
fn test_format_size_exact_boundary() {
    assert_eq!(format_size(1024), "1.0 KB");
}
