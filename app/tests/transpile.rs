use std::process::Command;
use tempfile::TempDir;

#[path = "transpile_support/syntax.rs"]
mod syntax;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

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
    bin()
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

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

fn sample_source() -> &'static str {
    r#"
channel=${RUNSEAL_CHANNEL:-stable}

release_run() {
  if empty "$channel"; then
    fail "channel missing"
  fi
  gh workflow run release.yml --ref main -f "channel=$channel"
}

case "$channel" in
  stable) print "stable release" ;;
  beta) release_run ;;
  *) fail "unknown channel: $channel" ;;
esac
"#
}

fn powershell_source() -> &'static str {
    r#"
$channel = $(if ($env:RUNSEAL_CHANNEL) { $env:RUNSEAL_CHANNEL } else { 'stable' })
function release_run {
    if ([string]::IsNullOrEmpty($channel)) {
        throw 'channel missing'
    }
    & 'gh' 'workflow' 'run' 'release.yml' '--ref' 'main' '-f' ('channel=' + $channel)
}

switch ($channel) {
    'stable' {
        Write-Output 'stable release'
        break
    }
    'beta' {
        release_run
        break
    }
    Default {
        throw ('unknown channel: ' + $channel)
        break
    }
}
"#
}

fn capture_source() -> &'static str {
    r#"
raw=$(gh run list --json databaseId)
print "$raw"
"#
}

fn powershell_capture_source() -> &'static str {
    r#"
$raw = & 'gh' 'run' 'list' '--json' 'databaseId'
Write-Output $raw
"#
}

fn trim_source() -> &'static str {
    r#"
raw="  value  "
trimmed=$(seal string trim "$raw")
print "$trimmed"
"#
}

fn powershell_trim_source() -> &'static str {
    r#"
$raw = '  value  '
$trimmed = seal string trim $raw
Write-Output $trimmed
"#
}

fn json_get_source() -> &'static str {
    r#"
raw='[{"databaseId":123}]'
run_id=$(seal json get "$raw" '.[0].databaseId')
print "$run_id"
"#
}

fn powershell_json_get_source() -> &'static str {
    r#"
$raw = '[{"databaseId":123}]'
$run_id = seal json get $raw '.[0].databaseId'
Write-Output $run_id
"#
}

#[test]
fn help_without_profile() {
    let fx = fixture("");

    let output = bin()
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--help")
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Usage: runseal @transpile"));
    assert!(stdout.contains("--input-lang"));
}

#[test]
fn sealir_without_profile() {
    let fx = fixture(sample_source());

    let output = run_transpile(&fx, "seal", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be JSON");
    assert_eq!(payload["version"], 1);
    assert!(stdout.contains("env_default"));
    assert!(stdout.contains("exec_checked"));
}

#[test]
fn bash_frontend_sealir() {
    let fx = fixture(sample_source());

    let output = run_transpile(&fx, "bash", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("env_default"));
    assert!(stdout.contains("exec_checked"));
}

#[test]
fn powershell_frontend_sealir() {
    let fx = fixture(powershell_source());

    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("env_default"));
    assert!(stdout.contains("call_function"));
    assert!(stdout.contains("exec_checked"));
}

#[test]
fn powershell_to_bash() {
    let fx = fixture(powershell_source());

    let output = run_transpile(&fx, "powershell", "bash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("release_run() {"));
    assert!(stdout.contains("gh workflow run release.yml --ref main -f \"channel=$channel\""));
    syntax::assert_bash(&stdout);
}

#[test]
fn bash_capture_ir() {
    let fx = fixture(capture_source());

    let output = run_transpile(&fx, "bash", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("capture_checked"));
    assert!(stdout.contains("databaseId"));
}

#[test]
fn powershell_capture_ir() {
    let fx = fixture(powershell_capture_source());

    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("capture_checked"));
    assert!(stdout.contains("databaseId"));
}

#[test]
fn capture_to_targets() {
    let fx = fixture(capture_source());

    let bash = run_transpile(&fx, "bash", "bash");
    let powershell = run_transpile(&fx, "bash", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("raw=$(gh run list --json databaseId)"));
    assert!(powershell.contains("$raw = & 'gh' 'run' 'list' '--json' 'databaseId'"));
    syntax::assert_bash(&bash);
    syntax::assert_pwsh(&powershell);
}

#[test]
fn string_trim_helper_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(trim_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("tool_capture"));
        assert!(stdout.contains("string"));
    }

    let fx = fixture(powershell_trim_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("tool_capture"));
    assert!(stdout.contains("string"));
}

#[test]
fn string_trim_emits_native() {
    let fx = fixture(trim_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("trimmed=$(runseal @tool string trim \"$raw\")"));
    assert!(powershell.contains("$trimmed = & 'runseal' '@tool' 'string' 'trim' $raw"));
    syntax::assert_bash(&bash);
    syntax::assert_pwsh(&powershell);
}

#[test]
fn json_get_helper_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(json_get_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("tool_capture"));
        assert!(stdout.contains("json"));
        assert!(stdout.contains("databaseId"));
    }

    let fx = fixture(powershell_json_get_source());
    let output = run_transpile(&fx, "powershell", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("tool_capture"));
    assert!(stdout.contains("json"));
    assert!(stdout.contains("databaseId"));
}

#[test]
fn json_get_emits_native() {
    let fx = fixture(json_get_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("run_id=$(runseal @tool json get \"$raw\" '.[0].databaseId')"));
    assert!(
        powershell.contains("$run_id = & 'runseal' '@tool' 'json' 'get' $raw '.[0].databaseId'")
    );
    syntax::assert_bash(&bash);
    syntax::assert_pwsh(&powershell);
}

#[test]
fn bash_syntax_valid() {
    let fx = fixture(sample_source());

    let output = run_transpile(&fx, "seal", "bash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("set -euo pipefail"));
    assert!(stdout.contains("gh workflow run release.yml --ref main -f \"channel=$channel\""));
    assert!(stdout.contains("case \"$channel\" in"));
    syntax::assert_bash(&stdout);
}

#[test]
fn repo_wrapper_syntax() {
    let root = repo_root();
    for wrapper in [
        ".runseal/wrappers/cloudflare.seal",
        ".runseal/wrappers/init.seal",
        ".runseal/wrappers/pr.seal",
        ".runseal/wrappers/release.seal",
    ] {
        let source = root.join(wrapper);

        let bash = bin()
            .current_dir(&root)
            .arg("@transpile")
            .arg("--input-lang=seal")
            .arg("--output-lang=bash")
            .arg(&source)
            .output()
            .expect("runseal should run");
        assert!(
            bash.status.success(),
            "{wrapper} bash stderr: {}",
            String::from_utf8_lossy(&bash.stderr)
        );
        let bash = String::from_utf8(bash.stdout).expect("bash output should be UTF-8");
        syntax::assert_bash(&bash);

        let powershell = bin()
            .current_dir(&root)
            .arg("@transpile")
            .arg("--input-lang=seal")
            .arg("--output-lang=powershell")
            .arg(&source)
            .output()
            .expect("runseal should run");
        assert!(
            powershell.status.success(),
            "{wrapper} powershell stderr: {}",
            String::from_utf8_lossy(&powershell.stderr)
        );
        let powershell =
            String::from_utf8(powershell.stdout).expect("powershell output should be UTF-8");
        syntax::assert_pwsh(&powershell);
    }
}

#[test]
fn powershell_readable() {
    let fx = fixture(sample_source());

    let output = run_transpile(&fx, "seal", "powershell");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("$ErrorActionPreference = 'Stop'"));
    assert!(stdout.contains("function release_run"));
    assert!(stdout.contains("& 'gh' 'workflow' 'run' 'release.yml' '--ref' 'main' '-f'"));
    assert!(stdout.contains("('channel=' + $channel)"));
    assert!(stdout.contains("switch ($channel)"));
    syntax::assert_pwsh(&stdout);
}

#[test]
fn empty_string_powershell() {
    let fx = fixture("print \"\"\n");

    let output = run_transpile(&fx, "seal", "powershell");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Write-Output ''"));
    syntax::assert_pwsh(&stdout);
}

#[test]
fn sealir_to_seal() {
    let fx = fixture(sample_source());
    let sealir = run_transpile(&fx, "seal", "sealir");
    assert!(sealir.status.success());
    let sealir_text = String::from_utf8(sealir.stdout).expect("stdout should be UTF-8");
    let sealir_path = fx.dir.join("operator.sealir.json");
    std::fs::write(&sealir_path, sealir_text).expect("sealir should be written");

    let output = bin()
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--input-lang=sealir")
        .arg("--output-lang=seal")
        .arg(&sealir_path)
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("release_run() {"));
    assert!(stdout.contains("case $channel in"));
}

#[test]
fn unsupported_input_fails() {
    let fx = fixture("print ok\n");

    let output = run_transpile(&fx, "python", "powershell");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("invalid --input-lang"));
}

#[test]
fn underscore_exec() {
    let fx = fixture("tool_name --version\n");

    let output = run_transpile(&fx, "seal", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("exec_checked"));
    assert!(!stdout.contains("call_function"));
}

#[test]
fn hyphen_exec() {
    let fx = fixture("git-lfs version\n");

    let output = run_transpile(&fx, "seal", "bash");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("git-lfs version"));
}

#[test]
fn metacharacters_fail() {
    for source in ["printf ok | cat\n", "eval something\n"] {
        let fx = fixture(source);

        let output = run_transpile(&fx, "seal", "sealir");

        assert!(!output.status.success(), "{source:?} should fail");
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert!(
            stderr.contains("unsupported"),
            "expected unsupported error, got {stderr:?}"
        );
    }
}
