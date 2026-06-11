use anyhow::{Result, bail};

use super::ast::{ExpansionOp, Value, ValueSource};

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
            return Ok(Value::Expand {
                source: ValueSource::Var {
                    name: name.to_string(),
                },
                op: ExpansionOp::Plain,
            });
        }
        if let Some(name) = name
            .strip_prefix('{')
            .and_then(|name| name.strip_suffix('}'))
        {
            return parse_braced_expansion(name, line);
        }
        validate_name(name, line)?;
        return Ok(Value::Expand {
            source: ValueSource::Var {
                name: name.to_string(),
            },
            op: ExpansionOp::Plain,
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
            parts.push(parse_braced_expansion(&inner, line)?);
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
            parts.push(Value::Expand {
                source: ValueSource::Var { name },
                op: ExpansionOp::Plain,
            });
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
        parts.push(Value::Expand {
            source: ValueSource::Var { name },
            op: ExpansionOp::Plain,
        });
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
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_braced_expansion(text: &str, line: usize) -> Result<Value> {
    if let Some((name, message)) = text.split_once(":?") {
        let source = parse_braced_source(name, line)?;
        return Ok(Value::Expand {
            source,
            op: ExpansionOp::RequireNonEmpty {
                message: message.to_string(),
            },
        });
    }
    if let Some((name, fallback)) = text.split_once(":-") {
        let source = parse_braced_source(name, line)?;
        return Ok(Value::Expand {
            source,
            op: ExpansionOp::DefaultIfUnsetOrEmpty {
                fallback: fallback.to_string(),
            },
        });
    }
    let source = parse_braced_source(text, line)?;
    Ok(Value::Expand {
        source,
        op: ExpansionOp::Plain,
    })
}

fn parse_braced_source(name: &str, line: usize) -> Result<ValueSource> {
    if is_positional_name(name) {
        return Ok(ValueSource::Var {
            name: name.to_string(),
        });
    }
    validate_name(name, line)?;
    Ok(ValueSource::Env {
        name: name.to_string(),
    })
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
