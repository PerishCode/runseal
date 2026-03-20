use std::io::{self, Cursor, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use reqwest::{StatusCode, blocking::Response};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::core::config::{RawEnv, resolve_runseal_home};

const REPO_OWNER: &str = "PerishCode";
const REPO_NAME: &str = "runseal";
pub const SKILL_INSTALL_HOME_ENV: &str = "RUNSEAL_SKILL_INSTALL_HOME";

#[derive(Debug, Clone)]
pub struct SkillInstallOptions {
    pub version: Option<String>,
    pub force: bool,
    pub yes: bool,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn run_install(options: SkillInstallOptions) -> Result<()> {
    let client = http_client()?;
    let release = fetch_release(&client, options.version.as_deref())?;
    let tag = normalize_release_tag(&release.tag_name);
    let skill_asset_name = format!("skill-{tag}.zip");

    let skill_asset = find_asset(&release.assets, &skill_asset_name)
        .with_context(|| format!("skill asset not found in release: {skill_asset_name}"))?;
    let checksums_asset = find_asset(&release.assets, "checksums.txt")
        .context("checksums.txt not found in release assets")?;

    let checksums_text = download_text(&client, &checksums_asset.browser_download_url)?;
    let expected_sha = parse_checksum(&checksums_text, &skill_asset_name)
        .with_context(|| format!("checksum entry not found for {skill_asset_name}"))?;

    let zip_bytes = download_bytes(&client, &skill_asset.browser_download_url)?;
    let actual_sha = sha256_hex(&zip_bytes);
    if actual_sha != expected_sha {
        bail!(
            "checksum mismatch for {} (expected {}, got {})",
            skill_asset_name,
            expected_sha,
            actual_sha
        );
    }

    let install_home = resolve_skill_install_home()?;
    let target_skill_dir = install_home.join("runseal");

    if target_skill_dir.exists() {
        if !options.force {
            bail!(
                "skill target already exists: {} (use --force to overwrite)",
                target_skill_dir.display()
            );
        }
        if !options.yes {
            prompt_for_overwrite(&target_skill_dir)?;
        }
        std::fs::remove_dir_all(&target_skill_dir).with_context(|| {
            format!(
                "failed to remove existing skill directory: {}",
                target_skill_dir.display()
            )
        })?;
    }

    println!(
        "Installing skill package {} to {}",
        tag,
        install_home.display()
    );
    std::fs::create_dir_all(&install_home).with_context(|| {
        format!(
            "failed to create skill install directory: {}",
            install_home.display()
        )
    })?;

    extract_skill_zip(&zip_bytes, &install_home)?;

    println!("Skill install complete.");
    println!("- version: {}", tag);
    println!("- path: {}", target_skill_dir.display());
    println!(
        "Tip: set {} to override install root.",
        SKILL_INSTALL_HOME_ENV
    );
    Ok(())
}

fn resolve_skill_install_home() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(SKILL_INSTALL_HOME_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
    {
        return Ok(path);
    }

    let runseal_home = resolve_runseal_home(&RawEnv::from_process())?;
    Ok(runseal_home.join("skills"))
}

fn extract_skill_zip(zip_bytes: &[u8], destination: &std::path::Path) -> Result<()> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).context("failed to open skill zip archive")?;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .with_context(|| format!("failed to read zip entry #{index}"))?;
        let Some(relative) = file.enclosed_name() else {
            bail!("skill archive contains unsafe path entry");
        };

        let output = destination.join(relative);
        if file.is_dir() {
            std::fs::create_dir_all(&output).with_context(|| {
                format!("failed to create extracted directory: {}", output.display())
            })?;
            continue;
        }

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create parent directory for extracted file: {}",
                    parent.display()
                )
            })?;
        }

        let mut writer = std::fs::File::create(&output)
            .with_context(|| format!("failed to create extracted file: {}", output.display()))?;
        std::io::copy(&mut file, &mut writer)
            .with_context(|| format!("failed to write extracted file: {}", output.display()))?;
    }

    Ok(())
}

fn prompt_for_overwrite(path: &std::path::Path) -> Result<()> {
    print!("Overwrite existing skill at {}? [y/N]: ", path.display());
    io::stdout()
        .flush()
        .context("failed to flush overwrite confirmation prompt")?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read overwrite confirmation input")?;
    let answer = answer.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        bail!("skill install cancelled by user");
    }

    Ok(())
}

fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent(format!("{REPO_NAME}-skill-install"))
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

#[cfg(test)]
#[path = "../../tests/unit/commands/skill.rs"]
mod tests;
