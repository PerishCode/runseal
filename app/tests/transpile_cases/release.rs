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

fn release_source() -> &'static str {
    r#"
seal argv parse --string channel=stable --string ref=main --string version= --flag watch --flag dry_run

if empty "$channel"; then
  fail "--channel is required"
fi

case "$channel" in
  stable) workflow=release-stable.yml ;;
  beta) workflow=release-beta.yml ;;
  *) fail "unknown channel: $channel" ;;
esac

if eq "$dry_run" true; then
  print "dry run"
else
  gh --version
  gh auth status
  trigger_output=$(gh workflow run "$workflow" --ref "$ref" -f "ref=$ref" -f "version_override=$version")
  if not_empty "$trigger_output"; then
    print "$trigger_output"
  fi
  print "triggered $workflow for ref $ref"
  if eq "$watch" true; then
    run_id=$(seal regex capture "$trigger_output" '/actions/runs/([0-9]+)' 1)
    if empty "$run_id"; then
      attempt=0
      raw='[]'
      while lt "$attempt" 6; do
        raw=$(gh run list --workflow "$workflow" --branch "$ref" --event workflow_dispatch --limit 1 --json databaseId)
        if json_not_empty "$raw"; then
          run_id=$(seal json get "$raw" '.[0].databaseId')
          break
        fi
        sleep 2
        attempt=$(seal int add "$attempt" 1)
      done
    fi
    gh run watch "$run_id" --interval 10
  fi
fi
"#
}

#[test]
fn release_fixture_roundtrip() {
    let fx = fixture(release_source());
    let sealir = run_transpile(&fx, "seal", "sealir");

    assert!(sealir.status.success());
    let sealir = String::from_utf8(sealir.stdout).expect("stdout should be UTF-8");
    assert!(sealir.contains("argv_parse"));
    assert!(sealir.contains("tool_capture"));
    assert!(sealir.contains("regex"));
    assert!(sealir.contains("json_not_empty"));
    assert!(sealir.contains("int"));
    assert!(sealir.contains("release-stable.yml"));
}

#[test]
fn release_fixture_emits_targets() {
    let fx = fixture(release_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("trigger_output=$(gh workflow run \"$workflow\""));
    assert!(bash.contains(
        "run_id=$(runseal @tool regex capture \"$trigger_output\" '/actions/runs/([0-9]+)' 1)"
    ));
    assert!(bash.contains("attempt=$(runseal @tool int add \"$attempt\" 1)"));
    assert!(powershell.contains("$trigger_output = & 'gh' 'workflow' 'run' $workflow"));
    assert!(powershell.contains("& 'runseal' '@tool' 'regex' 'capture' $trigger_output"));
    assert!(powershell.contains("& 'runseal' '@tool' 'json' 'empty' $raw"));
    super::syntax::assert_bash(&bash);
    super::syntax::assert_pwsh(&powershell);
}
