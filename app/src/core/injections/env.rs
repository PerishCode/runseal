use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};

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
                    if let Some(sep) = separator {
                        if sep != "os" && sep.is_empty() {
                            bail!("separator must not be empty");
                        }
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
        let resource_home = &app.config().resource_home;
        let mut env: BTreeMap<String, String> = self
            .cfg
            .vars
            .iter()
            .map(|(k, v)| Ok((k.clone(), resolve_resource_refs(v, resource_home)?)))
            .collect::<Result<_>>()?;
        apply_ops(app, &mut env, &self.cfg.ops, resource_home)?;
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
    resource_home: &Path,
) -> Result<()> {
    for op in ops {
        match op {
            EnvOpProfile::Set { key, value } => {
                env.insert(key.clone(), resolve_resource_refs(value, resource_home)?);
            }
            EnvOpProfile::SetIfAbsent { key, value } => {
                if !env.contains_key(key) && app.env().var(key).is_none() {
                    env.insert(key.clone(), resolve_resource_refs(value, resource_home)?);
                }
            }
            EnvOpProfile::Prepend {
                key,
                value,
                separator,
                dedup,
            } => {
                let merged =
                    merge_env_op(app, env, key, value, separator, *dedup, true, resource_home)?;
                env.insert(key.clone(), merged);
            }
            EnvOpProfile::Append {
                key,
                value,
                separator,
                dedup,
            } => {
                let merged = merge_env_op(
                    app,
                    env,
                    key,
                    value,
                    separator,
                    *dedup,
                    false,
                    resource_home,
                )?;
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
    resource_home: &Path,
) -> Result<String> {
    let sep = separator_value(separator);
    let base = env
        .get(key)
        .cloned()
        .or_else(|| app.env().var(key))
        .unwrap_or_default();
    let resolved = resolve_resource_refs(value, resource_home)?;
    let merged = if prepend {
        merge_values(&resolved, &base, sep, dedup)
    } else {
        merge_values(&base, &resolved, sep, dedup)
    };
    Ok(merged)
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

const RESOURCE_URI_PREFIX: &str = "resource://";
const RESOURCE_CONTENT_URI_PREFIX: &str = "resource-content://";

fn resolve_resource_refs(value: &str, resource_home: &Path) -> Result<String> {
    let mut out = String::new();
    let mut rest = value;

    while let Some((idx, prefix)) = find_next_resource_prefix(rest) {
        out.push_str(&rest[..idx]);
        let token_start = idx + prefix.len();
        let after = &rest[token_start..];
        let token_end = after
            .char_indices()
            .find(|(_, c)| is_resource_token_delimiter(*c))
            .map(|(i, _)| i)
            .unwrap_or(after.len());
        let rel = &after[..token_end];
        if rel.is_empty() {
            out.push_str(prefix);
        } else {
            let abs = resource_home.join(rel);
            if prefix == RESOURCE_CONTENT_URI_PREFIX {
                let content = std::fs::read_to_string(&abs).with_context(|| {
                    format!("failed to read resource content: {}", abs.display())
                })?;
                out.push_str(&content);
            } else {
                out.push_str(&abs.to_string_lossy());
            }
        }
        rest = &after[token_end..];
    }
    out.push_str(rest);
    Ok(out)
}

fn find_next_resource_prefix(input: &str) -> Option<(usize, &'static str)> {
    let file_idx = input.find(RESOURCE_URI_PREFIX);
    let content_idx = input.find(RESOURCE_CONTENT_URI_PREFIX);

    match (file_idx, content_idx) {
        (Some(a), Some(b)) => {
            if a <= b {
                Some((a, RESOURCE_URI_PREFIX))
            } else {
                Some((b, RESOURCE_CONTENT_URI_PREFIX))
            }
        }
        (Some(a), None) => Some((a, RESOURCE_URI_PREFIX)),
        (None, Some(b)) => Some((b, RESOURCE_CONTENT_URI_PREFIX)),
        (None, None) => None,
    }
}

fn is_resource_token_delimiter(c: char) -> bool {
    matches!(
        c,
        ':' | ';' | ',' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ' ' | '\t' | '\r' | '\n'
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/core/injections/env.rs"]
mod tests;
