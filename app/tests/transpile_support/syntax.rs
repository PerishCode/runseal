use std::{
    io::Write,
    process::{Command, Stdio},
};

use tempfile::TempDir;

#[path = "tool.rs"]
mod tool;

pub fn assert_bash(source: &str) {
    if !tool::exists("bash") || !bash_accepts_stdin() {
        return;
    }
    let mut child = Command::new("bash")
        .arg("-n")
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("bash should run");
    child
        .stdin
        .as_mut()
        .expect("bash stdin should be piped")
        .write_all(source.as_bytes())
        .expect("bash source should be written");
    let output = child.wait_with_output().expect("bash should finish");
    assert!(
        output.status.success(),
        "bash syntax should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn bash_accepts_stdin() -> bool {
    let output = Command::new("bash")
        .arg("-n")
        .arg("-s")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
    output.is_ok_and(|output| output.status.success())
}

pub fn assert_pwsh(source: &str) {
    if !tool::exists("pwsh") {
        return;
    }
    let temp = TempDir::new().expect("temp dir should be created");
    let source_path = temp.path().join("source.ps1");
    let checker_path = temp.path().join("check.ps1");
    std::fs::write(&source_path, source).expect("PowerShell source should be written");
    std::fs::write(
        &checker_path,
        r#"
param([string]$Path)
$tokens = $null
$errors = $null
[System.Management.Automation.Language.Parser]::ParseInput(
  (Get-Content -Raw -LiteralPath $Path),
  [ref]$tokens,
  [ref]$errors
) | Out-Null
if ($errors.Count -gt 0) {
  $errors | ForEach-Object { Write-Error $_.Message }
  exit 1
}
"#,
    )
    .expect("PowerShell checker should be written");
    let output = Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-File")
        .arg(&checker_path)
        .arg(&source_path)
        .output()
        .expect("pwsh should run");
    assert!(
        output.status.success(),
        "PowerShell syntax should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
