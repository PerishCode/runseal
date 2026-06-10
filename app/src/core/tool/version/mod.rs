use anyhow::{Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "part" => part(args),
        "compare" => compare(args),
        _ => bail!("unknown tool command: version {command}"),
    }
}

fn part(args: &[String]) -> Result<Option<String>> {
    let [version, name] = args else {
        bail!("usage: runseal @tool version part <version> <major|minor|patch>");
    };
    let parsed = parse(version)?;
    let output = match name.as_str() {
        "major" => parsed.0.to_string(),
        "minor" => parsed.1.to_string(),
        "patch" => parsed.2.to_string(),
        _ => bail!("usage: runseal @tool version part <version> <major|minor|patch>"),
    };
    Ok(Some(output))
}

fn compare(args: &[String]) -> Result<Option<String>> {
    let [left, right] = args else {
        bail!("usage: runseal @tool version compare <left> <right>");
    };
    let left = parse(left)?;
    let right = parse(right)?;
    let output = if left < right {
        "lt"
    } else if left > right {
        "gt"
    } else {
        "eq"
    };
    Ok(Some(output.to_string()))
}

fn parse(version: &str) -> Result<(u64, u64, u64)> {
    let value = version.strip_prefix('v').unwrap_or(version);
    let mut parts = value.split('.');
    let major = parse_part(parts.next(), version, "major")?;
    let minor = parse_part(parts.next(), version, "minor")?;
    let patch = parse_part(parts.next(), version, "patch")?;
    if parts.next().is_some() {
        bail!("expected stable semantic version, got {version}");
    }
    Ok((major, minor, patch))
}

fn parse_part(value: Option<&str>, version: &str, name: &str) -> Result<u64> {
    let Some(value) = value else {
        bail!("expected stable semantic version, got {version}");
    };
    value
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("invalid {name} version part in {version}"))
}
