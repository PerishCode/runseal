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
    state: PathBuf,
}

fn fixture() -> Option<Fixture> {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let bin = temp.path().join("bin");
    let state = temp.path().join("state");
    std::fs::create_dir_all(&project).expect("project should be created");
    std::fs::create_dir_all(&bin).expect("stub bin dir should be created");
    std::fs::create_dir_all(&state).expect("stub state dir should be created");
    write_stub(
        &bin.join("git"),
        r#"#!/usr/bin/env sh
set -eu
case "${1:-}" in
  --version)
    ;;
  branch)
    [ "${2:-}" = "--show-current" ] || exit 9
    printf '%s\n' "${RUNSEAL_TEST_BRANCH:-feat/seal}"
    ;;
  *)
    printf 'git %s\n' "$*" >> "${RUNSEAL_TEST_LOG:?}"
    ;;
esac
"#,
    );
    write_stub(
        &bin.join("gh"),
        r#"#!/usr/bin/env sh
set -eu

log() {
  printf 'gh %s\n' "$*" >> "${RUNSEAL_TEST_LOG:?}"
}

case "${1:-}" in
  --version)
    ;;
  auth)
    [ "${2:-}" = status ] || exit 9
    ;;
  workflow)
    log "$@"
    [ "${2:-}" = run ] || exit 9
    printf '%s\n' "${RUNSEAL_TEST_WORKFLOW_OUTPUT:-}"
    ;;
  run)
    log "$@"
    case "${2:-}" in
      list)
        printf '%s\n' "${RUNSEAL_TEST_RUN_LIST:-[]}"
        ;;
      watch)
        ;;
      *)
        exit 9
        ;;
    esac
    ;;
  pr)
    log "$@"
    case "${2:-}" in
      list)
        count_file="${RUNSEAL_TEST_STATE:?}/pr_list_count"
        count=0
        if [ -f "$count_file" ]; then
          count=$(cat "$count_file")
        fi
        next=$((count + 1))
        printf '%s\n' "$next" > "$count_file"
        if [ "$count" -eq 0 ] && [ "${RUNSEAL_TEST_PR_LIST_FIRST+x}" ]; then
          printf '%s\n' "$RUNSEAL_TEST_PR_LIST_FIRST"
        elif [ "$count" -gt 0 ] && [ "${RUNSEAL_TEST_PR_LIST_NEXT+x}" ]; then
          printf '%s\n' "$RUNSEAL_TEST_PR_LIST_NEXT"
        elif [ "${RUNSEAL_TEST_PR_LIST+x}" ]; then
          printf '%s\n' "$RUNSEAL_TEST_PR_LIST"
        else
          printf '%s\n' '[{"number":42,"title":"Seal","state":"OPEN","url":"https://example.test/pull/42","isDraft":false}]'
        fi
        ;;
      create|ready|checks|merge)
        ;;
      *)
        exit 9
        ;;
    esac
    ;;
  *)
    log "$@"
    ;;
esac
"#,
    );
    Some(Fixture {
        _temp: temp,
        project,
        bin,
        state,
    })
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

fn write_stub(path: &Path, content: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, content).expect("stub should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("stub metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("stub should be executable");
}

fn run_active_wrapper(fx: &Fixture, name: &str, args: &[&str]) -> std::process::Output {
    run_wrapper_env(fx, name, args, &[])
}

fn run_wrapper_env(
    fx: &Fixture,
    name: &str,
    args: &[&str],
    envs: &[(&str, &str)],
) -> std::process::Output {
    let log = fx.project.join("commands.log");
    let path = prepend_path(&fx.bin);
    Command::new(env!("CARGO_BIN_EXE_runseal"))
        .current_dir(&fx.project)
        .env("PATH", path)
        .env("RUNSEAL_TEST_LOG", &log)
        .env("RUNSEAL_TEST_STATE", &fx.state)
        .arg("-p")
        .arg(repo_root().join("runseal.toml"))
        .arg(format!(":{name}"))
        .args(args)
        .envs(envs.iter().copied())
        .output()
        .expect("active operator wrapper should run")
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

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn command_log(fx: &Fixture) -> String {
    std::fs::read_to_string(fx.project.join("commands.log")).unwrap_or_default()
}

#[test]
fn pr_help_option() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "pr", &["--help"]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("Usage: runseal :pr [options]"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn pr_dry_run_matches() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "pr", &["--dry-run"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "\
branch: feat/seal
base: main
push: True
pr: create if missing, otherwise reuse existing
draft: False
ready: True
watch: True
squash_merge: True
"
    );
}

#[test]
fn pr_rejects_draft_merge() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "pr", &["--draft", "--dry-run"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("pr: --draft requires --no-merge"));
}

#[test]
fn pr_rejects_base_branch() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--dry-run"],
        &[("RUNSEAL_TEST_BRANCH", "main")],
    );

    assert!(!output.status.success());
    assert!(stderr(&output).contains("pr: refusing to open a PR from base branch: main"));
}

#[test]
fn pr_reuses_draft() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--no-push", "--no-watch", "--no-merge"],
        &[(
            "RUNSEAL_TEST_PR_LIST",
            r#"[{"number":42,"title":"Seal","state":"OPEN","url":"https://example.test/pull/42","isDraft":true}]"#,
        )],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "\
found PR #42: https://example.test/pull/42
marked PR #42 ready
"
    );
    assert_eq!(
        command_log(&fx),
        "\
gh pr list --head feat/seal --json number,title,state,url,isDraft
gh pr ready 42
"
    );
}

#[test]
fn pr_creates_and_merges() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_wrapper_env(
        &fx,
        "pr",
        &[
            "--title",
            "Seal migration",
            "--body-file",
            "body.md",
            "--base",
            "develop",
        ],
        &[
            ("RUNSEAL_TEST_PR_LIST_FIRST", "[]"),
            (
                "RUNSEAL_TEST_PR_LIST_NEXT",
                r#"[{"number":77,"title":"Seal migration","state":"OPEN","url":"https://example.test/pull/77","isDraft":false}]"#,
            ),
        ],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "\
created PR #77: https://example.test/pull/77
squash-merged PR #77
"
    );
    assert_eq!(
        command_log(&fx),
        "\
git push -u origin feat/seal
gh pr list --head feat/seal --json number,title,state,url,isDraft
gh pr create --base develop --head feat/seal --title Seal migration --body-file body.md
gh pr list --head feat/seal --json number,title,state,url,isDraft
gh pr checks 77 --watch --interval 10
gh pr merge 77 --squash --delete-branch
"
    );
}

#[test]
fn release_help_without_args() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "release", &[]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("Usage: runseal :release --channel=stable|beta [options]"));
    assert!(stdout.contains("--watch"));
}

#[test]
fn release_dry_run_matches() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(
        &fx,
        "release",
        &[
            "--channel",
            "beta",
            "--ref",
            "feature/ref",
            "--version",
            "v1.2.3-beta.4",
            "--dry-run",
        ],
    );

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "gh workflow run release-beta.yml --ref feature/ref -f ref=feature/ref -f version_override=v1.2.3-beta.4\n"
    );
}

#[test]
fn release_requires_channel() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "release", &["--dry-run"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("release: --channel is required"));
}

#[test]
fn release_rejects_invalid_channel() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_active_wrapper(&fx, "release", &["--channel", "nightly", "--dry-run"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("invalid choice"));
}

#[test]
fn release_watches_trigger_url() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_wrapper_env(
        &fx,
        "release",
        &["--channel", "stable", "--watch"],
        &[(
            "RUNSEAL_TEST_WORKFLOW_OUTPUT",
            "https://github.com/acme/runseal/actions/runs/12345",
        )],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "\
https://github.com/acme/runseal/actions/runs/12345
triggered release-stable.yml for ref main
"
    );
    assert_eq!(
        command_log(&fx),
        "\
gh workflow run release-stable.yml --ref main -f ref=main -f version_override=
gh run watch 12345 --interval 10
"
    );
}

#[test]
fn release_uses_latest_run() {
    let Some(fx) = fixture() else {
        return;
    };

    let output = run_wrapper_env(
        &fx,
        "release",
        &["--channel", "beta", "--ref", "feature/ref", "--watch"],
        &[("RUNSEAL_TEST_RUN_LIST", r#"[{"databaseId":67890}]"#)],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "triggered release-beta.yml for ref feature/ref\n"
    );
    assert_eq!(
        command_log(&fx),
        "\
gh workflow run release-beta.yml --ref feature/ref -f ref=feature/ref -f version_override=
gh run list --workflow release-beta.yml --branch feature/ref --event workflow_dispatch --limit 1 --json databaseId
gh run watch 67890 --interval 10
"
    );
}
