mod basic;
mod cloudflare;
mod github;
mod hash_version;
mod json;
mod ssh;

#[derive(Clone, Copy)]
pub struct Entry {
    key: &'static str,
    usage: &'static str,
    about: Option<&'static str>,
    sections: &'static [Section],
    examples: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub struct Section {
    title: &'static str,
    items: &'static [(&'static str, &'static str)],
}

const ENTRIES: &[Entry] = &[
    json::JSON,
    json::JSON_GET,
    json::JSON_HAS,
    json::JSON_EMPTY,
    json::JSON_LEN,
    json::JSON_PRETTY,
    json::JSON_PRETTY_VALUE,
    json::JSON_PRETTY_STDIN,
    json::JSON_PRETTY_FILE,
    json::JSON_FIND,
    json::JSON_FILTER,
    hash_version::HASH,
    hash_version::HASH_TREE,
    basic::STRING,
    basic::STRING_TRIM,
    basic::STRING_JOIN,
    basic::STRING_SLUG,
    basic::REGEX,
    basic::REGEX_CAPTURE,
    basic::INT,
    basic::INT_ADD,
    basic::PROCESS,
    basic::PROCESS_EXISTS,
    basic::PROCESS_WRITE,
    basic::ARCHIVE,
    basic::ARCHIVE_LOCAL,
    basic::ARCHIVE_LOCAL_EXPORT,
    basic::ARCHIVE_LOCAL_IMPORT,
    basic::FS,
    basic::FS_LIST,
    basic::GITEE,
    basic::GITEE_REPO,
    basic::GITEE_REPO_PARSE_ORIGIN,
    basic::GITEE_PR,
    basic::GITEE_PR_CREATE,
    basic::GITEE_PR_PASS_GATES,
    basic::GITEE_PR_MERGE,
    hash_version::VERSION,
    hash_version::VERSION_PART,
    hash_version::VERSION_COMPARE,
    github::GITHUB,
    github::GITHUB_ISSUE,
    github::GITHUB_ISSUE_COMMENT,
    github::GITHUB_ISSUE_COMMENT_CREATE,
    github::GITHUB_ISSUE_BODY,
    github::GITHUB_ISSUE_BODY_UPDATE,
    ssh::SSH,
    ssh::SSH_CONFIG,
    ssh::SSH_CONFIG_HOST,
    ssh::SSH_CONFIG_IDENTITIES,
    ssh::SSH_SCRIPT,
    ssh::SSH_SCRIPT_RUN,
    ssh::SSH_SCRIPT_CAPTURE,
    cloudflare::CLOUDFLARE,
    cloudflare::CLOUDFLARE_CONFIG,
    cloudflare::CLOUDFLARE_CONFIG_GET,
    cloudflare::CLOUDFLARE_CONFIG_JSON,
    cloudflare::CLOUDFLARE_API,
    cloudflare::CLOUDFLARE_API_REQUEST,
    cloudflare::CLOUDFLARE_ZONE,
    cloudflare::CLOUDFLARE_ZONE_GET,
    cloudflare::CLOUDFLARE_ZONE_RULESET,
    cloudflare::CLOUDFLARE_ZONE_RULESET_LIST,
    cloudflare::CLOUDFLARE_ZONE_RULESET_GET,
    cloudflare::CLOUDFLARE_ZONE_RULESET_CREATE,
    cloudflare::CLOUDFLARE_ZONE_RULESET_RULE,
    cloudflare::CLOUDFLARE_ZONE_RULESET_RULE_ADD,
    cloudflare::CLOUDFLARE_ZONE_RULESET_RULE_UPDATE,
    cloudflare::CLOUDFLARE_ZONE_DNS_RECORD,
    cloudflare::CLOUDFLARE_ZONE_DNS_RECORD_UPDATE,
    cloudflare::CLOUDFLARE_ACCOUNT,
    cloudflare::CLOUDFLARE_ACCOUNT_GET,
    cloudflare::CLOUDFLARE_ACCOUNT_R2,
    cloudflare::CLOUDFLARE_ACCOUNT_R2_BUCKET,
    cloudflare::CLOUDFLARE_ACCOUNT_R2_BUCKET_LIST,
    cloudflare::CLOUDFLARE_REDIRECT_RULE,
    cloudflare::CLOUDFLARE_REDIRECT_RULE_EXACT,
];

pub fn top() -> &'static str {
    "\
Usage: runseal @tool <namespace> <command> [args]

Run an atomic runseal tool command.

Tools:
  json ...                               JSON helpers
  hash ...                               hash helpers
  string ...                             string helpers
  regex ...                              regex helpers
  int ...                                integer helpers
  process ...                            process helpers
  archive ...                            archive helpers
  fs ...                                 filesystem helpers
  version ...                            version helpers
  gitee ...                              gitee helpers
  ssh ...                                ssh helpers
  github ...                             github helpers
  cloudflare ...                         cloudflare helpers
"
}

pub fn progressive(args: &[String]) -> Option<String> {
    let last = args.last()?;
    if !matches!(last.as_str(), "-h" | "--help" | "help") {
        return None;
    }
    let path = &args[..args.len() - 1];
    if path.is_empty() {
        return Some(top().to_string());
    }
    let key = path.join(".");
    let entry = ENTRIES.iter().find(|entry| entry.key == key)?;
    Some(render(entry))
}

fn render(entry: &Entry) -> String {
    if entry.about.is_none() && entry.sections.is_empty() && entry.examples.is_empty() {
        return format!("usage: {}", entry.usage);
    }
    let mut out = format!("Usage: {}\n", entry.usage);
    if let Some(about) = entry.about {
        out.push('\n');
        out.push_str(about);
        out.push('\n');
    }
    for section in entry.sections {
        out.push('\n');
        out.push_str(section.title);
        out.push_str(":\n");
        for (name, desc) in section.items {
            out.push_str("  ");
            out.push_str(name);
            let padding = 44usize.saturating_sub(name.len()).max(2);
            out.push_str(&" ".repeat(padding));
            out.push_str(desc);
            out.push('\n');
        }
    }
    if !entry.examples.is_empty() {
        out.push('\n');
        out.push_str("Examples:\n");
        for example in entry.examples {
            out.push_str("  ");
            out.push_str(example);
            out.push('\n');
        }
    }
    out
}
