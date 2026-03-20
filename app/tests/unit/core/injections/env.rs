use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;
use crate::core::app::{AppContext, CommandRunner, EnvReader};
use crate::core::config::{LogFormat, OutputMode, RuntimeConfig};
use tracing_subscriber::filter::LevelFilter;

struct TestEnv {
    vars: BTreeMap<String, String>,
}

impl EnvReader for TestEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
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
    fn new(resource_home: &str, vars: BTreeMap<String, String>) -> Self {
        Self {
            cfg: RuntimeConfig {
                profile_path: PathBuf::from("/tmp/unused.json"),
                output_mode: OutputMode::Shell,
                strict: false,
                log_level: LevelFilter::WARN,
                log_format: LogFormat::Text,
                command: None,
                runseal_home: PathBuf::from("/tmp/runseal-home"),
                resource_home: PathBuf::from(resource_home),
            },
            env: TestEnv { vars },
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
fn rejects_empty_env_key() {
    let mut vars = BTreeMap::new();
    vars.insert("   ".to_string(), "x".to_string());
    let injection = EnvInjection::new(EnvProfile {
        enabled: true,
        vars,
        ops: Vec::new(),
    });
    let err = injection.validate().expect_err("empty key should fail");
    assert!(err.to_string().contains("env var key must not be empty"));
}

#[test]
fn prepend_path_with_dedup() {
    let mut vars = BTreeMap::new();
    vars.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    let injection = EnvInjection::new(EnvProfile {
        enabled: true,
        vars,
        ops: vec![EnvOpProfile::Prepend {
            key: "PATH".to_string(),
            value: "/custom/bin:/usr/bin".to_string(),
            separator: Some("os".to_string()),
            dedup: true,
        }],
    });
    let app = TestApp::new("/tmp/runseal-res", BTreeMap::new());

    let exports = injection.export(&app).expect("export should pass");
    let path = exports
        .into_iter()
        .find(|(k, _)| k == "PATH")
        .map(|(_, v)| v)
        .expect("PATH should exist");
    assert_eq!(path, "/custom/bin:/usr/bin:/bin");
}

#[test]
fn set_if_absent_uses_current_env() {
    let key = "RUNSEAL_TEST_SET_IF_ABSENT";
    let app = TestApp::new(
        "/tmp/runseal-res",
        BTreeMap::from([(key.to_string(), "present".to_string())]),
    );
    let injection = EnvInjection::new(EnvProfile {
        enabled: true,
        vars: BTreeMap::new(),
        ops: vec![EnvOpProfile::SetIfAbsent {
            key: key.to_string(),
            value: "fallback".to_string(),
        }],
    });
    let exports = injection.export(&app).expect("export should pass");
    assert!(!exports.iter().any(|(k, _)| k == key));
}

#[test]
fn resolves_resource_uri_with_default_home() {
    let resolved = resolve_resource_refs(
        "resource://kubeconfig/xx.yaml",
        std::path::Path::new("/tmp/runseal-res"),
    )
    .expect("resource path should resolve");
    assert_eq!(resolved, "/tmp/runseal-res/kubeconfig/xx.yaml");
}

#[test]
fn resolves_multiple_resource_uris_in_one_value() {
    let resolved = resolve_resource_refs(
        "resource://kubeconfig/xx.yaml:resource://kubeconfig/yy.yaml",
        std::path::Path::new("/tmp/runseal-res"),
    )
    .expect("multiple resource paths should resolve");
    assert_eq!(
        resolved,
        "/tmp/runseal-res/kubeconfig/xx.yaml:/tmp/runseal-res/kubeconfig/yy.yaml"
    );
}

#[test]
fn resolves_resource_content_uri() {
    let temp = tempfile::tempdir().expect("temp dir should exist");
    let dir = temp.path().join("opencode");
    std::fs::create_dir_all(&dir).expect("resource dir should exist");
    let cfg = dir.join("alpha.json");
    std::fs::write(&cfg, "{\"default_agent\":\"alpha\"}")
        .expect("resource content should be written");

    let resolved = resolve_resource_refs("resource-content://opencode/alpha.json", temp.path())
        .expect("resource content should resolve");
    assert_eq!(resolved, "{\"default_agent\":\"alpha\"}");
}

#[test]
fn resolves_resource_content_followed_by_separator() {
    let temp = tempfile::tempdir().expect("temp dir should exist");
    std::fs::write(temp.path().join("token.txt"), "ALPHA_ONLY")
        .expect("resource content should be written");

    let resolved = resolve_resource_refs("A=resource-content://token.txt;B=1", temp.path())
        .expect("resource content with separator should resolve");
    assert_eq!(resolved, "A=ALPHA_ONLY;B=1");
}

#[test]
fn errors_when_resource_content_missing() {
    let err = resolve_resource_refs(
        "resource-content://missing.json",
        std::path::Path::new("/tmp/runseal-res"),
    )
    .expect_err("missing content file should fail");
    assert!(err.to_string().contains("failed to read resource content"));
}
