use super::*;

#[test]
fn normalize_release_tag_adds_prefix_once() {
    assert_eq!(normalize_release_tag("0.1.0"), "v0.1.0");
    assert_eq!(normalize_release_tag("v0.1.0"), "v0.1.0");
}

#[test]
fn parse_checksum_reads_standard_and_star_format() {
    let checksums = "abc123  skill-v0.1.0.zip\ndef456 *skill-v0.1.1.zip\n";
    assert_eq!(
        parse_checksum(checksums, "skill-v0.1.0.zip").as_deref(),
        Some("abc123")
    );
    assert_eq!(
        parse_checksum(checksums, "skill-v0.1.1.zip").as_deref(),
        Some("def456")
    );
}
