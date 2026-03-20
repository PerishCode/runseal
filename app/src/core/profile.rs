use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use path_absolutize::Absolutize;
use serde::Deserialize;

fn default_enabled() -> bool {
    true
}

fn default_cleanup() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub injections: Vec<InjectionProfile>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InjectionProfile {
    Env(EnvProfile),
    Command(CommandProfile),
    Symlink(SymlinkProfile),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvProfile {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    #[serde(default)]
    pub ops: Vec<EnvOpProfile>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandProfile {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EnvOpProfile {
    Set {
        key: String,
        value: String,
    },
    SetIfAbsent {
        key: String,
        value: String,
    },
    Prepend {
        key: String,
        value: String,
        #[serde(default)]
        separator: Option<String>,
        #[serde(default)]
        dedup: bool,
    },
    Append {
        key: String,
        value: String,
        #[serde(default)]
        separator: Option<String>,
        #[serde(default)]
        dedup: bool,
    },
    Unset {
        key: String,
    },
}

impl EnvOpProfile {
    pub fn key(&self) -> &str {
        match self {
            Self::Set { key, .. }
            | Self::SetIfAbsent { key, .. }
            | Self::Prepend { key, .. }
            | Self::Append { key, .. }
            | Self::Unset { key } => key,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SymlinkProfile {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub source: PathBuf,
    pub target: PathBuf,
    #[serde(default)]
    pub on_exist: SymlinkOnExist,
    #[serde(default = "default_cleanup")]
    pub cleanup: bool,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum SymlinkOnExist {
    #[default]
    Error,
    Replace,
}

pub fn load(path: &Path) -> Result<Profile> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read profile file: {}", path.display()))?;
    let mut profile: Profile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse JSON: {}", path.display()))?;
    normalize_symlink_paths(path, &mut profile)?;
    Ok(profile)
}

fn normalize_symlink_paths(profile_path: &Path, profile: &mut Profile) -> Result<()> {
    let base_dir = profile_path.parent().unwrap_or(Path::new("."));
    for injection in &mut profile.injections {
        if let InjectionProfile::Symlink(spec) = injection {
            spec.source = normalize_path(&spec.source, base_dir)?;
            spec.target = normalize_path(&spec.target, base_dir)?;
        }
    }
    Ok(())
}

fn normalize_path(path: &Path, base_dir: &Path) -> Result<PathBuf> {
    let raw = path.to_string_lossy();
    let expanded = shellexpand::tilde(&raw);
    let expanded_path = PathBuf::from(expanded.as_ref());
    if expanded_path.is_absolute() {
        return Ok(expanded_path);
    }
    Ok(expanded_path.absolutize_from(base_dir)?.to_path_buf())
}

#[cfg(test)]
#[path = "../../tests/unit/core/profile.rs"]
mod tests;
