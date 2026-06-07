use anyhow::{Result, bail};

use super::ast::{ArgvKind, ArgvSpec, Statement, ToolInvocation, Value};
use super::json_path::parse_json_path;

pub(crate) fn parse_statement_helper(args: &[String], line: usize) -> Result<Statement> {
    match args {
        [argv, parse, rest @ ..] if argv == "argv" && parse == "parse" => {
            Ok(Statement::ArgvParse {
                specs: parse_argv_specs(rest, line)?,
            })
        }
        [passthrough, start, namespace, command, rest @ ..] if passthrough == "passthrough" => {
            let start = start
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("{line}: invalid passthrough start: {start}"))?;
            Ok(Statement::ToolPassthrough {
                start,
                invocation: ToolInvocation {
                    path: vec![namespace.clone(), command.clone()],
                    argv: rest
                        .iter()
                        .map(|arg| super::value::parse_value_text(arg, line))
                        .collect::<Result<Vec<_>>>()?,
                },
            })
        }
        [namespace, command, rest @ ..] => Ok(Statement::ToolExec {
            invocation: ToolInvocation {
                path: vec![namespace.clone(), command.clone()],
                argv: rest
                    .iter()
                    .map(|arg| super::value::parse_value_text(arg, line))
                    .collect::<Result<Vec<_>>>()?,
            },
        }),
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
            Some(tool_capture(name, ["string", "trim"], vec![value.clone()]))
        }
        [
            Value::Literal { text: seal },
            Value::Literal { text: json },
            Value::Literal { text: get },
            value,
            Value::Literal { text: path },
        ] if seal == "seal" && json == "json" && get == "get" => {
            parse_json_path(path, line)?;
            Some(tool_capture(
                name,
                ["json", "get"],
                vec![value.clone(), Value::Literal { text: path.clone() }],
            ))
        }
        [
            Value::Literal { text: seal },
            Value::Literal { text: regex },
            Value::Literal { text: capture },
            value,
            Value::Literal { text: pattern },
            Value::Literal { text: group },
        ] if seal == "seal" && regex == "regex" && capture == "capture" => {
            parse_group(group, line)?;
            Some(tool_capture(
                name,
                ["regex", "capture"],
                vec![
                    value.clone(),
                    Value::Literal {
                        text: pattern.clone(),
                    },
                    Value::Literal {
                        text: group.clone(),
                    },
                ],
            ))
        }
        [
            Value::Literal { text: seal },
            Value::Literal { text: int },
            Value::Literal { text: add },
            left,
            right,
        ] if seal == "seal" && int == "int" && add == "add" => Some(tool_capture(
            name,
            ["int", "add"],
            vec![left.clone(), right.clone()],
        )),
        [
            Value::Literal { text: seal },
            Value::Literal { text: namespace },
            Value::Literal { text: command },
            rest @ ..,
        ] if seal == "seal" => Some(Statement::ToolCapture {
            name: name.to_string(),
            invocation: ToolInvocation {
                path: vec![namespace.clone(), command.clone()],
                argv: rest.to_vec(),
            },
        }),
        _ => None,
    };
    Ok(statement)
}

fn tool_capture<const N: usize>(name: &str, path: [&str; N], argv: Vec<Value>) -> Statement {
    Statement::ToolCapture {
        name: name.to_string(),
        invocation: ToolInvocation {
            path: path.into_iter().map(str::to_string).collect(),
            argv,
        },
    }
}

fn parse_group(group: &str, line: usize) -> Result<usize> {
    let group = group
        .parse()
        .map_err(|_| anyhow::anyhow!("{line}: invalid regex capture group: {group}"))?;
    if !(1..=9).contains(&group) {
        bail!("{line}: regex capture group must be between 1 and 9");
    }
    Ok(group)
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
