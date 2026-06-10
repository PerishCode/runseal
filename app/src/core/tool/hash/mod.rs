use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "tree" => tree(args),
        _ => bail!("unknown tool command: hash {command}"),
    }
}

fn tree(args: &[String]) -> Result<Option<String>> {
    if args.is_empty() {
        bail!("usage: runseal @tool hash tree <path>...");
    }
    let mut entries = Vec::new();
    for input in args {
        let path = Path::new(input);
        collect(path, PathBuf::from(input), &mut entries)?;
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = Sha256::new();
    for (label, file) in entries {
        hasher.update(label.as_bytes());
        hasher.update([0]);
        let mut handle = fs::File::open(&file)
            .with_context(|| format!("failed to open file: {}", file.display()))?;
        let mut buffer = Vec::new();
        handle
            .read_to_end(&mut buffer)
            .with_context(|| format!("failed to read file: {}", file.display()))?;
        hasher.update(&buffer);
        hasher.update([0]);
    }

    Ok(Some(format!("{:x}", hasher.finalize())))
}

fn collect(path: &Path, label: PathBuf, entries: &mut Vec<(String, PathBuf)>) -> Result<()> {
    let metadata =
        fs::metadata(path).with_context(|| format!("path not found: {}", path.display()))?;
    if metadata.is_file() {
        entries.push((normalize(&label), path.to_path_buf()));
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .with_context(|| format!("failed to read directory: {}", path.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read entry: {}", path.display()))?;
            let name = entry.file_name();
            let child_path = entry.path();
            let child_label = label.join(&name);
            collect(&child_path, child_label, entries)?;
        }
        return Ok(());
    }
    bail!("unsupported path for hash tree: {}", path.display())
}

fn normalize(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
