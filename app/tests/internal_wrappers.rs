use std::{
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[cfg(unix)]
fn wrapper_file(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.sh"))
}

#[cfg(windows)]
fn wrapper_file(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.cmd"))
}

#[cfg(unix)]
fn make_wrapper(path: &Path, label: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, format!("#!/usr/bin/env sh\nprintf '{}'\n", label))
        .expect("wrapper should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("wrapper metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("wrapper should be executable");
}

#[cfg(windows)]
fn make_wrapper(path: &Path, label: &str) {
    std::fs::write(
        path,
        format!("@echo off\r\n<nul set /p=\"{}\"\r\nexit /b 0\r\n", label),
    )
    .expect("wrapper should be written");
}

fn make_seal_wrapper(path: &Path, source: &str) {
    std::fs::write(path, source).expect("seal wrapper should be written");
}

struct Fixture {
    _temp: TempDir,
    project: PathBuf,
    home: PathBuf,
    project_wrappers: PathBuf,
    home_wrappers: PathBuf,
}

fn fixture() -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    let project_wrappers = project.join(".runseal").join("wrappers");
    let home_wrappers = home.join("wrappers");
    std::fs::create_dir_all(&project_wrappers).expect("project wrappers should be created");
    std::fs::create_dir_all(&home_wrappers).expect("home wrappers should be created");
    std::fs::write(
        project.join("runseal.toml"),
        "injections = []\n[resources]\nroot = \".resource\"\n",
    )
    .expect("profile should be written");
    Fixture {
        _temp: temp,
        project,
        home,
        project_wrappers,
        home_wrappers,
    }
}

fn run_in(fx: &Fixture, args: &[&str]) -> std::process::Output {
    bin()
        .current_dir(&fx.project)
        .env("RUNSEAL_HOME", &fx.home)
        .args(args)
        .output()
        .expect("runseal should run")
}

fn path_suffix(path: &Path, count: usize) -> PathBuf {
    path.components()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn assert_path_ends_with(actual: &str, expected: &Path) {
    let expected_suffix = path_suffix(expected, 4);
    assert!(
        Path::new(actual).ends_with(&expected_suffix),
        "expected {actual:?} to end with {}",
        expected_suffix.display()
    );
}

#[test]
fn wrappers_show_effective() {
    let fx = fixture();
    make_wrapper(&wrapper_file(&fx.project_wrappers, "wrap"), "project");
    make_wrapper(&wrapper_file(&fx.home_wrappers, "wrap"), "home");
    make_wrapper(&wrapper_file(&fx.home_wrappers, "home-only"), "home");

    let output = run_in(&fx, &["@wrappers"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains(":wrap"));
    assert!(stdout.contains(":home-only"));
    assert!(stdout.contains("profile"));
    assert!(stdout.contains("home"));
    let wrap_line = stdout
        .lines()
        .find(|line| line.contains(":wrap"))
        .expect("wrap should be listed");
    let wrap_file = wrap_line
        .split_whitespace()
        .last()
        .expect("wrap line should include a file");
    assert!(wrap_line.contains("profile"));
    assert!(
        std::path::Path::new(wrap_file)
            .ends_with(path_suffix(&wrapper_file(&fx.project_wrappers, "wrap"), 4)),
        "expected {wrap_file} to point at the profile wrapper"
    );
}

#[test]
fn seal_wrapper_resolves() {
    let fx = fixture();
    let wrapper = fx.project_wrappers.join("seal-tool.seal");
    make_seal_wrapper(&wrapper, "print seal\n");

    let which = run_in(&fx, &["@which", ":seal-tool"]);
    assert!(which.status.success());
    let stdout = String::from_utf8(which.stdout).expect("stdout should be UTF-8");
    assert_path_ends_with(stdout.trim(), &wrapper);

    let wrappers = run_in(&fx, &["@wrappers"]);
    assert!(wrappers.status.success());
    let stdout = String::from_utf8(wrappers.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains(":seal-tool"));
    assert!(stdout.contains("seal-tool.seal"));
}

#[test]
#[cfg(unix)]
fn extensionless_is_ignored() {
    let fx = fixture();
    make_wrapper(&fx.project_wrappers.join("legacy"), "legacy");

    let output = run_in(&fx, &[":legacy"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("wrapper not found: :legacy"));
    assert!(stderr.contains("legacy.seal"));
    assert!(stderr.contains("legacy.sh"));
    assert!(!stderr.contains(".runseal/wrappers/legacy\n"));
}

#[test]
fn seal_wrapper_runs_directly() {
    let fx = fixture();
    make_seal_wrapper(
        &fx.project_wrappers.join("seal-tool.seal"),
        r#"
__seal_argc=$#
__seal_help=false
name=world
loud=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --name)
      if [ "$#" -lt 2 ]; then fail "missing value for --name"; fi
      name=$2
      shift 2
      ;;
    --name=*)
      name=${1#--name=}
      shift
      ;;
    --loud)
      loud=true
      shift
      ;;
    --)
      shift
      break
      ;;
    -h|--help|help)
      __seal_help=true
      shift
      ;;
    *) fail "unknown option: $1" ;;
  esac
done
if [ "$__seal_argc" = 0 ]; then
  print "hello $name"
else
  if [ "$loud" = true ]; then
    print "HELLO $name from ${RUNSEAL_WRAPPER_NAME}"
  else
    print "hello $name"
  fi
fi
"#,
    );

    let output = run_in(&fx, &[":seal-tool", "--name", "seal", "--loud"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "HELLO seal from seal-tool\n");
}

#[test]
fn seal_wrapper_shadows() {
    let fx = fixture();
    make_wrapper(&wrapper_file(&fx.project_wrappers, "tool"), "shell");
    make_seal_wrapper(&fx.project_wrappers.join("tool.seal"), "print seal\n");

    let output = run_in(&fx, &[":tool"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "seal\n");
}

#[test]
fn seal_env_overlay() {
    let fx = fixture();
    make_seal_wrapper(
        &fx.project_wrappers.join("env-tool.seal"),
        r#"
RUNSEAL_MARKER=sealed sh -c 'printf %s "$RUNSEAL_MARKER"'
"#,
    );

    let output = run_in(&fx, &[":env-tool"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "sealed");
}

#[test]
fn wrappers_hide_shadow() {
    let fx = fixture();
    let project_wrapper = wrapper_file(&fx.project_wrappers, "wrap");
    make_wrapper(&project_wrapper, "project");
    make_wrapper(&wrapper_file(&fx.home_wrappers, "wrap"), "home");

    let which = run_in(&fx, &["@which", ":wrap"]);
    assert!(which.status.success());
    let stdout = String::from_utf8(which.stdout).expect("stdout should be UTF-8");
    assert_path_ends_with(stdout.trim(), &project_wrapper);

    let output = run_in(&fx, &["@wrappers"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let wrap_lines = stdout
        .lines()
        .filter(|line| line.starts_with(":wrap "))
        .collect::<Vec<_>>();
    assert_eq!(wrap_lines.len(), 1);
    assert!(wrap_lines[0].contains("profile"));
}
