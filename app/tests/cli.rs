use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn no_command_prints_help() {
    let output = bin().output().expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--profile"));
}

#[test]
fn explicit_toml_profile_runs_command_with_env_and_context() {
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
        .args([
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
            "bash",
            "--",
            "-lc",
            "printf '%s|%s' \"$RUNSEAL_TEST_VALUE\" \"$(basename \"$RUNSEAL_PROFILE_PATH\")\"",
        ])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "from-toml|profile.toml");
}

#[test]
fn cwd_profile_beats_runseal_home_default() {
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
        .args(["bash", "--", "-lc", "printf '%s' \"$PICKED\""])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "cwd");
}

#[test]
fn symlink_is_available_during_command_and_cleaned_afterward() {
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
        .args([
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
            "bash",
            "--",
            "-lc",
            &format!("test -L {}", target.display()),
        ])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    assert!(!target.exists(), "symlink should be cleaned after command");
}

#[test]
fn propagates_child_exit_code() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("profile.json");
    std::fs::write(&profile, r#"{"injections":[]}"#).expect("profile should be written");

    let output = bin()
        .args([
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
            "bash",
            "--",
            "-lc",
            "exit 17",
        ])
        .output()
        .expect("runseal should run");

    assert_eq!(output.status.code(), Some(17));
}
