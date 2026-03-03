pub mod app;
pub mod config;
pub mod injections;
pub mod preview;
pub mod profile;
pub mod self_update;

use std::{collections::BTreeMap, process::Command};

use anyhow::{Context, Result, bail};
use tracing::{debug, info};

use crate::app::AppContext;
use crate::config::OutputMode;

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
        "envlock run started"
    );
    let profile = profile::load(&config.profile_path).context("unable to load envlock profile")?;
    let run_result = injections::with_registered_exports(app, profile.injections, |exports| {
        info!(
            export_count = exports.len(),
            "injections lifecycle completed"
        );
        if let Some(command) = &config.command {
            let code = run_command(command, exports)?;
            return Ok(RunResult {
                exit_code: Some(code),
            });
        }
        print_outputs(
            exports.to_vec(),
            matches!(config.output_mode, OutputMode::Json),
            config.strict,
        )?;
        Ok(RunResult { exit_code: None })
    })?;
    info!("envlock run completed");
    Ok(run_result)
}

fn print_outputs(exports: Vec<(String, String)>, as_json: bool, strict: bool) -> Result<()> {
    let env = to_env_map(exports, strict)?;
    debug!(
        output_mode = if as_json { "json" } else { "shell" },
        "rendering output"
    );
    if as_json {
        println!("{}", serde_json::to_string_pretty(&env)?);
    } else {
        for (key, value) in env {
            println!("export {}='{}'", key, shell_single_quote_escape(&value));
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

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
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
mod tests {
    use super::*;

    #[test]
    fn escape_single_quotes_for_shell() {
        assert_eq!(shell_single_quote_escape("a'b"), "a'\"'\"'b");
    }

    #[test]
    fn env_map_keeps_last_value_for_duplicate_keys() {
        let map = to_env_map(
            vec![
                ("A".to_string(), "1".to_string()),
                ("B".to_string(), "2".to_string()),
                ("A".to_string(), "3".to_string()),
            ],
            false,
        )
        .expect("non-strict mode should allow duplicate keys");
        assert_eq!(map.get("A"), Some(&"3".to_string()));
        assert_eq!(map.get("B"), Some(&"2".to_string()));
    }

    #[test]
    fn env_map_rejects_duplicate_keys_in_strict_mode() {
        let err = to_env_map(
            vec![
                ("A".to_string(), "1".to_string()),
                ("A".to_string(), "2".to_string()),
            ],
            true,
        )
        .expect_err("strict mode should reject duplicate keys");
        assert!(err.to_string().contains("duplicate exported key"));
    }

    #[test]
    fn env_map_rejects_invalid_key() {
        let err = to_env_map(vec![("BAD-KEY".to_string(), "1".to_string())], false)
            .expect_err("invalid env key should fail");
        assert!(err.to_string().contains("invalid exported key"));
    }
}
