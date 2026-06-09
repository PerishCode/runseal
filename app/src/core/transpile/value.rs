use anyhow::{Result, bail};

use super::ast::Value;

pub(crate) fn parse_value_text(text: &str, line: usize) -> Result<Value> {
    if text == "$@" || text == "\"$@\"" {
        return Ok(Value::Args);
    }
    if text == "$#" || text == "\"$#\"" {
        return Ok(Value::Argc);
    }
    if let Some(value) = text
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return Ok(Value::Literal {
            text: value.to_string(),
        });
    }
    if let Some(value) = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return parse_template(value, line);
    }
    if let Some(name) = text.strip_prefix('$') {
        if is_positional_name(name) {
            return Ok(Value::Var {
                name: name.to_string(),
            });
        }
        if let Some(name) = name
            .strip_prefix('{')
            .and_then(|name| name.strip_suffix('}'))
        {
            if let Some((name, default)) = name.split_once(":-") {
                validate_name(name, line)?;
                return Ok(Value::EnvDefault {
                    name: name.to_string(),
                    default: default.to_string(),
                });
            }
            validate_name(name, line)?;
            return Ok(Value::Env {
                name: name.to_string(),
            });
        }
        validate_name(name, line)?;
        return Ok(Value::Var {
            name: name.to_string(),
        });
    }
    if text.contains('$') {
        return parse_template(text, line);
    }
    Ok(Value::Literal {
        text: text.to_string(),
    })
}

fn parse_template(text: &str, line: usize) -> Result<Value> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '$' {
            literal.push(ch);
            continue;
        }
        if !literal.is_empty() {
            parts.push(Value::Literal {
                text: std::mem::take(&mut literal),
            });
        }
        if chars.peek() == Some(&'{') {
            chars.next();
            let mut inner = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                inner.push(next);
            }
            if let Some((name, default)) = inner.split_once(":-") {
                validate_name(name, line)?;
                parts.push(Value::EnvDefault {
                    name: name.to_string(),
                    default: default.to_string(),
                });
            } else {
                validate_name(&inner, line)?;
                parts.push(Value::Env { name: inner });
            }
            continue;
        }
        let mut name = String::new();
        if let Some(next) = chars.peek().copied()
            && next == '@'
        {
            bail!("{line}: $@ is only supported as a standalone argument");
        }
        if let Some(next) = chars.peek().copied()
            && next.is_ascii_digit()
        {
            name.push(next);
            chars.next();
            parts.push(Value::Var { name });
            continue;
        }
        while let Some(next) = chars.peek().copied() {
            if next.is_ascii_alphanumeric() || next == '_' {
                name.push(next);
                chars.next();
            } else {
                break;
            }
        }
        validate_name(&name, line)?;
        parts.push(Value::Var { name });
    }
    if !literal.is_empty() {
        parts.push(Value::Literal { text: literal });
    }
    match parts.as_slice() {
        [single] => Ok(single.clone()),
        _ => Ok(Value::Concat { parts }),
    }
}

fn is_positional_name(name: &str) -> bool {
    name.len() == 1 && name.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_name(name: &str, line: usize) -> Result<()> {
    let mut bytes = name.bytes();
    let valid = matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if !valid {
        bail!("{line}: invalid variable name: {name}");
    }
    Ok(())
}
