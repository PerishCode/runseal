use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::core::app::AppContext;
use crate::core::profile::{EnvOpProfile, EnvProfile};

pub(crate) struct EnvInjection {
    cfg: EnvProfile,
}

impl EnvInjection {
    pub(crate) fn new(cfg: EnvProfile) -> Self {
        Self { cfg }
    }

    pub(crate) fn name(&self) -> &'static str {
        "env"
    }

    pub(crate) fn validate(&self) -> Result<()> {
        for key in self.cfg.vars.keys() {
            if key.trim().is_empty() {
                bail!("env var key must not be empty");
            }
        }
        for op in &self.cfg.ops {
            match op {
                EnvOpProfile::Set { key, value } | EnvOpProfile::SetIfAbsent { key, value } => {
                    validate_key_value(key, value)?
                }
                EnvOpProfile::Prepend {
                    key,
                    value,
                    separator,
                    ..
                }
                | EnvOpProfile::Append {
                    key,
                    value,
                    separator,
                    ..
                } => {
                    validate_key_value(key, value)?;
                    if let Some(sep) = separator
                        && sep != "os"
                        && sep.is_empty()
                    {
                        bail!("separator must not be empty");
                    }
                }
                EnvOpProfile::Unset { key } => {
                    if key.trim().is_empty() {
                        bail!("env var key must not be empty");
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn register(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn export(&self, app: &dyn AppContext) -> Result<Vec<(String, String)>> {
        let mut env: BTreeMap<String, String> = self
            .cfg
            .vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        apply_ops(app, &mut env, &self.cfg.ops)?;
        Ok(env.into_iter().collect())
    }

    pub(crate) fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

fn validate_key_value(key: &str, value: &str) -> Result<()> {
    if key.trim().is_empty() {
        bail!("env var key must not be empty");
    }
    if value.trim().is_empty() {
        bail!("env var value must not be empty");
    }
    Ok(())
}

fn apply_ops(
    app: &dyn AppContext,
    env: &mut BTreeMap<String, String>,
    ops: &[EnvOpProfile],
) -> Result<()> {
    for op in ops {
        match op {
            EnvOpProfile::Set { key, value } => {
                env.insert(key.clone(), value.clone());
            }
            EnvOpProfile::SetIfAbsent { key, value } => {
                if !env.contains_key(key) && app.env().var(key).is_none() {
                    env.insert(key.clone(), value.clone());
                }
            }
            EnvOpProfile::Prepend {
                key,
                value,
                separator,
                dedup,
            } => {
                let merged = merge_env_op(app, env, key, value, separator, *dedup, true);
                env.insert(key.clone(), merged);
            }
            EnvOpProfile::Append {
                key,
                value,
                separator,
                dedup,
            } => {
                let merged = merge_env_op(app, env, key, value, separator, *dedup, false);
                env.insert(key.clone(), merged);
            }
            EnvOpProfile::Unset { key } => {
                env.remove(key);
            }
        }
    }
    Ok(())
}

fn merge_env_op(
    app: &dyn AppContext,
    env: &BTreeMap<String, String>,
    key: &str,
    value: &str,
    separator: &Option<String>,
    dedup: bool,
    prepend: bool,
) -> String {
    let sep = separator_value(separator);
    let base = env
        .get(key)
        .cloned()
        .or_else(|| app.env().var(key))
        .unwrap_or_default();
    if prepend {
        merge_values(value, &base, sep, dedup)
    } else {
        merge_values(&base, value, sep, dedup)
    }
}

fn separator_value(separator: &Option<String>) -> &str {
    match separator.as_deref() {
        None | Some("os") => {
            if cfg!(windows) {
                ";"
            } else {
                ":"
            }
        }
        Some(custom) => custom,
    }
}

fn merge_values(left: &str, right: &str, separator: &str, dedup: bool) -> String {
    let mut out = Vec::new();
    let left_parts = split_parts(left, separator);
    let right_parts = split_parts(right, separator);

    out.extend(left_parts);
    out.extend(right_parts);

    if dedup {
        let mut deduped = Vec::new();
        for entry in out {
            if !deduped.contains(&entry) {
                deduped.push(entry);
            }
        }
        return deduped.join(separator);
    }
    out.join(separator)
}

fn split_parts(value: &str, separator: &str) -> Vec<String> {
    value
        .split(separator)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}
