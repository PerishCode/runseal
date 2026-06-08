use std::process::Command;

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
while [ "$attempt" -lt 6 ]; do
  raw=$(gh run list --json databaseId)
  if [ "$(runseal @tool json empty "$raw")" = false ]; then
    run_id=$(runseal @tool json get "$raw" '.[0].databaseId')
    break
  fi
  sleep 2
  attempt=$(runseal @tool int add "$attempt" 1)
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
        $run_id = & 'runseal' '@tool' 'json' 'get' $raw '.[0].databaseId'
        break
    }
    Start-Sleep -Seconds 2
    $attempt = & 'runseal' '@tool' 'int' 'add' $attempt '1'
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
        assert!(stdout.contains("capture_checked"));
        assert!(stdout.contains("runseal"));
        assert!(stdout.contains("\"type\": \"break\""));
    }

    let fx = fixture(powershell_retry_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"type\": \"while\""));
    assert!(stdout.contains("json_not_empty"));
    assert!(stdout.contains("capture_checked"));
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
    super::syntax::assert_bash(&bash);
    super::syntax::assert_pwsh(&powershell);
}
