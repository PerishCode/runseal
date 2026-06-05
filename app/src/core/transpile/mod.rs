use std::fs;

use anyhow::{Result, bail};

mod ast;
mod emit;
mod lower;
mod parse;
mod powershell;

use emit::{emit_bash, emit_powershell, emit_seal};
use parse::parse_seal;
use powershell::parse_powershell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Seal,
    Bash,
    PowerShell,
    SealIr,
}

impl Lang {
    fn parse(value: &str, flag: &str) -> Result<Self> {
        match value {
            "seal" => Ok(Self::Seal),
            "bash" => Ok(Self::Bash),
            "powershell" => Ok(Self::PowerShell),
            "sealir" => Ok(Self::SealIr),
            _ => bail!("invalid {flag}: {value}; expected seal, bash, powershell, or sealir"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub input_lang: Lang,
    pub output_lang: Lang,
    pub source: String,
}

pub fn parse_args(args: &[String]) -> Result<Options> {
    let mut input_lang = None;
    let mut output_lang = None;
    let mut source = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--input-lang=") {
            input_lang = Some(Lang::parse(value, "--input-lang")?);
            index += 1;
            continue;
        }
        if arg == "--input-lang" {
            index += 1;
            let Some(value) = args.get(index) else {
                bail!("--input-lang requires a value");
            };
            input_lang = Some(Lang::parse(value, "--input-lang")?);
            index += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--output-lang=") {
            output_lang = Some(Lang::parse(value, "--output-lang")?);
            index += 1;
            continue;
        }
        if arg == "--output-lang" {
            index += 1;
            let Some(value) = args.get(index) else {
                bail!("--output-lang requires a value");
            };
            output_lang = Some(Lang::parse(value, "--output-lang")?);
            index += 1;
            continue;
        }
        if arg.starts_with('-') {
            bail!("unknown @transpile option: {arg}");
        }
        if source.replace(arg.clone()).is_some() {
            bail!("@transpile requires exactly one source file");
        }
        index += 1;
    }
    Ok(Options {
        input_lang: input_lang.ok_or_else(|| anyhow::anyhow!("--input-lang is required"))?,
        output_lang: output_lang.ok_or_else(|| anyhow::anyhow!("--output-lang is required"))?,
        source: source.ok_or_else(|| anyhow::anyhow!("@transpile requires one source file"))?,
    })
}

pub fn transpile_file(options: &Options) -> Result<String> {
    let source = fs::read_to_string(&options.source)
        .map_err(|err| anyhow::anyhow!("failed to read {}: {err}", options.source))?;
    transpile_source(
        options.input_lang,
        options.output_lang,
        &source,
        Some(&options.source),
    )
}

pub fn transpile_source(
    input_lang: Lang,
    output_lang: Lang,
    source: &str,
    source_name: Option<&str>,
) -> Result<String> {
    let program = match input_lang {
        Lang::Seal | Lang::Bash => parse_seal(source)?,
        Lang::PowerShell => parse_powershell(source)?,
        Lang::SealIr => serde_json::from_str(source)?,
    };
    match output_lang {
        Lang::SealIr => Ok(serde_json::to_string_pretty(&program)? + "\n"),
        Lang::Seal => Ok(emit_seal(&program)),
        Lang::Bash => Ok(emit_bash(&program, source_name)),
        Lang::PowerShell => Ok(emit_powershell(&program, source_name)),
    }
}
