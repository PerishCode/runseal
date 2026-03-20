mod command;
mod env;
mod symlink;

use anyhow::{Context, Result, anyhow};
use std::collections::BTreeMap;
use tracing::{debug, info};

use crate::core::app::AppContext;
use crate::core::profile::InjectionProfile;
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

    let (registered, register_result) = register_injections(&mut injections);
    if let Err(register_err) = register_result {
        let shutdown_result = shutdown_registered(&mut injections, registered);
        return match shutdown_result {
            Ok(()) => Err(register_err),
            Err(shutdown_err) => Err(anyhow!(
                "{register_err}; also failed shutdown: {shutdown_err}"
            )),
        };
    }

    let work_result = run_export_and_work(app, &injections, work);
    let shutdown_result = shutdown_registered(&mut injections, registered);

    match (work_result, shutdown_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(shutdown_err)) => Err(shutdown_err),
        (Err(primary), Err(shutdown_err)) => {
            Err(anyhow!("{primary}; also failed shutdown: {shutdown_err}"))
        }
    }
}

fn register_injections(injections: &mut [RuntimeInjection]) -> (usize, Result<()>) {
    let mut registered = 0usize;
    for injection in injections {
        debug!(
            injection = injection.name(),
            stage = "register",
            "running stage"
        );
        if let Err(err) = injection.register() {
            return (
                registered,
                Err(err).with_context(|| format!("{} registration failed", injection.name())),
            );
        }
        registered += 1;
    }
    (registered, Ok(()))
}

fn run_export_and_work<T, F>(
    app: &dyn AppContext,
    injections: &[RuntimeInjection],
    work: F,
) -> Result<T>
where
    F: FnOnce(&[(String, String)]) -> Result<T>,
{
    let exports = collect_exports(app, injections)?;
    work(&exports)
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
#[path = "../../../tests/unit/core/injections/mod.rs"]
mod tests;
