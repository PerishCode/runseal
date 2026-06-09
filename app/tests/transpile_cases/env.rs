use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

use super::syntax;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

struct Fixture {
    _temp: TempDir,
    dir: PathBuf,
    source: PathBuf,
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

fn run_transpile(fx: &Fixture, output_lang: &str) -> std::process::Output {
    bin()
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--input-lang=seal")
        .arg("--output-lang")
        .arg(output_lang)
        .arg(&fx.source)
        .output()
        .expect("runseal should run")
}

#[test]
fn env_overlay_emits_targets() {
    let fx = fixture(
        r#"
kubeconfig=/tmp/a.yaml
KUBECONFIG="$kubeconfig" kubectl "$@"
"#,
    );

    let bash = run_transpile(&fx, "bash");
    let powershell = run_transpile(&fx, "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("KUBECONFIG=\"$kubeconfig\" kubectl \"$@\""));
    assert!(powershell.contains("$__seal_old_env_KUBECONFIG = $env:KUBECONFIG"));
    assert!(powershell.contains("$env:KUBECONFIG = $kubeconfig"));
    assert!(powershell.contains("& 'kubectl' @args"));
    syntax::assert_bash(&bash);
    syntax::assert_pwsh(&powershell);
}
