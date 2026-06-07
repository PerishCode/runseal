use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    dir: std::path::PathBuf,
    source: std::path::PathBuf,
}

fn fixture(source: &str) -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let dir = temp.path().join("project-without-profile");
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    let source_path = dir.join("operator.seal");
    std::fs::write(&source_path, source).expect("source should be written");
    Fixture {
        _temp: temp,
        dir,
        source: source_path,
    }
}

fn run_transpile(fx: &Fixture, input_lang: &str, output_lang: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--input-lang")
        .arg(input_lang)
        .arg("--output-lang")
        .arg(output_lang)
        .arg(&fx.source)
        .output()
        .expect("runseal should run")
}

fn retry_source() -> &'static str {
    r#"
attempt=0
raw='[]'
while lt "$attempt" 6; do
  raw=$(gh run list --json databaseId)
  if json_not_empty "$raw"; then
    run_id=$(seal json get "$raw" '.[0].databaseId')
    break
  fi
  sleep 2
  attempt=$(seal int add "$attempt" 1)
done
print "$run_id"
"#
}

fn powershell_retry_source() -> &'static str {
    r#"
$attempt = '0'
$raw = '[]'
while ([int]$attempt -lt '6') {
    $raw = & 'gh' 'run' 'list' '--json' 'databaseId'
    if ((($raw | ConvertFrom-Json).Count -gt 0)) {
        $run_id = seal json get $raw '.[0].databaseId'
        break
    }
    Start-Sleep -Seconds 2
    $attempt = [int]$attempt + '1'
}
Write-Output $run_id
"#
}

#[test]
fn retry_loop_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(retry_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("\"type\": \"while\""));
        assert!(stdout.contains("json_not_empty"));
        assert!(stdout.contains("tool_capture"));
        assert!(stdout.contains("int"));
        assert!(stdout.contains("\"type\": \"break\""));
    }

    let fx = fixture(powershell_retry_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"type\": \"while\""));
    assert!(stdout.contains("json_not_empty"));
    assert!(stdout.contains("int_add"));
}

#[test]
fn retry_loop_emits_targets() {
    let fx = fixture(retry_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("while [ $attempt -lt 6 ]; do"));
    assert!(bash.contains("attempt=$(runseal @tool int add \"$attempt\" 1)"));
    assert!(bash.contains("break"));
    assert!(powershell.contains("while ([int]$attempt -lt '6') {"));
    assert!(powershell.contains("& 'runseal' '@tool' 'json' 'empty' $raw"));
    assert!(powershell.contains("$attempt = & 'runseal' '@tool' 'int' 'add' $attempt '1'"));
    assert_bash_syntax(&bash);
    assert_pwsh_syntax(&powershell);
}

fn assert_bash_syntax(source: &str) {
    if !tool_exists("bash") || !bash_accepts_stdin() {
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

fn assert_pwsh_syntax(source: &str) {
    if !tool_exists("pwsh") {
        return;
    }
    let output = Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg("[scriptblock]::Create($args[0]) | Out-Null")
        .arg(source)
        .output()
        .expect("pwsh should run");
    assert!(
        output.status.success(),
        "PowerShell syntax should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn tool_exists(name: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).any(|dir| executable_exists(&dir.join(name)))
}

#[cfg(unix)]
fn executable_exists(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_file()
        && path
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn executable_exists(path: &Path) -> bool {
    path.with_extension("exe").is_file()
        || path.with_extension("cmd").is_file()
        || path.with_extension("bat").is_file()
}
