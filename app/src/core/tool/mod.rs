use anyhow::{Result, bail};

mod archive;
mod cloudflare;
mod fs;
mod gitee;
mod github;
mod int;
mod json;
mod process;
mod regex;
mod ssh;
mod string;

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

pub fn help() -> &'static str {
    "\
Usage: runseal @tool <namespace> <command> [args]

Run an atomic runseal tool command.

Tools:
  json get <json> <path>                 print a JSON value
  json empty <json>                      print true when JSON length is zero
  json len <json>                        print JSON array/object/string length
  json pretty <json>                     print formatted JSON
  json find <array> <field> <value>      print first object with field=value
  json filter <array> <field> <value>... print objects with field matching values
  string trim <value>                    trim leading and trailing whitespace
  string join <array> --separator <sep>   join a JSON string array; sep=path uses platform path-list separator
  string slug <value>                    normalize text for branch/file slugs
  regex capture <value> <pattern> <n>    print regex capture group n, or empty
  int add <left> <right>                 print integer sum
  process exists <name>                  print true when command exists on PATH
  archive local export                   encrypt a .local-style directory archive
  archive local import                   decrypt and restore a .local-style archive
  fs mkdir <path> [mode]                 create a directory and parents
  fs write <path> <text> [mode]          write text to a file
  fs write-base64 <path> <base64>        write decoded bytes to a file
  fs chmod <path> <mode>                 set a file mode on Unix
  fs mode <path>                         print file mode on Unix
  fs touch <path> [mode]                 create a file without truncating it
  fs list <path> [--glob <pattern>]      list matching direct children as JSON
  ssh config host <host> --config <path> print true when Host patterns match
  ssh config identities --config <path>  print IdentityFile paths as JSON
  ssh script run --config <path>         run a local script on an SSH host
  ssh script capture --config <path>     run a local script and print stdout
  fs contains-any <path> <text>...       print true when file contains any text
  fs backup-numbered <path>              move path to .bak or .bak.N and print it
  gitee repo parse-origin <url>          parse Gitee owner/repo from origin URL
  gitee pr create                        create a Gitee pull request
  gitee pr pass-gates                    best-effort pass Gitee PR gates
  gitee pr merge                         merge a Gitee pull request
  github pr checks probe <number>        print true when PR checks are reported
  cloudflare zone dns-record list        list DNS records in a zone
  cloudflare zone dns-record create      create a DNS record from JSON
  cloudflare zone dns-record update      update a DNS record from JSON
  cloudflare ...                         run an atomic Cloudflare resource op

@tool is the runseal atomic tool runtime. Tool inputs use argv/env, output is
stdout, diagnostics are stderr, and failure is a non-zero exit code.
"
}
