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

fn profile_path(runseal_home: &std::path::Path, version: &str) -> std::path::PathBuf {
    let profile = runseal_home.join("profiles").join("node-test.json");
    std::fs::create_dir_all(profile.parent().unwrap()).expect("profiles dir should exist");
    let content = format!(
        r#"{{
  "schema": "runseal.profile.v1",
  "meta": {{ "name": "node-test" }},
  "injections": [
    {{
      "type": "env",
      "ops": [
        {{ "op": "set", "key": "RUNSEAL_HELPER_NODE_HOME", "value": "{home}/helpers/node" }},
        {{ "op": "set", "key": "RUNSEAL_NODE_VERSION", "value": "{version}" }},
        {{ "op": "set", "key": "RUNSEAL_NODE_BIN", "value": "{home}/helpers/node/versions/v{version}/bin/node" }},
        {{ "op": "set", "key": "RUNSEAL_COREPACK_SHIMS", "value": "{home}/helpers/node/versions/v{version}/corepack-bin" }},
        {{ "op": "set", "key": "COREPACK_HOME", "value": "{home}/helpers/node/versions/v{version}/cache/corepack" }},
        {{ "op": "set", "key": "NPM_CONFIG_CACHE", "value": "{home}/helpers/node/versions/v{version}/cache/npm" }},
        {{ "op": "set", "key": "NPM_CONFIG_PREFIX", "value": "{home}/helpers/node/versions/v{version}" }},
        {{ "op": "set", "key": "npm_config_prefix", "value": "{home}/helpers/node/versions/v{version}" }},
        {{ "op": "prepend", "key": "PATH", "value": "{home}/helpers/node/versions/v{version}/node_modules/.bin", "separator": ":" }},
        {{ "op": "prepend", "key": "PATH", "value": "{home}/helpers/node/versions/v{version}/corepack-bin", "separator": ":" }},
        {{ "op": "prepend", "key": "PATH", "value": "{home}/helpers/node/versions/v{version}/bin", "separator": ":" }}
      ]
    }}
  ]
}}"#,
        home = runseal_home.display(),
        version = version
    );
    std::fs::write(&profile, content).expect("profile should be written");
    profile
}

#[test]
fn helper_node_example_prints_profile_and_dirty_boundaries() {
    let output = runseal_command()
        .args(["helper", ":node", "example"])
        .output()
        .expect("example command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"schema\": \"runseal.profile.v1\""));
    assert!(stdout.contains("RUNSEAL_HELPER_NODE_HOME"));
    assert!(stdout.contains("dirty boundaries to keep sealed"));
    assert!(stdout.contains("versions/vX.Y.Z/bin"));
}

#[test]
fn helper_node_remote_list_returns_versions() {
    let output = runseal_command()
        .args(["helper", ":node", "remote", "list"])
        .output()
        .expect("remote list should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let first = stdout
        .lines()
        .next()
        .expect("remote list should have entries");
    assert!(first.chars().next().unwrap().is_ascii_digit());
}

#[test]
fn helper_node_install_requires_explicit_version() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");

    let output = runseal_command()
        .args(["helper", ":node", "install"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("install requires --node-version"));
}

#[test]
fn helper_node_install_creates_layout() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");

    let output = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runseal command should run");

    assert!(output.status.success());
    let install_home = runseal_home.join("helpers/node/versions/v24.12.0");
    assert!(install_home.join("bin/node").exists());
    assert!(install_home.join("bin/npm").exists());
    assert!(install_home.join("node_modules").exists());
    assert!(install_home.join("lib/node_modules/npm").exists());
    assert!(install_home.join(".lock").exists() || !install_home.join(".lock").exists());
}

#[test]
fn helper_node_install_emits_patch() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");

    let output = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"schema\": \"runseal.patch.v1\""));
    assert!(stdout.contains("\"RUNSEAL_COREPACK_SHIMS\""));
    assert!(stdout.contains("\"COREPACK_HOME\""));
    assert!(stdout.contains("\"RUNSEAL_NODE_BIN\""));
    assert!(stdout.contains("\"NPM_CONFIG_PREFIX\""));
    assert!(stdout.contains("corepack-bin"));
    assert!(stdout.contains("node_modules/.bin"));
}

#[test]
fn helper_node_list_and_which_report_installed_version() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let list = runseal_command()
        .args(["helper", ":node", "list"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("list should run");
    assert!(list.status.success());
    let list_stdout = String::from_utf8(list.stdout).expect("stdout should be UTF-8");
    assert!(list_stdout.contains("24.12.0"));

    let which = runseal_command()
        .args(["helper", ":node", "which", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("which should run");
    assert!(which.status.success());
    let which_stdout = String::from_utf8(which.stdout).expect("stdout should be UTF-8");
    assert!(which_stdout.contains("version=24.12.0"));
    assert!(which_stdout.contains("bin/node"));
    assert!(which_stdout.contains("bin/npm"));
    assert!(which_stdout.contains("corepack_shims="));
}

#[test]
fn helper_node_snapshot_reports_existing_install() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let snapshot = runseal_command()
        .args(["helper", ":node", "snapshot", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("snapshot should run");
    assert!(snapshot.status.success());
    let stdout = String::from_utf8(snapshot.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"schema\": \"runseal.node.snapshot.v1\""));
    assert!(stdout.contains("\"version\": \"24.12.0\""));
    assert!(stdout.contains("\"bin_dir\""));
}

#[test]
fn helper_node_uninstall_removes_single_version() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let version_root = runseal_home.join("helpers/node/versions/v24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());
    assert!(version_root.exists());

    let uninstall = runseal_command()
        .args(["helper", ":node", "uninstall", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("uninstall should run");
    assert!(uninstall.status.success());
    assert!(!version_root.exists());
}

#[test]
fn helper_node_profile_runtime_can_install_pnpm_globally() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let runtime = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "npm",
            "i",
            "-g",
            "pnpm",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runtime command should run");
    assert!(runtime.status.success());

    let version_root = runseal_home.join("helpers/node/versions/v24.12.0");
    assert!(version_root.join("bin/pnpm").exists());
    assert!(version_root.join("lib/node_modules/pnpm").exists());
}

#[test]
fn helper_node_profile_runtime_can_prepare_pnpm_with_corepack() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let runtime = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "corepack",
            "prepare",
            "pnpm@10.30.3",
            "--activate",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runtime command should run");
    assert!(runtime.status.success());

    let version_root = runseal_home.join("helpers/node/versions/v24.12.0");
    assert!(version_root.join("cache/corepack").exists());
    assert!(version_root.join("corepack-bin/pnpm").exists());

    let pnpm = runseal_command()
        .args(["--profile", profile.to_str().unwrap(), "pnpm", "--version"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("pnpm should run through profile");
    assert!(pnpm.status.success());
}

#[test]
fn helper_node_profile_runtime_can_enable_pnpm_with_corepack() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let enable = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "corepack",
            "enable",
            "pnpm",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("corepack enable should run");
    assert!(enable.status.success());

    let which = runseal_command()
        .args(["helper", ":node", "which", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("which should run");
    assert!(which.status.success());
    let stdout = String::from_utf8(which.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("corepack_shims="));
}

#[test]
fn helper_node_profile_runtime_records_corepack_yarn_prepare_state() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let runtime = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "corepack",
            "prepare",
            "yarn@1.22.22",
            "--activate",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("runtime command should run");
    assert!(runtime.status.success());

    let version_root = runseal_home.join("helpers/node/versions/v24.12.0");
    assert!(version_root.join("cache/corepack").exists());
    assert!(version_root.join("corepack-bin").exists());
    let corepack_cache_entries = std::fs::read_dir(version_root.join("cache/corepack"))
        .expect("corepack cache dir should exist")
        .count();
    assert!(corepack_cache_entries > 0);
}

#[test]
fn helper_node_profile_runtime_can_enable_yarn_with_corepack() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let enable = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "corepack",
            "enable",
            "yarn",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("corepack enable should run");
    assert!(enable.status.success());

    let version_root = runseal_home.join("helpers/node/versions/v24.12.0");
    assert!(version_root.join("corepack-bin/yarn").exists());
}

#[test]
fn helper_node_profile_runtime_supports_project_local_pnpm_workflow() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let project_dir = temp.path().join("pnpm-project");
    let profile = profile_path(&runseal_home, "24.12.0");
    std::fs::create_dir_all(&project_dir).expect("project dir should be created");
    std::fs::write(
        project_dir.join("package.json"),
        r#"{"name":"pnpm-project","version":"1.0.0","scripts":{"hello":"node -e \"console.log('hello from pnpm')\""}}"#,
    )
    .expect("package json should be written");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let absorb = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "npm",
            "i",
            "-g",
            "pnpm",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("pnpm absorb should run");
    assert!(absorb.status.success());

    let add = runseal_command()
        .current_dir(&project_dir)
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "pnpm",
            "add",
            "is-number@7.0.0",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("pnpm add should run");
    assert!(
        add.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let run = runseal_command()
        .current_dir(&project_dir)
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "pnpm",
            "run",
            "hello",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("pnpm run should execute");
    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("hello from pnpm"));
    assert!(project_dir.join("pnpm-lock.yaml").exists());
    assert!(project_dir.join("node_modules").exists());
}

#[test]
fn helper_node_profile_runtime_supports_project_local_yarn_workflow() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let project_dir = temp.path().join("yarn-project");
    let profile = profile_path(&runseal_home, "24.12.0");
    std::fs::create_dir_all(&project_dir).expect("project dir should be created");
    std::fs::write(
        project_dir.join("package.json"),
        r#"{"name":"yarn-project","version":"1.0.0","scripts":{"hello":"node -e \"console.log('hello from yarn')\""}}"#,
    )
    .expect("package json should be written");

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let absorb = runseal_command()
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "npm",
            "i",
            "-g",
            "yarn",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("yarn absorb should run");
    assert!(absorb.status.success());

    let add = runseal_command()
        .current_dir(&project_dir)
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "yarn",
            "add",
            "is-number@7.0.0",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("yarn add should run");
    assert!(
        add.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let run = runseal_command()
        .current_dir(&project_dir)
        .args([
            "--profile",
            profile.to_str().unwrap(),
            "yarn",
            "run",
            "hello",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("yarn run should execute");
    assert!(run.status.success());
    let stdout = String::from_utf8(run.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("hello from yarn"));
    assert!(project_dir.join("yarn.lock").exists());
    assert!(project_dir.join("node_modules").exists());
}

#[test]
fn helper_node_profile_runtime_beats_host_path_contamination() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile = profile_path(&runseal_home, "24.12.0");
    let host_bin = temp.path().join("host-bin");
    std::fs::create_dir_all(&host_bin).expect("host bin dir should be created");
    std::fs::write(
        host_bin.join("node"),
        "#!/usr/bin/env bash\necho host-node\n",
    )
    .expect("fake host node should be written");
    std::fs::write(host_bin.join("npm"), "#!/usr/bin/env bash\necho host-npm\n")
        .expect("fake host npm should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [host_bin.join("node"), host_bin.join("npm")] {
            let mut permissions = std::fs::metadata(&path)
                .expect("metadata should exist")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).expect("permissions should be set");
        }
    }

    let install = runseal_command()
        .args(["helper", ":node", "install", "--node-version", "24.12.0"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("install should run");
    assert!(install.status.success());

    let runtime = runseal_command()
        .args(["--profile", profile.to_str().unwrap(), "node", "--version"])
        .env("RUNSEAL_HOME", &runseal_home)
        .env(
            "PATH",
            format!("{}:{}", host_bin.display(), std::env::var("PATH").unwrap()),
        )
        .output()
        .expect("runtime command should run");
    assert!(runtime.status.success());
    let stdout = String::from_utf8(runtime.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("v24.12.0"));
    assert!(!stdout.contains("host-node"));
}

#[test]
fn helper_node_versions_stay_isolated() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profile24 = profile_path(&runseal_home, "24.12.0");
    let profile22 = runseal_home.join("profiles").join("node22.json");
    let content22 = std::fs::read_to_string(&profile24)
        .expect("profile24 should exist")
        .replace("24.12.0", "22.12.0")
        .replace("node-test", "node22");
    std::fs::write(&profile22, content22).expect("profile22 should be written");

    for version in ["24.12.0", "22.12.0"] {
        let install = runseal_command()
            .args(["helper", ":node", "install", "--node-version", version])
            .env("RUNSEAL_HOME", &runseal_home)
            .output()
            .expect("install should run");
        assert!(install.status.success());
    }

    let list = runseal_command()
        .args(["helper", ":node", "list"])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("list should run");
    assert!(list.status.success());
    let stdout = String::from_utf8(list.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("24.12.0"));
    assert!(stdout.contains("22.12.0"));

    let pnpm24 = runseal_command()
        .args([
            "--profile",
            profile24.to_str().unwrap(),
            "npm",
            "i",
            "-g",
            "pnpm",
        ])
        .env("RUNSEAL_HOME", &runseal_home)
        .output()
        .expect("pnpm install should run");
    assert!(pnpm24.status.success());

    let root24 = runseal_home.join("helpers/node/versions/v24.12.0");
    let root22 = runseal_home.join("helpers/node/versions/v22.12.0");
    assert!(root24.join("bin/pnpm").exists());
    assert!(!root22.join("bin/pnpm").exists());
}
