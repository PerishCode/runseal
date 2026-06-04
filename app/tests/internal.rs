use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[cfg(unix)]
fn wrapper_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    dir.join(name)
}

#[cfg(windows)]
fn wrapper_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    dir.join(format!("{name}.cmd"))
}

#[cfg(unix)]
fn make_wrapper(path: &std::path::Path, label: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, format!("#!/usr/bin/env sh\nprintf '{}'\n", label))
        .expect("wrapper should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("wrapper metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("wrapper should be executable");
}

#[cfg(windows)]
fn make_wrapper(path: &std::path::Path, label: &str) {
    std::fs::write(
        path,
        format!("@echo off\r\n<nul set /p=\"{}\"\r\nexit /b 0\r\n", label),
    )
    .expect("wrapper should be written");
}

#[cfg(unix)]
fn make_probe(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, "#!/usr/bin/env sh\nprintf '%s|' \"$@\"\n")
        .expect("probe should be written");
    let mut permissions = std::fs::metadata(path)
        .expect("probe metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("probe should be executable");
}

fn path_suffix(path: &std::path::Path, count: usize) -> std::path::PathBuf {
    path.components()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[test]
fn profile_prints_paths() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let profile = project.join("runseal.toml");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&project).expect("project should be created");
    std::fs::write(&profile, "not valid profile toml").expect("profile should be written");

    let output = bin()
        .current_dir(&project)
        .env("RUNSEAL_HOME", &home)
        .args(["@profile"])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("RUNSEAL_HOME="));
    assert!(stdout.contains("RUNSEAL_PROFILE_HOME="));
    assert!(stdout.contains("RUNSEAL_PROFILE_PATH="));
    assert!(stdout.contains("RUNSEAL_WRAPPER_PATH="));
    assert!(stdout.contains(profile.to_str().expect("path should be UTF-8")));
}

#[test]
fn wrappers_show_effective() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let project_wrappers = project.join(".runseal/wrappers");
    let home = temp.path().join("home");
    let home_wrappers = home.join("wrappers");
    std::fs::create_dir_all(&project_wrappers).expect("project wrappers should be created");
    std::fs::create_dir_all(&home_wrappers).expect("home wrappers should be created");
    std::fs::write(project.join("runseal.toml"), "injections = []\n")
        .expect("profile should be written");
    make_wrapper(&wrapper_file(&project_wrappers, "wrap"), "project");
    make_wrapper(&wrapper_file(&home_wrappers, "wrap"), "home");
    make_wrapper(&wrapper_file(&home_wrappers, "home-only"), "home");

    let output = bin()
        .current_dir(&project)
        .env("RUNSEAL_HOME", &home)
        .args(["@wrappers"])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains(":wrap"));
    assert!(stdout.contains(":home-only"));
    assert!(stdout.contains("profile"));
    assert!(stdout.contains("home"));
    let wrap_line = stdout
        .lines()
        .find(|line| line.contains(":wrap"))
        .expect("wrap should be listed");
    let wrap_file = wrap_line
        .split_whitespace()
        .last()
        .expect("wrap line should include a file");
    assert!(wrap_line.contains("profile"));
    assert!(
        std::path::Path::new(wrap_file)
            .ends_with(path_suffix(&wrapper_file(&project_wrappers, "wrap"), 4)),
        "expected {wrap_file} to point at the profile wrapper"
    );
}

#[test]
fn which_resolves_wrapper() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    let project_wrappers = project.join(".runseal/wrappers");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&project_wrappers).expect("project wrappers should be created");
    std::fs::write(project.join("runseal.toml"), "injections = []\n")
        .expect("profile should be written");
    let wrapper = wrapper_file(&project_wrappers, "wrap");
    make_wrapper(&wrapper, "project");

    let output = bin()
        .current_dir(&project)
        .env("RUNSEAL_HOME", &home)
        .args(["@which", ":wrap"])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let expected_suffix = path_suffix(&wrapper, 4);
    assert!(
        std::path::Path::new(stdout.trim()).ends_with(&expected_suffix),
        "expected {stdout:?} to end with {}",
        expected_suffix.display()
    );
}

#[test]
fn which_rejects_external() {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    std::fs::create_dir_all(&project).expect("project should be created");
    std::fs::write(project.join("runseal.toml"), "injections = []\n")
        .expect("profile should be written");

    let output = bin()
        .current_dir(&project)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .args(["@which", "ssh"])
        .output()
        .expect("runseal should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("@which currently supports only :wrapper arguments"));
}

#[cfg(unix)]
#[test]
fn namespace_keeps_external() {
    let temp = TempDir::new().expect("temp dir should be created");
    let bin_dir = temp.path().join("bin");
    let profile = temp.path().join("profile.toml");
    std::fs::create_dir_all(&bin_dir).expect("bin dir should be created");
    make_probe(&bin_dir.join("profile"));
    std::fs::write(
        &profile,
        format!(
            r#"
[[injections]]
type = "env"

[[injections.ops]]
op = "prepend"
key = "PATH"
value = "{}"
separator = "os"
dedup = true
"#,
            bin_dir.display()
        ),
    )
    .expect("profile should be written");

    let output = bin()
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .arg("--profile")
        .arg(profile.to_str().expect("path should be UTF-8"))
        .args(["profile", "arg"])
        .output()
        .expect("runseal should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, "arg|");
}
