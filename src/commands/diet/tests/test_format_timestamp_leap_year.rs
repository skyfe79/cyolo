use super::super::*;
use super::common::*;

#[test]
fn test_format_timestamp_leap_year() {
    // 2024-02-29 12:00:00 UTC = 1709208000
    assert_eq!(format_timestamp(1_709_208_000), "20240229120000");
}
