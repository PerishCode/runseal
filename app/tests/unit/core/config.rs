use super::*;
use tempfile::TempDir;

fn raw_env(home: PathBuf, runseal_home: Option<PathBuf>) -> RawEnv {
    RawEnv {
        home: Some(home),
        runseal_home,
        runseal_profile_home: None,
    }
}

#[test]
fn explicit_profile_wins() {
    let temp = TempDir::new().expect("temp dir should be created");
    let explicit = temp.path().join("explicit.yaml");
    let cwd = temp.path().join("cwd");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&cwd).expect("cwd should exist");
    std::fs::write(&explicit, "injections: []").expect("profile should be written");
    std::fs::write(cwd.join("runseal.toml"), "[[injections]]\ntype = \"env\"\n")
        .expect("cwd profile should be written");

    let cfg = RuntimeConfig::from_cli_env_and_cwd(
        CliInput {
            profile: Some(explicit.clone()),
            command: vec!["true".to_string()],
        },
        raw_env(home, None),
        &cwd,
    )
    .expect("config should resolve");

    assert_eq!(cfg.profile_path, explicit);
}

#[test]
fn cwd_toml_wins_over_cwd_yaml() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("cwd");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&cwd).expect("cwd should exist");
    std::fs::write(cwd.join("runseal.toml"), "injections = []")
        .expect("toml profile should be written");
    std::fs::write(cwd.join("runseal.yaml"), "injections: []")
        .expect("yaml profile should be written");

    let cfg = RuntimeConfig::from_cli_env_and_cwd(
        CliInput {
            profile: None,
            command: vec!["true".to_string()],
        },
        raw_env(home, None),
        &cwd,
    )
    .expect("config should resolve");

    assert_eq!(cfg.profile_path, cwd.join("runseal.toml"));
}

#[test]
fn profile_home_default_is_under_runseal_home() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("cwd");
    let runseal_home = temp.path().join("runseal-home");
    let profile_home = runseal_home.join("profiles");
    std::fs::create_dir_all(&cwd).expect("cwd should exist");
    std::fs::create_dir_all(&profile_home).expect("profile home should exist");
    std::fs::write(profile_home.join("default.json"), r#"{"injections":[]}"#)
        .expect("default profile should be written");

    let cfg = RuntimeConfig::from_cli_env_and_cwd(
        CliInput {
            profile: None,
            command: vec!["true".to_string()],
        },
        raw_env(temp.path().join("home"), Some(runseal_home.clone())),
        &cwd,
    )
    .expect("config should resolve");

    assert_eq!(cfg.runseal_home, runseal_home);
    assert_eq!(cfg.profile_home, profile_home);
    assert_eq!(cfg.profile_path, cfg.profile_home.join("default.json"));
}

#[test]
fn missing_profile_lists_searched_paths() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("cwd");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&cwd).expect("cwd should exist");

    let err = RuntimeConfig::from_cli_env_and_cwd(
        CliInput {
            profile: None,
            command: vec!["true".to_string()],
        },
        raw_env(home, None),
        &cwd,
    )
    .expect_err("missing profile should fail");

    let msg = err.to_string();
    assert!(msg.contains("runseal.toml"));
    assert!(msg.contains("default.json"));
}
