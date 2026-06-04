use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use runseal::core::app::AppState;
use runseal::core::config::{CliInput, RawEnv, RuntimeConfig};
use runseal::run;

#[derive(Debug, Parser)]
#[command(
    name = "runseal",
    version = build_version(),
    about = "Run a command inside an env, symlink, argv, and wrapper profile.",
    after_help = "Profile discovery: --profile, ./runseal.toml|yaml|yml|json, then $RUNSEAL_PROFILE_HOME/default.toml|yaml|yml|json."
)]
struct Cli {
    #[arg(short = 'p', long = "profile")]
    profile: Option<PathBuf>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.command.is_empty() {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }

    let config = build_runtime_config(cli)?;
    let app = AppState::new(config);
    let result = run(&app)?;
    if let Some(code) = result.exit_code {
        process::exit(code);
    }
    Ok(())
}

fn build_runtime_config(cli: Cli) -> Result<RuntimeConfig> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    RuntimeConfig::from_input(
        CliInput {
            profile: cli.profile,
            command: normalize_command(cli.command),
        },
        RawEnv::from_process(),
        &cwd,
    )
}

fn normalize_command(mut command: Vec<String>) -> Vec<String> {
    if command.len() > 1 && command.get(1).map(String::as_str) == Some("--") {
        command.remove(1);
    }
    command
}

fn build_version() -> &'static str {
    option_env!("RUNSEAL_BUILD_VERSION").unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}
