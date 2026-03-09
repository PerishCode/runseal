use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::core::config::{resolve_envlock_home, RawEnv};
use crate::logging::current_log_file;

use super::{builtin_plugin_script, patch::validate_patch_json};

#[derive(Debug, Clone)]
pub struct PluginHostOptions {
    pub plugin: String,
    pub method: String,
    pub args: Vec<String>,
    pub force_install: bool,
}

#[derive(Debug)]
pub struct PluginCommandError {
    exit_code: i32,
    plugin: String,
    method: String,
    stderr: String,
}

impl PluginCommandError {
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

impl std::fmt::Display for PluginCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "plugin `{}` method `{}` failed with exit code {}: {}",
            self.plugin, self.method, self.exit_code, self.stderr
        )
    }
}

impl std::error::Error for PluginCommandError {}

pub fn plugin_exit_code(error: &anyhow::Error) -> Option<i32> {
    error
        .downcast_ref::<PluginCommandError>()
        .map(PluginCommandError::exit_code)
}

pub fn run_plugin(options: PluginHostOptions) -> Result<()> {
    validate_name("plugin", &options.plugin)?;
    validate_name("plugin method", &options.method)?;
    tracing::info!(plugin = %options.plugin, method = %options.method, "plugin invocation starting");

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

    let mut command = Command::new("bash");
    command
        .arg(&script_path)
        .arg(&options.method)
        .args(&options.args)
        .env("ENVLOCK_HOME", &envlock_home)
        .env("ENVLOCK_PLUGIN_NAME", &options.plugin)
        .env("ENVLOCK_PLUGIN_METHOD", &options.method);
    if let Some(path) = current_log_file() {
        command.env("ENVLOCK_LOG_FILE", path);
    }
    tracing::info!(plugin = %options.plugin, method = %options.method, script = %script_path.display(), "plugin command prepared");
    let output = command
        .output()
        .with_context(|| format!("failed to execute plugin script: {}", script_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(1);
        tracing::warn!(plugin = %options.plugin, method = %options.method, exit_code = code, "plugin invocation failed");
        return Err(anyhow!(PluginCommandError {
            exit_code: code,
            plugin: options.plugin,
            method: options.method,
            stderr: stderr.trim().to_owned(),
        }));
    }

    let stdout = String::from_utf8(output.stdout).context("plugin output is not valid UTF-8")?;
    validate_patch_json(&stdout)?;
    tracing::info!(plugin = %options.plugin, method = %options.method, "plugin patch validated");
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
