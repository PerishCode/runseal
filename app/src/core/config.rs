use std::path::PathBuf;

use anyhow::{Result, bail};
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Shell,
    Json,
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct CliInput {
    pub profile: Option<PathBuf>,
    pub output_mode: OutputMode,
    pub strict: bool,
    pub log_level: LevelFilter,
    pub log_format: LogFormat,
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawEnv {
    pub home: Option<PathBuf>,
    pub runseal_home: Option<PathBuf>,
    pub runseal_resource_home: Option<PathBuf>,
}

impl RawEnv {
    pub fn from_process() -> Self {
        Self {
            home: std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(non_empty_path),
            runseal_home: std::env::var_os("RUNSEAL_HOME")
                .map(PathBuf::from)
                .filter(non_empty_path),
            runseal_resource_home: std::env::var_os("RUNSEAL_RESOURCE_HOME")
                .map(PathBuf::from)
                .filter(non_empty_path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub profile_path: PathBuf,
    pub output_mode: OutputMode,
    pub strict: bool,
    pub log_level: LevelFilter,
    pub log_format: LogFormat,
    pub command: Option<Vec<String>>,
    pub runseal_home: PathBuf,
    pub resource_home: PathBuf,
}

impl RuntimeConfig {
    pub fn from_cli_and_env(cli: CliInput, env: RawEnv) -> Result<Self> {
        let runseal_home = resolve_runseal_home(&env)?;
        let resource_home = env
            .runseal_resource_home
            .filter(non_empty_path)
            .unwrap_or_else(|| runseal_home.join("resources"));

        let profile_path = if let Some(profile) = cli.profile {
            profile
        } else {
            runseal_home.join("profiles/default.json")
        };

        if !profile_path.is_file() {
            bail!(
                "profile file not found: {}. create default profile at {}/profiles/default.json or pass --profile",
                profile_path.display(),
                runseal_home.display()
            );
        }

        Ok(Self {
            profile_path,
            output_mode: cli.output_mode,
            strict: cli.strict,
            log_level: cli.log_level,
            log_format: cli.log_format,
            command: if cli.command.is_empty() {
                None
            } else {
                Some(cli.command)
            },
            runseal_home,
            resource_home,
        })
    }
}

pub fn resolve_runseal_home(env: &RawEnv) -> Result<PathBuf> {
    env.runseal_home
        .clone()
        .filter(non_empty_path)
        .or_else(|| {
            env.home
                .clone()
                .filter(non_empty_path)
                .map(|home| home.join(".runseal"))
        })
        .ok_or_else(|| anyhow::anyhow!("HOME is not set; pass --profile or set RUNSEAL_HOME"))
}

fn non_empty_path(path: &PathBuf) -> bool {
    !path.as_os_str().is_empty()
}

#[cfg(test)]
#[path = "../../tests/unit/core/config.rs"]
mod tests;
