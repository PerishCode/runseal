use std::process::{Command, Output};

use anyhow::{Context, Result};

use super::config::RuntimeConfig;

pub trait EnvReader: Send + Sync {
    fn var(&self, key: &str) -> Option<String>;
}

pub trait CommandRunner: Send + Sync {
    fn output(&self, program: &str, args: &[String]) -> Result<Output>;

    fn output_with_env(
        &self,
        program: &str,
        args: &[String],
        env_overrides: &[(String, String)],
    ) -> Result<Output> {
        let _ = env_overrides;
        self.output(program, args)
    }
}

pub trait AppContext: Send + Sync {
    fn config(&self) -> &RuntimeConfig;
    fn env(&self) -> &dyn EnvReader;
    fn command_runner(&self) -> &dyn CommandRunner;
}

pub struct ProcessEnv;

impl EnvReader for ProcessEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn output(&self, program: &str, args: &[String]) -> Result<Output> {
        Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("failed to run command: {program}"))
    }

    fn output_with_env(
        &self,
        program: &str,
        args: &[String],
        env_overrides: &[(String, String)],
    ) -> Result<Output> {
        Command::new(program)
            .args(args)
            .envs(env_overrides.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .output()
            .with_context(|| format!("failed to run command: {program}"))
    }
}

pub struct App {
    config: RuntimeConfig,
    env: ProcessEnv,
    command_runner: ProcessCommandRunner,
}

impl App {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            env: ProcessEnv,
            command_runner: ProcessCommandRunner,
        }
    }
}

impl AppContext for App {
    fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    fn env(&self) -> &dyn EnvReader {
        &self.env
    }

    fn command_runner(&self) -> &dyn CommandRunner {
        &self.command_runner
    }
}
