use std::process::Command;

use tempfile::TempDir;

#[test]
fn alias_append_and_list_work() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    std::fs::create_dir_all(runseal_home.join("profiles"))
        .expect("profiles directory should be created");

    let profile = runseal_home.join("profiles/work.json");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"work"}}]}"#,
    )
    .expect("profile should be written");

    let append = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "alias",
            "append",
            "work",
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");
    assert!(append.status.success());

    let list = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args(["alias", "list"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");
    assert!(list.status.success());

    let stdout = String::from_utf8(list.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("work ->"));
}

#[test]
fn shortcut_alias_runs_profile() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    std::fs::create_dir_all(runseal_home.join("profiles"))
        .expect("profiles directory should be created");

    let profile = runseal_home.join("profiles/work.json");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"from-alias"}}]}"#,
    )
    .expect("profile should be written");

    let append = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "alias",
            "append",
            "work",
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");
    assert!(append.status.success());

    let run = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([":work"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");

    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("export RUNSEAL_PROFILE='from-alias'"));
}

#[test]
fn alias_run_executes_explicit_entrypoint() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    std::fs::create_dir_all(runseal_home.join("profiles"))
        .expect("profiles directory should be created");

    let profile = runseal_home.join("profiles/work.json");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"from-run"}}]}"#,
    )
    .expect("profile should be written");

    let append = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "alias",
            "append",
            "work",
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");
    assert!(append.status.success());

    let run = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args(["alias", "run", "work"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");

    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("export RUNSEAL_PROFILE='from-run'"));
}

#[test]
fn bare_alias_name_is_not_supported() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    std::fs::create_dir_all(runseal_home.join("profiles"))
        .expect("profiles directory should be created");

    let profile = runseal_home.join("profiles/work.json");
    let default_profile = runseal_home.join("profiles/default.json");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"from-run"}}]}"#,
    )
    .expect("profile should be written");
    std::fs::write(
        &default_profile,
        r#"{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"default"}}]}"#,
    )
    .expect("default profile should be written");

    let append = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "alias",
            "append",
            "work",
            "--profile",
            profile.to_str().expect("path should be UTF-8"),
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");
    assert!(append.status.success());

    let run = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args(["work"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");

    assert!(!run.status.success());
    let stderr = String::from_utf8(run.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("failed to execute child command"));
}
