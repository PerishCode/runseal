#![cfg(unix)]

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    project: PathBuf,
    bin: PathBuf,
}

fn fixture() -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(project.join(".runseal/wrappers"))
        .expect("wrapper dir should be created");
    std::fs::create_dir_all(project.join("app/tests")).expect("app tests dir should be created");
    std::fs::create_dir_all(&bin).expect("bin dir should be created");

    std::fs::write(project.join("runseal.toml"), "injections = []\n")
        .expect("profile should be written");
    std::fs::write(
        project.join(".runseal/wrappers/guard.seal"),
        std::fs::read_to_string(repo_root().join(".runseal/wrappers/guard.seal"))
            .expect("repo guard seal should be readable"),
    )
    .expect("guard seal should be copied");
    std::fs::write(project.join("app/tests/sample.txt"), "sample\n")
        .expect("sample test file should be written");

    write_executable(
        &bin.join("git"),
        r#"#!/usr/bin/env sh
set -eu
if [ "${1:-}" = "rev-parse" ] && [ "${2:-}" = "--show-toplevel" ]; then
  printf '%s\n' "${RUNSEAL_TEST_ROOT:?}"
  exit 0
fi
exit 0
"#,
    );
    write_executable(
        &bin.join("cargo"),
        r#"#!/usr/bin/env sh
set -eu
if [ "${1:-}" = "metadata" ]; then
  if [ -n "${RUNSEAL_TEST_CARGO_METADATA:-}" ]; then
    printf '%s\n' "$RUNSEAL_TEST_CARGO_METADATA"
  else
    printf '%s\n' '{"packages":[{"version":"0.6.1"}]}'
  fi
  exit 0
fi
exit 0
"#,
    );
    write_executable(
        &bin.join("curl"),
        r#"#!/usr/bin/env sh
set -eu
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      out="$2"
      shift 2
      ;;
    -w)
      shift 2
      ;;
    -s|-S)
      shift
      ;;
    *)
      shift
      ;;
  esac
done
if [ -n "${RUNSEAL_TEST_CURL_BODY:-}" ] && [ -n "$out" ]; then
  printf '%s' "$RUNSEAL_TEST_CURL_BODY" > "$out"
fi
printf '%s' "${RUNSEAL_TEST_CURL_STATUS:-404}"
"#,
    );
    write_executable(
        &bin.join("cat"),
        r#"#!/usr/bin/env sh
set -eu
/bin/cat "$@"
"#,
    );

    Fixture {
        _temp: temp,
        project,
        bin,
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

fn write_executable(path: &Path, content: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, content).expect("stub should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("stub metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("stub should be executable");
}

fn prepend_path(first: &Path) -> OsString {
    let mut paths = vec![first.to_path_buf()];
    if let Some(runseal_dir) = Path::new(env!("CARGO_BIN_EXE_runseal")).parent() {
        paths.push(runseal_dir.to_path_buf());
    }
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).expect("PATH should be joinable")
}

fn run_guard(fx: &Fixture, args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command
        .current_dir(&fx.project)
        .env("PATH", prepend_path(&fx.bin))
        .env("RUNSEAL_TEST_ROOT", &fx.project)
        .arg("-p")
        .arg(fx.project.join("runseal.toml"))
        .arg(":guard")
        .args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("guard should run")
}

#[test]
fn version_hash() {
    let fx = fixture();

    let wrapper = run_guard(&fx, &["version-hash"], &[]);
    assert!(
        wrapper.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&wrapper.stderr)
    );

    let tool = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .current_dir(&fx.project)
        .env("PATH", prepend_path(&fx.bin))
        .env("RUNSEAL_HOME", fx.project.join(".home"))
        .args(["@tool", "hash", "tree", "app/tests"])
        .output()
        .expect("hash tool should run");
    assert!(tool.status.success());

    assert_eq!(wrapper.stdout, tool.stdout);
}

#[test]
fn skip_no_stable() {
    let fx = fixture();

    let output = run_guard(
        &fx,
        &["version-check"],
        &[("RUNSEAL_TEST_CURL_STATUS", "404")],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("guard version policy: no stable metadata; skipping")
    );
}

#[test]
fn reject_hash_change_patch() {
    let fx = fixture();

    let output = run_guard(
        &fx,
        &["version-check"],
        &[
            (
                "RUNSEAL_TEST_CARGO_METADATA",
                r#"{"packages":[{"version":"0.6.1"}]}"#,
            ),
            ("RUNSEAL_TEST_CURL_STATUS", "200"),
            (
                "RUNSEAL_TEST_CURL_BODY",
                r#"{"stableVersion":"0.6.0","guard":{"version":{"hash":"different"}}}"#,
            ),
        ],
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("changed guard.version.hash requires a minor-or-higher bump")
    );
}
