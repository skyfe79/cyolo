use super::super::*;
use super::common::*;

#[test]
fn test_format_size_exact_mb() {
    assert_eq!(format_size(1_048_576), "1.0 MB");
}
