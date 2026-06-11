use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command.env_remove("RUNSEAL_PROFILE_HOME");
    command.env_remove("RUNSEAL_PROFILE_PATH");
    command
}

#[cfg(unix)]
fn shell_args(script: &str) -> Vec<String> {
    vec!["bash".into(), "--".into(), "-lc".into(), script.into()]
}

#[cfg(windows)]
fn shell_args(script: &str) -> Vec<String> {
    vec![
        "pwsh".into(),
        "--".into(),
        "-NoProfile".into(),
        "-Command".into(),
        script.into(),
    ]
}

#[cfg(unix)]
fn print_env_script(key: &str) -> String {
    format!("printf '%s' \"${key}\"")
}

#[cfg(windows)]
fn print_env_script(key: &str) -> String {
    format!("[Console]::Write($env:{key})")
}

fn env_profile(value: &str) -> String {
    format!("[[injections]]\ntype = \"env\"\n[injections.vars]\nPICKED = \"{value}\"\n")
}

fn run_picked(cwd: &std::path::Path, home: &std::path::Path) -> String {
    let output = bin()
        .current_dir(cwd)
        .env("RUNSEAL_HOME", home)
        .args(shell_args(&print_env_script("PICKED")))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn explicit_is_absolute() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let nested = project.join("nested");
    let profile = project.join("runseal.toml");
    std::fs::create_dir_all(&nested).expect("nested dir should be created");
    std::fs::write(&profile, "injections = []\n").expect("profile should be written");

    let output = bin()
        .current_dir(&nested)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["--profile", "../runseal.toml"])
        .args(shell_args(&print_env_script("RUNSEAL_PROFILE_PATH")))
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let printed = std::path::Path::new(stdout.as_str());
    assert!(printed.is_absolute());
    assert!(
        !printed
            .components()
            .any(|component| { matches!(component, std::path::Component::ParentDir) })
    );
    assert!(printed.ends_with("runseal.toml"));
}

#[test]
fn ancestor_is_found() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let nested = project.join("a/b/c");
    std::fs::create_dir_all(&nested).expect("nested dir should be created");
    std::fs::write(project.join("runseal.toml"), env_profile("parent"))
        .expect("parent profile should be written");

    assert_eq!(run_picked(&nested, &temp.path().join("home")), "parent");
}

#[test]
fn nearest_wins() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let nested = project.join("a/b/c");
    std::fs::create_dir_all(&nested).expect("nested dir should be created");
    std::fs::write(project.join("runseal.toml"), env_profile("parent"))
        .expect("parent profile should be written");
    std::fs::write(
        nested.join("runseal.yaml"),
        "injections:\n  - type: env\n    vars:\n      PICKED: nested\n",
    )
    .expect("nested profile should be written");

    assert_eq!(run_picked(&nested, &temp.path().join("home")), "nested");
}

#[test]
fn priority_per_dir() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let nested = project.join("nested");
    std::fs::create_dir_all(&nested).expect("nested dir should be created");
    std::fs::write(
        nested.join("runseal.yaml"),
        "injections:\n  - type: env\n    vars:\n      PICKED: nested-yaml\n",
    )
    .expect("nested yaml should be written");
    std::fs::write(project.join("runseal.toml"), env_profile("parent-toml"))
        .expect("parent toml should be written");
    std::fs::write(nested.join("runseal.toml"), env_profile("nested-toml"))
        .expect("nested toml should be written");

    assert_eq!(
        run_picked(&nested, &temp.path().join("home")),
        "nested-toml"
    );
}

#[test]
fn default_is_fallback() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("work/a/b");
    let runseal_home = temp.path().join("home");
    let profile_home = runseal_home.join("profiles");
    std::fs::create_dir_all(&cwd).expect("cwd should be created");
    std::fs::create_dir_all(&profile_home).expect("profile home should be created");
    std::fs::write(profile_home.join("default.toml"), env_profile("home"))
        .expect("default profile should be written");

    assert_eq!(run_picked(&cwd, &runseal_home), "home");
}
