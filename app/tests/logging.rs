use std::process::Command;

use tempfile::TempDir;

fn helper_alias_template() -> String {
    format!("{}/../helpers/{{name}}.sh", env!("CARGO_MANIFEST_DIR"))
}

fn runseal_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command.env("RUNSEAL_HELPER_ALIAS_TEMPLATE", helper_alias_template());
    command
}

#[test]
fn logs_go_to_stderr_and_exports_stay_on_stdout() {
    let output = runseal_command()
        .args(["-p", "examples/runseal.sample.json", "--log-level", "info"])
        .env_remove("RUST_LOG")
        .output()
        .expect("runseal command should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(stdout.contains("export RUNSEAL_PROFILE='dev'"));
    assert!(stderr.contains("runseal run started"));
    assert!(!stdout.contains("runseal run started"));
}

#[test]
fn helper_node_writes_per_invocation_log_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let log_home = temp.path().join("logs");

    std::fs::create_dir_all(&log_home).expect("log dir should be created");

    let before = std::fs::read_dir(&log_home)
        .expect("log dir should exist")
        .count();

    let preview = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .env("RUNSEAL_LOG_HOME", &log_home)
        .output()
        .expect("install command should run");
    assert!(preview.status.success());

    let entries: Vec<_> = std::fs::read_dir(&log_home)
        .expect("log dir should be readable")
        .map(|entry| entry.expect("entry should be readable").path())
        .collect();
    assert!(entries.len() > before);
    let newest = entries
        .iter()
        .max_by_key(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
        .expect("at least one log file should exist");
    let contents = std::fs::read_to_string(newest).expect("log file should be readable");
    assert!(contents.contains("helper execution starting") || contents.contains("method=install"));
    assert!(contents.contains("node_mirror=") || contents.contains("downloaded node runtime"));
    assert!(
        contents.contains("dirs prepared node_home=") || contents.contains("lock acquired dir=")
    );
    assert!(
        contents.contains("patch emitted env_count=6")
            || contents.contains("patch emitted env_count=8")
    );
}

#[test]
fn helper_node_failure_prints_log_path_and_writes_failure_trail() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let log_home = temp.path().join("logs");

    std::fs::create_dir_all(&log_home).expect("log dir should be created");

    let preview = runseal_command()
        .args([
            "helper",
            ":node",
            "install",
            "--node-version",
            "not-a-version",
        ])
        .env("RUNSEAL_HELPER_NODE_MIRROR", "https://nodejs.org/dist")
        .env("RUNSEAL_HOME", &runseal_home)
        .env("RUNSEAL_LOG_HOME", &log_home)
        .output()
        .expect("install command should run");
    assert!(!preview.status.success());

    let stderr = String::from_utf8(preview.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("See log:"));

    let entries: Vec<_> = std::fs::read_dir(&log_home)
        .expect("log dir should be readable")
        .map(|entry| entry.expect("entry should be readable").path())
        .collect();
    let newest = entries
        .iter()
        .max_by_key(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
        .expect("at least one log file should exist");
    let contents = std::fs::read_to_string(newest).expect("log file should be readable");
    assert!(
        contents.contains("helper execution failed")
            || contents.contains("failed to fetch helper script")
            || contents.contains("download node runtime")
    );
    assert!(contents.contains("runseal invocation failed"));
    assert!(contents.contains("runseal invocation failed"));
}

#[test]
fn unwritable_log_home_disables_file_logging_without_failing_command() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let log_home = temp.path().join("logs-file");
    let profile = runseal_home.join("profiles/default.json");

    std::fs::create_dir_all(profile.parent().expect("profile dir should exist"))
        .expect("profile dir should be created");
    std::fs::write(&log_home, "not a directory").expect("log path sentinel should be written");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"default"}}]}"#,
    )
    .expect("profile should be written");

    let output = runseal_command()
        .args(["preview", "-p", profile.to_str().unwrap()])
        .env("RUNSEAL_HOME", &runseal_home)
        .env("RUNSEAL_LOG_HOME", &log_home)
        .output()
        .expect("preview command should run");

    assert!(
        output.status.success(),
        "preview should continue without file logging, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("Warning: session file logging disabled"));
    assert!(
        stderr.contains("failed to create log directory")
            || stderr.contains("failed to open session log file")
    );
    assert!(!stderr.contains("See log:"));
    assert!(log_home.is_file());
}
