use super::*;

#[test]
fn parse_semver_accepts_v_prefix() {
    let v = parse_semver("v1.2.3").expect("semver should parse");
    assert_eq!(v, Version::new(1, 2, 3));
}

#[test]
fn parse_checksum_reads_sha256sum_format() {
    let checksums = "abc123  runseal-v0.2.0-x86_64-unknown-linux-gnu.tar.gz\n";
    let v = parse_checksum(checksums, "runseal-v0.2.0-x86_64-unknown-linux-gnu.tar.gz");
    assert_eq!(v.as_deref(), Some("abc123"));
}

#[test]
fn parse_checksum_reads_shasum_star_format() {
    let checksums = "def456 *runseal-v0.2.0-x86_64-unknown-linux-gnu.tar.gz\n";
    let v = parse_checksum(checksums, "runseal-v0.2.0-x86_64-unknown-linux-gnu.tar.gz");
    assert_eq!(v.as_deref(), Some("def456"));
}

#[test]
fn managed_install_path_is_home_scoped() {
    let path = managed_install_binary_path_with_home(Some(PathBuf::from("/tmp/runseal-home")))
        .expect("managed path should build");
    assert_eq!(
        path,
        PathBuf::from("/tmp/runseal-home/.runseal/bin/runseal")
    );
}

#[test]
fn normalize_release_tag_adds_prefix_once() {
    assert_eq!(normalize_release_tag("0.2.1"), "v0.2.1");
    assert_eq!(normalize_release_tag("v0.2.1"), "v0.2.1");
}

#[test]
fn release_metadata_url_uses_expected_endpoint() {
    assert_eq!(
        release_metadata_url(None),
        "https://api.github.com/repos/PerishCode/runseal/releases/latest"
    );
    assert_eq!(
        release_metadata_url(Some("0.2.1")),
        "https://api.github.com/repos/PerishCode/runseal/releases/tags/v0.2.1"
    );
}

#[test]
fn release_highlights_extracts_light_changelog_lines() {
    let body = "# Release v0.3.0\n\n- add meta-first docs\n- tighten converge checks\n\nSee details below.";
    let items = release_highlights(Some(body), 3);
    assert_eq!(
        items,
        vec![
            "add meta-first docs".to_string(),
            "tighten converge checks".to_string(),
            "See details below.".to_string()
        ]
    );
}

#[test]
fn release_highlights_handles_missing_body() {
    let items = release_highlights(None, 3);
    assert!(items.is_empty());
}

#[test]
fn docs_changelog_tag_match_normalizes_prefix() {
    let release = DocsChangelogRelease {
        tag: "0.2.1".to_string(),
        highlights: vec!["line".to_string()],
    };

    let matched = normalize_release_tag(&release.tag) == normalize_release_tag("v0.2.1");
    assert!(matched);
}
