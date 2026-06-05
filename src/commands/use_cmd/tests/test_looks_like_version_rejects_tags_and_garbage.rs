use super::super::*;

/// Dist-tags (`latest`/`stable`) and malformed input are rejected — `use`
/// wants a concrete version; "give me the newest" is `cyolo update`.
#[test]
fn test_looks_like_version_rejects_tags_and_garbage() {
    assert!(!looks_like_version("latest"));
    assert!(!looks_like_version("stable"));
    assert!(!looks_like_version("2.1"));
    assert!(!looks_like_version("2.1.x"));
    assert!(!looks_like_version("v2.1.158"));
    assert!(!looks_like_version(""));
    assert!(!looks_like_version("2..1"));
}
