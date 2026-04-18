use super::super::*;
use super::common::*;

#[test]
fn test_format_size_mb() {
    assert_eq!(format_size(1_500_000), "1.4 MB");
}
