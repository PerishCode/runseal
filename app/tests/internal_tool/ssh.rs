use std::{path::PathBuf, process::Command};

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn ssh_config() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    let base = cwd.join("ssh");
    std::fs::create_dir_all(&base).expect("ssh dir should be created");
    let config = base.join("config");
    std::fs::write(
        &config,
        r#"
Host 10m.hk.zxi *.lisa !blocked.lisa
  IdentityFile id_root
  IdentityFile ~/.ssh/id_extra
"#,
    )
    .expect("config should be written");

    let exact = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "ssh",
            "config",
            "host",
            "10m.hk.zxi",
            "--config",
            config.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(exact.status.success());
    assert_eq!(String::from_utf8(exact.stdout).unwrap(), "true\n");

    let wildcard = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "ssh",
            "config",
            "host",
            "ny.lisa",
            "--config",
            config.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(wildcard.status.success());
    assert_eq!(String::from_utf8(wildcard.stdout).unwrap(), "true\n");

    let blocked = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "ssh",
            "config",
            "host",
            "blocked.lisa",
            "--config",
            config.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(blocked.status.success());
    assert_eq!(String::from_utf8(blocked.stdout).unwrap(), "false\n");

    let identities = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "ssh",
            "config",
            "identities",
            "--config",
            config.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(identities.status.success());
    let files: Vec<String> =
        serde_json::from_slice(&identities.stdout).expect("stdout should be JSON array");
    assert_eq!(files.len(), 2);
    assert_eq!(PathBuf::from(&files[0]), base.join("id_root"));
    assert!(files[1].ends_with(".ssh/id_extra"));

    let override_base = cwd.join("repo");
    let identities = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args([
            "@tool",
            "ssh",
            "config",
            "identities",
            "--config",
            config.to_str().unwrap(),
            "--base",
            override_base.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(identities.status.success());
    let files: Vec<String> =
        serde_json::from_slice(&identities.stdout).expect("stdout should be JSON array");
    assert_eq!(PathBuf::from(&files[0]), override_base.join("id_root"));
}

#[test]
#[cfg(unix)]
fn ssh_script() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    let bin_dir = temp.path().join("bin");
    let ssh_dir = cwd.join("ssh");
    std::fs::create_dir_all(&bin_dir).expect("bin dir should be created");
    std::fs::create_dir_all(&ssh_dir).expect("ssh dir should be created");
    let log = cwd.join("ssh.log");
    let stdin_file = cwd.join("stdin.txt");
    write_ssh_tool_stub(&bin_dir.join("ssh"), &log, &stdin_file);

    let config = ssh_dir.join("config");
    std::fs::write(&config, "Host demo.host\n  HostName 127.0.0.1\n")
        .expect("config should be written");
    let script = cwd.join("probe.sh");
    std::fs::write(&script, "echo from-script\n").expect("script should be written");

    let run = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("PATH", prepend_path_for_test(&bin_dir))
        .args([
            "@tool",
            "ssh",
            "script",
            "run",
            "--config",
            config.to_str().unwrap(),
            "--host",
            "demo.host",
            "--file",
            script.to_str().unwrap(),
            "--",
            "one",
            "two",
        ])
        .output()
        .expect("runseal should run");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&log).expect("log should be readable"),
        format!("ssh|-F|{}|demo.host|bash|-s|--|one|two\n", config.display())
    );
    assert_eq!(
        std::fs::read_to_string(&stdin_file).expect("stdin should be readable"),
        "echo from-script\n"
    );

    let capture = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("PATH", prepend_path_for_test(&bin_dir))
        .args([
            "@tool",
            "ssh",
            "script",
            "capture",
            "--config",
            config.to_str().unwrap(),
            "--host",
            "demo.host",
            "--file",
            script.to_str().unwrap(),
        ])
        .output()
        .expect("runseal should run");
    assert!(
        capture.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&capture.stderr)
    );
    assert_eq!(String::from_utf8(capture.stdout).unwrap(), "captured\n\n");
}

#[cfg(unix)]
fn write_ssh_tool_stub(
    path: &std::path::Path,
    log: &std::path::Path,
    stdin_file: &std::path::Path,
) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        format!(
            r#"#!/usr/bin/env sh
set -eu
printf 'ssh' > '{}'
for arg in "$@"; do
  printf '|%s' "$arg" >> '{}'
done
printf '\n' >> '{}'
cat > '{}'
printf 'captured\n'
"#,
            log.display(),
            log.display(),
            log.display(),
            stdin_file.display()
        ),
    )
    .expect("ssh stub should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("ssh stub metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("ssh stub should be executable");
}

fn prepend_path_for_test(first: &std::path::Path) -> std::ffi::OsString {
    let mut paths = vec![first.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).expect("PATH should be joinable")
}
