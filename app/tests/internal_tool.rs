#[path = "internal_tool/archive.rs"]
#[cfg(unix)]
mod archive;
#[path = "internal_tool/gitee.rs"]
mod gitee;
#[path = "internal_tool/github.rs"]
#[cfg(unix)]
mod github;
#[path = "internal_tool/hash_version.rs"]
mod hash_version;
#[path = "internal_tool/process.rs"]
mod process;
#[path = "internal_tool/ssh.rs"]
mod ssh;
#[path = "internal_tool/string.rs"]
mod string;

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

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
            vec!["@tool", "json", "pretty", "value", r#"{"a":1}"#],
            "{\n  \"a\": 1\n}\n",
        ),
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
        (
            vec![
                "@tool",
                "json",
                "has",
                r#"{"guard":{"version":{"hash":"x"}}}"#,
                ".guard.version.hash",
            ],
            "true\n",
        ),
        (vec!["@tool", "string", "trim", "  value  "], "value\n"),
        (
            vec![
                "@tool",
                "string",
                "slug",
                "Land Change: ship .seal!",
                "--max-len",
                "48",
                "--fallback",
                "change",
            ],
            "land-change-ship-seal\n",
        ),
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
fn tool_help_is_progressive() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for (args, expected) in [
        (
            vec!["@tool", "json", "--help"],
            "Usage: runseal @tool json <command> [args]",
        ),
        (
            vec!["@tool", "json", "get", "--help"],
            "Usage: runseal @tool json get <json> <path>",
        ),
        (
            vec!["@tool", "json", "has", "--help"],
            "Usage: runseal @tool json has <json> <path>",
        ),
        (
            vec!["@tool", "json", "pretty", "--help"],
            "Usage: runseal @tool json pretty <mode> [args]",
        ),
        (
            vec!["@tool", "json", "pretty", "value", "--help"],
            "Usage: runseal @tool json pretty value <json>",
        ),
        (
            vec!["@tool", "json", "pretty", "stdin", "--help"],
            "Usage: runseal @tool json pretty stdin",
        ),
        (
            vec!["@tool", "json", "pretty", "file", "--help"],
            "Usage: runseal @tool json pretty file <input> <output>",
        ),
        (
            vec!["@tool", "string", "--help"],
            "Usage: runseal @tool string <command> [args]",
        ),
        (
            vec!["@tool", "archive", "local", "--help"],
            "Usage: runseal @tool archive local <command> [args]",
        ),
        (
            vec!["@tool", "cloudflare", "config", "--help"],
            "Cloudflare config helpers:",
        ),
        (
            vec!["@tool", "ssh", "script", "--help"],
            "Usage: runseal @tool ssh script <command> [options] -- [args...]",
        ),
        (
            vec!["@tool", "cloudflare", "zone", "dns-record", "--help"],
            "Usage: runseal @tool cloudflare zone dns-record <command> [args]",
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
        assert!(
            stdout.contains(expected),
            "{args:?} should contain {expected:?}, got {stdout:?}"
        );
    }
}

#[test]
fn richer_help() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for (args, expected) in [
        (
            vec!["@tool", "ssh", "config", "--help"],
            "identities --config <path> [--base <path>]",
        ),
        (
            vec!["@tool", "ssh", "config", "identities", "--help"],
            "Relative IdentityFile paths resolve from `--base`",
        ),
        (
            vec!["@tool", "ssh", "script", "capture", "--help"],
            "Run one local script on the SSH host and print stdout to the caller.",
        ),
        (
            vec!["@tool", "cloudflare", "zone", "--help"],
            "Cloudflare zone helpers:",
        ),
        (
            vec!["@tool", "cloudflare", "zone", "ruleset", "--help"],
            "Cloudflare zone ruleset helpers:",
        ),
        (
            vec![
                "@tool",
                "cloudflare",
                "zone",
                "dns-record",
                "update",
                "--help",
            ],
            "--record-id <id> --json <json>",
        ),
        (
            vec!["@tool", "fs", "list", "--help"],
            "[--require-nonempty]",
        ),
        (
            vec!["@tool", "gitee", "repo", "--help"],
            "Gitee repo helpers:",
        ),
        (
            vec!["@tool", "cloudflare", "redirect-rule", "exact", "--help"],
            "Build one exact-match redirect rule payload as JSON.",
        ),
        (
            vec!["@tool", "github", "issue", "comment", "create", "--help"],
            "--prefix-enable=<true|false>",
        ),
        (
            vec!["@tool", "gitee", "pr", "merge", "--help"],
            "Merge a Gitee pull request and print the API response JSON.",
        ),
        (
            vec!["@tool", "archive", "local", "import", "--help"],
            "Decrypt one .local-style directory archive into the source directory.",
        ),
        (
            vec!["@tool", "process", "write", "--help"],
            "<stdout|stderr> <path> [--append] -- <command> [args...]",
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
        assert!(
            stdout.contains(expected),
            "{args:?} should contain {expected:?}, got {stdout:?}"
        );
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

    let text_file = cwd.join("text");
    let write_text = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "fs",
            "write",
            text_file.to_str().unwrap(),
            "plain text",
            "600",
        ])
        .output()
        .expect("runseal should run");
    assert!(write_text.status.success());
    assert_eq!(
        std::fs::read_to_string(&text_file).expect("file should be readable"),
        "plain text"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&text_file)
                .expect("metadata should be readable")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

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

#[test]
fn fs_mode_touch_list() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let dir = cwd.join("kube");
    let first = dir.join("b.yaml");
    let second = dir.join("a.yaml");
    let ignored = dir.join("notes.txt");

    let touch = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "fs", "touch", first.to_str().unwrap(), "600"])
        .output()
        .expect("runseal should run");
    assert!(touch.status.success());
    std::fs::write(&first, "existing").expect("file should be writable");

    let touch_again = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "fs", "touch", first.to_str().unwrap(), "600"])
        .output()
        .expect("runseal should run");
    assert!(touch_again.status.success());
    assert_eq!(
        std::fs::read_to_string(&first).expect("file should be readable"),
        "existing"
    );

    std::fs::write(&second, "").expect("file should be written");
    std::fs::write(&ignored, "").expect("file should be written");

    let mode = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "fs", "mode", first.to_str().unwrap()])
        .output()
        .expect("runseal should run");
    assert!(mode.status.success());
    #[cfg(unix)]
    assert_eq!(String::from_utf8(mode.stdout).unwrap(), "600\n");

    let list = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "fs",
            "list",
            dir.to_str().unwrap(),
            "--glob",
            "*.yaml",
            "--files",
            "--require-nonempty",
        ])
        .output()
        .expect("runseal should run");
    assert!(list.status.success());
    let paths: Vec<String> =
        serde_json::from_slice(&list.stdout).expect("stdout should be JSON array");
    assert_eq!(paths.len(), 2);
    assert!(PathBuf::from(&paths[0]).is_absolute());
    assert!(paths[0].ends_with("a.yaml"));
    assert!(paths[1].ends_with("b.yaml"));
}

#[test]
fn json_pretty_stdin() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    let mut child = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "json", "pretty", "stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("runseal should start");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(br#"{"a":1,"b":[2]}"#)
        .expect("stdin write should succeed");

    let output = child.wait_with_output().expect("runseal should finish");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "{\n  \"a\": 1,\n  \"b\": [\n    2\n  ]\n}\n"
    );
}

#[test]
fn json_pretty_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let input = cwd.join("input.json");
    let output = cwd.join("output.json");
    std::fs::write(&input, r#"{"a":1,"b":[2]}"#).expect("input file should be written");

    let result = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "json",
            "pretty",
            "file",
            input.to_str().expect("input path should be UTF-8"),
            output.to_str().expect("output path should be UTF-8"),
        ])
        .output()
        .expect("runseal should run");

    assert!(result.status.success());
    assert_eq!(
        std::fs::read_to_string(&output).expect("output file should be readable"),
        "{\n  \"a\": 1,\n  \"b\": [\n    2\n  ]\n}\n"
    );
}
