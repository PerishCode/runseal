use super::*;
use crate::core::app::{AppContext, CommandRunner, EnvReader};
use crate::core::config::{LogFormat, OutputMode, RuntimeConfig};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing_subscriber::filter::LevelFilter;

struct TestEnv;

impl EnvReader for TestEnv {
    fn var(&self, _key: &str) -> Option<String> {
        None
    }
}

struct TestRunner;

impl CommandRunner for TestRunner {
    fn output(&self, program: &str, args: &[String]) -> Result<std::process::Output> {
        std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(Into::into)
    }
}

struct TestApp {
    cfg: RuntimeConfig,
    env: TestEnv,
    runner: TestRunner,
}

impl TestApp {
    fn new() -> Self {
        Self {
            cfg: RuntimeConfig {
                profile_path: PathBuf::from("/tmp/unused.json"),
                output_mode: OutputMode::Shell,
                strict: false,
                log_level: LevelFilter::WARN,
                log_format: LogFormat::Text,
                command: None,
                runseal_home: PathBuf::from("/tmp/runseal-home"),
                resource_home: PathBuf::from("/tmp/runseal-res"),
            },
            env: TestEnv,
            runner: TestRunner,
        }
    }
}

impl AppContext for TestApp {
    fn config(&self) -> &RuntimeConfig {
        &self.cfg
    }

    fn env(&self) -> &dyn EnvReader {
        &self.env
    }

    fn command_runner(&self) -> &dyn CommandRunner {
        &self.runner
    }
}

#[test]
fn skip_disabled_env_injection() {
    let specs = vec![
        InjectionProfile::Env(crate::core::profile::EnvProfile {
            enabled: false,
            vars: BTreeMap::from([("A".to_string(), "1".to_string())]),
            ops: Vec::new(),
        }),
        InjectionProfile::Env(crate::core::profile::EnvProfile {
            enabled: true,
            vars: BTreeMap::from([("B".to_string(), "2".to_string())]),
            ops: Vec::new(),
        }),
    ];

    let app = TestApp::new();
    let exports = execute_lifecycle(&app, specs).expect("lifecycle should pass");
    assert_eq!(exports.len(), 1);
    assert!(exports.contains(&("B".to_string(), "2".to_string())));
}

#[test]
fn fail_validation_when_env_key_is_empty() {
    let specs = vec![InjectionProfile::Env(crate::core::profile::EnvProfile {
        enabled: true,
        vars: BTreeMap::from([("   ".to_string(), "1".to_string())]),
        ops: Vec::new(),
    })];

    let app = TestApp::new();
    let err = execute_lifecycle(&app, specs).expect_err("empty env key should fail");
    assert!(err.to_string().contains("validation failed"));
}

#[test]
fn command_injection_exports_values() {
    let specs = vec![InjectionProfile::Command(
        crate::core::profile::CommandProfile {
            enabled: true,
            program: "bash".to_string(),
            args: vec![
                "-lc".to_string(),
                "printf \"export CMD_A='1'\\nCMD_B=2\\n\"".to_string(),
            ],
        },
    )];

    let app = TestApp::new();
    let exports = execute_lifecycle(&app, specs).expect("command lifecycle should pass");
    assert!(exports.contains(&("CMD_A".to_string(), "1".to_string())));
    assert!(exports.contains(&("CMD_B".to_string(), "2".to_string())));
}

#[test]
fn command_injection_observes_prior_exports() {
    let specs = vec![
        InjectionProfile::Env(crate::core::profile::EnvProfile {
            enabled: true,
            vars: BTreeMap::from([("BASE".to_string(), "seed".to_string())]),
            ops: Vec::new(),
        }),
        InjectionProfile::Command(crate::core::profile::CommandProfile {
            enabled: true,
            program: "bash".to_string(),
            args: vec![
                "-lc".to_string(),
                "printf 'export DERIVED=${BASE}-ok\\n'".to_string(),
            ],
        }),
    ];

    let app = TestApp::new();
    let exports = execute_lifecycle(&app, specs).expect("command should see prior exports");
    assert!(exports.contains(&("BASE".to_string(), "seed".to_string())));
    assert!(exports.contains(&("DERIVED".to_string(), "seed-ok".to_string())));
}

#[test]
fn register_failure_rolls_back_prior_registered_injections() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source_a = temp.path().join("source-a");
    let source_b = temp.path().join("source-b");
    let target_a = temp.path().join("target-a");
    let target_b = temp.path().join("target-b");

    std::fs::write(&source_a, "a").expect("source-a should exist");
    std::fs::write(&source_b, "b").expect("source-b should exist");
    std::fs::write(&target_b, "occupied").expect("target-b should exist");

    let specs = vec![
        InjectionProfile::Symlink(crate::core::profile::SymlinkProfile {
            enabled: true,
            source: source_a.clone(),
            target: target_a.clone(),
            on_exist: crate::core::profile::SymlinkOnExist::Error,
            cleanup: true,
        }),
        InjectionProfile::Symlink(crate::core::profile::SymlinkProfile {
            enabled: true,
            source: source_b,
            target: target_b,
            on_exist: crate::core::profile::SymlinkOnExist::Error,
            cleanup: true,
        }),
    ];

    let app = TestApp::new();
    let err = execute_lifecycle(&app, specs).expect_err("second register should fail");
    assert!(err.to_string().contains("registration failed"));
    assert!(
        std::fs::symlink_metadata(&target_a).is_err(),
        "first symlink should be rolled back on later register failure"
    );
}
