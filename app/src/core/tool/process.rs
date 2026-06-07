use anyhow::{Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "exists" => exists(args),
        _ => bail!("unknown tool command: process {command}"),
    }
}

fn exists(args: &[String]) -> Result<Option<String>> {
    let [name] = args else {
        bail!("usage: runseal @tool process exists <name>");
    };
    Ok(Some(command_exists(name).to_string()))
}

fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}
