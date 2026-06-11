use anyhow::{Result, bail};

use super::ast::{OutputStream, Statement};
use super::parse::parse_values;
use super::parse_lex::is_safe_command_name;
use super::value::parse_value_text;

pub(super) fn parse_exec_write(tokens: &[String], line: usize) -> Result<Option<Statement>> {
    let redirects = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| matches!(token.as_str(), ">" | ">>" | "2>" | "2>>" | "|"))
        .collect::<Vec<_>>();
    if redirects.is_empty() {
        return Ok(None);
    }
    if redirects.iter().any(|(_, token)| token.as_str() == "|") {
        bail!("{line}: unsupported shell metacharacter: |");
    }
    if redirects.len() != 1 {
        bail!("{line}: unsupported redirect combination");
    }
    let (index, token) = redirects[0];
    if index == 0 || index + 2 != tokens.len() {
        bail!("{line}: redirect requires one command and one file target");
    }
    let argv_tokens = &tokens[..index];
    validate_external_tokens(argv_tokens, line)?;
    let Some((command, args)) = argv_tokens.split_first() else {
        bail!("{line}: redirect requires one command");
    };
    if command == "printf" && token.as_str() == ">" && tokens[index + 1] == "&2" {
        return Ok(None);
    }
    validate_shell_command(command, args, line)?;
    if !is_safe_command_name(command) {
        bail!("{line}: unsupported statement: {}", tokens.join(" "));
    }
    if matches!(tokens[index + 1].as_str(), "&1" | "&2") {
        bail!("{line}: unsupported redirect combination");
    }
    let path = parse_value_text(&tokens[index + 1], line)?;
    let (stream, append) = match token.as_str() {
        ">" => (OutputStream::Stdout, false),
        ">>" => (OutputStream::Stdout, true),
        "2>" => (OutputStream::Stderr, false),
        "2>>" => (OutputStream::Stderr, true),
        _ => unreachable!(),
    };
    Ok(Some(Statement::ExecWrite {
        stream,
        path,
        append,
        argv: parse_values(argv_tokens, line)?,
    }))
}

pub(super) fn validate_external_tokens(tokens: &[String], line: usize) -> Result<()> {
    for token in tokens {
        if matches!(token.as_str(), ">" | ">>" | "2>" | "2>>" | "|") {
            bail!("{line}: unsupported shell metacharacter: {token}");
        }
        if token.starts_with('"') || token.starts_with('\'') {
            continue;
        }
        if token
            .chars()
            .any(|ch| matches!(ch, '|' | '>' | '<' | '&' | ';' | '`'))
        {
            bail!("{line}: unsupported shell metacharacter in token: {token}");
        }
    }
    Ok(())
}

pub(super) fn validate_shell_command(command: &str, args: &[String], line: usize) -> Result<()> {
    if SHELL_ONLY_COMMANDS.contains(&command) {
        bail!(
            "{line}: shell-specific construct is not supported in .seal: {command}; use .sh/.ps1 or file an issue for first-class support"
        );
    }
    let shell_launch = matches!(
        (command, args.first().map(String::as_str)),
        ("sh", Some("-c"))
            | ("bash", Some("-c"))
            | ("pwsh", Some("-Command" | "-command"))
            | ("powershell", Some("-Command" | "-command"))
    );
    if shell_launch {
        bail!(
            "{line}: shell-specific construct is not supported in .seal: {command} {}; use .sh/.ps1 or file an issue for first-class support",
            args.first().expect("checked first arg exists")
        );
    }
    Ok(())
}

const SHELL_ONLY_COMMANDS: &[&str] = &[
    ".", "alias", "exec", "export", "local", "readonly", "source", "trap", "unalias", "unset",
];
