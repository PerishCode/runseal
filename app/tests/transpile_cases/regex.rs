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

fn regex_source() -> &'static str {
    r#"
trigger_output='https://github.com/PerishCode/runseal/actions/runs/12345'
run_id=$(runseal @tool regex capture "$trigger_output" '/actions/runs/([0-9]+)' 1)
if [ -z "$run_id" ]; then
  run_id=$(latest_run_id "$workflow" "$ref")
fi
print "$run_id"
"#
}

fn powershell_regex_source() -> &'static str {
    r#"
$trigger_output = 'https://github.com/PerishCode/runseal/actions/runs/12345'
$run_id = & 'runseal' '@tool' 'regex' 'capture' $trigger_output '/actions/runs/([0-9]+)' '1'
if ([string]::IsNullOrEmpty($run_id)) {
    $run_id = & 'latest_run_id' $workflow $ref
}
Write-Output $run_id
"#
}

#[test]
fn regex_capture_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(regex_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("capture_checked"));
        assert!(stdout.contains("regex"));
        assert!(stdout.contains("/actions/runs/([0-9]+)"));
    }

    let fx = fixture(powershell_regex_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("capture_checked"));
    assert!(stdout.contains("regex"));
    assert!(stdout.contains("\"1\""));
}

#[test]
fn regex_capture_emits_targets() {
    let fx = fixture(regex_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains(
        "run_id=$(runseal @tool regex capture \"$trigger_output\" '/actions/runs/([0-9]+)' 1)"
    ));
    assert!(
        powershell.contains(
            "$run_id = & 'runseal' '@tool' 'regex' 'capture' $trigger_output '/actions/runs/([0-9]+)' '1'"
        )
    );
    super::syntax::assert_bash(&bash);
    super::syntax::assert_pwsh(&powershell);
}
