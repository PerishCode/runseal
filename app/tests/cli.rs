use std::process::Command;

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
fn explicit_profile_script() -> String {
    "printf '%s|%s' \"$RUNSEAL_TEST_VALUE\" \"$(basename \"$RUNSEAL_PROFILE_PATH\")\"".into()
}

#[cfg(windows)]
fn explicit_profile_script() -> String {
    "[Console]::Write(\"$env:RUNSEAL_TEST_VALUE|$(Split-Path -Leaf $env:RUNSEAL_PROFILE_PATH)\")"
        .into()
}

#[cfg(unix)]
fn symlink_check_script(path: &std::path::Path) -> String {
    format!("test -L {}", path.display())
}

#[test]
fn help_without_command() {
    let output = bin().output().expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--profile"));
}

#[test]
fn explicit_profile_runs() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("profile.toml");
    std::fs::write(
        &profile,
        r#"
[[injections]]
type = "env"

[injections.vars]
RUNSEAL_TEST_VALUE = "from-toml"
"#,
    )
    .expect("profile should be written");

    let output = bin()
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .arg("--profile")
        .arg(profile.to_str().expect("path should be UTF-8"))
        .args(shell_args(&explicit_profile_script()))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "from-toml|profile.toml");
}

#[test]
fn cwd_beats_home() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("work");
    let runseal_home = temp.path().join("home");
    let profile_home = runseal_home.join("profiles");
    std::fs::create_dir_all(&cwd).expect("cwd should be created");
    std::fs::create_dir_all(&profile_home).expect("profile home should be created");
    std::fs::write(
        cwd.join("runseal.yaml"),
        "injections:\n  - type: env\n    vars:\n      PICKED: cwd\n",
    )
    .expect("cwd profile should be written");
    std::fs::write(
        profile_home.join("default.toml"),
        "[[injections]]\ntype = \"env\"\n[injections.vars]\nPICKED = \"home\"\n",
    )
    .expect("default profile should be written");

    let output = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", &runseal_home)
        .args(shell_args(&print_env_script("PICKED")))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "cwd");
}

#[cfg(unix)]
#[test]
fn symlink_lifecycle() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source.txt");
    let target = temp.path().join("links/source.txt");
    let profile = temp.path().join("profile.json");
    std::fs::write(&source, "sealed").expect("source should be written");
    std::fs::write(
        &profile,
        format!(
            r#"{{
  "injections": [
    {{
      "type": "symlink",
      "source": "{}",
      "target": "{}",
      "cleanup": true
    }}
  ]
}}"#,
            source.display(),
            target.display()
        ),
    )
    .expect("profile should be written");

    let output = bin()
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .arg("--profile")
        .arg(profile.to_str().expect("path should be UTF-8"))
        .args(shell_args(&symlink_check_script(&target)))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    assert!(!target.exists(), "symlink should be cleaned after command");
}

#[test]
fn child_exit_code() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("profile.json");
    std::fs::write(&profile, r#"{"injections":[]}"#).expect("profile should be written");

    let output = bin()
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .arg("--profile")
        .arg(profile.to_str().expect("path should be UTF-8"))
        .args(shell_args("exit 17"))
        .output()
        .expect("runseal should run");

    assert_eq!(output.status.code(), Some(17));
}

#[test]
fn toml_beats_yaml() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("work");
    std::fs::create_dir_all(&cwd).expect("cwd should be created");
    std::fs::write(
        cwd.join("runseal.toml"),
        "[[injections]]\ntype = \"env\"\n[injections.vars]\nPICKED = \"toml\"\n",
    )
    .expect("toml profile should be written");
    std::fs::write(
        cwd.join("runseal.yaml"),
        "injections:\n  - type: env\n    vars:\n      PICKED: yaml\n",
    )
    .expect("yaml profile should be written");

    let output = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(shell_args(&print_env_script("PICKED")))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "toml");
}

#[test]
fn missing_profile_paths() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("work");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&cwd).expect("cwd should be created");

    let output = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", &home)
        .args(shell_args("true"))
        .output()
        .expect("runseal should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("runseal.toml"));
    assert!(stderr.contains("default.json"));
}
