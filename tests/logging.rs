use std::process::Command;

use tempfile::TempDir;

#[test]
fn logs_go_to_stderr_and_exports_stay_on_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args(["-p", "examples/envlock.sample.json", "--log-level", "info"])
        .env_remove("RUST_LOG")
        .output()
        .expect("envlock command should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    assert!(stdout.contains("export ENVLOCK_PROFILE='dev'"));
    assert!(stderr.contains("envlock run started"));
    assert!(!stdout.contains("envlock run started"));
}

#[test]
fn plugin_node_writes_per_invocation_log_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");
    let log_home = temp.path().join("logs");
    let state_dir = temp.path().join("node-state");
    let node_bin = temp.path().join("fake-node.sh");
    let npm_bin = temp.path().join("fake-npm.sh");
    let pnpm_bin = temp.path().join("fake-pnpm.sh");
    let yarn_bin = temp.path().join("fake-yarn.sh");

    std::fs::write(&node_bin, "#!/usr/bin/env bash\necho v24.12.0\n")
        .expect("fake node should be written");
    std::fs::write(&npm_bin, "#!/usr/bin/env bash\necho 10.9.2\n")
        .expect("fake npm should be written");
    std::fs::write(&pnpm_bin, "#!/usr/bin/env bash\necho 10.30.3\n")
        .expect("fake pnpm should be written");
    std::fs::write(&yarn_bin, "#!/usr/bin/env bash\necho 1.22.22\n")
        .expect("fake yarn should be written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&node_bin, &npm_bin, &pnpm_bin, &yarn_bin] {
            let mut permissions = std::fs::metadata(path)
                .expect("metadata should exist")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).expect("permissions should be set");
        }
    }

    let init = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "init",
            "--state-dir",
            state_dir.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .env("ENVLOCK_LOG_HOME", &log_home)
        .output()
        .expect("init command should run");
    assert!(init.status.success());

    let before = std::fs::read_dir(&log_home)
        .expect("log dir should exist")
        .count();

    let preview = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "preview",
            "--state-dir",
            state_dir.to_str().unwrap(),
            "--node-bin",
            node_bin.to_str().unwrap(),
            "--npm-bin",
            npm_bin.to_str().unwrap(),
            "--pnpm-bin",
            pnpm_bin.to_str().unwrap(),
            "--yarn-bin",
            yarn_bin.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .env("ENVLOCK_LOG_HOME", &log_home)
        .output()
        .expect("preview command should run");
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
    assert!(contents.contains("plugin command prepared"));
    assert!(contents.contains("plugin.node resolve tool=node source=override"));
    assert!(contents.contains("plugin.node patch emitted env_count=8 symlink_count=4"));
}
