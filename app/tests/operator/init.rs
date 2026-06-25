#![cfg(unix)]

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
}

fn fixture() -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&project).expect("project should be created");
    std::fs::create_dir_all(&bin).expect("bin should be created");
    Command::new("git")
        .arg("init")
        .arg(&project)
        .output()
        .expect("git init should run");
    write_required_files(&project);
    write_stub(&bin.join("python3"));
    write_stub(&bin.join("cargo"));
    write_stub(&bin.join("flavor"));
    write_stub(&bin.join("sh"));
    write_stub(&bin.join("bash"));
    write_stub(&bin.join("sed"));
    write_stub(&bin.join("grep"));
    Fixture {
        _temp: temp,
        project,
        bin,
    }
}

fn write_required_files(project: &Path) {
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "flavor.toml",
        "manage.sh",
        "manage.ps1",
        "runseal.toml",
        ".runseal/deno.json",
        ".runseal/hooks/pre-commit",
        ".runseal/hooks/commit-msg",
        ".runseal/lib/runseal.ts",
        ".runseal/wrappers/cloudflare.ts",
        ".runseal/wrappers/guard.ts",
        ".runseal/wrappers/init.ts",
        ".runseal/wrappers/pr.ts",
        ".runseal/wrappers/release.ts",
        ".github/workflows/guard.yml",
        ".github/workflows/release-beta.yml",
        ".github/workflows/release-stable.yml",
        ".github/scripts/release/assets/checksums.sh",
        ".github/scripts/release/assets/package.sh",
        ".github/scripts/release/assets/package.ps1",
        ".github/scripts/release/assets/verify.sh",
        ".github/scripts/release/github/cleanup-artifacts.sh",
        ".github/scripts/release/metadata/beta.py",
        ".github/scripts/release/metadata/stable.py",
        ".github/scripts/release/r2/check.sh",
        ".github/scripts/release/r2/publish.sh",
        ".github/scripts/release/r2/summary.sh",
        ".github/scripts/release/r2/verify.sh",
        ".github/scripts/release/smoke/smoke.sh",
        ".github/scripts/release/smoke/smoke.ps1",
    ] {
        let file = project.join(path);
        std::fs::create_dir_all(file.parent().expect("file should have a parent"))
            .expect("parent should be created");
        std::fs::write(&file, "").expect("required file should be written");
    }
    std::fs::write(
        project.join(".runseal/wrappers/init.ts"),
        std::fs::read_to_string(repo_root().join(".runseal/wrappers/init.ts"))
            .expect("repo init wrapper should be readable"),
    )
    .expect("init wrapper should be copied");
    std::fs::write(
        project.join(".runseal/wrappers/guard.ts"),
        std::fs::read_to_string(repo_root().join(".runseal/wrappers/guard.ts"))
            .expect("repo guard wrapper should be readable"),
    )
    .expect("guard wrapper should be copied");
    std::fs::write(
        project.join(".runseal/lib/runseal.ts"),
        std::fs::read_to_string(repo_root().join(".runseal/lib/runseal.ts"))
            .expect("repo deno helper should be readable"),
    )
    .expect("deno helper should be copied");
    std::fs::write(
        project.join(".runseal/deno.json"),
        std::fs::read_to_string(repo_root().join(".runseal/deno.json"))
            .expect("repo deno config should be readable"),
    )
    .expect("deno config should be copied");
    std::fs::write(
        project.join(".runseal/hooks/pre-commit"),
        std::fs::read_to_string(repo_root().join(".runseal/hooks/pre-commit"))
            .expect("repo pre-commit hook should be readable"),
    )
    .expect("pre-commit hook should be copied");
    std::fs::write(
        project.join(".runseal/hooks/commit-msg"),
        std::fs::read_to_string(repo_root().join(".runseal/hooks/commit-msg"))
            .expect("repo commit-msg hook should be readable"),
    )
    .expect("commit-msg hook should be copied");
    std::fs::write(
        project.join("runseal.toml"),
        r#"
injections = []

[deno]
config = ".runseal/deno.json"
permissions = [
  "--allow-read=.",
  "--allow-write=.",
  "--allow-env",
  "--allow-run=git,deno,python3,cargo,runseal,flavor,sh,bash,sed,grep",
]
"#,
    )
    .expect("profile should be written");
}

fn write_stub(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, "#!/usr/bin/env sh\nexit 0\n").expect("stub should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("stub metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("stub should be executable");
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

fn run_init(fx: &Fixture, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
        .current_dir(&fx.project)
        .env("PATH", prepend_path(&fx.bin))
        .arg("-p")
        .arg(fx.project.join("runseal.toml"))
        .arg(":init")
        .args(args)
        .output()
        .expect("runseal init should run")
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

#[test]
fn init_installs_generated_hooks() {
    let fx = fixture();

    let output = run_init(&fx, &[]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pre_commit = fx.project.join(".git/hooks/pre-commit");
    let commit_msg = fx.project.join(".git/hooks/commit-msg");
    let pre_commit_text = std::fs::read_to_string(&pre_commit).expect("pre-commit should exist");
    let commit_msg_text = std::fs::read_to_string(&commit_msg).expect("commit-msg should exist");
    assert!(pre_commit_text.contains("runseal init hook"));
    assert!(pre_commit_text.contains("runseal :guard"));
    assert!(commit_msg_text.contains("runseal init hook"));
}

#[test]
fn init_help_is_readonly() {
    let fx = fixture();

    let output = run_init(&fx, &["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: runseal :init"));
    assert!(!fx.project.join(".git/hooks/pre-commit").exists());
    assert!(!fx.project.join(".git/hooks/commit-msg").exists());
}

#[test]
fn force_backs_up_hook() {
    let fx = fixture();
    let pre_commit = fx.project.join(".git/hooks/pre-commit");
    std::fs::write(&pre_commit, "#!/usr/bin/env sh\necho custom\n")
        .expect("custom hook should be written");

    let rejected = run_init(&fx, &[]);

    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("rerun with --force"));

    let forced = run_init(&fx, &["--force"]);

    assert!(
        forced.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert!(fx.project.join(".git/hooks/pre-commit.bak").is_file());
    let pre_commit_text = std::fs::read_to_string(&pre_commit).expect("pre-commit should exist");
    assert!(pre_commit_text.contains("runseal init hook"));
}
