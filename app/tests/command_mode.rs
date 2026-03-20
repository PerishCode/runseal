use std::process::Command;

use tempfile::TempDir;

fn write_profile(dir: &TempDir) -> String {
    let profile = dir.path().join("cmd-profile.json");
    std::fs::write(
        &profile,
        r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "RUNSEAL_PROFILE": "from-command-mode"
      }
    }
  ]
}"#,
    )
    .expect("profile should be written");
    profile
        .to_str()
        .expect("profile path should be UTF-8")
        .to_string()
}

#[test]
fn command_mode_runs_child_with_exported_envs() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile_path = write_profile(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "-p",
            &profile_path,
            "--log-level",
            "error",
            "--",
            "bash",
            "-lc",
            "printf '%s' \"$RUNSEAL_PROFILE\"",
        ])
        .output()
        .expect("runseal command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "from-command-mode");
}

#[test]
fn command_mode_propagates_child_exit_code() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile_path = write_profile(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "-p",
            &profile_path,
            "--log-level",
            "error",
            "--",
            "bash",
            "-lc",
            "exit 17",
        ])
        .output()
        .expect("runseal command should run");

    assert_eq!(output.status.code(), Some(17));
}

#[test]
fn command_mode_resolves_resource_content_uri() {
    let temp = TempDir::new().expect("temp dir should be created");
    let resource_home = temp.path().join("resources");
    std::fs::create_dir_all(resource_home.join("opencode"))
        .expect("resource directory should be created");
    std::fs::write(
        resource_home.join("opencode/config.json"),
        "{\"default_agent\":\"alpha\"}",
    )
    .expect("resource file should be written");

    let profile = temp.path().join("resource-content-profile.json");
    std::fs::write(
        &profile,
        r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "OPENCODE_CONFIG_CONTENT": "resource-content://opencode/config.json"
      }
    }
  ]
}"#,
    )
    .expect("profile should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "-p",
            profile
                .to_str()
                .expect("profile path should be valid UTF-8"),
            "--log-level",
            "error",
            "--",
            "bash",
            "-lc",
            "printf '%s' \"$OPENCODE_CONFIG_CONTENT\"",
        ])
        .env("RUNSEAL_RESOURCE_HOME", &resource_home)
        .output()
        .expect("runseal command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "{\"default_agent\":\"alpha\"}");
}

#[test]
fn command_mode_honors_strict_duplicate_key_checks() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("strict-profile.json");
    std::fs::write(
        &profile,
        r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "DUP": "from-env"
      }
    },
    {
      "type": "command",
      "program": "bash",
      "args": [
        "-lc",
        "printf 'export DUP=from-command\\n'"
      ]
    }
  ]
}"#,
    )
    .expect("profile should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "-p",
            profile
                .to_str()
                .expect("profile path should be valid UTF-8"),
            "--strict",
            "--log-level",
            "error",
            "--",
            "bash",
            "-lc",
            "printf '%s' should-not-run",
        ])
        .output()
        .expect("runseal command should run");

    assert!(
        !output.status.success(),
        "strict mode should reject duplicate key"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("duplicate exported key"));
}
