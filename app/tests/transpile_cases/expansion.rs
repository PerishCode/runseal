use std::process::Command;

use tempfile::TempDir;

use super::syntax;

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

fn expansion_source() -> &'static str {
    r#"
channel=${RUNSEAL_CHANNEL:-stable}
required=${RUNSEAL_TOKEN:?missing token}
target=${1:-origin}
branch=${2:?missing branch}
print "$channel $required $target $branch"
"#
}

#[test]
fn forms_roundtrip() {
    let fx = fixture(expansion_source());

    let sealir = run_transpile(&fx, "seal", "sealir");
    assert!(sealir.status.success());
    let sealir = String::from_utf8(sealir.stdout).expect("stdout should be UTF-8");
    assert!(sealir.contains("require_non_empty"));
    assert!(sealir.contains("default_if_unset_or_empty"));

    let bash = run_transpile(&fx, "seal", "bash");
    assert!(bash.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("channel=\"${RUNSEAL_CHANNEL:-stable}\""));
    assert!(bash.contains("required=\"${RUNSEAL_TOKEN:?missing token}\""));
    assert!(bash.contains("target=\"${1:-origin}\""));
    assert!(bash.contains("branch=\"${2:?missing branch}\""));
    syntax::assert_bash(&bash);

    let powershell = run_transpile(&fx, "seal", "powershell");
    assert!(powershell.status.success());
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(powershell.contains(
        "$channel = $(if ([string]::IsNullOrEmpty($env:RUNSEAL_CHANNEL)) { 'stable' } else { $env:RUNSEAL_CHANNEL })"
    ));
    assert!(powershell.contains(
        "$required = $(if ([string]::IsNullOrEmpty($env:RUNSEAL_TOKEN)) { throw 'missing token' } else { $env:RUNSEAL_TOKEN })"
    ));
    assert!(powershell.contains(
        "$target = $(if (($args.Count -lt 1) -or [string]::IsNullOrEmpty($1)) { 'origin' } else { $1 })"
    ));
    assert!(powershell.contains(
        "$branch = $(if (($args.Count -lt 2) -or [string]::IsNullOrEmpty($2)) { throw 'missing branch' } else { $2 })"
    ));
    syntax::assert_pwsh(&powershell);

    let powershell_fx = fixture(&powershell);
    let roundtrip = run_transpile(&powershell_fx, "powershell", "sealir");
    assert!(roundtrip.status.success());
    let roundtrip = String::from_utf8(roundtrip.stdout).expect("stdout should be UTF-8");
    assert!(roundtrip.contains("require_non_empty"));
    assert!(roundtrip.contains("default_if_unset_or_empty"));
}

fn run_wrapper(source: &str, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let wrappers = project.join(".runseal").join("wrappers");
    std::fs::create_dir_all(&wrappers).expect("wrappers should be created");
    std::fs::write(project.join("runseal.toml"), "injections = []\n")
        .expect("profile should be written");
    std::fs::write(wrappers.join("expansion.seal"), source).expect("wrapper should be written");
    let mut command = bin();
    command.current_dir(&project).arg(":expansion").args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("runseal should run")
}

#[test]
fn env_default_runtime() {
    let source = "print \"${RUNSEAL_CHANNEL:-stable}\"\n";
    let missing = run_wrapper(source, &[], &[]);
    assert!(missing.status.success());
    assert_eq!(
        String::from_utf8(missing.stdout).expect("stdout should be UTF-8"),
        "stable\n"
    );
    let empty = run_wrapper(source, &[], &[("RUNSEAL_CHANNEL", "")]);
    assert!(empty.status.success());
    assert_eq!(
        String::from_utf8(empty.stdout).expect("stdout should be UTF-8"),
        "stable\n"
    );
}

#[test]
fn positional_default_runtime() {
    let source = "print \"${1:-origin}\"\n";
    let missing = run_wrapper(source, &[], &[]);
    assert!(missing.status.success());
    assert_eq!(
        String::from_utf8(missing.stdout).expect("stdout should be UTF-8"),
        "origin\n"
    );
    let empty = run_wrapper(source, &[""], &[]);
    assert!(empty.status.success());
    assert_eq!(
        String::from_utf8(empty.stdout).expect("stdout should be UTF-8"),
        "origin\n"
    );
}

#[test]
fn env_require_runtime() {
    let source = "print \"${RUNSEAL_TOKEN:?missing token}\"\n";
    let missing = run_wrapper(source, &[], &[]);
    assert!(!missing.status.success());
    assert!(
        String::from_utf8(missing.stderr)
            .expect("stderr should be UTF-8")
            .contains("missing token")
    );
    let empty = run_wrapper(source, &[], &[("RUNSEAL_TOKEN", "")]);
    assert!(!empty.status.success());
    assert!(
        String::from_utf8(empty.stderr)
            .expect("stderr should be UTF-8")
            .contains("missing token")
    );
}

#[test]
fn positional_require_runtime() {
    let source = "print \"${1:?missing branch}\"\n";
    let missing = run_wrapper(source, &[], &[]);
    assert!(!missing.status.success());
    assert!(
        String::from_utf8(missing.stderr)
            .expect("stderr should be UTF-8")
            .contains("missing branch")
    );
    let empty = run_wrapper(source, &[""], &[]);
    assert!(!empty.status.success());
    assert!(
        String::from_utf8(empty.stderr)
            .expect("stderr should be UTF-8")
            .contains("missing branch")
    );
}
