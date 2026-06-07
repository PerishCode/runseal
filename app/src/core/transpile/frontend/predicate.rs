use anyhow::{Result, bail};

use super::powershell::parse_value;
use crate::core::transpile::ast::Predicate;

pub(crate) fn parse_powershell_predicate(text: &str, line: usize) -> Result<Predicate> {
    if let Some(value) = text
        .strip_prefix("[string]::IsNullOrEmpty(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return Ok(Predicate::Empty {
            value: parse_value(value, line)?,
        });
    }
    if let Some(value) = text
        .strip_prefix("![string]::IsNullOrEmpty(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return Ok(Predicate::NotEmpty {
            value: parse_value(value, line)?,
        });
    }
    if let Some(value) = json_count_value(text, "-eq 0") {
        return Ok(Predicate::JsonEmpty {
            value: parse_value(value, line)?,
        });
    }
    if let Some(value) = json_count_value(text, "-gt 0") {
        return Ok(Predicate::JsonNotEmpty {
            value: parse_value(value, line)?,
        });
    }
    if let Some((operator, left, right)) = powershell_compare(text) {
        return int_predicate(operator, left, right, line);
    }
    bail!("{line}: unsupported PowerShell predicate: {text}")
}

fn int_predicate(operator: &str, left: &str, right: &str, line: usize) -> Result<Predicate> {
    let left = left.strip_prefix("[int]").unwrap_or(left);
    let left = parse_value(left, line)?;
    let right = parse_value(right, line)?;
    match operator {
        "-lt" => Ok(Predicate::IntLt { left, right }),
        "-le" => Ok(Predicate::IntLte { left, right }),
        "-gt" => Ok(Predicate::IntGt { left, right }),
        "-ge" => Ok(Predicate::IntGte { left, right }),
        _ => bail!("{line}: unsupported PowerShell comparison operator: {operator}"),
    }
}

fn powershell_compare(text: &str) -> Option<(&str, &str, &str)> {
    for operator in [" -lt ", " -le ", " -gt ", " -ge "] {
        if let Some((left, right)) = text.split_once(operator) {
            return Some((operator.trim(), left, right));
        }
    }
    None
}

fn json_count_value<'a>(text: &'a str, comparison: &str) -> Option<&'a str> {
    let inner = text.strip_prefix("((")?;
    let suffix = format!(").Count {comparison})");
    let inner = inner.strip_suffix(&suffix)?;
    inner.strip_suffix(" | ConvertFrom-Json")
}
