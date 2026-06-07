use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "get" => get(args),
        "empty" => empty(args),
        "len" => len(args),
        "pretty" => pretty(args),
        "find" => find(args),
        "filter" => filter(args),
        _ => bail!("unknown tool command: json {command}"),
    }
}

fn get(args: &[String]) -> Result<Option<String>> {
    let [json, path] = args else {
        bail!("usage: runseal @tool json get <json> <path>");
    };
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    let selected = select_path(&value, path)?;
    let output = match selected {
        JsonValue::Null => None,
        JsonValue::String(value) => Some(value.clone()),
        JsonValue::Bool(value) => Some(value.to_string()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::Array(_) | JsonValue::Object(_) => Some(serde_json::to_string(selected)?),
    };
    Ok(output)
}

fn empty(args: &[String]) -> Result<Option<String>> {
    let [json] = args else {
        bail!("usage: runseal @tool json empty <json>");
    };
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    Ok(Some(value_is_empty(&value).to_string()))
}

fn len(args: &[String]) -> Result<Option<String>> {
    let [json] = args else {
        bail!("usage: runseal @tool json len <json>");
    };
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    let len = match value {
        JsonValue::Null => 0,
        JsonValue::String(value) => value.len(),
        JsonValue::Array(value) => value.len(),
        JsonValue::Object(value) => value.len(),
        JsonValue::Bool(_) | JsonValue::Number(_) => 1,
    };
    Ok(Some(len.to_string()))
}

fn pretty(args: &[String]) -> Result<Option<String>> {
    let [json] = args else {
        bail!("usage: runseal @tool json pretty <json>");
    };
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    Ok(Some(serde_json::to_string_pretty(&value)?))
}

fn find(args: &[String]) -> Result<Option<String>> {
    let [json, field, expected] = args else {
        bail!("usage: runseal @tool json find <array> <field> <value>");
    };
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    let Some(found) = json_array(&value)?
        .iter()
        .find(|item| field_string(item, field).as_deref() == Some(expected.as_str()))
    else {
        return Ok(None);
    };
    Ok(Some(serde_json::to_string(found)?))
}

fn filter(args: &[String]) -> Result<Option<String>> {
    let [json, field, expected @ ..] = args else {
        bail!("usage: runseal @tool json filter <array> <field> <value>...");
    };
    if expected.is_empty() {
        bail!("json filter requires at least one expected value");
    }
    let value: JsonValue = serde_json::from_str(json).context("invalid JSON input")?;
    let filtered = json_array(&value)?
        .iter()
        .filter(|item| {
            field_string(item, field)
                .as_deref()
                .is_some_and(|actual| expected.iter().any(|value| value == actual))
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(Some(serde_json::to_string(&filtered)?))
}

fn json_array(value: &JsonValue) -> Result<&[JsonValue]> {
    let JsonValue::Array(values) = value else {
        bail!("expected JSON array");
    };
    Ok(values)
}

fn field_string(value: &JsonValue, field: &str) -> Option<String> {
    value.get(field).map(|value| match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null | JsonValue::Array(_) | JsonValue::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    })
}

fn value_is_empty(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => true,
        JsonValue::String(value) => value.is_empty(),
        JsonValue::Array(value) => value.is_empty(),
        JsonValue::Object(value) => value.is_empty(),
        JsonValue::Bool(_) | JsonValue::Number(_) => false,
    }
}

fn select_path<'a>(value: &'a JsonValue, path: &str) -> Result<&'a JsonValue> {
    let mut input = path.strip_prefix('.').unwrap_or(path);
    if input.is_empty() {
        bail!("json path cannot be empty");
    }
    let mut current = value;
    while !input.is_empty() {
        if let Some(rest) = input.strip_prefix('[') {
            let Some((index, rest)) = rest.split_once(']') else {
                bail!("unsupported json path: {path}");
            };
            let index = index
                .parse::<usize>()
                .with_context(|| format!("invalid json path index: {index}"))?;
            current = current
                .get(index)
                .with_context(|| format!("json path not found: {path}"))?;
            input = rest.strip_prefix('.').unwrap_or(rest);
            continue;
        }
        let end = input.find(['.', '[']).unwrap_or(input.len());
        let field = &input[..end];
        validate_field(field)?;
        current = current
            .get(field)
            .with_context(|| format!("json path not found: {path}"))?;
        input = input[end..].strip_prefix('.').unwrap_or(&input[end..]);
    }
    Ok(current)
}

fn validate_field(field: &str) -> Result<()> {
    let mut bytes = field.bytes();
    let valid = matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if !valid {
        bail!("invalid json path field: {field}");
    }
    Ok(())
}
