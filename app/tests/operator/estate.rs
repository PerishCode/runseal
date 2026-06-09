#![cfg(unix)]

#[path = "estate/pr.rs"]
mod pr;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    project: PathBuf,
    bin: PathBuf,
    log: PathBuf,
}

fn fixture() -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let bin = temp.path().join("bin");
    let log = temp.path().join("commands.log");
    std::fs::create_dir_all(project.join(".runseal/wrappers"))
        .expect("wrapper dir should be created");
    std::fs::create_dir_all(&bin).expect("bin dir should be created");
    write_profile(&project);
    write_wrappers(&project);
    write_git_stub(&bin.join("git"));
    write_ssh_stub(&bin.join("ssh"));
    write_kubectl_stub(&bin.join("kubectl"));
    Fixture {
        _temp: temp,
        project,
        bin,
        log,
    }
}

fn write_profile(project: &Path) {
    std::fs::write(
        project.join("runseal.toml"),
        r#"
[resources]
root = ".local"

[[injections]]
type = "env"

[injections.vars]
PERISH_TOP_LOCAL_DIR = "resource://"
PERISH_TOP_SSH_DIR = "resource://ssh"
PERISH_TOP_SSH_CONFIG = "resource://ssh/config"
PERISH_TOP_KUBE_DIR = "resource://kube"
PERISH_TOP_SECRETS_DIR = "resource://secrets"
PERISH_TOP_TMP_DIR = "resource://tmp"
"#,
    )
    .expect("profile should be written");
}

fn write_wrappers(project: &Path) {
    let wrappers = project.join(".runseal/wrappers");
    std::fs::write(
        wrappers.join("admin.seal"),
        ADMIN_SEAL.replace("__SSH_CONFIG_BASE64__", SSH_CONFIG_BASE64),
    )
    .expect("admin seal should be written");
    std::fs::write(wrappers.join("ssh.seal"), SSH_SEAL).expect("ssh seal should be written");
    std::fs::write(wrappers.join("ssh-run.seal"), SSH_RUN_SEAL)
        .expect("ssh-run seal should be written");
    std::fs::write(wrappers.join("kube.seal"), KUBE_SEAL).expect("kube seal should be written");
    std::fs::write(wrappers.join("pr.seal"), PR_SEAL).expect("pr seal should be written");
}

fn write_git_stub(path: &Path) {
    write_executable(
        path,
        r#"#!/usr/bin/env sh
set -eu
case "$1" in
  checkout)
    if [ "${RUNSEAL_TEST_CHECKOUT_FAIL:-}" = "${2:-}" ]; then
      printf 'git' >> "$RUNSEAL_TEST_LOG"
      for arg in "$@"; do
        printf '|%s' "$arg" >> "$RUNSEAL_TEST_LOG"
      done
      printf '\n' >> "$RUNSEAL_TEST_LOG"
      exit 1
    fi
    ;;
  branch)
    if [ "${2:-}" = "--show-current" ]; then
      printf '%s\n' "${RUNSEAL_TEST_BRANCH:-main}"
      exit 0
    fi
    ;;
  remote)
    if [ "${2:-}" = "get-url" ] && [ "${3:-}" = "origin" ]; then
      printf 'git@gitee.com:perishme/perish.top.git\n'
      exit 0
    fi
    ;;
  status)
    if [ "${2:-}" = "--short" ]; then
      printf '%s\n' "${RUNSEAL_TEST_STATUS- M docs.md}"
      exit 0
    fi
    ;;
esac
printf 'git' >> "$RUNSEAL_TEST_LOG"
for arg in "$@"; do
  printf '|%s' "$arg" >> "$RUNSEAL_TEST_LOG"
done
printf '\n' >> "$RUNSEAL_TEST_LOG"
"#,
    );
}

fn write_ssh_stub(path: &Path) {
    write_executable(
        path,
        r#"#!/usr/bin/env sh
set -eu
printf 'ssh' >> "$RUNSEAL_TEST_LOG"
for arg in "$@"; do
  printf '|%s' "$arg" >> "$RUNSEAL_TEST_LOG"
done
printf '\n' >> "$RUNSEAL_TEST_LOG"
cat >/dev/null
printf 'captured-kube'
"#,
    );
}

fn write_kubectl_stub(path: &Path) {
    write_executable(
        path,
        r#"#!/usr/bin/env sh
set -eu
printf 'kubectl|KUBECONFIG=%s' "${KUBECONFIG:-}" >> "$RUNSEAL_TEST_LOG"
for arg in "$@"; do
  printf '|%s' "$arg" >> "$RUNSEAL_TEST_LOG"
done
printf '\n' >> "$RUNSEAL_TEST_LOG"
"#,
    );
}

fn write_executable(path: &Path, content: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, content).expect("stub should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("stub metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("stub should be executable");
}

fn run_wrapper(fx: &Fixture, name: &str, args: &[&str]) -> std::process::Output {
    run_wrapper_env(fx, name, args, &[])
}

fn run_wrapper_env(
    fx: &Fixture,
    name: &str,
    args: &[&str],
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command
        .current_dir(&fx.project)
        .env("PATH", prepend_path(&fx.bin))
        .env("RUNSEAL_TEST_LOG", &fx.log)
        .arg("-p")
        .arg(fx.project.join("runseal.toml"))
        .arg(format!(":{name}"))
        .args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("runseal wrapper should run")
}

fn prepend_path(first: &Path) -> OsString {
    let mut paths = vec![first.to_path_buf()];
    if let Some(runseal_dir) = Path::new(env!("CARGO_BIN_EXE_runseal")).parent() {
        paths.push(runseal_dir.to_path_buf());
    }
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).expect("PATH should be joinable")
}

fn log(fx: &Fixture) -> String {
    std::fs::read_to_string(&fx.log).unwrap_or_default()
}

#[test]
fn admin_init_check() {
    let fx = fixture();

    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(
        init.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    for path in [".local/ssh", ".local/kube", ".local/secrets", ".local/tmp"] {
        assert!(fx.project.join(path).is_dir(), "{path} should exist");
    }
    assert!(fx.project.join(".local/ssh/config").is_file());
    assert!(fx.project.join(".local/ssh/id_perish_top_root").is_file());
    assert!(fx.project.join(".local/ssh/known_hosts").is_file());

    std::fs::write(fx.project.join(".local/ssh/id_perish_top_root"), "key")
        .expect("root key should be filled");
    let kubeconfig = fx.project.join(".local/kube/hk-zxi.yaml");
    std::fs::write(&kubeconfig, "kube").expect("kubeconfig should be written");
    set_mode(&kubeconfig, 0o600);

    let check = run_wrapper(&fx, "admin", &["check"]);
    assert!(
        check.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(String::from_utf8_lossy(&check.stdout).contains("admin check: ok"));
}

#[test]
fn ssh_remote_args() {
    let fx = fixture();
    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(init.status.success());

    let ok = run_wrapper(&fx, "ssh", &["10m.hk.zxi", "--", "uptime", "-p"]);
    assert!(
        ok.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
    assert_eq!(
        log(&fx),
        format!(
            "ssh|-F|{}|10m.hk.zxi|uptime|-p\n",
            fx.project.join(".local/ssh/config").display()
        )
    );

    let denied = run_wrapper(&fx, "ssh", &["unknown.example"]);
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("host is not declared"));
}

#[test]
fn ssh_help() {
    let fx = fixture();

    let output = run_wrapper(&fx, "ssh", &["--help"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "Usage: runseal :ssh <host> [--run <script> [-- <args>...] | -- <remote-command>...]\n"
    );
    assert!(output.stderr.is_empty());
    assert!(log(&fx).is_empty());
}

#[test]
fn ssh_run_mode() {
    let fx = fixture();
    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(init.status.success());
    let script = fx.project.join("probe.sh");
    std::fs::write(&script, "echo probe").expect("script should be written");

    let output = run_wrapper(
        &fx,
        "ssh",
        &[
            "10m.hk.zxi",
            "--run",
            script.to_str().unwrap(),
            "--",
            "one",
            "two",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        log(&fx),
        format!(
            "ssh|-F|{}|10m.hk.zxi|bash|-s|--|one|two\n",
            fx.project.join(".local/ssh/config").display()
        )
    );
}

#[test]
fn ssh_run_forward() {
    let fx = fixture();
    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(init.status.success());
    let script = fx.project.join("probe.sh");
    std::fs::write(&script, "echo probe").expect("script should be written");

    let output = run_wrapper(
        &fx,
        "ssh-run",
        &["10m.hk.zxi", script.to_str().unwrap(), "--", "one"],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = log(&fx);
    assert!(log.contains("ssh|-F|"));
    assert!(log.ends_with("|10m.hk.zxi|bash|-s|--|one\n"));
}

#[test]
fn admin_bootstrap_kube() {
    let fx = fixture();
    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(init.status.success());
    let ops_admin = fx.project.join("infra/k8s/access/ops-admin.yaml");
    let setup = fx
        .project
        .join("nodes/10m-hk-zxi/k3s/bootstrap/35-emit-kubeconfig.sh");
    std::fs::create_dir_all(ops_admin.parent().expect("ops parent should exist"))
        .expect("ops parent should be created");
    std::fs::create_dir_all(setup.parent().expect("setup parent should exist"))
        .expect("setup parent should be created");
    std::fs::write(&ops_admin, "ops").expect("ops file should be written");
    std::fs::write(&setup, "setup").expect("setup file should be written");

    let output = run_wrapper(&fx, "admin", &["bootstrap-kube"]);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let kubeconfig = fx.project.join(".local/kube/hk-zxi.yaml");
    assert_eq!(
        std::fs::read_to_string(&kubeconfig).expect("kubeconfig should be readable"),
        "captured-kube"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&kubeconfig)
                .expect("metadata should be readable")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(
        log(&fx),
        format!(
            "ssh|-F|{}|10m.hk.zxi|bash|-s|--\nssh|-F|{}|10m.hk.zxi|bash|-s|--|https://k8s.perish.top:6443|hk-zxi\n",
            fx.project.join(".local/ssh/config").display(),
            fx.project.join(".local/ssh/config").display()
        )
    );
}

#[test]
fn admin_archive() {
    let fx = fixture();
    let init = run_wrapper(&fx, "admin", &["init"]);
    assert!(init.status.success());
    std::fs::write(
        fx.project.join(".local/secrets/gitee.env"),
        "GITEE_TOKEN=test-token\n",
    )
    .expect("secret should be written");
    let archive = fx.project.join("local.enc");

    let export = run_wrapper_env(
        &fx,
        "admin",
        &["export", archive.to_str().unwrap()],
        &[("PERISH_TOP_LOCAL_PASSWORD", "secret".to_string())],
    );
    assert!(
        export.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&export.stdout),
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(archive.is_file());

    std::fs::remove_dir_all(fx.project.join(".local")).expect(".local should be removable");
    let import = run_wrapper_env(
        &fx,
        "admin",
        &["import", "--force", archive.to_str().unwrap()],
        &[("PERISH_TOP_LOCAL_PASSWORD", "secret".to_string())],
    );
    assert!(
        import.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&import.stdout),
        String::from_utf8_lossy(&import.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(fx.project.join(".local/secrets/gitee.env"))
            .expect("secret should be restored"),
        "GITEE_TOKEN=test-token\n"
    );
    assert!(String::from_utf8_lossy(&export.stdout).contains("admin export: wrote"));
    assert!(String::from_utf8_lossy(&import.stdout).contains("admin import: restored .local"));
}

#[test]
fn kube_env() {
    let fx = fixture();
    std::fs::create_dir_all(fx.project.join(".local/kube")).expect("kube dir should be created");
    std::fs::write(fx.project.join(".local/kube/b.yaml"), "b").expect("kubeconfig should exist");
    std::fs::write(fx.project.join(".local/kube/a.yaml"), "a").expect("kubeconfig should exist");

    let output = run_wrapper(&fx, "kube", &["auth", "whoami"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        log(&fx),
        format!(
            "kubectl|KUBECONFIG={}:{}|auth|whoami\n",
            fx.project
                .join(".local/kube/a.yaml")
                .canonicalize()
                .expect("a path should canonicalize")
                .display(),
            fx.project
                .join(".local/kube/b.yaml")
                .canonicalize()
                .expect("b path should canonicalize")
                .display()
        )
    );
}

fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("metadata should be readable")
        .permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions).expect("mode should be set");
}

const SSH_CONFIG_BASE64: &str = "SG9zdCAxMG0uaGsuenhpCiAgSG9zdE5hbWUgNDMuMjUxLjIyNS4xMTMKICBVc2VyIHJvb3QKICBJZGVudGl0eUZpbGUgaWRfcGVyaXNoX3RvcF9yb290CgpIb3N0IDVtLmhrLnp4aQogIEhvc3ROYW1lIDQzLjI1MS4yMjUuODUKICBVc2VyIHJvb3QKICBJZGVudGl0eUZpbGUgaWRfcGVyaXNoX3RvcF9yb290CgpIb3N0IGxhLnVzLmxpc2EKICBIb3N0TmFtZSAxNTQuMjkuMTU4LjEzNAogIFBvcnQgMjc2OTEKICBVc2VyIHJvb3QKICBJZGVudGl0eUZpbGUgaWRfcGVyaXNoX3RvcF9yb290CgpIb3N0IG55LnVzLmxpc2EKICBIb3N0TmFtZSAzOC43Ny4xMzMuMTExCiAgUG9ydCAyMTM2OQogIFVzZXIgcm9vdAogIElkZW50aXR5RmlsZSBpZF9wZXJpc2hfdG9wX3Jvb3QK";

const ADMIN_SEAL: &str = include_str!("../fixtures/estate/admin.seal");

const SSH_SEAL: &str = include_str!("../fixtures/estate/ssh.seal");

const SSH_RUN_SEAL: &str = include_str!("../fixtures/estate/ssh-run.seal");

const KUBE_SEAL: &str = include_str!("../fixtures/estate/kube.seal");

const PR_SEAL: &str = include_str!("../fixtures/estate/pr.seal");
