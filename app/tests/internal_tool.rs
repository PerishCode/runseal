use std::{path::PathBuf, process::Command};

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn tool_runs_without_profile() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for (args, expected) in [
        (
            vec![
                "@tool",
                "json",
                "get",
                r#"[{"databaseId":123}]"#,
                ".[0].databaseId",
            ],
            "123\n",
        ),
        (vec!["@tool", "string", "trim", "  value  "], "value\n"),
        (
            vec![
                "@tool",
                "regex",
                "capture",
                "https://github.test/actions/runs/456",
                "/actions/runs/([0-9]+)",
                "1",
            ],
            "456\n",
        ),
        (vec!["@tool", "int", "add", "2", "3"], "5\n"),
        (
            vec!["@tool", "process", "exists", "definitely-not-runseal-tool"],
            "false\n",
        ),
    ] {
        let output = bin()
            .current_dir(&cwd)
            .env("RUNSEAL_HOME", temp.path().join("home"))
            .args(args.clone())
            .output()
            .expect("runseal should run");

        assert!(output.status.success(), "{args:?} should succeed");
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert_eq!(stdout, expected, "{args:?} stdout should match");
    }
}

#[test]
fn fs_runs_without_profile() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let file = cwd.join("hook");
    let nested = cwd.join("nested");

    let mkdir = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "fs", "mkdir", nested.to_str().unwrap(), "700"])
        .output()
        .expect("runseal should run");
    assert!(mkdir.status.success());
    assert!(nested.is_dir());

    let write = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "fs",
            "write-base64",
            file.to_str().unwrap(),
            "c2VhbCBtYXJrZXIK",
        ])
        .output()
        .expect("runseal should run");
    assert!(write.status.success());
    assert_eq!(
        std::fs::read_to_string(&file).expect("file should be readable"),
        "seal marker\n"
    );

    let contains = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "fs",
            "contains-any",
            file.to_str().unwrap(),
            "missing",
            "seal marker",
        ])
        .output()
        .expect("runseal should run");
    assert!(contains.status.success());
    assert_eq!(String::from_utf8(contains.stdout).unwrap(), "true\n");

    let backup = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "fs", "backup-numbered", file.to_str().unwrap()])
        .output()
        .expect("runseal should run");
    assert!(backup.status.success());
    assert!(!file.exists());
    let backup_path = PathBuf::from(String::from_utf8(backup.stdout).unwrap().trim());
    assert!(backup_path.is_file());
}
