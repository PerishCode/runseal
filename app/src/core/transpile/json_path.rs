use anyhow::{Result, bail};

use super::ast::{JsonPath, JsonPathSegment};

pub(crate) fn parse_json_path(text: &str, line: usize) -> Result<JsonPath> {
    let mut input = text.strip_prefix('.').unwrap_or(text);
    let mut segments = Vec::new();
    while !input.is_empty() {
        if let Some(rest) = input.strip_prefix('[') {
            let Some((index, rest)) = rest.split_once(']') else {
                bail!("{line}: unsupported json path: {text}");
            };
            segments.push(JsonPathSegment::Index {
                index: index
                    .parse()
                    .map_err(|_| anyhow::anyhow!("{line}: invalid json path index: {index}"))?,
            });
            input = rest.strip_prefix('.').unwrap_or(rest);
            continue;
        }
        let end = input.find(['.', '[']).unwrap_or(input.len());
        let field = &input[..end];
        if !is_valid_name(field) {
            bail!("{line}: invalid json path field: {field}");
        }
        segments.push(JsonPathSegment::Field {
            name: field.to_string(),
        });
        input = input[end..].strip_prefix('.').unwrap_or(&input[end..]);
    }
    if segments.is_empty() {
        bail!("{line}: json path cannot be empty");
    }
    Ok(JsonPath { segments })
}

pub(crate) fn json_path(path: &JsonPath) -> String {
    let mut output = String::from(".");
    for segment in &path.segments {
        match segment {
            JsonPathSegment::Field { name } => {
                if output != "." {
                    output.push('.');
                }
                output.push_str(name);
            }
            JsonPathSegment::Index { index } => output.push_str(&format!("[{index}]")),
        }
    }
    output
}

pub(crate) fn powershell_json_get(json: &str, path: &JsonPath) -> String {
    let mut output = format!("({json} | ConvertFrom-Json)");
    for segment in &path.segments {
        match segment {
            JsonPathSegment::Field { name } => {
                output.push('.');
                output.push_str(name);
            }
            JsonPathSegment::Index { index } => output.push_str(&format!("[{index}]")),
        }
    }
    output
}

fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
