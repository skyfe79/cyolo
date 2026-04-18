use super::super::*;
use super::common::*;

#[test]
fn test_format_size_kb() {
    assert_eq!(format_size(1536), "1.5 KB");
}
