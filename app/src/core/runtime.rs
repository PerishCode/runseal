use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use super::app::AppContext;
use super::config::RuntimeConfig;
use super::env_key::is_valid_env_key;
use super::profile::InjectionProfile;
use super::{injections, profile};

pub struct RunResult {
    pub exit_code: Option<i32>,
}

struct ResolvedCommand {
    argv: Vec<String>,
    wrapper: Option<ResolvedWrapper>,
}

struct ResolvedWrapper {
    name: String,
    file: PathBuf,
}

pub fn run(app: &dyn AppContext) -> Result<RunResult> {
    let config = app.config();
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    let command = resolve_command(config, &profile.injections)?;
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

fn resolve_command(
    config: &RuntimeConfig,
    injections: &[InjectionProfile],
) -> Result<ResolvedCommand> {
    if config.command.is_empty() {
        bail!("command mode requires at least one command token");
    }
    if let Some(name) = wrapper_name(&config.command[0])? {
        let file = resolve_wrapper(config, &name)?;
        let mut argv = Vec::with_capacity(config.command.len());
        argv.push(file.to_string_lossy().into_owned());
        argv.extend_from_slice(&config.command[1..]);
        return Ok(ResolvedCommand {
            argv,
            wrapper: Some(ResolvedWrapper { name, file }),
        });
    }

    Ok(ResolvedCommand {
        argv: apply_argv_injections(&config.command, injections)?,
        wrapper: None,
    })
}

fn wrapper_name(token: &str) -> Result<Option<String>> {
    let Some(name) = token.strip_prefix(':') else {
        return Ok(None);
    };
    if name.is_empty() {
        bail!("wrapper name must not be empty");
    }
    if name == "." || name == ".." {
        bail!("invalid wrapper name: :{name}");
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("invalid wrapper name: :{name}");
    }
    Ok(Some(name.to_string()))
}

fn resolve_wrapper(config: &RuntimeConfig, name: &str) -> Result<PathBuf> {
    let searched = wrapper_search_dirs(config)
        .into_iter()
        .flat_map(|dir| wrapper_candidates(&dir, name))
        .collect::<Vec<_>>();

    for candidate in &searched {
        if wrapper_is_executable(candidate) {
            return Ok(candidate.clone());
        }
    }

    let searched = searched
        .iter()
        .map(|path| format!("- {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");
    bail!("wrapper not found: :{name}\nsearched:\n{searched}")
}

fn wrapper_search_dirs(config: &RuntimeConfig) -> Vec<PathBuf> {
    vec![
        profile_root(&config.profile_path).join(".runseal/wrappers"),
        config.runseal_home.join("wrappers"),
    ]
}

fn profile_root(profile_path: &Path) -> &Path {
    profile_path.parent().unwrap_or(Path::new("."))
}

#[cfg(unix)]
fn wrapper_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![dir.join(name)]
}

#[cfg(windows)]
fn wrapper_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    let exact = dir.join(name);
    if Path::new(name).extension().is_some() {
        return vec![exact];
    }
    [exact]
        .into_iter()
        .chain(
            ["exe", "cmd", "bat"]
                .into_iter()
                .map(|ext| dir.join(format!("{name}.{ext}"))),
        )
        .collect()
}

#[cfg(unix)]
fn wrapper_is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(windows)]
fn wrapper_is_executable(path: &Path) -> bool {
    path.is_file()
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
    resolved: &ResolvedCommand,
    exports: &[(String, String)],
) -> Result<i32> {
    let command = &resolved.argv;
    if command.is_empty() {
        bail!("command mode requires at least one command token");
    }

    let mut child = child_command(resolved);
    child.envs(exports.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    child.env("RUNSEAL_HOME", &config.runseal_home);
    child.env("RUNSEAL_PROFILE_HOME", &config.profile_home);
    child.env("RUNSEAL_PROFILE_PATH", &config.profile_path);
    child.env("RUNSEAL_WRAPPER_PATH", wrapper_path_env(config)?);
    child.env_remove("RUNSEAL_WRAPPER_NAME");
    child.env_remove("RUNSEAL_WRAPPER_FILE");
    if let Some(wrapper) = &resolved.wrapper {
        child.env("RUNSEAL_WRAPPER_NAME", &wrapper.name);
        child.env("RUNSEAL_WRAPPER_FILE", &wrapper.file);
    }

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

fn child_command(resolved: &ResolvedCommand) -> Command {
    #[cfg(windows)]
    if let Some(wrapper) = &resolved.wrapper
        && wrapper_uses_cmd(&wrapper.file)
    {
        let mut child = Command::new("cmd");
        child.arg("/C").arg(&wrapper.file);
        if resolved.argv.len() > 1 {
            child.args(&resolved.argv[1..]);
        }
        return child;
    }

    let mut child = Command::new(&resolved.argv[0]);
    if resolved.argv.len() > 1 {
        child.args(&resolved.argv[1..]);
    }
    child
}

#[cfg(windows)]
fn wrapper_uses_cmd(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat")
    )
}

fn wrapper_path_env(config: &RuntimeConfig) -> Result<std::ffi::OsString> {
    env::join_paths(wrapper_search_dirs(config)).context("failed to build RUNSEAL_WRAPPER_PATH")
}
