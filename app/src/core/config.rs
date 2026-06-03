use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct CliInput {
    pub profile: Option<PathBuf>,
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawEnv {
    pub home: Option<PathBuf>,
    pub runseal_home: Option<PathBuf>,
    pub runseal_profile_home: Option<PathBuf>,
}

impl RawEnv {
    pub fn from_process() -> Self {
        Self {
            home: std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| non_empty_path(path)),
            runseal_home: std::env::var_os("RUNSEAL_HOME")
                .map(PathBuf::from)
                .filter(|path| non_empty_path(path)),
            runseal_profile_home: std::env::var_os("RUNSEAL_PROFILE_HOME")
                .map(PathBuf::from)
                .filter(|path| non_empty_path(path)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub profile_path: PathBuf,
    pub command: Vec<String>,
    pub runseal_home: PathBuf,
    pub profile_home: PathBuf,
}

impl RuntimeConfig {
    pub fn from_input(cli: CliInput, env: RawEnv, cwd: &Path) -> Result<Self> {
        let runseal_home = resolve_runseal_home(&env)?;
        let profile_home = env
            .runseal_profile_home
            .filter(|path| non_empty_path(path))
            .unwrap_or_else(|| runseal_home.join("profiles"));
        let profile_path = resolve_profile_path(cli.profile, cwd, &profile_home)?;

        Ok(Self {
            profile_path,
            command: cli.command,
            runseal_home,
            profile_home,
        })
    }
}

pub fn resolve_runseal_home(env: &RawEnv) -> Result<PathBuf> {
    env.runseal_home
        .clone()
        .filter(|path| non_empty_path(path))
        .or_else(|| {
            env.home
                .clone()
                .filter(|path| non_empty_path(path))
                .map(|home| home.join(".runseal"))
        })
        .ok_or_else(|| anyhow::anyhow!("HOME is not set; pass --profile or set RUNSEAL_HOME"))
}

fn non_empty_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
}

fn resolve_profile_path(
    explicit: Option<PathBuf>,
    cwd: &Path,
    profile_home: &Path,
) -> Result<PathBuf> {
    if let Some(profile) = explicit {
        let profile = if profile.is_absolute() {
            profile
        } else {
            cwd.join(profile)
        };
        if !profile.is_file() {
            bail!("profile file not found: {}", profile.display());
        }
        return Ok(profile);
    }

    let mut searched = Vec::new();
    for candidate in discovery_candidates(cwd, profile_home) {
        if candidate.is_file() {
            return Ok(candidate);
        }
        searched.push(candidate);
    }

    let searched = searched
        .iter()
        .map(|path| format!("- {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");
    bail!("profile file not found. searched:\n{searched}")
}

fn discovery_candidates(cwd: &Path, profile_home: &Path) -> Vec<PathBuf> {
    profile_extensions()
        .iter()
        .map(|ext| cwd.join(format!("runseal.{ext}")))
        .chain(
            profile_extensions()
                .iter()
                .map(|ext| profile_home.join(format!("default.{ext}"))),
        )
        .collect()
}

pub fn profile_extensions() -> &'static [&'static str] {
    &["toml", "yaml", "yml", "json"]
}
