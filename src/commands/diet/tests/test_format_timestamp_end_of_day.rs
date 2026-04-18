use super::super::*;
use super::common::*;

#[test]
fn test_format_timestamp_end_of_day() {
    // 2026-04-17 23:59:59 UTC = 1776499199
    // Let's use a known: 2025-12-31 23:59:59 UTC = 1767225599
    assert_eq!(format_timestamp(1_767_225_599), "20251231235959");
}
