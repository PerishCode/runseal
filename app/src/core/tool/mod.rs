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
    if let Some(help) = progressive_help(args) {
        return Ok(Some(help.to_string()));
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

fn progressive_help(args: &[String]) -> Option<&'static str> {
    let last = args.last()?;
    if !matches!(last.as_str(), "-h" | "--help" | "help") {
        return None;
    }
    let path = &args[..args.len() - 1];
    match path {
        [] => Some(help()),
        [namespace] if namespace == "json" => Some(JSON_HELP),
        [namespace] if namespace == "string" => Some(STRING_HELP),
        [namespace] if namespace == "regex" => Some(REGEX_HELP),
        [namespace] if namespace == "int" => Some(INT_HELP),
        [namespace] if namespace == "process" => Some(PROCESS_HELP),
        [namespace] if namespace == "archive" => Some(ARCHIVE_HELP),
        [namespace] if namespace == "fs" => Some(FS_HELP),
        [namespace] if namespace == "gitee" => Some(GITEE_HELP),
        [namespace] if namespace == "ssh" => Some(SSH_HELP),
        [namespace] if namespace == "github" => Some(GITHUB_HELP),
        [namespace] if namespace == "cloudflare" => Some(CLOUDFLARE_HELP),
        [namespace, command] if namespace == "json" && command == "get" => Some(JSON_GET_HELP),
        [namespace, command] if namespace == "json" && command == "empty" => Some(JSON_EMPTY_HELP),
        [namespace, command] if namespace == "json" && command == "len" => Some(JSON_LEN_HELP),
        [namespace, command] if namespace == "json" && command == "pretty" => {
            Some(JSON_PRETTY_HELP)
        }
        [namespace, command] if namespace == "json" && command == "find" => Some(JSON_FIND_HELP),
        [namespace, command] if namespace == "json" && command == "filter" => {
            Some(JSON_FILTER_HELP)
        }
        [namespace, command] if namespace == "string" && command == "trim" => {
            Some(STRING_TRIM_HELP)
        }
        [namespace, command] if namespace == "string" && command == "join" => {
            Some(STRING_JOIN_HELP)
        }
        [namespace, command] if namespace == "string" && command == "slug" => {
            Some(STRING_SLUG_HELP)
        }
        [namespace, command] if namespace == "regex" && command == "capture" => {
            Some(REGEX_CAPTURE_HELP)
        }
        [namespace, command] if namespace == "int" && command == "add" => Some(INT_ADD_HELP),
        [namespace, command] if namespace == "process" && command == "exists" => {
            Some(PROCESS_EXISTS_HELP)
        }
        [namespace, scope] if namespace == "archive" && scope == "local" => {
            Some(ARCHIVE_LOCAL_HELP)
        }
        [namespace, scope, command]
            if namespace == "archive"
                && scope == "local"
                && matches!(command.as_str(), "export" | "import") =>
        {
            Some(ARCHIVE_LOCAL_COMMAND_HELP)
        }
        [namespace, command] if namespace == "fs" && command == "mkdir" => Some(FS_MKDIR_HELP),
        [namespace, command] if namespace == "fs" && command == "write" => Some(FS_WRITE_HELP),
        [namespace, command] if namespace == "fs" && command == "write-base64" => {
            Some(FS_WRITE_BASE64_HELP)
        }
        [namespace, command] if namespace == "fs" && command == "chmod" => Some(FS_CHMOD_HELP),
        [namespace, command] if namespace == "fs" && command == "mode" => Some(FS_MODE_HELP),
        [namespace, command] if namespace == "fs" && command == "touch" => Some(FS_TOUCH_HELP),
        [namespace, command] if namespace == "fs" && command == "list" => Some(FS_LIST_HELP),
        [namespace, command] if namespace == "fs" && command == "contains-any" => {
            Some(FS_CONTAINS_ANY_HELP)
        }
        [namespace, command] if namespace == "fs" && command == "backup-numbered" => {
            Some(FS_BACKUP_NUMBERED_HELP)
        }
        [namespace, scope] if namespace == "gitee" && scope == "repo" => Some(GITEE_REPO_HELP),
        [namespace, scope] if namespace == "gitee" && scope == "pr" => Some(GITEE_PR_HELP),
        [namespace, scope, command]
            if namespace == "gitee" && scope == "repo" && command == "parse-origin" =>
        {
            Some(GITEE_REPO_PARSE_ORIGIN_HELP)
        }
        [namespace, scope, command]
            if namespace == "gitee"
                && scope == "pr"
                && matches!(command.as_str(), "create" | "pass-gates" | "merge") =>
        {
            Some(GITEE_PR_COMMAND_HELP)
        }
        [namespace, scope] if namespace == "ssh" && scope == "config" => Some(SSH_CONFIG_HELP),
        [namespace, scope] if namespace == "ssh" && scope == "script" => Some(SSH_SCRIPT_HELP),
        [namespace, scope, command]
            if namespace == "ssh"
                && scope == "config"
                && matches!(command.as_str(), "host" | "identities") =>
        {
            Some(SSH_CONFIG_COMMAND_HELP)
        }
        [namespace, scope, command]
            if namespace == "ssh"
                && scope == "script"
                && matches!(command.as_str(), "run" | "capture") =>
        {
            Some(SSH_SCRIPT_COMMAND_HELP)
        }
        [namespace, scope] if namespace == "github" && scope == "pr" => Some(GITHUB_PR_HELP),
        [namespace, scope, sub] if namespace == "github" && scope == "pr" && sub == "checks" => {
            Some(GITHUB_PR_CHECKS_HELP)
        }
        [namespace, scope, sub, command]
            if namespace == "github" && scope == "pr" && sub == "checks" && command == "probe" =>
        {
            Some(GITHUB_PR_CHECKS_PROBE_HELP)
        }
        [namespace, scope] if namespace == "cloudflare" && scope == "config" => {
            Some(CLOUDFLARE_CONFIG_HELP)
        }
        [namespace, scope] if namespace == "cloudflare" && scope == "api" => {
            Some(CLOUDFLARE_API_HELP)
        }
        [namespace, scope] if namespace == "cloudflare" && scope == "zone" => {
            Some(CLOUDFLARE_ZONE_HELP)
        }
        [namespace, scope] if namespace == "cloudflare" && scope == "account" => {
            Some(CLOUDFLARE_ACCOUNT_HELP)
        }
        [namespace, scope] if namespace == "cloudflare" && scope == "redirect-rule" => {
            Some(CLOUDFLARE_REDIRECT_RULE_HELP)
        }
        [namespace, scope, command]
            if namespace == "cloudflare"
                && scope == "config"
                && matches!(command.as_str(), "get" | "json") =>
        {
            Some(CLOUDFLARE_CONFIG_COMMAND_HELP)
        }
        [namespace, scope, command]
            if namespace == "cloudflare" && scope == "api" && command == "request" =>
        {
            Some(CLOUDFLARE_API_REQUEST_HELP)
        }
        [namespace, scope, sub]
            if namespace == "cloudflare" && scope == "zone" && sub == "ruleset" =>
        {
            Some(CLOUDFLARE_ZONE_RULESET_HELP)
        }
        [namespace, scope, sub]
            if namespace == "cloudflare" && scope == "zone" && sub == "dns-record" =>
        {
            Some(CLOUDFLARE_ZONE_DNS_RECORD_HELP)
        }
        [namespace, scope, sub, command]
            if namespace == "cloudflare"
                && scope == "zone"
                && sub == "ruleset"
                && matches!(command.as_str(), "list" | "get" | "create" | "rule") =>
        {
            Some(CLOUDFLARE_ZONE_RULESET_COMMAND_HELP)
        }
        [namespace, scope, sub, command]
            if namespace == "cloudflare"
                && scope == "zone"
                && sub == "dns-record"
                && matches!(command.as_str(), "list" | "create" | "update") =>
        {
            Some(CLOUDFLARE_ZONE_DNS_RECORD_COMMAND_HELP)
        }
        [namespace, scope, command]
            if namespace == "cloudflare"
                && scope == "zone"
                && matches!(command.as_str(), "get" | "ruleset" | "dns-record") =>
        {
            Some(CLOUDFLARE_ZONE_COMMAND_HELP)
        }
        [namespace, scope, sub, command]
            if namespace == "cloudflare"
                && scope == "zone"
                && sub == "ruleset"
                && command == "rule" =>
        {
            Some(CLOUDFLARE_ZONE_RULESET_RULE_HELP)
        }
        [namespace, scope, sub, command, action]
            if namespace == "cloudflare"
                && scope == "zone"
                && sub == "ruleset"
                && command == "rule"
                && matches!(action.as_str(), "add" | "update") =>
        {
            Some(CLOUDFLARE_ZONE_RULESET_RULE_COMMAND_HELP)
        }
        [namespace, scope, command]
            if namespace == "cloudflare"
                && scope == "account"
                && matches!(command.as_str(), "get" | "r2") =>
        {
            Some(CLOUDFLARE_ACCOUNT_COMMAND_HELP)
        }
        [namespace, scope, sub]
            if namespace == "cloudflare" && scope == "account" && sub == "r2" =>
        {
            Some(CLOUDFLARE_ACCOUNT_R2_HELP)
        }
        [namespace, scope, sub, command]
            if namespace == "cloudflare"
                && scope == "account"
                && sub == "r2"
                && command == "bucket" =>
        {
            Some(CLOUDFLARE_ACCOUNT_R2_BUCKET_HELP)
        }
        [namespace, scope, sub, command, leaf]
            if namespace == "cloudflare"
                && scope == "account"
                && sub == "r2"
                && command == "bucket"
                && leaf == "list" =>
        {
            Some(CLOUDFLARE_ACCOUNT_R2_BUCKET_LIST_HELP)
        }
        [namespace, scope, command]
            if namespace == "cloudflare" && scope == "redirect-rule" && command == "exact" =>
        {
            Some(CLOUDFLARE_REDIRECT_RULE_EXACT_HELP)
        }
        _ => None,
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

const JSON_HELP: &str = "\
Usage: runseal @tool json <command> [args]

JSON helpers:
  get <json> <path>                 print one JSON value
  empty <json>                      print true when JSON length is zero
  len <json>                        print JSON array/object/string length
  pretty <json>                     print formatted JSON
  find <array> <field> <value>      print first object with field=value
  filter <array> <field> <value>... print objects with field matching values
";
const JSON_GET_HELP: &str = "usage: runseal @tool json get <json> <path>";
const JSON_EMPTY_HELP: &str = "usage: runseal @tool json empty <json>";
const JSON_LEN_HELP: &str = "usage: runseal @tool json len <json>";
const JSON_PRETTY_HELP: &str = "usage: runseal @tool json pretty <json>";
const JSON_FIND_HELP: &str = "usage: runseal @tool json find <array> <field> <value>";
const JSON_FILTER_HELP: &str = "usage: runseal @tool json filter <array> <field> <value>...";

const STRING_HELP: &str = "\
Usage: runseal @tool string <command> [args]

String helpers:
  trim <value>                          trim leading and trailing whitespace
  join <json-array> --separator <text>  join a JSON string array
  slug <value>                          normalize text for branch/file slugs
";
const STRING_TRIM_HELP: &str = "usage: runseal @tool string trim <value>";
const STRING_JOIN_HELP: &str =
    "usage: runseal @tool string join <json-array> --separator <text|path>";
const STRING_SLUG_HELP: &str =
    "usage: runseal @tool string slug <value> [--max-len <n>] [--fallback <text>]";

const REGEX_HELP: &str = "\
Usage: runseal @tool regex <command> [args]

Regex helpers:
  capture <value> <pattern> <group>  print regex capture group n, or empty
";
const REGEX_CAPTURE_HELP: &str = "usage: runseal @tool regex capture <value> <pattern> <group>";

const INT_HELP: &str = "\
Usage: runseal @tool int <command> [args]

Integer helpers:
  add <left> <right>  print integer sum
";
const INT_ADD_HELP: &str = "usage: runseal @tool int add <left> <right>";

const PROCESS_HELP: &str = "\
Usage: runseal @tool process <command> [args]

Process helpers:
  exists <name>  print true when command exists on PATH
";
const PROCESS_EXISTS_HELP: &str = "usage: runseal @tool process exists <name>";

const ARCHIVE_HELP: &str = "\
Usage: runseal @tool archive <scope> <command> [args]

Archive helpers:
  local export  encrypt a .local-style directory archive
  local import  decrypt and restore a .local-style archive
";
const ARCHIVE_LOCAL_HELP: &str = "usage: runseal @tool archive local export|import ...";
const ARCHIVE_LOCAL_COMMAND_HELP: &str = "\
usage: runseal @tool archive local export|import --source <dir> --archive <path> (--password <text>|--password-env <name>) [--force]";

const FS_HELP: &str = "\
Usage: runseal @tool fs <command> [args]

Filesystem helpers:
  mkdir <path> [mode]                    create a directory and parents
  write <path> <text> [mode]            write text to a file
  write-base64 <path> <base64>          write decoded bytes to a file
  chmod <path> <mode>                   set a file mode on Unix
  mode <path>                           print file mode on Unix
  touch <path> [mode]                   create a file without truncating it
  list <path> [--glob <pattern>]        list matching direct children as JSON
  contains-any <path> <text>...         print true when file contains any text
  backup-numbered <path>                move path to .bak or .bak.N and print it
";
const FS_MKDIR_HELP: &str = "usage: runseal @tool fs mkdir <path> [mode]";
const FS_WRITE_HELP: &str = "usage: runseal @tool fs write <path> <text> [mode]";
const FS_WRITE_BASE64_HELP: &str = "usage: runseal @tool fs write-base64 <path> <base64>";
const FS_CHMOD_HELP: &str = "usage: runseal @tool fs chmod <path> <mode>";
const FS_MODE_HELP: &str = "usage: runseal @tool fs mode <path>";
const FS_TOUCH_HELP: &str = "usage: runseal @tool fs touch <path> [mode]";
const FS_LIST_HELP: &str = "usage: runseal @tool fs list <path> [--glob <pattern>] [--files] [--dirs] [--require-nonempty]";
const FS_CONTAINS_ANY_HELP: &str = "usage: runseal @tool fs contains-any <path> <text>...";
const FS_BACKUP_NUMBERED_HELP: &str = "usage: runseal @tool fs backup-numbered <path>";

const GITEE_HELP: &str = "\
Usage: runseal @tool gitee <scope> <command> [args]

Gitee helpers:
  repo parse-origin <url>   parse owner/repo from origin URL
  pr create                 create a pull request
  pr pass-gates             best-effort pass PR gates
  pr merge                  merge a pull request
";
const GITEE_REPO_HELP: &str = "usage: runseal @tool gitee repo parse-origin <url>";
const GITEE_REPO_PARSE_ORIGIN_HELP: &str = "usage: runseal @tool gitee repo parse-origin <url>";
const GITEE_PR_HELP: &str = "usage: runseal @tool gitee pr create|pass-gates|merge ...";
const GITEE_PR_COMMAND_HELP: &str = "usage: runseal @tool gitee pr create|pass-gates|merge ...";

const SSH_HELP: &str = "\
Usage: runseal @tool ssh <scope> <command> [args]

SSH helpers:
  config host <host> --config <path>    print true when Host patterns match
  config identities --config <path>     print IdentityFile paths as JSON
  script run --config <path>            run a local script on an SSH host
  script capture --config <path>        run a local script and print stdout
";
const SSH_CONFIG_HELP: &str = "usage: runseal @tool ssh config host|identities ...";
const SSH_CONFIG_COMMAND_HELP: &str = "usage: runseal @tool ssh config host <host> --config <path>";
const SSH_SCRIPT_HELP: &str = "usage: runseal @tool ssh script run|capture --config <path> --host <host> --file <path> -- <args...>";
const SSH_SCRIPT_COMMAND_HELP: &str = "usage: runseal @tool ssh script run|capture --config <path> --host <host> --file <path> -- <args...>";

const GITHUB_HELP: &str = "\
Usage: runseal @tool github <scope> <command> [args]

GitHub helpers:
  pr checks probe <number>  print true when PR checks are reported
";
const GITHUB_PR_HELP: &str = "usage: runseal @tool github pr checks probe <number>";
const GITHUB_PR_CHECKS_HELP: &str = "usage: runseal @tool github pr checks probe <number>";
const GITHUB_PR_CHECKS_PROBE_HELP: &str = "usage: runseal @tool github pr checks probe <number>";

const CLOUDFLARE_HELP: &str = "\
Usage: runseal @tool cloudflare <scope> <command> [args]

Cloudflare helpers:
  config get|json          inspect configured account/zone defaults
  api request              send one authenticated API request
  zone get|ruleset|dns-record
  account get|r2 bucket list
  redirect-rule exact      build one redirect rule payload
";
const CLOUDFLARE_CONFIG_HELP: &str = "usage: runseal @tool cloudflare config get <key>|json";
const CLOUDFLARE_CONFIG_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare config get <key>|json";
const CLOUDFLARE_API_HELP: &str =
    "usage: runseal @tool cloudflare api request <method> <path> [--query k=v]... [--json <json>]";
const CLOUDFLARE_API_REQUEST_HELP: &str =
    "usage: runseal @tool cloudflare api request <method> <path> [--query k=v]... [--json <json>]";
const CLOUDFLARE_ZONE_HELP: &str =
    "usage: runseal @tool cloudflare zone get|ruleset|dns-record ...";
const CLOUDFLARE_ZONE_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare zone get|ruleset|dns-record ...";
const CLOUDFLARE_ZONE_RULESET_HELP: &str =
    "usage: runseal @tool cloudflare zone ruleset list|get|create|rule ...";
const CLOUDFLARE_ZONE_RULESET_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare zone ruleset list|get|create|rule ...";
const CLOUDFLARE_ZONE_RULESET_RULE_HELP: &str =
    "usage: runseal @tool cloudflare zone ruleset rule add|update ...";
const CLOUDFLARE_ZONE_RULESET_RULE_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare zone ruleset rule add|update ...";
const CLOUDFLARE_ZONE_DNS_RECORD_HELP: &str =
    "usage: runseal @tool cloudflare zone dns-record list|create|update ...";
const CLOUDFLARE_ZONE_DNS_RECORD_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare zone dns-record list|create|update ...";
const CLOUDFLARE_ACCOUNT_HELP: &str =
    "usage: runseal @tool cloudflare account get|r2 bucket list ...";
const CLOUDFLARE_ACCOUNT_COMMAND_HELP: &str =
    "usage: runseal @tool cloudflare account get|r2 bucket list ...";
const CLOUDFLARE_ACCOUNT_R2_HELP: &str =
    "usage: runseal @tool cloudflare account get|r2 bucket list ...";
const CLOUDFLARE_ACCOUNT_R2_BUCKET_HELP: &str =
    "usage: runseal @tool cloudflare account get|r2 bucket list ...";
const CLOUDFLARE_ACCOUNT_R2_BUCKET_LIST_HELP: &str =
    "usage: runseal @tool cloudflare account get|r2 bucket list ...";
const CLOUDFLARE_REDIRECT_RULE_HELP: &str =
    "usage: runseal @tool cloudflare redirect-rule exact ...";
const CLOUDFLARE_REDIRECT_RULE_EXACT_HELP: &str =
    "usage: runseal @tool cloudflare redirect-rule exact ...";
