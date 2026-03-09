use std::process::Command;

use tempfile::TempDir;

fn write_fake_tool(path: &std::path::Path, version: &str) {
    std::fs::write(path, format!("#!/usr/bin/env bash\necho {}\n", version))
        .expect("fake tool script should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .expect("fake tool metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("fake tool should be executable");
    }
}

#[test]
fn plugin_node_init_creates_embedded_script() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");

    let output = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args(["plugin", "node", "init"])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("envlock command should run");

    assert!(output.status.success());
    assert!(envlock_home.join("plugins/node.sh").is_file());
}

#[test]
fn plugin_node_preview_and_apply_emit_patch() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");
    let state_dir = temp.path().join("node-state");
    let node_bin = temp.path().join("fake-node.sh");
    let npm_bin = temp.path().join("fake-npm.sh");
    let pnpm_bin = temp.path().join("fake-pnpm.sh");
    let yarn_bin = temp.path().join("fake-yarn.sh");

    write_fake_tool(&node_bin, "v24.12.0");
    write_fake_tool(&npm_bin, "10.9.2");
    write_fake_tool(&pnpm_bin, "10.30.3");
    write_fake_tool(&yarn_bin, "1.22.22");

    let init = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "init",
            "--state-dir",
            state_dir.to_str().expect("state dir should be UTF-8"),
            "--node-bin",
            node_bin.to_str().expect("node bin path should be UTF-8"),
            "--npm-bin",
            npm_bin.to_str().expect("npm bin path should be UTF-8"),
            "--pnpm-bin",
            pnpm_bin.to_str().expect("pnpm bin path should be UTF-8"),
            "--yarn-bin",
            yarn_bin.to_str().expect("yarn bin path should be UTF-8"),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("envlock command should run");
    assert!(init.status.success());

    let preview = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "preview",
            "--state-dir",
            state_dir.to_str().expect("state dir should be UTF-8"),
            "--node-bin",
            node_bin.to_str().expect("node bin path should be UTF-8"),
            "--npm-bin",
            npm_bin.to_str().expect("npm bin path should be UTF-8"),
            "--pnpm-bin",
            pnpm_bin.to_str().expect("pnpm bin path should be UTF-8"),
            "--yarn-bin",
            yarn_bin.to_str().expect("yarn bin path should be UTF-8"),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("envlock command should run");
    assert!(preview.status.success());

    let stdout = String::from_utf8(preview.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"schema\": \"envlock.patch.v1\""));
    assert!(stdout.contains("\"ENVLOCK_NODE_BIN\""));
    assert!(stdout.contains("\"PNPM_STORE_PATH\""));
    assert!(stdout.contains("\"YARN_CACHE_FOLDER\""));

    let apply = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "apply",
            "--state-dir",
            state_dir.to_str().expect("state dir should be UTF-8"),
            "--node-bin",
            node_bin.to_str().expect("node bin path should be UTF-8"),
            "--npm-bin",
            npm_bin.to_str().expect("npm bin path should be UTF-8"),
            "--pnpm-bin",
            pnpm_bin.to_str().expect("pnpm bin path should be UTF-8"),
            "--yarn-bin",
            yarn_bin.to_str().expect("yarn bin path should be UTF-8"),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("envlock command should run");
    assert!(apply.status.success());

    let link = state_dir.join("current/bin/node");
    let metadata = std::fs::symlink_metadata(&link).expect("symlink metadata should exist");
    assert!(metadata.file_type().is_symlink());

    assert!(state_dir.join("versions/node/v24.12.0/bin/node").exists());
    assert!(state_dir.join("versions/npm/v10.9.2/bin/npm").exists());
    assert!(state_dir.join("versions/pnpm/v10.30.3/bin/pnpm").exists());
    assert!(state_dir.join("versions/yarn/v1.22.22/bin/yarn").exists());
    assert!(state_dir.join("state.v2.json").is_file());
}

#[test]
fn plugin_node_preview_reports_missing_override_binary() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");
    let state_dir = temp.path().join("node-state");
    let npm_bin = temp.path().join("fake-npm.sh");
    let pnpm_bin = temp.path().join("fake-pnpm.sh");
    let yarn_bin = temp.path().join("fake-yarn.sh");

    write_fake_tool(&npm_bin, "10.9.2");
    write_fake_tool(&pnpm_bin, "10.30.3");
    write_fake_tool(&yarn_bin, "1.22.22");

    let init = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "init",
            "--state-dir",
            state_dir.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("init command should run");
    assert!(init.status.success());

    let missing = temp.path().join("does-not-exist");
    let preview = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "preview",
            "--state-dir",
            state_dir.to_str().expect("state dir should be UTF-8"),
            "--node-bin",
            missing.to_str().expect("missing path should be UTF-8"),
            "--npm-bin",
            npm_bin.to_str().expect("npm bin path should be UTF-8"),
            "--pnpm-bin",
            pnpm_bin.to_str().expect("pnpm bin path should be UTF-8"),
            "--yarn-bin",
            yarn_bin.to_str().expect("yarn bin path should be UTF-8"),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("preview command should run");

    assert!(!preview.status.success());
    let stderr = String::from_utf8(preview.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("configured node binary does not exist or cannot be read"),
        "stderr should contain actionable missing binary error, got: {stderr}"
    );
}

#[test]
fn plugin_node_preview_rejects_invalid_plugin_json() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");
    let state_dir = temp.path().join("node-state");

    let init = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "init",
            "--state-dir",
            state_dir.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("init command should run");
    assert!(init.status.success());

    let script_path = envlock_home.join("plugins/node.sh");
    std::fs::write(&script_path, "#!/usr/bin/env bash\necho not-json\n")
        .expect("script should be rewritten");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&script_path)
            .expect("script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script_path, permissions).expect("script should be executable");
    }

    let preview = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "preview",
            "--state-dir",
            state_dir.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("preview command should run");

    assert!(!preview.status.success());
    let stderr = String::from_utf8(preview.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("invalid plugin patch JSON output"));
}

#[test]
#[cfg(unix)]
fn plugin_node_preview_rejects_looped_symlink_override() {
    let temp = TempDir::new().expect("temp dir should be created");
    let envlock_home = temp.path().join("envlock-home");
    let state_dir = temp.path().join("node-state");
    let loop_bin = temp.path().join("node-loop");
    let npm_bin = temp.path().join("fake-npm.sh");
    let pnpm_bin = temp.path().join("fake-pnpm.sh");
    let yarn_bin = temp.path().join("fake-yarn.sh");

    write_fake_tool(&npm_bin, "10.9.2");
    write_fake_tool(&pnpm_bin, "10.30.3");
    write_fake_tool(&yarn_bin, "1.22.22");
    std::os::unix::fs::symlink(&loop_bin, &loop_bin).expect("looped symlink should be created");

    let init = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "init",
            "--state-dir",
            state_dir.to_str().unwrap(),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("init command should run");
    assert!(init.status.success());

    let preview = Command::new(env!("CARGO_BIN_EXE_envlock"))
        .args([
            "plugin",
            "node",
            "preview",
            "--state-dir",
            state_dir.to_str().expect("state dir should be UTF-8"),
            "--node-bin",
            loop_bin.to_str().expect("loop path should be UTF-8"),
            "--npm-bin",
            npm_bin.to_str().expect("npm bin path should be UTF-8"),
            "--pnpm-bin",
            pnpm_bin.to_str().expect("pnpm bin path should be UTF-8"),
            "--yarn-bin",
            yarn_bin.to_str().expect("yarn bin path should be UTF-8"),
        ])
        .env("ENVLOCK_HOME", &envlock_home)
        .output()
        .expect("preview command should run");

    assert!(!preview.status.success());
    let stderr = String::from_utf8(preview.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("configured node binary symlink has invalid or looped target"),
        "stderr should contain looped symlink error, got: {stderr}"
    );
}
