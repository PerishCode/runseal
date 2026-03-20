use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;

use crate::core::app::{AppContext, EnvReader};
use crate::core::env_key::is_valid_env_key;
use crate::core::profile::CommandProfile;

pub(crate) struct CommandInjection {
    cfg: CommandProfile,
}

impl CommandInjection {
    pub(crate) fn new(cfg: CommandProfile) -> Self {
        Self { cfg }
    }

    pub(crate) fn name(&self) -> &'static str {
        "command"
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.cfg.program.trim().is_empty() {
            bail!("program must not be empty");
        }
        Ok(())
    }

    pub(crate) fn register(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn export(
        &self,
        app: &dyn AppContext,
        inherited: &BTreeMap<String, String>,
    ) -> Result<Vec<(String, String)>> {
        let inherited_pairs: Vec<(String, String)> = inherited
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let output = app
            .command_runner()
            .output_with_env(&self.cfg.program, &self.cfg.args, &inherited_pairs)
            .with_context(|| format!("failed to run command: {}", self.cfg.program))?;

        if !output.status.success() {
            bail!(
                "command exited with non-zero status: {}",
                output
                    .status
                    .code()
                    .map_or_else(|| "unknown".to_string(), |code| code.to_string())
            );
        }

        let stdout =
            String::from_utf8(output.stdout).context("command stdout is not valid UTF-8")?;
        Ok(parse_exports(
            &stdout,
            &OverlayEnv::new(app.env(), inherited),
        ))
    }

    pub(crate) fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

fn parse_exports(stdout: &str, env: &dyn EnvReader) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let assignment = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        let Some((key_raw, value_raw)) = assignment.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        if !is_valid_env_key(key) {
            continue;
        }
        let value = normalize_value(value_raw.trim(), env);
        out.push((key.to_string(), value));
    }
    out
}

fn normalize_value(raw: &str, env: &dyn EnvReader) -> String {
    let unquoted = strip_quote_delimiters(raw);
    expand_vars(&unquoted, env)
}

fn strip_quote_delimiters(raw: &str) -> String {
    let mut out = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for ch in raw.chars() {
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        out.push(ch);
    }
    out
}

struct OverlayEnv<'a> {
    base: &'a dyn EnvReader,
    overlay: BTreeMap<String, String>,
}

impl<'a> OverlayEnv<'a> {
    fn new(base: &'a dyn EnvReader, inherited: &BTreeMap<String, String>) -> Self {
        Self {
            base,
            overlay: inherited.clone(),
        }
    }
}

impl EnvReader for OverlayEnv<'_> {
    fn var(&self, key: &str) -> Option<String> {
        self.overlay
            .get(key)
            .cloned()
            .or_else(|| self.base.var(key))
    }
}

fn expand_vars(input: &str, env: &dyn EnvReader) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if i + 1 < chars.len() && chars[i + 1] == '{' {
            let mut j = i + 2;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j < chars.len() {
                let key: String = chars[i + 2..j].iter().collect();
                if !key.is_empty() {
                    out.push_str(&env.var(&key).unwrap_or_default());
                }
                i = j + 1;
                continue;
            }
        }

        let mut j = i + 1;
        while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
            j += 1;
        }
        if j > i + 1 {
            let key: String = chars[i + 1..j].iter().collect();
            out.push_str(&env.var(&key).unwrap_or_default());
            i = j;
            continue;
        }

        out.push('$');
        i += 1;
    }
    out
}

#[cfg(test)]
#[path = "../../../tests/unit/core/injections/command.rs"]
mod tests;
