use std::{path::Path, process::Command};

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[cfg(unix)]
fn shell_args(script: &str) -> Vec<String> {
    vec!["bash".into(), "--".into(), "-lc".into(), script.into()]
}

#[cfg(windows)]
fn shell_args(script: &str) -> Vec<String> {
    vec![
        "pwsh".into(),
        "--".into(),
        "-NoProfile".into(),
        "-Command".into(),
        script.into(),
    ]
}

#[cfg(unix)]
fn print_env_script(key: &str) -> String {
    format!("printf '%s' \"${key}\"")
}

#[cfg(windows)]
fn print_env_script(key: &str) -> String {
    format!("[Console]::Write($env:{key})")
}

#[cfg(unix)]
fn print_two_env_script(left: &str, right: &str) -> String {
    format!("printf '%s|%s' \"${left}\" \"${right}\"")
}

#[cfg(windows)]
fn print_two_env_script(left: &str, right: &str) -> String {
    format!("[Console]::Write(\"$env:{left}|$env:{right}\")")
}

struct Fixture {
    _temp: TempDir,
    project: std::path::PathBuf,
    profile: std::path::PathBuf,
    home: std::path::PathBuf,
}

fn fixture(profile_text: &str) -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let profile = project.join("runseal.toml");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&project).expect("project should be created");
    std::fs::write(&profile, profile_text).expect("profile should be written");
    Fixture {
        _temp: temp,
        project,
        profile,
        home,
    }
}

fn resource_profile() -> Fixture {
    fixture(
        r#"
[resources]
root = ".resource"

[[injections]]
type = "env"

[injections.vars]
RUNSEAL_RESOURCE_ROOT_A = "resource://"
RUNSEAL_RESOURCE_ROOT_B = "resource://."
RUNSEAL_RESOURCE_A = "resource://local/ssh/config"

[[injections.ops]]
op = "set"
key = "RUNSEAL_RESOURCE_B"
value = "resource://state/export.json"
"#,
    )
}

fn run_in(fx: &Fixture, args: &[&str]) -> std::process::Output {
    bin()
        .current_dir(&fx.project)
        .env("RUNSEAL_HOME", &fx.home)
        .args(args)
        .output()
        .expect("runseal should run")
}

fn run_profile(fx: &Fixture, args: Vec<String>) -> std::process::Output {
    bin()
        .env("RUNSEAL_HOME", &fx.home)
        .arg("--profile")
        .arg(fx.profile.to_str().expect("path should be UTF-8"))
        .args(args)
        .output()
        .expect("runseal should run")
}

fn assert_resource_root(value: &str) {
    let path = Path::new(value);
    assert!(path.is_absolute(), "expected {value} to be absolute");
    assert!(
        path.ends_with(Path::new(".resource")),
        "unexpected resource root: {}",
        path.display()
    );
}

fn assert_fails(fx: &Fixture, args: &[&str], expected: &str) {
    let output = run_in(fx, args);
    assert!(!output.status.success(), "{args:?} should fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains(expected),
        "expected stderr for {args:?} to contain {expected:?}, got {stderr:?}"
    );
}

#[test]
fn env_values_resolve() {
    let fx = resource_profile();

    let output = run_profile(
        &fx,
        shell_args(&print_env_script("RUNSEAL_RESOURCE_ROOT_A")),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_resource_root(&stdout);

    let output = run_profile(
        &fx,
        shell_args(&print_two_env_script(
            "RUNSEAL_RESOURCE_ROOT_B",
            "RUNSEAL_RESOURCE_A",
        )),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let (left, right) = stdout
        .split_once('|')
        .expect("stdout should include two env values");
    assert_resource_root(left);
    assert!(
        Path::new(right).ends_with(
            Path::new(".resource")
                .join("local")
                .join("ssh")
                .join("config")
        ),
        "unexpected resource path: {right}"
    );

    let output = run_profile(&fx, shell_args(&print_env_script("RUNSEAL_RESOURCE_B")));
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        Path::new(&stdout).ends_with(Path::new(".resource").join("state").join("export.json")),
        "unexpected resource path: {stdout}"
    );
}

#[test]
fn internal_prints_resources() {
    let fx = resource_profile();

    let output = run_in(&fx, &["@resources"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let value = stdout
        .trim()
        .strip_prefix("RUNSEAL_RESOURCE_ROOT=")
        .expect("output should include resource root");
    assert_resource_root(value);

    let output = run_in(&fx, &["@resolve", "resource://"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_resource_root(stdout.trim());

    let output = run_in(&fx, &["@resolve", "resource://local/ssh/config"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        Path::new(stdout.trim()).ends_with(
            Path::new(".resource")
                .join("local")
                .join("ssh")
                .join("config")
        ),
        "unexpected resource path: {stdout}"
    );
}

#[test]
fn root_is_required() {
    let fx = fixture(
        r#"
[[injections]]
type = "env"

[injections.vars]
RUNSEAL_RESOURCE_A = "resource://local/ssh/config"
"#,
    );

    let output = run_profile(&fx, shell_args(&print_env_script("RUNSEAL_RESOURCE_A")));
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("resource root is not configured"));
}

#[test]
fn invalid_uri_fails() {
    let fx = resource_profile();
    for (args, expected) in [
        (
            vec!["@resolve", "local/ssh/config"],
            "expected resource URI to start with resource://",
        ),
        (
            vec!["@resolve", "resource://../secret"],
            "resource URI path must not contain '.' or '..'",
        ),
        (
            vec!["@resolve", "resource://./secret"],
            "resource URI path must not contain '.' or '..'",
        ),
        (
            vec!["@resolve", "resource://local//config"],
            "resource URI path segment must not be empty",
        ),
        (
            vec!["@resolve", "resource://C:/config"],
            "resource URI path segment must not contain ':'",
        ),
    ] {
        assert_fails(&fx, &args, expected);
    }
}
