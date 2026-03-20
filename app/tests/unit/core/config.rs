use super::*;
use tempfile::TempDir;

fn base_cli() -> CliInput {
    CliInput {
        profile: None,
        output_mode: OutputMode::Shell,
        strict: false,
        log_level: LevelFilter::WARN,
        log_format: LogFormat::Text,
        command: Vec::new(),
    }
}

#[test]
fn default_profile_uses_runseal_home() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    let profiles = runseal_home.join("profiles");
    std::fs::create_dir_all(&profiles).expect("profiles dir should be created");
    std::fs::write(profiles.join("default.json"), "{\"injections\":[]}")
        .expect("default profile should be written");

    let cfg = RuntimeConfig::from_cli_and_env(
        base_cli(),
        RawEnv {
            home: Some(PathBuf::from("/Users/tester")),
            runseal_home: Some(runseal_home.clone()),
            runseal_resource_home: None,
        },
    )
    .expect("config should build");

    assert_eq!(cfg.runseal_home, runseal_home);
    assert_eq!(
        cfg.profile_path,
        temp.path().join("runseal-home/profiles/default.json")
    );
}

#[test]
fn profile_flag_overrides_default_resolution() {
    let temp = TempDir::new().expect("temp dir should be created");
    let explicit = temp.path().join("explicit.json");
    std::fs::write(&explicit, "{\"injections\":[]}").expect("profile should be written");

    let mut cli = base_cli();
    cli.profile = Some(explicit.clone());

    let cfg = RuntimeConfig::from_cli_and_env(
        cli,
        RawEnv {
            home: Some(PathBuf::from("/Users/tester")),
            runseal_home: None,
            runseal_resource_home: None,
        },
    )
    .expect("config should build");

    assert_eq!(cfg.profile_path, explicit);
}

#[test]
fn resource_home_defaults_from_runseal_home() {
    let temp = TempDir::new().expect("temp dir should be created");
    let runseal_home = temp.path().join("runseal-home");
    std::fs::create_dir_all(runseal_home.join("profiles")).expect("profiles dir should exist");
    std::fs::write(
        runseal_home.join("profiles/default.json"),
        "{\"injections\":[]}",
    )
    .expect("default profile should be written");

    let cfg = RuntimeConfig::from_cli_and_env(
        base_cli(),
        RawEnv {
            home: Some(PathBuf::from("/Users/tester")),
            runseal_home: Some(runseal_home.clone()),
            runseal_resource_home: None,
        },
    )
    .expect("config should build");

    assert_eq!(cfg.resource_home, runseal_home.join("resources"));
}

#[test]
fn missing_default_profile_returns_actionable_error() {
    let err = RuntimeConfig::from_cli_and_env(
        base_cli(),
        RawEnv {
            home: Some(PathBuf::from("/Users/tester")),
            runseal_home: Some(PathBuf::from("/tmp/does-not-exist")),
            runseal_resource_home: None,
        },
    )
    .expect_err("missing default profile should fail");
    assert!(err.to_string().contains("profiles/default.json"));
}

#[test]
fn missing_home_and_runseal_home_fails() {
    let err = RuntimeConfig::from_cli_and_env(
        base_cli(),
        RawEnv {
            home: None,
            runseal_home: None,
            runseal_resource_home: None,
        },
    )
    .expect_err("missing home should fail");
    assert!(err.to_string().contains("HOME is not set"));
}

#[test]
fn empty_runseal_home_is_treated_as_unset() {
    let temp = TempDir::new().expect("temp dir should be created");
    let home = temp.path().join("home");
    std::fs::create_dir_all(home.join(".runseal/profiles")).expect("profiles dir should exist");
    std::fs::write(
        home.join(".runseal/profiles/default.json"),
        "{\"injections\":[]}",
    )
    .expect("default profile should be written");

    let cfg = RuntimeConfig::from_cli_and_env(
        base_cli(),
        RawEnv {
            home: Some(home.clone()),
            runseal_home: Some(PathBuf::new()),
            runseal_resource_home: None,
        },
    )
    .expect("config should fall back to HOME/.runseal");

    assert_eq!(cfg.runseal_home, home.join(".runseal"));
}
