use anyhow::{Result, bail};

use crate::core::transpile::ast::{ExpansionOp, Value, ValueSource};

pub(super) fn parse_argv(text: &str, line: usize) -> Result<Vec<Value>> {
    split_exprs(text, line)?
        .iter()
        .map(|arg| parse_argv_value(arg, line))
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn parse_value(text: &str, line: usize) -> Result<Value> {
    let text = text.trim();
    if let Some(value) = text
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return Ok(Value::Literal {
            text: value.replace("''", "'"),
        });
    }
    if let Some((source, op)) = guarded_expand(text, line)? {
        return Ok(Value::Expand { source, op });
    }
    if let Some(name) = text.strip_prefix("$env:") {
        validate_name(name, line)?;
        return Ok(Value::Expand {
            source: ValueSource::Env {
                name: name.to_string(),
            },
            op: ExpansionOp::Plain,
        });
    }
    if let Some(name) = text.strip_prefix('$') {
        if is_valid_positional(name) {
            return Ok(Value::Expand {
                source: ValueSource::Var {
                    name: name.to_string(),
                },
                op: ExpansionOp::Plain,
            });
        }
        validate_name(name, line)?;
        return Ok(Value::Expand {
            source: ValueSource::Var {
                name: name.to_string(),
            },
            op: ExpansionOp::Plain,
        });
    }
    if text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(Value::Literal {
            text: text.to_string(),
        });
    }
    if let Some(inner) = text
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        && inner.contains(" + ")
    {
        return Ok(Value::Concat {
            parts: split_concat(inner, line)?
                .iter()
                .map(|part| parse_value(part, line))
                .collect::<Result<Vec<_>>>()?,
        });
    }
    bail!("{line}: unsupported PowerShell value: {text}")
}

pub(super) fn assignment(text: &str) -> Option<(String, &str)> {
    let (name, value) = text.split_once(" = ")?;
    let name = name.strip_prefix('$')?;
    is_valid_var_name(name).then_some((name.to_string(), value))
}

pub(super) fn parse_pattern(pattern: &str) -> String {
    if pattern == "Default" {
        return "*".to_string();
    }
    pattern
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(pattern)
        .to_string()
}

pub(super) fn strip_comment(line: &str) -> String {
    let mut output = String::new();
    let mut quote = false;
    for ch in line.chars() {
        match ch {
            '\'' => {
                quote = !quote;
                output.push(ch);
            }
            '#' if !quote => break,
            _ => output.push(ch),
        }
    }
    output
}

pub(super) fn is_generated_positional_binding(text: &str) -> bool {
    if text == "$0 = $args.Count" {
        return true;
    }
    let Some(rest) = text.strip_prefix('$') else {
        return false;
    };
    let Some((index, rest)) = rest.split_once(" = if ($args.Count -ge ") else {
        return false;
    };
    let Some((index2, rest)) = rest.split_once(") { $args[") else {
        return false;
    };
    let Some((offset, _rest)) = rest.split_once("] } else { '' }") else {
        return false;
    };
    if index != index2 {
        return false;
    }
    let Ok(index) = index.parse::<usize>() else {
        return false;
    };
    let Ok(offset) = offset.parse::<usize>() else {
        return false;
    };
    index.checked_sub(1) == Some(offset)
}

pub(super) fn validate_name(name: &str, line: usize) -> Result<()> {
    if !is_valid_name(name) {
        bail!("{line}: invalid PowerShell name: {name}");
    }
    Ok(())
}

fn parse_argv_value(text: &str, line: usize) -> Result<Value> {
    parse_value(text, line).or_else(|_| {
        if is_valid_name(text) {
            Ok(Value::Literal {
                text: text.to_string(),
            })
        } else {
            bail!("{line}: unsupported PowerShell argv value: {text}")
        }
    })
}

fn guarded_expand(text: &str, line: usize) -> Result<Option<(ValueSource, ExpansionOp)>> {
    let Some(inner) = text
        .strip_prefix("$(if (")
        .and_then(|value| value.strip_suffix(" })"))
    else {
        return Ok(None);
    };
    if let Some(parsed) = env_guarded_expand(inner, line)? {
        return Ok(Some(parsed));
    }
    if let Some(parsed) = positional_guarded_expand(inner, line)? {
        return Ok(Some(parsed));
    }
    if let Some(parsed) = var_guarded_expand(inner, line)? {
        return Ok(Some(parsed));
    }
    Ok(None)
}

fn env_guarded_expand(text: &str, line: usize) -> Result<Option<(ValueSource, ExpansionOp)>> {
    let Some(rest) = text.strip_prefix("[string]::IsNullOrEmpty($env:") else {
        return Ok(None);
    };
    let (name, rest) = rest
        .split_once(")) { ")
        .ok_or_else(|| anyhow::anyhow!("{line}: unsupported PowerShell env expansion"))?;
    validate_name(name, line)?;
    let source = ValueSource::Env {
        name: name.to_string(),
    };
    let plain = format!("$env:{name}");
    parse_guarded_action(source, &plain, rest, line).map(Some)
}

fn positional_guarded_expand(
    text: &str,
    line: usize,
) -> Result<Option<(ValueSource, ExpansionOp)>> {
    let Some(rest) = text.strip_prefix("($args.Count -lt ") else {
        return Ok(None);
    };
    let (index, rest) = rest
        .split_once(") -or [string]::IsNullOrEmpty($")
        .ok_or_else(|| anyhow::anyhow!("{line}: unsupported PowerShell positional expansion"))?;
    let (name, rest) = rest
        .split_once(")) { ")
        .ok_or_else(|| anyhow::anyhow!("{line}: unsupported PowerShell positional expansion"))?;
    if index != name {
        bail!("{line}: unsupported PowerShell positional expansion");
    }
    let source = ValueSource::Var {
        name: name.to_string(),
    };
    let plain = format!("${name}");
    parse_guarded_action(source, &plain, rest, line).map(Some)
}

fn var_guarded_expand(text: &str, line: usize) -> Result<Option<(ValueSource, ExpansionOp)>> {
    let Some(rest) = text.strip_prefix("[string]::IsNullOrEmpty($") else {
        return Ok(None);
    };
    let (name, rest) = rest
        .split_once(")) { ")
        .ok_or_else(|| anyhow::anyhow!("{line}: unsupported PowerShell variable expansion"))?;
    validate_name(name, line)?;
    let source = ValueSource::Var {
        name: name.to_string(),
    };
    let plain = format!("${name}");
    parse_guarded_action(source, &plain, rest, line).map(Some)
}

fn parse_guarded_action(
    source: ValueSource,
    plain: &str,
    rest: &str,
    line: usize,
) -> Result<(ValueSource, ExpansionOp)> {
    let (action, expected_plain) = rest
        .split_once(" } else { ")
        .ok_or_else(|| anyhow::anyhow!("{line}: unsupported PowerShell guarded expansion"))?;
    if expected_plain != plain {
        bail!("{line}: unsupported PowerShell guarded expansion");
    }
    if let Some(message) = action.strip_prefix("throw ") {
        let message = parse_single_quoted(message, line)?;
        return Ok((source, ExpansionOp::RequireNonEmpty { message }));
    }
    let fallback = parse_single_quoted(action, line)?;
    Ok((source, ExpansionOp::DefaultIfUnsetOrEmpty { fallback }))
}

fn parse_single_quoted(text: &str, line: usize) -> Result<String> {
    let Some(value) = text
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    else {
        bail!("{line}: unsupported PowerShell quoted literal: {text}");
    };
    Ok(value.replace("''", "'"))
}

fn split_exprs(text: &str, line: usize) -> Result<Vec<String>> {
    split_top_level(text, line, ' ')
}

fn split_concat(text: &str, line: usize) -> Result<Vec<String>> {
    split_top_level(text, line, '+').map(|items| {
        items
            .into_iter()
            .map(|item| item.trim().to_string())
            .collect()
    })
}

fn split_top_level(text: &str, line: usize, delimiter: char) -> Result<Vec<String>> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote = false;
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '\'' => {
                quote = !quote;
                current.push(ch);
            }
            '(' if !quote => {
                depth += 1;
                current.push(ch);
            }
            ')' if !quote => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ch if ch == delimiter && !quote && depth == 0 => {
                if !current.trim().is_empty() {
                    items.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if quote {
        bail!("{line}: unterminated PowerShell string");
    }
    if !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }
    Ok(items)
}

fn is_valid_var_name(name: &str) -> bool {
    is_valid_name(name) || is_valid_positional(name)
}

fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_valid_positional(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_digit())
}
