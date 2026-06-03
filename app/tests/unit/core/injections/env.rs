use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;
use crate::core::app::{AppContext, EnvReader};
use crate::core::config::RuntimeConfig;

struct TestEnv {
    vars: BTreeMap<String, String>,
}

impl EnvReader for TestEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

struct TestApp {
    cfg: RuntimeConfig,
    env: TestEnv,
}

impl TestApp {
    fn new(vars: BTreeMap<String, String>) -> Self {
        Self {
            cfg: RuntimeConfig {
                profile_path: PathBuf::from("/tmp/profile.toml"),
                command: vec!["true".to_string()],
                runseal_home: PathBuf::from("/tmp/runseal"),
                profile_home: PathBuf::from("/tmp/runseal/profiles"),
            },
            env: TestEnv { vars },
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
    let app = TestApp::new(BTreeMap::new());

    let exports = injection.export(&app).expect("export should pass");
    let path = exports
        .into_iter()
        .find(|(k, _)| k == "PATH")
        .map(|(_, v)| v)
        .expect("PATH should exist");
    assert_eq!(path, "/custom/bin:/usr/bin:/bin");
}

#[test]
fn set_if_absent_reads_process_env_overlay() {
    let key = "RUNSEAL_TEST_SET_IF_ABSENT";
    let app = TestApp::new(BTreeMap::from([(key.to_string(), "present".to_string())]));
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
