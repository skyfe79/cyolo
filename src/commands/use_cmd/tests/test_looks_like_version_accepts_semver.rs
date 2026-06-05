use super::super::*;

/// Concrete MAJOR.MINOR.PATCH, with or without a prerelease suffix, is accepted.
#[test]
fn test_looks_like_version_accepts_semver() {
    assert!(looks_like_version("2.1.158"));
    assert!(looks_like_version("10.0.0"));
    assert!(looks_like_version("2.1.158-beta.1"));
    assert!(looks_like_version("0.0.1"));
}
