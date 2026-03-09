use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use envlock::commands::alias::{
    resolve_profile_for_alias, run_append as run_alias_append, run_list as run_alias_list,
    AliasAppendOptions,
};
use envlock::commands::plugin::{run as run_plugin, PluginRunOptions};
use envlock::commands::preview::{run as run_preview, PreviewOutputMode};
use envlock::commands::profiles::{
    run_init as run_profiles_init, run_status as run_profiles_status, InitProfileType,
    ProfilesInitOptions,
};
use envlock::commands::self_update::{run as run_self_update, SelfUpdateOptions};
use envlock::commands::skill::{run_install as run_skill_install, SkillInstallOptions};
use envlock::core::app::App;
use envlock::core::config::{
    CliInput, LogFormat as RuntimeLogFormat, OutputMode, RawEnv, RuntimeConfig,
};
use envlock::logging::{current_log_file, make_file_writer, prepare_session_log, SessionLog};
use envlock::plugins::host::plugin_exit_code;
use envlock::run;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Parser)]
#[command(
    name = "envlock",
    version,
    about = "Build environment sessions from JSON profile",
    after_help = "Docs: https://perishcode.github.io/envlock/"
)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Commands>,

    #[command(flatten)]
    run_args: RunArgs,
}

#[derive(Debug, Subcommand)]
enum Commands {
    SelfUpdate(SelfUpdateArgs),
    Preview(PreviewArgs),
    Profiles(ProfilesArgs),
    Alias(AliasArgs),
    Skill(SkillArgs),
    Plugin(PluginArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Debug, Args)]
struct ProfilesArgs {
    #[command(subcommand)]
    command: ProfilesSubcommand,
}

#[derive(Debug, Subcommand)]
enum ProfilesSubcommand {
    Status,
    Init(ProfilesInitArgs),
}

#[derive(Debug, Args)]
struct ProfilesInitArgs {
    #[arg(long = "type", value_enum, default_value = "minimal")]
    profile_type: ProfileTemplateType,

    #[arg(long = "name")]
    name: Option<String>,

    #[arg(long = "force")]
    force: bool,
}

#[derive(Debug, Args)]
struct AliasArgs {
    #[command(subcommand)]
    command: AliasSubcommand,
}

#[derive(Debug, Subcommand)]
enum AliasSubcommand {
    List,
    Append(AliasAppendArgs),
    Run(AliasRunArgs),
}

#[derive(Debug, Args)]
struct AliasAppendArgs {
    name: String,

    #[arg(long = "profile")]
    profile: String,
}

#[derive(Debug, Args)]
struct AliasRunArgs {
    name: String,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

#[derive(Debug, Args)]
struct SkillArgs {
    #[command(subcommand)]
    command: SkillSubcommand,
}

#[derive(Debug, Subcommand)]
enum SkillSubcommand {
    Install(SkillInstallArgs),
}

#[derive(Debug, Args)]
struct PluginArgs {
    plugin: String,

    method: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Debug, Args)]
struct SkillInstallArgs {
    #[arg(long = "version")]
    version: Option<String>,

    #[arg(long = "force")]
    force: bool,

    #[arg(long = "yes", short = 'y')]
    yes: bool,
}

#[derive(Debug, Args)]
struct SelfUpdateArgs {
    #[arg(long = "check")]
    check: bool,

    #[arg(long = "version")]
    version: Option<String>,

    #[arg(long = "yes", short = 'y')]
    yes: bool,
}

#[derive(Debug, Args)]
struct PreviewArgs {
    #[arg(short = 'p', long = "profile")]
    profile: PathBuf,

    #[arg(long = "output", default_value = "text", value_enum)]
    output: PreviewOutputFormat,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(short = 'p', long = "profile")]
    profile: Option<PathBuf>,

    #[arg(long = "output", default_value = "shell", value_enum)]
    output: OutputFormat,

    #[arg(long = "strict")]
    strict: bool,

    #[arg(long = "log-level", default_value = "warn", value_enum)]
    log_level: LogLevel,

    #[arg(long = "log-format", default_value = "text", value_enum)]
    log_format: LogFormat,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let raw_env = RawEnv::from_process();
    let session_log = prepare_session_log(&raw_env, &command_slug(&cli)).ok();
    init_logging(
        cli.run_args.log_level.into(),
        cli.run_args.log_format.into(),
        session_log.as_ref(),
    )?;
    tracing::info!(command = %command_slug(&cli), "envlock invocation started");

    if let Some(command) = cli.subcommand {
        let result = match command {
            Commands::SelfUpdate(args) => run_self_update(SelfUpdateOptions {
                check_only: args.check,
                version: args.version,
                yes: args.yes,
            }),
            Commands::Preview(args) => run_preview(
                &args.profile,
                match args.output {
                    PreviewOutputFormat::Text => PreviewOutputMode::Text,
                    PreviewOutputFormat::Json => PreviewOutputMode::Json,
                },
            ),
            Commands::Profiles(args) => match args.command {
                ProfilesSubcommand::Status => run_profiles_status(),
                ProfilesSubcommand::Init(init) => run_profiles_init(ProfilesInitOptions {
                    profile_type: match init.profile_type {
                        ProfileTemplateType::Minimal => InitProfileType::Minimal,
                        ProfileTemplateType::Sample => InitProfileType::Sample,
                    },
                    name: init.name,
                    force: init.force,
                }),
            },
            Commands::Alias(args) => match args.command {
                AliasSubcommand::List => run_alias_list(),
                AliasSubcommand::Append(append) => run_alias_append(AliasAppendOptions {
                    name: append.name,
                    profile: append.profile,
                }),
                AliasSubcommand::Run(run_args) => {
                    run_alias_named(&run_args.name, &cli.run_args, Some(run_args.command))
                }
            },
            Commands::Skill(args) => match args.command {
                SkillSubcommand::Install(install) => run_skill_install(SkillInstallOptions {
                    version: install.version,
                    force: install.force,
                    yes: install.yes,
                }),
            },
            Commands::Plugin(args) => {
                let result = run_plugin(PluginRunOptions {
                    force_install: args.method == "init"
                        && args.args.iter().any(|arg| arg == "--force"),
                    plugin: args.plugin,
                    method: args.method,
                    args: args.args,
                });
                if let Err(error) = result {
                    if let Some(code) = plugin_exit_code(&error) {
                        eprintln!("{error}");
                        process::exit(code);
                    }
                    Err(error)
                } else {
                    Ok(())
                }
            }
            Commands::External(tokens) => run_external_command(&tokens, &cli.run_args),
        };
        return finish_command(result);
    }

    if let Some(alias_name) = parse_shortcut_alias_name(&cli.run_args.command) {
        let command = if cli.run_args.command.len() > 1 {
            Some(cli.run_args.command[1..].to_vec())
        } else {
            Some(Vec::new())
        };
        return finish_command(run_alias_named(alias_name, &cli.run_args, command));
    }

    let config = match build_runtime_config(&cli.run_args, None, None) {
        Ok(config) => config,
        Err(error) => return finish_command(Err(error)),
    };
    let app = App::new(config);
    let result = match run(&app) {
        Ok(result) => result,
        Err(error) => return finish_command(Err(error)),
    };
    if let Some(code) = result.exit_code {
        process::exit(code);
    }
    finish_command(Ok(()))
}

fn finish_command(result: Result<()>) -> Result<()> {
    match result {
        Ok(()) => {
            tracing::info!(exit_code = 0, "envlock invocation completed");
            Ok(())
        }
        Err(error) => {
            if let Some(code) = plugin_exit_code(&error) {
                tracing::error!(exit_code = code, error = %error, "envlock invocation failed");
                eprintln!("{error}");
                if let Some(path) = current_log_file() {
                    eprintln!("See log: {}", path.display());
                }
                process::exit(code);
            }
            tracing::error!(error = %error, "envlock invocation failed");
            if let Some(path) = current_log_file() {
                eprintln!("See log: {}", path.display());
            }
            Err(error)
        }
    }
}

fn run_external_command(tokens: &[String], run_args: &RunArgs) -> Result<()> {
    let alias_name = parse_shortcut_alias_name(tokens)
        .context("unknown command. alias shortcut must use envlock :<name>")?;
    let command = if tokens.len() > 1 {
        Some(tokens[1..].to_vec())
    } else {
        Some(Vec::new())
    };
    run_alias_named(alias_name, run_args, command)
}

fn run_alias_named(
    alias_name: &str,
    run_args: &RunArgs,
    command_override: Option<Vec<String>>,
) -> Result<()> {
    let profile = resolve_profile_for_alias(alias_name)?;
    let Some(profile) = profile else {
        anyhow::bail!("unknown alias: {}", alias_name);
    };

    let command_override = command_override.map(|mut command| {
        if command.first().map(String::as_str) == Some("--") {
            command.remove(0);
        }
        command
    });
    let config = build_runtime_config(run_args, Some(PathBuf::from(profile)), command_override)?;
    let app = App::new(config);
    let result = run(&app)?;
    if let Some(code) = result.exit_code {
        process::exit(code);
    }
    Ok(())
}

fn parse_shortcut_alias_name(tokens: &[String]) -> Option<&str> {
    let first = tokens.first()?;
    let name = first.strip_prefix(':')?;
    if name.is_empty() {
        return None;
    }
    Some(name)
}

fn build_runtime_config(
    run_args: &RunArgs,
    profile_override: Option<PathBuf>,
    command_override: Option<Vec<String>>,
) -> Result<RuntimeConfig> {
    RuntimeConfig::from_cli_and_env(
        CliInput {
            profile: profile_override.or_else(|| run_args.profile.clone()),
            output_mode: match run_args.output {
                OutputFormat::Shell => OutputMode::Shell,
                OutputFormat::Json => OutputMode::Json,
            },
            strict: run_args.strict,
            log_level: run_args.log_level.into(),
            log_format: match run_args.log_format {
                LogFormat::Text => RuntimeLogFormat::Text,
                LogFormat::Json => RuntimeLogFormat::Json,
            },
            command: command_override.unwrap_or_else(|| run_args.command.clone()),
        },
        RawEnv::from_process(),
    )
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Shell,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing_subscriber::filter::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn => Self::WARN,
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PreviewOutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProfileTemplateType {
    Minimal,
    Sample,
}

fn init_logging(
    level: tracing_subscriber::filter::LevelFilter,
    format: RuntimeLogFormat,
    session_log: Option<&SessionLog>,
) -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    let stderr_layer = match format {
        RuntimeLogFormat::Text => tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .boxed(),
        RuntimeLogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_writer(std::io::stderr)
            .boxed(),
    };

    let registry = tracing_subscriber::registry().with(stderr_layer.with_filter(env_filter));

    if let Some(session_log) = session_log {
        let writer = make_file_writer(session_log)?;
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
        registry
            .with(file_layer)
            .try_init()
            .context("failed to initialize logger")?;
    } else {
        registry.try_init().context("failed to initialize logger")?;
    }

    if let Some(path) = current_log_file() {
        tracing::info!(log_file = %path.display(), "session log initialized");
    }

    Ok(())
}

fn command_slug(cli: &Cli) -> String {
    match &cli.subcommand {
        Some(Commands::Plugin(args)) => format!("plugin-{}-{}", args.plugin, args.method),
        Some(Commands::SelfUpdate(_)) => "self-update".to_owned(),
        Some(Commands::Preview(_)) => "preview".to_owned(),
        Some(Commands::Profiles(_)) => "profiles".to_owned(),
        Some(Commands::Alias(_)) => "alias".to_owned(),
        Some(Commands::Skill(_)) => "skill".to_owned(),
        Some(Commands::External(tokens)) => tokens
            .first()
            .map(|token| format!("external-{token}"))
            .unwrap_or_else(|| "external".to_owned()),
        None => {
            if let Some(alias) = parse_shortcut_alias_name(&cli.run_args.command) {
                format!("alias-{alias}")
            } else {
                "run".to_owned()
            }
        }
    }
}

impl From<LogFormat> for RuntimeLogFormat {
    fn from(value: LogFormat) -> Self {
        match value {
            LogFormat::Text => RuntimeLogFormat::Text,
            LogFormat::Json => RuntimeLogFormat::Json,
        }
    }
}
