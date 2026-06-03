use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;
use crate::core::app::{AppContext, EnvReader};
use crate::core::config::RuntimeConfig;
use crate::core::profile::{EnvProfile, SymlinkOnExist, SymlinkProfile};
use tempfile::TempDir;

struct TestEnv;

impl EnvReader for TestEnv {
    fn var(&self, _key: &str) -> Option<String> {
        None
    }
}

struct TestApp {
    cfg: RuntimeConfig,
    env: TestEnv,
}

impl TestApp {
    fn new(profile_path: PathBuf) -> Self {
        Self {
            cfg: RuntimeConfig {
                profile_path,
                command: vec!["true".to_string()],
                runseal_home: PathBuf::from("/tmp/runseal"),
                profile_home: PathBuf::from("/tmp/runseal/profiles"),
            },
            env: TestEnv,
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
fn lifecycle_cleans_registered_symlink_when_work_fails() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    std::fs::write(&source, "x").expect("source should be written");
    let app = TestApp::new(temp.path().join("profile.json"));

    let result: anyhow::Result<()> = with_registered_exports(
        &app,
        vec![
            InjectionProfile::Env(EnvProfile {
                enabled: true,
                vars: BTreeMap::new(),
                ops: Vec::new(),
            }),
            InjectionProfile::Symlink(SymlinkProfile {
                enabled: true,
                source,
                target: target.clone(),
                on_exist: SymlinkOnExist::Error,
                cleanup: true,
            }),
        ],
        |_exports| anyhow::bail!("work failed"),
    );
    let err = result.expect_err("work should fail");

    assert!(err.to_string().contains("work failed"));
    assert!(!target.exists());
}
