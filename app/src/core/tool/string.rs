use anyhow::{Context, Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "trim" => trim(args),
        "join" => join(args),
        "slug" => slug(args),
        _ => bail!("unknown tool command: string {command}"),
    }
}

fn trim(args: &[String]) -> Result<Option<String>> {
    let [value] = args else {
        bail!("usage: runseal @tool string trim <value>");
    };
    Ok(Some(value.trim().to_string()))
}

fn join(args: &[String]) -> Result<Option<String>> {
    let [json, separator_flag, separator] = args else {
        bail!("usage: runseal @tool string join <json-array> --separator <text|path>");
    };
    if separator_flag != "--separator" {
        bail!("usage: runseal @tool string join <json-array> --separator <text|path>");
    }
    let values: Vec<String> = serde_json::from_str(json).context("invalid string array JSON")?;
    if separator == "path" {
        let joined =
            std::env::join_paths(values.iter()).context("failed to join path-list values")?;
        return Ok(Some(joined.to_string_lossy().into_owned()));
    }
    Ok(Some(values.join(separator)))
}

fn slug(args: &[String]) -> Result<Option<String>> {
    let Some(value) = args.first() else {
        bail!("usage: runseal @tool string slug <value> [--max-len <n>] [--fallback <text>]");
    };

    let mut max_len: Option<usize> = None;
    let mut fallback = "value".to_string();
    let mut rest = &args[1..];
    while let Some((flag, tail)) = rest.split_first() {
        match flag.as_str() {
            "--max-len" => {
                let Some((raw, next)) = tail.split_first() else {
                    bail!("missing value for --max-len");
                };
                let parsed = raw.parse::<usize>().context("invalid --max-len value")?;
                if parsed == 0 {
                    bail!("--max-len must be greater than zero");
                }
                max_len = Some(parsed);
                rest = next;
            }
            "--fallback" => {
                let Some((raw, next)) = tail.split_first() else {
                    bail!("missing value for --fallback");
                };
                fallback = raw.clone();
                rest = next;
            }
            _ => bail!(
                "usage: runseal @tool string slug <value> [--max-len <n>] [--fallback <text>]"
            ),
        }
    }

    let mut output = slugify(value);
    if let Some(limit) = max_len {
        output.truncate(limit);
        output = trim_hyphens(&output);
    }
    if output.is_empty() {
        output = slugify(&fallback);
        if let Some(limit) = max_len {
            output.truncate(limit);
            output = trim_hyphens(&output);
        }
    }
    if output.is_empty() {
        output = "value".to_string();
    }
    Ok(Some(output))
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut pending_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !output.is_empty() {
                output.push('-');
            }
            output.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    output
}

fn trim_hyphens(value: &str) -> String {
    value.trim_matches('-').to_string()
}
