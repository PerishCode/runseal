use std::{collections::BTreeMap, process::Command};

use anyhow::{Context, Result, bail};

use super::app::AppContext;
use super::config::RuntimeConfig;
use super::env_key::is_valid_env_key;
use super::{injections, profile};

pub struct RunResult {
    pub exit_code: Option<i32>,
}

pub fn run(app: &dyn AppContext) -> Result<RunResult> {
    let config = app.config();
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    let run_result = injections::with_registered_exports(app, profile.injections, |exports| {
        let env = to_env_map(exports.to_vec())?;
        let run_exports: Vec<(String, String)> = env.into_iter().collect();
        let code = run_command(config, &run_exports)?;
        Ok(RunResult {
            exit_code: Some(code),
        })
    })?;
    Ok(run_result)
}

fn to_env_map(exports: Vec<(String, String)>) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for (key, value) in exports {
        if !is_valid_env_key(&key) {
            bail!("invalid exported key: {}", key);
        }
        env.insert(key, value);
    }
    Ok(env)
}

fn run_command(config: &RuntimeConfig, exports: &[(String, String)]) -> Result<i32> {
    let command = &config.command;
    if command.is_empty() {
        bail!("command mode requires at least one command token");
    }

    let mut child = Command::new(&command[0]);
    if command.len() > 1 {
        child.args(&command[1..]);
    }
    child.envs(exports.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    child.env("RUNSEAL_HOME", &config.runseal_home);
    child.env("RUNSEAL_PROFILE_HOME", &config.profile_home);
    child.env("RUNSEAL_PROFILE_PATH", &config.profile_path);

    let status = child.status().context("failed to execute child command")?;
    if let Some(code) = status.code() {
        return Ok(code);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }

    Ok(1)
}
