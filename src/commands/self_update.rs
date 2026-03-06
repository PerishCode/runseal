use std::io::{self, Cursor, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::{blocking::Response, StatusCode};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tar::Archive;
use tempfile::TempDir;

const REPO_OWNER: &str = "PerishCode";
const REPO_NAME: &str = "envlock";
const DOCS_CHANGELOG_URL: &str = "https://perishcode.github.io/envlock/changelog";

#[derive(Debug, Clone)]
pub struct SelfUpdateOptions {
    pub check_only: bool,
    pub version: Option<String>,
    pub yes: bool,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn run(options: SelfUpdateOptions) -> Result<()> {
    let client = http_client()?;
    let release = fetch_release(&client, options.version.as_deref())?;
    let current = current_version()?;
    let target = parse_semver(&release.tag_name)?;

    if target <= current {
        println!(
            "envlock is up to date (current: v{}, target: {}).",
            current, release.tag_name
        );
        return Ok(());
    }

    if options.check_only {
        println!("Update available: v{} -> {}", current, release.tag_name);
        print_release_notes(&release);
        if let Err(err) = resolve_update_target_path() {
            eprintln!("note: {}", err);
        }
        return Ok(());
    }

    if !options.yes {
        prompt_for_confirmation(&current, &release.tag_name)?;
    }

    let target_binary =
        resolve_update_target_path().context("unable to resolve managed install target")?;

    let target_triple = current_target_triple()?;
    let archive_name = format!("envlock-{}-{target_triple}.tar.gz", release.tag_name);
    let archive_asset = find_asset(&release.assets, &archive_name)
        .with_context(|| format!("release asset not found: {archive_name}"))?;
    let checksums_asset = find_asset(&release.assets, "checksums.txt")
        .context("checksums.txt not found in release assets")?;

    let checksums_text = download_text(&client, &checksums_asset.browser_download_url)?;
    let expected_sha = parse_checksum(&checksums_text, &archive_name)
        .with_context(|| format!("checksum entry not found for {archive_name}"))?;

    let archive_bytes = download_bytes(&client, &archive_asset.browser_download_url)?;
    let actual_sha = sha256_hex(&archive_bytes);
    if actual_sha != expected_sha {
        bail!(
            "checksum mismatch for {} (expected {}, got {})",
            archive_name,
            expected_sha,
            actual_sha
        );
    }

    let temp_dir = TempDir::new().context("failed to create temp directory")?;
    let extracted_binary = extract_binary(&archive_bytes, temp_dir.path())?;
    replace_binary_at_path(extracted_binary, &target_binary)?;

    println!("Updated envlock to {}", release.tag_name);
    print_release_notes(&release);
    Ok(())
}

fn print_release_notes(release: &Release) {
    let highlights = release_highlights(release.body.as_deref(), 3);
    if !highlights.is_empty() {
        println!("Light changelog:");
        for item in highlights {
            println!("- {}", item);
        }
    }
    println!("Release notes: {}", release.html_url);
    println!("Docs changelog index: {}", DOCS_CHANGELOG_URL);
}

fn release_highlights(body: Option<&str>, limit: usize) -> Vec<String> {
    let Some(body) = body else {
        return Vec::new();
    };

    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            line.trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim_start_matches("+ ")
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .take(limit)
        .collect()
}

fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent(format!("{REPO_NAME}-self-update"))
        .build()
        .context("failed to build HTTP client")
}

fn fetch_release(client: &Client, version: Option<&str>) -> Result<Release> {
    let url = release_metadata_url(version);
    let response = client
        .get(url)
        .send()
        .context("failed to fetch release metadata")?;
    if response.status() == StatusCode::NOT_FOUND {
        if let Some(v) = version {
            bail!("release tag not found: {v}");
        }
        bail!("no published release found yet");
    }
    response
        .error_for_status()
        .context("release metadata request failed")?
        .json::<Release>()
        .context("failed to parse release metadata")
}

fn release_metadata_url(version: Option<&str>) -> String {
    match version {
        Some(version) => {
            let tag = normalize_release_tag(version);
            format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/tags/{tag}")
        }
        None => format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest"),
    }
}

fn normalize_release_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

fn current_version() -> Result<Version> {
    Version::parse(env!("CARGO_PKG_VERSION")).context("invalid current version")
}

fn parse_semver(tag: &str) -> Result<Version> {
    let normalized = tag.strip_prefix('v').unwrap_or(tag);
    Version::parse(normalized).with_context(|| format!("invalid release tag version: {tag}"))
}

fn prompt_for_confirmation(current: &Version, next_tag: &str) -> Result<()> {
    print!("Upgrade envlock from v{} to {}? [y/N]: ", current, next_tag);
    io::stdout()
        .flush()
        .context("failed to flush confirmation prompt")?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read confirmation input")?;
    let answer = answer.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        bail!("update cancelled by user");
    }
    Ok(())
}

fn find_asset<'a>(assets: &'a [ReleaseAsset], name: &str) -> Option<&'a ReleaseAsset> {
    assets.iter().find(|asset| asset.name == name)
}

fn download_text(client: &Client, url: &str) -> Result<String> {
    fetch_response(client, url)?
        .text()
        .with_context(|| format!("failed to parse text response: {url}"))
}

fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    fetch_response(client, url)?
        .bytes()
        .with_context(|| format!("failed to read bytes response: {url}"))
        .map(|bytes| bytes.to_vec())
}

fn fetch_response(client: &Client, url: &str) -> Result<Response> {
    client
        .get(url)
        .send()
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .with_context(|| format!("download request failed: {url}"))
}

fn parse_checksum(checksums_text: &str, asset_name: &str) -> Option<String> {
    checksums_text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let file = parts.next()?;
        if file.trim_start_matches('*') == asset_name {
            return Some(hash.to_string());
        }
        None
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn extract_binary(archive_bytes: &[u8], temp_root: &std::path::Path) -> Result<PathBuf> {
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(decoder);
    archive
        .unpack(temp_root)
        .context("failed to unpack release archive")?;

    let candidate = temp_root.join("envlock");
    if candidate.is_file() {
        return Ok(candidate);
    }
    bail!("envlock binary not found in release archive")
}

fn replace_binary_at_path(new_binary: PathBuf, target: &PathBuf) -> Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create target directory for update: {}",
                parent.display()
            )
        })?;
    }
    let parent = target
        .parent()
        .context("failed to resolve executable parent directory")?;

    let staged = parent.join(format!(".envlock.new.{}", std::process::id()));
    std::fs::copy(&new_binary, &staged).context("failed to stage replacement binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&staged)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&staged, perms).context("failed to set executable permissions")?;
    }

    let backup = parent.join(format!(".envlock.old.{}", std::process::id()));

    if target.exists() {
        std::fs::rename(target, &backup).context("failed to rotate current binary")?;
        if let Err(err) = std::fs::rename(&staged, target) {
            let _ = std::fs::rename(&backup, target);
            bail!("failed to install updated binary: {}", err);
        }
        let _ = std::fs::remove_file(&backup);
    } else {
        std::fs::rename(&staged, target).context("failed to install updated binary")?;
    }
    Ok(())
}

fn resolve_update_target_path() -> Result<PathBuf> {
    let managed = managed_install_binary_path()?;
    let exe = std::env::current_exe().context("failed to resolve current executable")?;
    if exe == managed {
        return Ok(managed);
    }
    let canonical = exe
        .canonicalize()
        .with_context(|| format!("failed to resolve executable realpath: {}", exe.display()))?;
    if canonical == managed {
        return Ok(managed);
    }

    bail!(
        "self-update only supports installs under {}. Reinstall via scripts/install.sh",
        managed.display()
    )
}

fn managed_install_binary_path() -> Result<PathBuf> {
    managed_install_binary_path_with_home(std::env::var_os("HOME").map(PathBuf::from))
}

fn managed_install_binary_path_with_home(home: Option<PathBuf>) -> Result<PathBuf> {
    let home = home.context("HOME is not set; unable to resolve managed install path")?;
    Ok(home.join(".envlock/bin/envlock"))
}

fn current_target_triple() -> Result<&'static str> {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        return Ok("x86_64-unknown-linux-gnu");
    }
    if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        return Ok("x86_64-apple-darwin");
    }
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        return Ok("aarch64-apple-darwin");
    }
    bail!("self-update is not supported on this target yet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_semver_accepts_v_prefix() {
        let v = parse_semver("v1.2.3").expect("semver should parse");
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn parse_checksum_reads_sha256sum_format() {
        let checksums = "abc123  envlock-v0.2.0-x86_64-unknown-linux-gnu.tar.gz\n";
        let v = parse_checksum(checksums, "envlock-v0.2.0-x86_64-unknown-linux-gnu.tar.gz");
        assert_eq!(v.as_deref(), Some("abc123"));
    }

    #[test]
    fn parse_checksum_reads_shasum_star_format() {
        let checksums = "def456 *envlock-v0.2.0-x86_64-unknown-linux-gnu.tar.gz\n";
        let v = parse_checksum(checksums, "envlock-v0.2.0-x86_64-unknown-linux-gnu.tar.gz");
        assert_eq!(v.as_deref(), Some("def456"));
    }

    #[test]
    fn managed_install_path_is_home_scoped() {
        let path = managed_install_binary_path_with_home(Some(PathBuf::from("/tmp/envlock-home")))
            .expect("managed path should build");
        assert_eq!(
            path,
            PathBuf::from("/tmp/envlock-home/.envlock/bin/envlock")
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
            "https://api.github.com/repos/PerishCode/envlock/releases/latest"
        );
        assert_eq!(
            release_metadata_url(Some("0.2.1")),
            "https://api.github.com/repos/PerishCode/envlock/releases/tags/v0.2.1"
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
}
