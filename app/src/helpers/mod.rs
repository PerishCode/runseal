use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;

use crate::core::config::{RawEnv, resolve_runseal_home};
use crate::logging::current_log_file;

pub const HELPER_ALIAS_TEMPLATE_ENV: &str = "RUNSEAL_HELPER_ALIAS_TEMPLATE";

#[derive(Debug, Clone)]
pub struct HelperRunOptions {
    pub reference: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct HelperCommandError {
    exit_code: i32,
    reference: String,
}

impl HelperCommandError {
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

impl std::fmt::Display for HelperCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "helper `{}` failed with exit code {}",
            self.reference, self.exit_code
        )
    }
}

impl std::error::Error for HelperCommandError {}

pub fn helper_exit_code(error: &anyhow::Error) -> Option<i32> {
    error
        .downcast_ref::<HelperCommandError>()
        .map(HelperCommandError::exit_code)
}

pub fn run(options: HelperRunOptions) -> Result<()> {
    let raw_env = RawEnv::from_process();
    let runseal_home = resolve_runseal_home(&raw_env)?;
    let resolved = resolve_reference(&options.reference)?;
    let script_path = materialize_script(&resolved, &runseal_home)?;

    tracing::info!(
        helper_ref = %options.reference,
        script = %script_path.display(),
        "helper execution starting"
    );

    let mut command = Command::new("bash");
    command.arg(&script_path).args(&options.args);
    command.env("RUNSEAL_HOME", &runseal_home);
    command.env("RUNSEAL_HELPER_REF", &options.reference);
    if let Some(alias) = resolved.alias_name() {
        command.env("RUNSEAL_HELPER_ALIAS", alias);
    }
    if let Some(path) = current_log_file() {
        command.env("RUNSEAL_LOG_FILE", path);
    }

    let output = command
        .output()
        .with_context(|| format!("failed to execute helper script: {}", script_path.display()))?;

    io::stdout()
        .write_all(&output.stdout)
        .context("failed to write helper stdout")?;
    io::stderr()
        .write_all(&output.stderr)
        .context("failed to write helper stderr")?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        tracing::warn!(helper_ref = %options.reference, exit_code = code, "helper execution failed");
        return Err(anyhow!(HelperCommandError {
            exit_code: code,
            reference: options.reference,
        }));
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum ResolvedHelperReference {
    LocalPath(PathBuf),
    RemoteUrl {
        url: String,
        cache_path: Option<PathBuf>,
    },
}

impl ResolvedHelperReference {
    fn alias_name(&self) -> Option<&str> {
        match self {
            Self::RemoteUrl { cache_path, .. } => cache_path
                .as_ref()
                .and_then(|path| path.file_stem())
                .and_then(|name| name.to_str()),
            Self::LocalPath(_) => None,
        }
    }
}

fn resolve_reference(reference: &str) -> Result<ResolvedHelperReference> {
    if let Some(alias) = reference.strip_prefix(':') {
        validate_alias(alias)?;
        return resolve_alias(alias);
    }

    if is_http_url(reference) {
        return Ok(ResolvedHelperReference::RemoteUrl {
            url: reference.to_owned(),
            cache_path: None,
        });
    }

    Ok(ResolvedHelperReference::LocalPath(PathBuf::from(reference)))
}

fn resolve_alias(alias: &str) -> Result<ResolvedHelperReference> {
    let template = std::env::var(HELPER_ALIAS_TEMPLATE_ENV).with_context(|| {
        format!(
            "{} is required to resolve helper alias :{}",
            HELPER_ALIAS_TEMPLATE_ENV, alias
        )
    })?;
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let rendered = render_alias_template(&template, alias, &version);

    if is_http_url(&rendered) {
        return Ok(ResolvedHelperReference::RemoteUrl {
            url: rendered,
            cache_path: Some(
                PathBuf::from("helpers")
                    .join(&version)
                    .join(format!("{alias}.sh")),
            ),
        });
    }

    Ok(ResolvedHelperReference::LocalPath(PathBuf::from(rendered)))
}

fn materialize_script(reference: &ResolvedHelperReference, runseal_home: &Path) -> Result<PathBuf> {
    match reference {
        ResolvedHelperReference::LocalPath(path) => {
            if !path.is_file() {
                bail!("helper script not found: {}", path.display());
            }
            Ok(path.clone())
        }
        ResolvedHelperReference::RemoteUrl { url, cache_path } => {
            let bytes = download_bytes(url)?;
            let cache_relative = cache_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("helpers/adhoc/remote.sh"));
            let full = runseal_home.join(cache_relative);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "failed to create helper cache directory: {}",
                        parent.display()
                    )
                })?;
            }
            std::fs::write(&full, bytes).with_context(|| {
                format!("failed to write helper cache file: {}", full.display())
            })?;
            set_executable(&full)?;
            Ok(full)
        }
    }
}

fn download_bytes(url: &str) -> Result<Vec<u8>> {
    http_client()?
        .get(url)
        .send()
        .with_context(|| format!("failed to fetch helper script: {url}"))?
        .error_for_status()
        .with_context(|| format!("helper download request failed: {url}"))?
        .bytes()
        .context("failed to read helper script response body")
        .map(|bytes| bytes.to_vec())
}

fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent(format!("runseal-helper/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to build helper HTTP client")
}

fn render_alias_template(template: &str, name: &str, version: &str) -> String {
    template
        .replace("{name}", name)
        .replace("<name>", name)
        .replace("{version}", version)
        .replace("<version>", version)
}

fn validate_alias(alias: &str) -> Result<()> {
    if alias.is_empty() {
        bail!("helper alias cannot be empty")
    }
    if !alias
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        bail!(
            "helper alias must contain only ASCII letters, numbers, `-`, or `_`: {}",
            alias
        )
    }
    Ok(())
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://")
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("failed to read helper metadata: {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to set helper executable bit: {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}
