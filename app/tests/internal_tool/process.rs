use tempfile::TempDir;

use super::bin;

#[test]
fn process_write_routes_streams() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let output_path = cwd.join("logs").join("stderr.txt");

    let output = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "process",
            "write",
            "stderr",
            output_path.to_str().expect("path should be utf-8"),
            "--",
            "sh",
            "-c",
            "printf 'out\\n'; printf 'err\\n' >&2",
        ])
        .output()
        .expect("runseal should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "out\n");
    assert_eq!(std::fs::read_to_string(&output_path).unwrap(), "err\n");
}

#[test]
fn process_write_appends() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let output_path = cwd.join("stdout.txt");

    for value in ["one", "two"] {
        let mut args = vec![
            "@tool".to_string(),
            "process".to_string(),
            "write".to_string(),
            "stdout".to_string(),
            output_path
                .to_str()
                .expect("path should be utf-8")
                .to_string(),
        ];
        if value == "two" {
            args.push("--append".to_string());
        }
        args.push("--".to_string());
        args.push("printf".to_string());
        args.push(format!("{value}\\n"));

        let output = bin()
            .current_dir(&cwd)
            .env("RUNSEAL_HOME", temp.path().join("home"))
            .args(&args)
            .output()
            .expect("runseal should run");
        assert!(output.status.success(), "{args:?} should succeed");
    }

    assert_eq!(std::fs::read_to_string(&output_path).unwrap(), "one\ntwo\n");
}
