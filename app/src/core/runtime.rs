use std::{collections::BTreeMap, process::Command};

use anyhow::{Context, Result, bail};

use super::app::AppContext;
use super::config::RuntimeConfig;
use super::env_key::is_valid_env_key;
use super::profile::InjectionProfile;
use super::{injections, profile};

pub struct RunResult {
    pub exit_code: Option<i32>,
}

pub fn run(app: &dyn AppContext) -> Result<RunResult> {
    let config = app.config();
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    let command = apply_argv_injections(&config.command, &profile.injections)?;
    let run_result = injections::with_registered_exports(app, profile.injections, |exports| {
        let env = to_env_map(exports.to_vec())?;
        let run_exports: Vec<(String, String)> = env.into_iter().collect();
        let code = run_command(config, &command, &run_exports)?;
        Ok(RunResult {
            exit_code: Some(code),
        })
    })?;
    Ok(run_result)
}

fn apply_argv_injections(
    command: &[String],
    injections: &[InjectionProfile],
) -> Result<Vec<String>> {
    if command.is_empty() {
        bail!("command mode requires at least one command token");
    }

    let mut prefix_args = Vec::new();
    for injection in injections {
        let InjectionProfile::Argv(spec) = injection else {
            continue;
        };
        if !spec.enabled {
            continue;
        }
        if spec.command.trim().is_empty() {
            bail!("argv command must not be empty");
        }
        if spec.args.is_empty() {
            bail!("argv args must not be empty");
        }
        if spec.command == command[0] {
            prefix_args.extend(spec.args.clone());
        }
    }
    if prefix_args.is_empty() {
        return Ok(command.to_vec());
    }

    let mut rewritten = Vec::with_capacity(command.len() + prefix_args.len());
    rewritten.push(command[0].clone());
    rewritten.extend(prefix_args);
    rewritten.extend_from_slice(&command[1..]);
    Ok(rewritten)
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

fn run_command(
    config: &RuntimeConfig,
    command: &[String],
    exports: &[(String, String)],
) -> Result<i32> {
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
