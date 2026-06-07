use anyhow::{Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "trim" => trim(args),
        _ => bail!("unknown tool command: string {command}"),
    }
}

fn trim(args: &[String]) -> Result<Option<String>> {
    let [value] = args else {
        bail!("usage: runseal @tool string trim <value>");
    };
    Ok(Some(value.trim().to_string()))
}
