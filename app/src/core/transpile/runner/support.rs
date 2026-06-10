use std::path::Path;

use anyhow::{Context, Result};

use super::super::ast::{ArgvSpec, OutputStream};

pub(super) enum CaptureMode {
    None,
    Stdout,
    All,
}

pub(super) struct CommandOutput {
    pub(super) code: i32,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

pub(super) fn write_stream_file(
    stream: &OutputStream,
    path: &Path,
    append: bool,
    output: &CommandOutput,
) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }
    let captured = match stream {
        OutputStream::Stdout => output.stdout.as_bytes(),
        OutputStream::Stderr => output.stderr.as_bytes(),
    };
    if append {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("failed to append file: {}", path.display()))?;
        file.write_all(captured)
            .with_context(|| format!("failed to append file: {}", path.display()))?;
    } else {
        std::fs::write(path, captured)
            .with_context(|| format!("failed to write file: {}", path.display()))?;
    }
    match stream {
        OutputStream::Stdout => {
            std::io::stderr()
                .write_all(output.stderr.as_bytes())
                .context("failed to write stderr")?;
        }
        OutputStream::Stderr => {
            std::io::stdout()
                .write_all(output.stdout.as_bytes())
                .context("failed to write stdout")?;
        }
    }
    Ok(())
}

pub(super) fn find_spec<'a>(specs: &'a [ArgvSpec], arg: &str) -> Option<&'a ArgvSpec> {
    specs.iter().find(|spec| {
        let option = option_name(&spec.name);
        arg == option || arg.starts_with(&(option + "="))
    })
}

pub(super) fn option_name(name: &str) -> String {
    format!("--{}", name.replace('_', "-"))
}

pub(super) fn case_matches(pattern: &str, value: &str) -> bool {
    pattern == "*" || pattern == value
}

pub(super) fn shell_words(argv: &[String]) -> String {
    argv.join("\u{1f}")
}

pub(super) fn split_words(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split('\u{1f}').map(str::to_string).collect()
    }
}
