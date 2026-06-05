use std::{path::Path, process::Command};

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

fn argv_source() -> &'static str {
    r#"
seal argv parse --string channel=stable --string ref=main --string body_file= --flag dry_run --flag no_merge
if empty "$channel"; then
  fail "channel missing"
fi
print "$body_file"
"#
}

fn powershell_argv_source() -> &'static str {
    r#"
seal argv parse --string channel=stable --string ref=main --string body_file= --flag dry_run --flag no_merge
if ([string]::IsNullOrEmpty($channel)) {
    throw 'channel missing'
}
Write-Output $body_file
"#
}

#[test]
fn argv_parse_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(argv_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("argv_parse"));
        assert!(stdout.contains("body_file"));
        assert!(stdout.contains("dry_run"));
    }

    let fx = fixture(powershell_argv_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("argv_parse"));
    assert!(stdout.contains("no_merge"));
}

#[test]
fn argv_parse_emits_targets() {
    let fx = fixture(argv_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("body_file=${1#--body-file=}"));
    assert!(bash.contains("dry_run=true"));
    assert!(powershell.contains("$body_file = $__seal_arg.Substring(12)"));
    assert!(powershell.contains("$dry_run = $true"));
    assert_bash_syntax(&bash);
    assert_pwsh_syntax(&powershell);
}

fn assert_bash_syntax(source: &str) {
    if !tool_exists("bash") {
        return;
    }
    let output = Command::new("bash")
        .arg("-n")
        .arg("-c")
        .arg(source)
        .output()
        .expect("bash should run");
    assert!(
        output.status.success(),
        "bash syntax should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
