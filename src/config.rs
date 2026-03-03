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
    pub envlock_home: Option<PathBuf>,
    pub envlock_resource_home: Option<PathBuf>,
}

impl RawEnv {
    pub fn from_process() -> Self {
        Self {
            home: std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(non_empty_path),
            envlock_home: std::env::var_os("ENVLOCK_HOME")
                .map(PathBuf::from)
                .filter(non_empty_path),
            envlock_resource_home: std::env::var_os("ENVLOCK_RESOURCE_HOME")
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
    pub envlock_home: PathBuf,
    pub resource_home: PathBuf,
}

impl RuntimeConfig {
    pub fn from_cli_and_env(cli: CliInput, env: RawEnv) -> Result<Self> {
        let envlock_home = env
            .envlock_home
            .filter(non_empty_path)
            .or_else(|| {
                env.home
                    .filter(non_empty_path)
                    .map(|home| home.join(".envlock"))
            })
            .ok_or_else(|| {
                anyhow::anyhow!("HOME is not set; pass --profile or set ENVLOCK_HOME")
            })?;
        let resource_home = env
            .envlock_resource_home
            .filter(non_empty_path)
            .unwrap_or_else(|| envlock_home.join("resources"));

        let profile_path = if let Some(profile) = cli.profile {
            profile
        } else {
            envlock_home.join("profiles/default.json")
        };

        if !profile_path.is_file() {
            bail!(
                "profile file not found: {}. create default profile at {}/profiles/default.json or pass --profile",
                profile_path.display(),
                envlock_home.display()
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
            envlock_home,
            resource_home,
        })
    }
}

fn non_empty_path(path: &PathBuf) -> bool {
    !path.as_os_str().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn base_cli() -> CliInput {
        CliInput {
            profile: None,
            output_mode: OutputMode::Shell,
            strict: false,
            log_level: LevelFilter::WARN,
            log_format: LogFormat::Text,
            command: Vec::new(),
        }
    }

    #[test]
    fn default_profile_uses_envlock_home() {
        let temp = TempDir::new().expect("temp dir should be created");
        let envlock_home = temp.path().join("envlock-home");
        let profiles = envlock_home.join("profiles");
        std::fs::create_dir_all(&profiles).expect("profiles dir should be created");
        std::fs::write(profiles.join("default.json"), "{\"injections\":[]}")
            .expect("default profile should be written");

        let cfg = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                envlock_home: Some(envlock_home.clone()),
                envlock_resource_home: None,
            },
        )
        .expect("config should build");

        assert_eq!(cfg.envlock_home, envlock_home);
        assert_eq!(
            cfg.profile_path,
            temp.path().join("envlock-home/profiles/default.json")
        );
    }

    #[test]
    fn profile_flag_overrides_default_resolution() {
        let temp = TempDir::new().expect("temp dir should be created");
        let explicit = temp.path().join("explicit.json");
        std::fs::write(&explicit, "{\"injections\":[]}").expect("profile should be written");

        let mut cli = base_cli();
        cli.profile = Some(explicit.clone());

        let cfg = RuntimeConfig::from_cli_and_env(
            cli,
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                envlock_home: None,
                envlock_resource_home: None,
            },
        )
        .expect("config should build");

        assert_eq!(cfg.profile_path, explicit);
    }

    #[test]
    fn resource_home_defaults_from_envlock_home() {
        let temp = TempDir::new().expect("temp dir should be created");
        let envlock_home = temp.path().join("envlock-home");
        std::fs::create_dir_all(envlock_home.join("profiles")).expect("profiles dir should exist");
        std::fs::write(
            envlock_home.join("profiles/default.json"),
            "{\"injections\":[]}",
        )
        .expect("default profile should be written");

        let cfg = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                envlock_home: Some(envlock_home.clone()),
                envlock_resource_home: None,
            },
        )
        .expect("config should build");

        assert_eq!(cfg.resource_home, envlock_home.join("resources"));
    }

    #[test]
    fn missing_default_profile_returns_actionable_error() {
        let err = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                envlock_home: Some(PathBuf::from("/tmp/does-not-exist")),
                envlock_resource_home: None,
            },
        )
        .expect_err("missing default profile should fail");
        assert!(err.to_string().contains("profiles/default.json"));
    }

    #[test]
    fn missing_home_and_envlock_home_fails() {
        let err = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: None,
                envlock_home: None,
                envlock_resource_home: None,
            },
        )
        .expect_err("missing home should fail");
        assert!(err.to_string().contains("HOME is not set"));
    }

    #[test]
    fn empty_envlock_home_is_treated_as_unset() {
        let temp = TempDir::new().expect("temp dir should be created");
        let home = temp.path().join("home");
        std::fs::create_dir_all(home.join(".envlock/profiles")).expect("profiles dir should exist");
        std::fs::write(
            home.join(".envlock/profiles/default.json"),
            "{\"injections\":[]}",
        )
        .expect("default profile should be written");

        let cfg = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(home.clone()),
                envlock_home: Some(PathBuf::new()),
                envlock_resource_home: None,
            },
        )
        .expect("config should fall back to HOME/.envlock");

        assert_eq!(cfg.envlock_home, home.join(".envlock"));
    }
}
