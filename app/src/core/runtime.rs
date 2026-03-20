use std::{collections::BTreeMap, process::Command};

use anyhow::{Context, Result, bail};
use tracing::{debug, info};

use super::app::AppContext;
use super::config::OutputMode;
use super::env_key::is_valid_env_key;
use super::{injections, profile};

pub struct RunResult {
    pub exit_code: Option<i32>,
}

pub fn run(app: &dyn AppContext) -> Result<RunResult> {
    let config = app.config();
    info!(
        profile_path = %config.profile_path.display(),
        output_mode = match config.output_mode {
            OutputMode::Shell => "shell",
            OutputMode::Json => "json",
        },
        strict = config.strict,
        has_command = config.command.is_some(),
        "runseal run started"
    );
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    let run_result = injections::with_registered_exports(app, profile.injections, |exports| {
        info!(
            export_count = exports.len(),
            "injections lifecycle completed"
        );
        let env = to_env_map(exports.to_vec(), config.strict)?;
        if let Some(command) = &config.command {
            let run_exports: Vec<(String, String)> = env.into_iter().collect();
            let code = run_command(command, &run_exports)?;
            return Ok(RunResult {
                exit_code: Some(code),
            });
        }
        print_outputs(env, config.output_mode)?;
        Ok(RunResult { exit_code: None })
    })?;
    info!("runseal run completed");
    Ok(run_result)
}

fn print_outputs(env: BTreeMap<String, String>, mode: OutputMode) -> Result<()> {
    debug!(
        output_mode = match mode {
            OutputMode::Json => "json",
            OutputMode::Shell => "shell",
        },
        "rendering output"
    );
    match mode {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&env)?),
        OutputMode::Shell => {
            for (key, value) in env {
                println!("export {}='{}'", key, shell_single_quote_escape(&value));
            }
        }
    }
    Ok(())
}

fn to_env_map(exports: Vec<(String, String)>, strict: bool) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for (key, value) in exports {
        if !is_valid_env_key(&key) {
            bail!("invalid exported key: {}", key);
        }
        if strict && env.contains_key(&key) {
            bail!("duplicate exported key detected in strict mode: {}", key);
        }
        env.insert(key, value);
    }
    Ok(env)
}

fn shell_single_quote_escape(input: &str) -> String {
    input.replace('\'', "'\"'\"'")
}

fn run_command(command: &[String], exports: &[(String, String)]) -> Result<i32> {
    if command.is_empty() {
        bail!("command mode requires at least one command token");
    }

    let mut child = Command::new(&command[0]);
    if command.len() > 1 {
        child.args(&command[1..]);
    }
    child.envs(exports.iter().map(|(k, v)| (k.as_str(), v.as_str())));

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

#[cfg(test)]
#[path = "../../tests/unit/core/runtime.rs"]
mod tests;
