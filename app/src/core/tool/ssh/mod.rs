use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "config" => config(args),
        "script" => script(args),
        _ => bail!("unknown tool command: ssh {command}"),
    }
}

fn config(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!("usage: runseal @tool ssh config host|identities ...");
    };
    match command.as_str() {
        "host" => config_host(rest),
        "identities" => config_identities(rest),
        _ => bail!("usage: runseal @tool ssh config host|identities ..."),
    }
}

fn script(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!(
            "usage: runseal @tool ssh script run|capture --config <path> --host <host> --file <path> -- <args...>"
        );
    };
    match command.as_str() {
        "run" => script_run(rest, false),
        "capture" => script_run(rest, true),
        _ => bail!(
            "usage: runseal @tool ssh script run|capture --config <path> --host <host> --file <path> -- <args...>"
        ),
    }
}

fn script_run(args: &[String], capture: bool) -> Result<Option<String>> {
    let (options, script_args) = split_options(args);
    let config = required_option(options, "--config")?;
    let host = required_option(options, "--host")?;
    let file = required_option(options, "--file")?;
    if !Path::new(&file).is_file() {
        bail!("script not found: {file}");
    }
    let patterns = read_hosts(Path::new(&config))?;
    if !host_allowed(&host, &patterns) {
        bail!("host is not declared in {config}: {host}");
    }
    let input = std::fs::read(&file).with_context(|| format!("failed to read script: {file}"))?;
    let mut command = Command::new("ssh");
    command
        .arg("-F")
        .arg(&config)
        .arg(&host)
        .arg("bash")
        .arg("-s")
        .arg("--")
        .args(script_args)
        .stdin(Stdio::piped());
    if capture {
        command.stdout(Stdio::piped());
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to execute ssh for host: {host}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(&input)
            .with_context(|| format!("failed to send script to ssh for host: {host}"))?;
    }
    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to wait for ssh script on host: {host}"))?;
    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        bail!("ssh script failed with exit code {code}");
    }
    if capture {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
    }
    Ok(None)
}

fn split_options(args: &[String]) -> (&[String], &[String]) {
    if let Some(index) = args.iter().position(|arg| arg == "--") {
        (&args[..index], &args[index + 1..])
    } else {
        (args, &[])
    }
}

fn config_host(args: &[String]) -> Result<Option<String>> {
    let [host, rest @ ..] = args else {
        bail!("usage: runseal @tool ssh config host <host> --config <path>");
    };
    let config = required_option(rest, "--config")?;
    let patterns = read_hosts(Path::new(&config))?;
    Ok(Some(host_allowed(host, &patterns).to_string()))
}

fn config_identities(args: &[String]) -> Result<Option<String>> {
    let config = required_option(args, "--config")?;
    let config_path = Path::new(&config);
    let base = optional_option(args, "--base")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            config_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });
    let identities = read_identity_files(config_path, &base)?;
    Ok(Some(serde_json::to_string(&identities)?))
}

fn read_hosts(config: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(config)
        .with_context(|| format!("failed to read ssh config: {}", config.display()))?;
    Ok(text
        .lines()
        .filter_map(config_words)
        .filter(|words| {
            words
                .first()
                .is_some_and(|word| word.eq_ignore_ascii_case("host"))
        })
        .flat_map(|words| words.into_iter().skip(1))
        .collect())
}

fn read_identity_files(config: &Path, base: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(config)
        .with_context(|| format!("failed to read ssh config: {}", config.display()))?;
    let mut files = Vec::new();
    for words in text.lines().filter_map(config_words) {
        if words
            .first()
            .is_some_and(|word| word.eq_ignore_ascii_case("identityfile"))
            && let Some(value) = words.get(1)
        {
            files.push(
                resolve_identity_file(value, base)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    Ok(files)
}

fn config_words(line: &str) -> Option<Vec<String>> {
    let line = line.split_once('#').map_or(line, |(before, _)| before);
    let words = line
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!words.is_empty()).then_some(words)
}

fn host_allowed(host: &str, patterns: &[String]) -> bool {
    let mut matched = false;
    for pattern in patterns {
        if let Some(negative) = pattern.strip_prefix('!') {
            if glob_match(negative, host) {
                return false;
            }
        } else if glob_match(pattern, host) {
            matched = true;
        }
    }
    matched
}

fn resolve_identity_file(value: &str, base: &Path) -> PathBuf {
    if value == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn required_option(args: &[String], name: &str) -> Result<String> {
    optional_option(args, name).ok_or_else(|| anyhow::anyhow!("{name} is required"))
}

fn optional_option(args: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == name {
            return args.get(index + 1).cloned();
        }
        if let Some(value) = arg.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
        index += 1;
    }
    None
}

fn glob_match(pattern: &str, value: &str) -> bool {
    glob_match_inner(pattern.as_bytes(), value.as_bytes())
}

fn glob_match_inner(pattern: &[u8], value: &[u8]) -> bool {
    match (pattern.split_first(), value.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((&b'*', rest)), _) => {
            glob_match_inner(rest, value)
                || value
                    .split_first()
                    .is_some_and(|(_, tail)| glob_match_inner(pattern, tail))
        }
        (Some((&b'?', rest)), Some((_, tail))) => glob_match_inner(rest, tail),
        (Some((&expected, rest)), Some((&actual, tail))) if expected == actual => {
            glob_match_inner(rest, tail)
        }
        _ => false,
    }
}
