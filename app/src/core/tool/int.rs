use anyhow::{Context, Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "add" => add(args),
        _ => bail!("unknown tool command: int {command}"),
    }
}

fn add(args: &[String]) -> Result<Option<String>> {
    let [left, right] = args else {
        bail!("usage: runseal @tool int add <left> <right>");
    };
    let left = left
        .parse::<i64>()
        .with_context(|| format!("invalid integer: {left}"))?;
    let right = right
        .parse::<i64>()
        .with_context(|| format!("invalid integer: {right}"))?;
    Ok(Some((left + right).to_string()))
}
