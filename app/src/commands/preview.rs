use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::core::profile::{EnvProfile, InjectionProfile};

#[derive(Debug, Clone, Copy)]
pub enum PreviewOutputMode {
    Text,
    Json,
}

#[derive(Debug, Serialize)]
struct PreviewReport {
    profile: String,
    injections: Vec<PreviewInjection>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PreviewInjection {
    Env {
        enabled: bool,
        keys: Vec<String>,
    },
    Command {
        enabled: bool,
        program: String,
        arg_count: usize,
    },
    Symlink {
        enabled: bool,
        source: String,
        target: String,
        on_exist: String,
        cleanup: bool,
    },
}

pub fn run(profile_path: &Path, output_mode: PreviewOutputMode) -> Result<()> {
    let report = build_report(profile_path)?;

    match output_mode {
        PreviewOutputMode::Text => print_text(&report),
        PreviewOutputMode::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }

    Ok(())
}

fn build_report(profile_path: &Path) -> Result<PreviewReport> {
    let profile = crate::core::profile::load(profile_path)?;
    let injections = profile.injections.into_iter().map(map_injection).collect();
    Ok(PreviewReport {
        profile: profile_path.display().to_string(),
        injections,
    })
}

fn map_injection(injection: InjectionProfile) -> PreviewInjection {
    match injection {
        InjectionProfile::Env(env) => PreviewInjection::Env {
            enabled: env.enabled,
            keys: collect_env_keys(env),
        },
        InjectionProfile::Command(command) => PreviewInjection::Command {
            enabled: command.enabled,
            program: command.program,
            arg_count: command.args.len(),
        },
        InjectionProfile::Symlink(symlink) => PreviewInjection::Symlink {
            enabled: symlink.enabled,
            source: symlink.source.to_string_lossy().to_string(),
            target: symlink.target.to_string_lossy().to_string(),
            on_exist: format!("{:?}", symlink.on_exist).to_lowercase(),
            cleanup: symlink.cleanup,
        },
    }
}

fn collect_env_keys(env: EnvProfile) -> Vec<String> {
    let mut keys: BTreeSet<String> = env.vars.keys().cloned().collect();
    for op in env.ops {
        keys.insert(op.key().to_string());
    }
    keys.into_iter().collect()
}

fn print_text(report: &PreviewReport) {
    println!("profile: {}", report.profile);
    println!("injections: {}", report.injections.len());

    for injection in &report.injections {
        match injection {
            PreviewInjection::Env { enabled, keys } => {
                println!("- [env] enabled={} keys=[{}]", enabled, keys.join(", "));
            }
            PreviewInjection::Command {
                enabled,
                program,
                arg_count,
            } => {
                println!(
                    "- [command] enabled={} program={} arg_count={}",
                    enabled, program, arg_count
                );
            }
            PreviewInjection::Symlink {
                enabled,
                source,
                target,
                on_exist,
                cleanup,
            } => {
                println!(
                    "- [symlink] enabled={} source={} target={} on_exist={} cleanup={}",
                    enabled, source, target, on_exist, cleanup
                );
            }
        }
    }
}
