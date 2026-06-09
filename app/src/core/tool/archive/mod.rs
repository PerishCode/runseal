use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "local" => local(args),
        _ => bail!("unknown tool command: archive {command}"),
    }
}

fn local(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!("usage: runseal @tool archive local export|import ...");
    };
    match command.as_str() {
        "export" => export_local(rest),
        "import" => import_local(rest),
        _ => bail!("usage: runseal @tool archive local export|import ..."),
    }
}

fn export_local(args: &[String]) -> Result<Option<String>> {
    let options = LocalOptions::parse(args, false)?;
    let source = options.source()?;
    if !source.is_dir() {
        bail!("archive source is not a directory: {}", source.display());
    }
    let archive = options.archive()?;
    if archive.exists() {
        bail!(
            "refusing to overwrite existing archive: {}",
            archive.display()
        );
    }
    if let Some(parent) = archive.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
    }

    let password = options.password()?;
    let temp = TempDir::new().context("failed to create temporary archive directory")?;
    let tar_path = temp.path().join("local.tar.gz");
    let source_parent = source
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let source_name = source
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid source path: {}", source.display()))?;

    run(
        Command::new("tar")
            .arg("-czf")
            .arg(&tar_path)
            .arg("-C")
            .arg(source_parent)
            .arg(source_name),
        "tar export",
    )?;
    openssl_encrypt(&tar_path, &archive, &password)?;
    chmod_path(&archive, 0o600)?;
    Ok(None)
}

fn import_local(args: &[String]) -> Result<Option<String>> {
    let options = LocalOptions::parse(args, true)?;
    let dest = options.source()?;
    let archive = options.archive()?;
    if !archive.is_file() {
        bail!("archive not found: {}", archive.display());
    }
    if dest.exists() && !options.force {
        bail!("refusing to replace existing local directory without --force");
    }
    let password = options.password()?;
    let temp = TempDir::new().context("failed to create temporary archive directory")?;
    let tar_path = temp.path().join("local.tar.gz");
    openssl_decrypt(&archive, &tar_path, &password)?;
    validate_tar_entries(&tar_path)?;

    let extract_dir = temp.path().join("extract");
    std::fs::create_dir_all(&extract_dir)
        .with_context(|| format!("failed to create {}", extract_dir.display()))?;
    run(
        Command::new("tar")
            .arg("-xzf")
            .arg(&tar_path)
            .arg("-C")
            .arg(&extract_dir),
        "tar import",
    )?;

    let restored = extract_dir.join(
        dest.file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid destination path: {}", dest.display()))?,
    );
    if !restored.is_dir() {
        bail!("archive does not contain {}", restored.display());
    }
    if dest.exists() {
        std::fs::remove_dir_all(&dest)
            .with_context(|| format!("failed to remove {}", dest.display()))?;
    }
    copy_dir_all(&restored, &dest)?;
    fix_local_permissions(&dest)?;
    Ok(None)
}

#[derive(Default)]
struct LocalOptions {
    source: Option<String>,
    archive: Option<String>,
    password: Option<String>,
    password_env: Option<String>,
    force: bool,
}

impl LocalOptions {
    fn parse(args: &[String], allow_force: bool) -> Result<Self> {
        let mut options = LocalOptions::default();
        let mut rest = args;
        while let Some((flag, tail)) = rest.split_first() {
            match flag.as_str() {
                "--source" => {
                    let Some((value, next)) = tail.split_first() else {
                        bail!("missing value for --source");
                    };
                    options.source = Some(value.clone());
                    rest = next;
                }
                "--archive" => {
                    let Some((value, next)) = tail.split_first() else {
                        bail!("missing value for --archive");
                    };
                    options.archive = Some(value.clone());
                    rest = next;
                }
                "--password" => {
                    let Some((value, next)) = tail.split_first() else {
                        bail!("missing value for --password");
                    };
                    options.password = Some(value.clone());
                    rest = next;
                }
                "--password-env" => {
                    let Some((value, next)) = tail.split_first() else {
                        bail!("missing value for --password-env");
                    };
                    options.password_env = Some(value.clone());
                    rest = next;
                }
                "--force" if allow_force => {
                    options.force = true;
                    rest = tail;
                }
                _ => bail!(
                    "usage: runseal @tool archive local export|import --source <dir> --archive <path> (--password <text>|--password-env <name>) [--force]"
                ),
            }
        }
        Ok(options)
    }

    fn source(&self) -> Result<PathBuf> {
        self.source
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("missing --source"))
    }

    fn archive(&self) -> Result<PathBuf> {
        self.archive
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("missing --archive"))
    }

    fn password(&self) -> Result<String> {
        match (&self.password, &self.password_env) {
            (Some(password), None) if !password.is_empty() => Ok(password.clone()),
            (None, Some(name)) => {
                let value = std::env::var(name)
                    .with_context(|| format!("missing password env var: {name}"))?;
                if value.is_empty() {
                    bail!("password env var is empty: {name}");
                }
                Ok(value)
            }
            (Some(_), Some(_)) => bail!("use only one of --password or --password-env"),
            _ => bail!("missing --password or --password-env"),
        }
    }
}

fn openssl_encrypt(input: &Path, output: &Path, password: &str) -> Result<()> {
    let mut child = Command::new("openssl")
        .args(["enc", "-aes-256-cbc", "-salt", "-pbkdf2", "-out"])
        .arg(output)
        .arg("-pass")
        .arg("stdin")
        .arg("-in")
        .arg(input)
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start openssl encryption")?;
    write_password(&mut child, password)?;
    let status = child.wait().context("failed to wait for openssl")?;
    if !status.success() {
        bail!("openssl encryption failed");
    }
    Ok(())
}

fn openssl_decrypt(input: &Path, output: &Path, password: &str) -> Result<()> {
    let mut child = Command::new("openssl")
        .args(["enc", "-d", "-aes-256-cbc", "-pbkdf2", "-in"])
        .arg(input)
        .arg("-pass")
        .arg("stdin")
        .arg("-out")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start openssl decryption")?;
    write_password(&mut child, password)?;
    let status = child.wait().context("failed to wait for openssl")?;
    if !status.success() {
        bail!("openssl decryption failed");
    }
    Ok(())
}

fn write_password(child: &mut std::process::Child, password: &str) -> Result<()> {
    use std::io::Write;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to open openssl stdin"))?;
    stdin
        .write_all(password.as_bytes())
        .context("failed to write openssl password")?;
    stdin
        .write_all(b"\n")
        .context("failed to finish openssl password")?;
    Ok(())
}

fn validate_tar_entries(path: &Path) -> Result<()> {
    let output = Command::new("tar")
        .arg("-tzf")
        .arg(path)
        .output()
        .context("failed to list tar archive")?;
    if !output.status.success() {
        bail!("failed to list tar archive");
    }
    let stdout = String::from_utf8(output.stdout).context("tar listing was not UTF-8")?;
    for line in stdout.lines() {
        validate_relative_entry(line)?;
    }
    Ok(())
}

fn validate_relative_entry(entry: &str) -> Result<()> {
    let path = Path::new(entry);
    if path.is_absolute() {
        bail!("unsafe archive path: {entry}");
    }
    let mut components = path.components();
    let Some(Component::Normal(first)) = components.next() else {
        bail!("unsafe archive path: {entry}");
    };
    if first != OsStr::new(".local") {
        bail!("archive entry must be under .local/: {entry}");
    }
    for component in components {
        match component {
            Component::Normal(_) => {}
            _ => bail!("unsafe archive path: {entry}"),
        }
    }
    Ok(())
}

fn run(command: &mut Command, label: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("failed to execute {label}"))?;
    if !status.success() {
        bail!("{label} failed");
    }
    Ok(())
}

fn copy_dir_all(source: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("failed to create {}", dest.display()))?;
    for entry in
        std::fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", source.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type: {}", entry.path().display()))?;
        let target = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &target)
                .with_context(|| format!("failed to copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn fix_local_permissions(local: &Path) -> Result<()> {
    if local.is_dir() {
        chmod_path(local, 0o700)?;
    }
    for name in ["ssh", "kube", "secrets", "tmp"] {
        let path = local.join(name);
        if path.is_dir() {
            chmod_path(&path, 0o700)?;
        }
    }
    for name in ["ssh", "kube", "secrets"] {
        let path = local.join(name);
        if path.is_dir() {
            chmod_files_recursive(&path, 0o600)?;
        }
    }
    Ok(())
}

fn chmod_files_recursive(path: &Path, mode: u32) -> Result<()> {
    for entry in
        std::fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", path.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type: {}", entry.path().display()))?;
        if file_type.is_dir() {
            chmod_files_recursive(&entry.path(), mode)?;
        } else if file_type.is_file() {
            chmod_path(&entry.path(), mode)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn chmod_path(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata: {}", path.display()))?
        .permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn chmod_path(path: &Path, _mode: u32) -> Result<()> {
    std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata: {}", path.display()))?;
    Ok(())
}
