use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::core::config::{RawEnv, resolve_envlock_home};

use super::{builtin_plugin_script, patch::validate_patch_json};

#[derive(Debug, Clone)]
pub struct PluginHostOptions {
    pub plugin: String,
    pub method: String,
    pub args: Vec<String>,
    pub force_install: bool,
}

pub fn run_plugin(options: PluginHostOptions) -> Result<()> {
    validate_name("plugin", &options.plugin)?;
    validate_name("plugin method", &options.method)?;

    let envlock_home = resolve_envlock_home(&RawEnv::from_process())?;
    let script_path = plugin_script_path(&envlock_home, &options.plugin);

    if options.method == "init" {
        if let Some(script) = builtin_plugin_script(&options.plugin) {
            install_plugin_script(&script_path, script, options.force_install)?;
        }
    }

    if !script_path.is_file() {
        bail!(
            "plugin script not found: {} (run `envlock plugin {} init` first)",
            script_path.display(),
            options.plugin
        );
    }

    let output = Command::new("bash")
        .arg(&script_path)
        .arg(&options.method)
        .args(&options.args)
        .env("ENVLOCK_HOME", &envlock_home)
        .env("ENVLOCK_PLUGIN_NAME", &options.plugin)
        .env("ENVLOCK_PLUGIN_METHOD", &options.method)
        .output()
        .with_context(|| format!("failed to execute plugin script: {}", script_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(1);
        bail!(
            "plugin `{}` method `{}` failed with exit code {}: {}",
            options.plugin,
            options.method,
            code,
            stderr.trim()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("plugin output is not valid UTF-8")?;
    validate_patch_json(&stdout)?;
    println!("{}", stdout.trim_end());
    Ok(())
}

fn install_plugin_script(path: &Path, contents: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create plugin script directory: {}",
                parent.display()
            )
        })?;
    }

    std::fs::write(path, contents)
        .with_context(|| format!("failed to write plugin script file: {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .with_context(|| format!("failed to read metadata: {}", path.display()))?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)
            .with_context(|| format!("failed to set executable permission: {}", path.display()))?;
    }

    Ok(())
}

fn plugin_script_path(envlock_home: &Path, plugin: &str) -> PathBuf {
    envlock_home.join("plugins").join(format!("{plugin}.sh"))
}

fn validate_name(label: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{} cannot be empty", label);
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        bail!(
            "{} must contain only ASCII letters, numbers, `-`, or `_`: {}",
            label,
            value
        );
    }
    Ok(())
}
