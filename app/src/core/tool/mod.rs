use anyhow::{Result, bail};

mod archive;
mod cloudflare;
mod fs;
mod gitee;
mod github;
mod help;
mod int;
mod json;
mod process;
mod regex;
mod ssh;
mod string;

pub fn help() -> &'static str {
    help::top()
}

pub fn run(args: &[String]) -> Result<()> {
    if matches!(args, [arg] if matches!(arg.as_str(), "-h" | "--help" | "help")) {
        print!("{}", help());
        return Ok(());
    }
    if let Some(output) = eval(args)? {
        println!("{output}");
    }
    Ok(())
}

pub fn eval(args: &[String]) -> Result<Option<String>> {
    if let Some(help) = help::progressive(args) {
        return Ok(Some(help));
    }
    match args {
        [namespace, command, rest @ ..] if namespace == "json" => json::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "string" => string::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "regex" => regex::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "int" => int::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "process" => process::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "archive" => archive::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "fs" => fs::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "gitee" => gitee::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "ssh" => ssh::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "github" => github::eval(command, rest),
        [namespace, command, rest @ ..] if namespace == "cloudflare" => {
            cloudflare::eval(command, rest)
        }
        [] => bail!("@tool requires a tool path"),
        [namespace, command, ..] => bail!("unknown tool command: {namespace} {command}"),
        [namespace] => bail!("tool namespace requires a command: {namespace}"),
    }
}
