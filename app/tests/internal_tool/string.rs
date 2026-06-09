use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn string_join() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    let literal = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "string",
            "join",
            r#"["left","right"]"#,
            "--separator",
            ",",
        ])
        .output()
        .expect("runseal should run");
    assert!(literal.status.success());
    assert_eq!(String::from_utf8(literal.stdout).unwrap(), "left,right\n");

    let path = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "string",
            "join",
            r#"["left","right"]"#,
            "--separator",
            "path",
        ])
        .output()
        .expect("runseal should run");
    assert!(path.status.success());
    let expected = std::env::join_paths(["left", "right"])
        .expect("paths should join")
        .to_string_lossy()
        .into_owned()
        + "\n";
    assert_eq!(String::from_utf8(path.stdout).unwrap(), expected);
}

#[test]
fn string_slug() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    let limited = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "string", "slug", "One Two Three", "--max-len", "7"])
        .output()
        .expect("runseal should run");
    assert!(limited.status.success());
    assert_eq!(String::from_utf8(limited.stdout).unwrap(), "one-two\n");

    let fallback = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "string", "slug", "!!!", "--fallback", "change"])
        .output()
        .expect("runseal should run");
    assert!(fallback.status.success());
    assert_eq!(String::from_utf8(fallback.stdout).unwrap(), "change\n");
}
