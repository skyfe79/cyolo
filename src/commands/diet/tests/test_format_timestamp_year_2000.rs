use super::super::*;
use super::common::*;

#[test]
fn test_format_timestamp_year_2000() {
    // 2000-01-01 00:00:00 UTC = 946684800
    assert_eq!(format_timestamp(946_684_800), "20000101000000");
}
