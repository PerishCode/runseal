use anyhow::{Result, bail};

use super::ast::{ArgvKind, ArgvSpec, Statement, Value};
use super::json_path::parse_json_path;

pub(crate) fn parse_statement_helper(args: &[String], line: usize) -> Result<Statement> {
    match args {
        [argv, parse, rest @ ..] if argv == "argv" && parse == "parse" => {
            Ok(Statement::ArgvParse {
                specs: parse_argv_specs(rest, line)?,
            })
        }
        _ => bail!("{line}: unsupported seal helper statement"),
    }
}

pub(crate) fn parse_capture_helper(
    name: &str,
    argv: &[Value],
    line: usize,
) -> Result<Option<Statement>> {
    let statement = match argv {
        [
            Value::Literal { text: seal },
            Value::Literal { text: string },
            Value::Literal { text: trim },
            value,
        ] if seal == "seal" && string == "string" && trim == "trim" => {
            Some(Statement::StringTrim {
                name: name.to_string(),
                value: value.clone(),
            })
        }
        [
            Value::Literal { text: seal },
            Value::Literal { text: json },
            Value::Literal { text: get },
            value,
            Value::Literal { text: path },
        ] if seal == "seal" && json == "json" && get == "get" => Some(Statement::JsonGet {
            name: name.to_string(),
            json: value.clone(),
            path: parse_json_path(path, line)?,
        }),
        [
            Value::Literal { text: seal },
            Value::Literal { text: int },
            Value::Literal { text: add },
            left,
            right,
        ] if seal == "seal" && int == "int" && add == "add" => Some(Statement::IntAdd {
            name: name.to_string(),
            left: left.clone(),
            right: right.clone(),
        }),
        _ => None,
    };
    Ok(statement)
}

fn parse_argv_specs(args: &[String], line: usize) -> Result<Vec<ArgvSpec>> {
    let mut specs = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let kind = match args[index].as_str() {
            "--string" => ArgvKind::String,
            "--flag" => ArgvKind::Flag,
            other => bail!("{line}: unsupported argv parse spec: {other}"),
        };
        index += 1;
        let Some(raw) = args.get(index) else {
            bail!("{line}: argv parse spec requires a name");
        };
        let (name, default) = match raw.split_once('=') {
            Some((name, default)) => (name, Some(default.to_string())),
            None => (raw.as_str(), None),
        };
        validate_name(name, line)?;
        if matches!(kind, ArgvKind::Flag) && default.is_some() {
            bail!("{line}: argv flag cannot have a default: {name}");
        }
        specs.push(ArgvSpec {
            name: name.to_string(),
            kind,
            default,
        });
        index += 1;
    }
    Ok(specs)
}

fn validate_name(name: &str, line: usize) -> Result<()> {
    let mut bytes = name.bytes();
    let valid = matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if !valid {
        bail!("{line}: invalid argv parse name: {name}");
    }
    Ok(())
}
