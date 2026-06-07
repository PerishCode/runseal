use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use base64::Engine;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "mkdir" => mkdir(args),
        "write-base64" => write_base64(args),
        "chmod" => chmod(args),
        "contains-any" => contains_any(args),
        "backup-numbered" => backup_numbered(args),
        _ => bail!("unknown tool command: fs {command}"),
    }
}

fn mkdir(args: &[String]) -> Result<Option<String>> {
    let (path, mode) = match args {
        [path] => (path, None),
        [path, mode] => (path, Some(mode)),
        _ => bail!("usage: runseal @tool fs mkdir <path> [mode]"),
    };
    std::fs::create_dir_all(path).with_context(|| format!("failed to create directory: {path}"))?;
    if let Some(mode) = mode {
        chmod_path(Path::new(path), mode)?;
    }
    Ok(None)
}

fn write_base64(args: &[String]) -> Result<Option<String>> {
    let [path, encoded] = args else {
        bail!("usage: runseal @tool fs write-base64 <path> <base64>");
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("invalid base64 content")?;
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }
    std::fs::write(path, bytes).with_context(|| format!("failed to write file: {path}"))?;
    Ok(None)
}

fn chmod(args: &[String]) -> Result<Option<String>> {
    let [path, mode] = args else {
        bail!("usage: runseal @tool fs chmod <path> <mode>");
    };
    chmod_path(Path::new(path), mode)?;
    Ok(None)
}

fn contains_any(args: &[String]) -> Result<Option<String>> {
    let [path, needles @ ..] = args else {
        bail!("usage: runseal @tool fs contains-any <path> <text>...");
    };
    if needles.is_empty() {
        bail!("fs contains-any requires at least one text argument");
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).with_context(|| format!("failed to read file: {path}")),
    };
    Ok(Some(
        needles
            .iter()
            .any(|needle| text.contains(needle))
            .to_string(),
    ))
}

fn backup_numbered(args: &[String]) -> Result<Option<String>> {
    let [path] = args else {
        bail!("usage: runseal @tool fs backup-numbered <path>");
    };
    let path = PathBuf::from(path);
    let backup = next_backup_path(&path)?;
    std::fs::rename(&path, &backup)
        .with_context(|| format!("failed to move {} to {}", path.display(), backup.display()))?;
    Ok(Some(backup.to_string_lossy().into_owned()))
}

fn next_backup_path(path: &Path) -> Result<PathBuf> {
    let backup = path.with_file_name(format!(
        "{}.bak",
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("invalid path: {}", path.display()))?
    ));
    if !backup.exists() {
        return Ok(backup);
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid path: {}", path.display()))?;
    for index in 1..1000 {
        let candidate = path.with_file_name(format!("{file_name}.bak.{index}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("too many existing backups for {}", path.display())
}

#[cfg(unix)]
fn chmod_path(path: &Path, mode: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = u32::from_str_radix(mode.trim_start_matches("0o"), 8)
        .with_context(|| format!("invalid file mode: {mode}"))?;
    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata: {}", path.display()))?
        .permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn chmod_path(_path: &Path, _mode: &str) -> Result<()> {
    Ok(())
}
