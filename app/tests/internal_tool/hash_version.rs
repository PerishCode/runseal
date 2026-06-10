use tempfile::TempDir;

use super::bin;

#[test]
fn version_atoms() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for (args, expected) in [
        (vec!["@tool", "version", "part", "v1.2.3", "minor"], "2\n"),
        (
            vec!["@tool", "version", "compare", "0.6.1", "0.6.0"],
            "gt\n",
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
fn help() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for (args, expected) in [
        (
            vec!["@tool", "hash", "--help"],
            "Usage: runseal @tool hash <command> [args]",
        ),
        (
            vec!["@tool", "hash", "tree", "--help"],
            "Usage: runseal @tool hash tree <path>...",
        ),
        (
            vec!["@tool", "version", "--help"],
            "Usage: runseal @tool version <command> [args]",
        ),
        (
            vec!["@tool", "version", "part", "--help"],
            "Usage: runseal @tool version part <version> <major|minor|patch>",
        ),
        (
            vec!["@tool", "version", "compare", "--help"],
            "Usage: runseal @tool version compare <left> <right>",
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
fn hash_tree() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("hash");
    std::fs::create_dir_all(cwd.join("nested")).expect("tree should be created");
    std::fs::write(cwd.join("a.txt"), "alpha\n").expect("a.txt should be written");
    std::fs::write(cwd.join("nested/b.txt"), "beta\n").expect("b.txt should be written");

    let first = bin()
        .current_dir(temp.path())
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "hash", "tree", cwd.to_str().unwrap()])
        .output()
        .expect("runseal should run");
    assert!(first.status.success());

    let second = bin()
        .current_dir(temp.path())
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@tool", "hash", "tree", cwd.to_str().unwrap()])
        .output()
        .expect("runseal should run");
    assert!(second.status.success());

    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be UTF-8");
    assert_eq!(first_stdout, second_stdout);
    assert_eq!(first_stdout.trim().len(), 64);
}
