use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "exists" => exists(args),
        "write" => write(args),
        _ => bail!("unknown tool command: process {command}"),
    }
}

fn exists(args: &[String]) -> Result<Option<String>> {
    let [name] = args else {
        bail!("usage: runseal @tool process exists <name>");
    };
    Ok(Some(command_exists(name).to_string()))
}

fn write(args: &[String]) -> Result<Option<String>> {
    let [stream, path, rest @ ..] = args else {
        bail!(
            "usage: runseal @tool process write <stdout|stderr> <path> [--append] -- <command> [args...]"
        );
    };
    let stream = parse_stream(stream)?;
    let mut append = false;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--append" => {
                append = true;
                index += 1;
            }
            "--" => {
                index += 1;
                break;
            }
            other => bail!("unknown process write argument: {other}"),
        }
    }
    let command_argv = &rest[index..];
    let Some((program, command_args)) = command_argv.split_first() else {
        bail!("process write requires one command after `--`");
    };
    let output = Command::new(program)
        .args(command_args)
        .output()
        .with_context(|| format!("failed to execute command: {program}"))?;
    let captured = match stream {
        Stream::Stdout => &output.stdout,
        Stream::Stderr => &output.stderr,
    };
    let passthrough = match stream {
        Stream::Stdout => &output.stderr,
        Stream::Stderr => &output.stdout,
    };
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }
    if append {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("failed to append file: {path}"))?;
        file.write_all(captured)
            .with_context(|| format!("failed to append file: {path}"))?;
    } else {
        std::fs::write(path, captured).with_context(|| format!("failed to write file: {path}"))?;
    }
    match stream {
        Stream::Stdout => {
            use std::io::Write;
            std::io::stderr()
                .write_all(passthrough)
                .context("failed to write stderr")?;
        }
        Stream::Stderr => {
            use std::io::Write;
            std::io::stdout()
                .write_all(passthrough)
                .context("failed to write stdout")?;
        }
    }
    if output.status.success() {
        return Ok(None);
    }
    std::process::exit(output.status.code().unwrap_or(1));
}

enum Stream {
    Stdout,
    Stderr,
}

fn parse_stream(value: &str) -> Result<Stream> {
    match value {
        "stdout" => Ok(Stream::Stdout),
        "stderr" => Ok(Stream::Stderr),
        _ => bail!("process write stream must be stdout or stderr"),
    }
}

fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}
