use anyhow::{Context, Result, bail};
use regex::Regex;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "capture" => capture(args),
        _ => bail!("unknown tool command: regex {command}"),
    }
}

fn capture(args: &[String]) -> Result<Option<String>> {
    let [value, pattern, group] = args else {
        bail!("usage: runseal @tool regex capture <value> <pattern> <group>");
    };
    let group = group
        .parse::<usize>()
        .with_context(|| format!("invalid regex capture group: {group}"))?;
    if !(1..=9).contains(&group) {
        bail!("regex capture group must be between 1 and 9");
    }
    let regex = Regex::new(pattern).context("invalid regex pattern")?;
    let captured = regex
        .captures(value)
        .and_then(|captures| captures.get(group))
        .map(|capture| capture.as_str())
        .unwrap_or("");
    Ok(Some(captured.to_string()))
}
