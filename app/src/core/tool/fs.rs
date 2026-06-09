use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use base64::Engine;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "mkdir" => mkdir(args),
        "write" => write(args),
        "write-base64" => write_base64(args),
        "chmod" => chmod(args),
        "mode" => mode(args),
        "touch" => touch(args),
        "list" => list(args),
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

fn write(args: &[String]) -> Result<Option<String>> {
    let (path, text, mode) = match args {
        [path, text] => (path, text, None),
        [path, text, mode] => (path, text, Some(mode)),
        _ => bail!("usage: runseal @tool fs write <path> <text> [mode]"),
    };
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }
    std::fs::write(path, text).with_context(|| format!("failed to write file: {path}"))?;
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

fn mode(args: &[String]) -> Result<Option<String>> {
    let [path] = args else {
        bail!("usage: runseal @tool fs mode <path>");
    };
    mode_path(Path::new(path))
}

fn touch(args: &[String]) -> Result<Option<String>> {
    let (path, mode) = match args {
        [path] => (path, None),
        [path, mode] => (path, Some(mode)),
        _ => bail!("usage: runseal @tool fs touch <path> [mode]"),
    };
    let path = Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to touch {}", path.display()))?;
    if let Some(mode) = mode {
        chmod_path(path, mode)?;
    }
    Ok(None)
}

fn list(args: &[String]) -> Result<Option<String>> {
    let [dir, rest @ ..] = args else {
        bail!(
            "usage: runseal @tool fs list <path> [--glob <pattern>] [--files] [--dirs] [--require-nonempty]"
        );
    };
    let mut glob = "*".to_string();
    let mut files = false;
    let mut dirs = false;
    let mut require_nonempty = false;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--glob" => {
                let Some(value) = rest.get(index + 1) else {
                    bail!("--glob requires a value");
                };
                glob = value.clone();
                index += 2;
            }
            "--files" => {
                files = true;
                index += 1;
            }
            "--dirs" => {
                dirs = true;
                index += 1;
            }
            "--require-nonempty" => {
                require_nonempty = true;
                index += 1;
            }
            other => bail!("unknown fs list argument: {other}"),
        }
    }
    if !files && !dirs {
        files = true;
        dirs = true;
    }
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        bail!("fs list path is not a directory: {}", dir_path.display());
    }
    let mut matches = Vec::new();
    for entry in std::fs::read_dir(dir_path)
        .with_context(|| format!("failed to read directory: {}", dir_path.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", dir_path.display()))?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if !glob_match(&glob, name) {
            continue;
        }
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type: {}", entry.path().display()))?;
        if (file_type.is_file() && files) || (file_type.is_dir() && dirs) {
            let path = entry
                .path()
                .canonicalize()
                .with_context(|| format!("failed to canonicalize {}", entry.path().display()))?;
            matches.push(path.to_string_lossy().into_owned());
        }
    }
    matches.sort();
    if require_nonempty && matches.is_empty() {
        bail!("fs list found no matches in {}", dir_path.display());
    }
    Ok(Some(serde_json::to_string(&matches)?))
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

#[cfg(unix)]
fn mode_path(path: &Path) -> Result<Option<String>> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata: {}", path.display()))?
        .permissions()
        .mode()
        & 0o777;
    Ok(Some(format!("{mode:03o}")))
}

#[cfg(not(unix))]
fn mode_path(path: &Path) -> Result<Option<String>> {
    std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata: {}", path.display()))?;
    Ok(Some("".to_string()))
}
