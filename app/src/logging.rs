use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::core::config::{RawEnv, resolve_runseal_home};

static CURRENT_LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct SessionLog {
    file_path: PathBuf,
}

impl SessionLog {
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

#[derive(Clone)]
pub struct SharedFileWriter(Arc<Mutex<File>>);

impl Write for SharedFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("log file mutex should not be poisoned")
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0
            .lock()
            .expect("log file mutex should not be poisoned")
            .flush()
    }
}

pub fn prepare_session_log(raw_env: &RawEnv, command_slug: &str) -> Result<SessionLog> {
    let log_root = std::env::var_os("RUNSEAL_LOG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .map(Ok)
        .unwrap_or_else(|| resolve_runseal_home(raw_env).map(|path| path.join("logs")))?;

    std::fs::create_dir_all(&log_root)
        .with_context(|| format!("failed to create log directory: {}", log_root.display()))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock drifted before UNIX_EPOCH")?
        .as_secs();
    let pid = std::process::id();
    let slug = sanitize_slug(command_slug);
    let file_path = log_root.join(format!("{}-{}-{}.log", timestamp, pid, slug));
    Ok(SessionLog { file_path })
}

pub fn make_file_writer(session_log: &SessionLog) -> Result<SharedFileWriter> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(session_log.file_path())
        .with_context(|| {
            format!(
                "failed to open session log file: {}",
                session_log.file_path().display()
            )
        })?;
    let _ = CURRENT_LOG_FILE.set(session_log.file_path().to_path_buf());
    Ok(SharedFileWriter(Arc::new(Mutex::new(file))))
}

pub fn current_log_file() -> Option<&'static Path> {
    CURRENT_LOG_FILE.get().map(PathBuf::as_path)
}

fn sanitize_slug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_dash = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() { ch } else { '-' };
        if normalized == '-' {
            if last_dash {
                continue;
            }
            last_dash = true;
        } else {
            last_dash = false;
        }
        out.push(normalized.to_ascii_lowercase());
        if out.len() >= 40 {
            break;
        }
    }
    out.trim_matches('-').to_owned().if_empty_then("run")
}

trait IfEmptyThen {
    fn if_empty_then(self, fallback: &str) -> String;
}

impl IfEmptyThen for String {
    fn if_empty_then(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_owned()
        } else {
            self
        }
    }
}
