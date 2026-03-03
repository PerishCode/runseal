mod command;
mod env;
mod symlink;

use anyhow::{Context, Result, anyhow};
use std::collections::BTreeMap;
use tracing::{debug, info};

use crate::app::AppContext;
use crate::profile::InjectionProfile;
use command::CommandInjection;
use env::EnvInjection;
use symlink::SymlinkInjection;

pub fn execute_lifecycle(
    app: &dyn AppContext,
    specs: Vec<InjectionProfile>,
) -> Result<Vec<(String, String)>> {
    with_registered_exports(app, specs, |exports| Ok(exports.to_vec()))
}

pub fn with_registered_exports<T, F>(
    app: &dyn AppContext,
    specs: Vec<InjectionProfile>,
    work: F,
) -> Result<T>
where
    F: FnOnce(&[(String, String)]) -> Result<T>,
{
    let mut injections = build_injections(specs);
    info!(
        injection_count = injections.len(),
        "starting injection lifecycle"
    );

    for injection in &injections {
        debug!(
            injection = injection.name(),
            stage = "validate",
            "running stage"
        );
        injection
            .validate()
            .with_context(|| format!("{} validation failed", injection.name()))?;
    }

    let mut registered = 0usize;
    for injection in &mut injections {
        debug!(
            injection = injection.name(),
            stage = "register",
            "running stage"
        );
        injection
            .register()
            .with_context(|| format!("{} registration failed", injection.name()))?;
        registered += 1;
    }

    let export_result = collect_exports(app, &injections);
    let work_result = match &export_result {
        Ok(exports) => Some(work(exports)),
        Err(_) => None,
    };
    let shutdown_result = shutdown_registered(&mut injections, registered);

    match (export_result, work_result, shutdown_result) {
        (Ok(_), Some(Ok(result)), Ok(())) => Ok(result),
        (Ok(_), Some(Err(work_err)), Ok(())) => Err(work_err),
        (Err(err), _, Ok(())) => Err(err),
        (Ok(_), Some(Ok(_)), Err(shutdown_err)) => Err(shutdown_err),
        (Ok(_), Some(Err(work_err)), Err(shutdown_err)) => {
            Err(anyhow!("{work_err}; also failed shutdown: {shutdown_err}"))
        }
        (Err(export_err), _, Err(shutdown_err)) => Err(anyhow!(
            "{export_err}; also failed shutdown: {shutdown_err}"
        )),
        (Ok(_), None, Ok(())) => Err(anyhow!("internal lifecycle error: missing work result")),
        (Ok(_), None, Err(shutdown_err)) => Err(shutdown_err),
    }
}

fn collect_exports(
    app: &dyn AppContext,
    injections: &[RuntimeInjection],
) -> Result<Vec<(String, String)>> {
    let mut exports = Vec::new();
    let mut inherited = BTreeMap::new();
    for injection in injections {
        debug!(
            injection = injection.name(),
            stage = "export",
            "running stage"
        );
        let exported = injection
            .export(app, &inherited)
            .with_context(|| format!("{} export failed", injection.name()))?;
        debug!(
            injection = injection.name(),
            export_count = exported.len(),
            "export stage completed"
        );
        for (key, value) in &exported {
            inherited.insert(key.clone(), value.clone());
        }
        exports.extend(exported);
    }
    info!(export_count = exports.len(), "export collection completed");
    Ok(exports)
}

fn shutdown_registered(injections: &mut [RuntimeInjection], registered: usize) -> Result<()> {
    for idx in (0..registered).rev() {
        debug!(
            injection = injections[idx].name(),
            stage = "shutdown",
            "running stage"
        );
        injections[idx]
            .shutdown()
            .with_context(|| format!("{} shutdown failed", injections[idx].name()))?;
    }
    info!(registered_count = registered, "shutdown completed");
    Ok(())
}

fn build_injections(specs: Vec<InjectionProfile>) -> Vec<RuntimeInjection> {
    let mut injections = Vec::new();
    for spec in specs {
        match spec {
            InjectionProfile::Env(cfg) if cfg.enabled => {
                injections.push(RuntimeInjection::Env(EnvInjection::new(cfg)));
            }
            InjectionProfile::Command(cfg) if cfg.enabled => {
                injections.push(RuntimeInjection::Command(CommandInjection::new(cfg)));
            }
            InjectionProfile::Symlink(cfg) if cfg.enabled => {
                injections.push(RuntimeInjection::Symlink(SymlinkInjection::new(cfg)));
            }
            _ => {}
        }
    }
    debug!(
        enabled_injections = injections.len(),
        "built enabled injections"
    );
    injections
}

enum RuntimeInjection {
    Env(EnvInjection),
    Command(CommandInjection),
    Symlink(SymlinkInjection),
}

impl RuntimeInjection {
    fn name(&self) -> &'static str {
        match self {
            Self::Env(inner) => inner.name(),
            Self::Command(inner) => inner.name(),
            Self::Symlink(inner) => inner.name(),
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            Self::Env(inner) => inner.validate(),
            Self::Command(inner) => inner.validate(),
            Self::Symlink(inner) => inner.validate(),
        }
    }

    fn register(&mut self) -> Result<()> {
        match self {
            Self::Env(inner) => inner.register(),
            Self::Command(inner) => inner.register(),
            Self::Symlink(inner) => inner.register(),
        }
    }

    fn export(
        &self,
        app: &dyn AppContext,
        inherited: &BTreeMap<String, String>,
    ) -> Result<Vec<(String, String)>> {
        match self {
            Self::Env(inner) => inner.export(app),
            Self::Command(inner) => inner.export(app, inherited),
            Self::Symlink(inner) => inner.export(),
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        match self {
            Self::Env(inner) => inner.shutdown(),
            Self::Command(inner) => inner.shutdown(),
            Self::Symlink(inner) => inner.shutdown(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppContext, CommandRunner, EnvReader};
    use crate::config::{LogFormat, OutputMode, RuntimeConfig};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
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
                    envlock_home: PathBuf::from("/tmp/envlock-home"),
                    resource_home: PathBuf::from("/tmp/envlock-res"),
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
            InjectionProfile::Env(crate::profile::EnvProfile {
                enabled: false,
                vars: BTreeMap::from([("A".to_string(), "1".to_string())]),
                ops: Vec::new(),
            }),
            InjectionProfile::Env(crate::profile::EnvProfile {
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
        let specs = vec![InjectionProfile::Env(crate::profile::EnvProfile {
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
        let specs = vec![InjectionProfile::Command(crate::profile::CommandProfile {
            enabled: true,
            program: "bash".to_string(),
            args: vec![
                "-lc".to_string(),
                "printf \"export CMD_A='1'\\nCMD_B=2\\n\"".to_string(),
            ],
        })];

        let app = TestApp::new();
        let exports = execute_lifecycle(&app, specs).expect("command lifecycle should pass");
        assert!(exports.contains(&("CMD_A".to_string(), "1".to_string())));
        assert!(exports.contains(&("CMD_B".to_string(), "2".to_string())));
    }

    #[test]
    fn command_injection_observes_prior_exports() {
        let specs = vec![
            InjectionProfile::Env(crate::profile::EnvProfile {
                enabled: true,
                vars: BTreeMap::from([("BASE".to_string(), "seed".to_string())]),
                ops: Vec::new(),
            }),
            InjectionProfile::Command(crate::profile::CommandProfile {
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
}
