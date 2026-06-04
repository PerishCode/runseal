use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use path_absolutize::Absolutize;

use super::app::AppContext;
use super::config::RuntimeConfig;
use super::env_key::is_valid_env_key;
use super::profile::InjectionProfile;
use super::{injections, profile};

pub struct RunResult {
    pub exit_code: Option<i32>,
}

enum InternalCommand {
    Profile,
    ResolveResource(Vec<String>),
    Resources,
    Wrappers,
    WhichWrapper(String),
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
    if let Some(command) = resolve_internal_dispatch(config)? {
        run_internal(config, command)?;
        return Ok(RunResult { exit_code: Some(0) });
    }

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

fn resolve_internal_dispatch(config: &RuntimeConfig) -> Result<Option<InternalCommand>> {
    if config.command.is_empty() {
        bail!("command mode requires at least one command token");
    }
    let Some(name) = internal_name(&config.command[0])? else {
        return Ok(None);
    };
    Ok(Some(resolve_internal_command(&name, &config.command[1..])?))
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

fn internal_name(token: &str) -> Result<Option<String>> {
    let Some(name) = token.strip_prefix('@') else {
        return Ok(None);
    };
    if name.is_empty() {
        bail!("internal command name must not be empty");
    }
    validate_symbol_name(name)
        .with_context(|| format!("invalid internal command name: @{name}"))?;
    Ok(Some(name.to_string()))
}

fn resolve_internal_command(name: &str, args: &[String]) -> Result<InternalCommand> {
    match name {
        "profile" => {
            if !args.is_empty() {
                bail!("@profile does not accept arguments");
            }
            Ok(InternalCommand::Profile)
        }
        "resolve" => {
            if args.is_empty() {
                bail!("@resolve requires at least one resource:// URI argument");
            }
            Ok(InternalCommand::ResolveResource(args.to_vec()))
        }
        "resources" => {
            if !args.is_empty() {
                bail!("@resources does not accept arguments");
            }
            Ok(InternalCommand::Resources)
        }
        "wrappers" => {
            if !args.is_empty() {
                bail!("@wrappers does not accept arguments");
            }
            Ok(InternalCommand::Wrappers)
        }
        "which" => {
            if args.len() != 1 {
                bail!("@which requires exactly one :wrapper argument");
            }
            let Some(name) = wrapper_name(&args[0])? else {
                bail!("@which currently supports only :wrapper arguments");
            };
            Ok(InternalCommand::WhichWrapper(name))
        }
        _ => bail!("unknown internal command: @{name}"),
    }
}

fn wrapper_name(token: &str) -> Result<Option<String>> {
    let Some(name) = token.strip_prefix(':') else {
        return Ok(None);
    };
    if name.is_empty() {
        bail!("wrapper name must not be empty");
    }
    validate_symbol_name(name).with_context(|| format!("invalid wrapper name: :{name}"))?;
    Ok(Some(name.to_string()))
}

fn validate_symbol_name(name: &str) -> Result<()> {
    if name == "." || name == ".." {
        bail!("reserved name");
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("expected only ASCII letters, numbers, '.', '_', and '-'");
    }
    Ok(())
}

fn resolve_wrapper(config: &RuntimeConfig, name: &str) -> Result<PathBuf> {
    let searched = wrapper_search_paths(config, name);

    for candidate in &searched {
        if wrapper_is_executable(candidate) {
            return candidate
                .absolutize()
                .with_context(|| format!("failed to absolutize wrapper: {}", candidate.display()))
                .map(|path| path.to_path_buf());
        }
    }

    let searched = searched
        .iter()
        .map(|path| format!("- {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");
    bail!("wrapper not found: :{name}\nsearched:\n{searched}")
}

fn wrapper_search_paths(config: &RuntimeConfig, name: &str) -> Vec<PathBuf> {
    wrapper_search_dirs(config)
        .into_iter()
        .flat_map(|dir| wrapper_candidates(&dir, name))
        .collect()
}

fn wrapper_search_dirs(config: &RuntimeConfig) -> Vec<PathBuf> {
    vec![
        profile_root(&config.profile_path)
            .join(".runseal")
            .join("wrappers"),
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

fn run_internal(config: &RuntimeConfig, command: InternalCommand) -> Result<()> {
    match command {
        InternalCommand::Profile => print_profile(config)?,
        InternalCommand::ResolveResource(uris) => print_resolve_resources(config, &uris)?,
        InternalCommand::Resources => print_resources(config)?,
        InternalCommand::Wrappers => print_wrappers(config)?,
        InternalCommand::WhichWrapper(name) => print_which_wrapper(config, &name)?,
    }

    Ok(())
}

fn print_profile(config: &RuntimeConfig) -> Result<()> {
    println!("RUNSEAL_HOME={}", config.runseal_home.display());
    println!("RUNSEAL_PROFILE_HOME={}", config.profile_home.display());
    println!("RUNSEAL_PROFILE_PATH={}", config.profile_path.display());
    if let Ok(Some(resources)) = profile::load_resources(&config.profile_path)
        && let Ok(root) = profile::resolve_resource_root(&config.profile_path, Some(&resources))
    {
        println!("RUNSEAL_RESOURCE_ROOT={}", root.display());
    }
    println!(
        "RUNSEAL_WRAPPER_PATH={}",
        wrapper_path_env(config)?.to_string_lossy()
    );
    Ok(())
}

fn print_wrappers(config: &RuntimeConfig) -> Result<()> {
    for wrapper in effective_wrappers(config)? {
        println!(
            ":{:<20} {}\t{}",
            wrapper.name,
            wrapper.source,
            wrapper.file.display()
        );
    }
    Ok(())
}

fn print_resolve_resources(config: &RuntimeConfig, uris: &[String]) -> Result<()> {
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    for uri in uris {
        let path =
            profile::resolve_resource_uri(&config.profile_path, profile.resources.as_ref(), uri)?;
        println!("{}", path.display());
    }
    Ok(())
}

fn print_resources(config: &RuntimeConfig) -> Result<()> {
    let profile = profile::load(&config.profile_path).context("unable to load runseal profile")?;
    let root = profile::resolve_resource_root(&config.profile_path, profile.resources.as_ref())?;
    println!("RUNSEAL_RESOURCE_ROOT={}", root.display());
    Ok(())
}

fn print_which_wrapper(config: &RuntimeConfig, name: &str) -> Result<()> {
    let file = resolve_wrapper(config, name)?;
    println!("{}", file.display());
    Ok(())
}

struct ListedWrapper {
    name: String,
    source: &'static str,
    file: PathBuf,
}

fn effective_wrappers(config: &RuntimeConfig) -> Result<Vec<ListedWrapper>> {
    let dirs = wrapper_search_dirs(config);
    let mut names = BTreeSet::new();

    for dir in &dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read wrapper dir: {}", dir.display()))?;
            let file = entry.path();
            if !wrapper_is_executable(&file) {
                continue;
            }
            let Some(name) = listed_wrapper_name(&file) else {
                continue;
            };
            names.insert(name);
        }
    }

    let mut wrappers = Vec::new();
    for name in names {
        let file = resolve_wrapper(config, &name)?;
        let source = if file.starts_with(&dirs[0]) {
            "profile"
        } else {
            "home"
        };
        wrappers.push(ListedWrapper { name, source, file });
    }
    Ok(wrappers)
}

fn listed_wrapper_name(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension().and_then(std::ffi::OsStr::to_str)
            && matches_ignore_ascii_case(ext, &["exe", "cmd", "bat"])
        {
            let stem = path.file_stem()?.to_str()?;
            if validate_symbol_name(stem).is_ok() {
                return Some(stem.to_string());
            }
            return None;
        }
    }

    if validate_symbol_name(file_name).is_ok() {
        return Some(file_name.to_string());
    }
    None
}

#[cfg(windows)]
fn matches_ignore_ascii_case(value: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
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
