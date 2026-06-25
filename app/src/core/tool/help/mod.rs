mod cloudflare;
mod github;

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
    github::GITHUB,
    github::GITHUB_ISSUE,
    github::GITHUB_ISSUE_CREATE,
    github::GITHUB_ISSUE_COMMENT,
    github::GITHUB_ISSUE_COMMENT_CREATE,
    github::GITHUB_ISSUE_BODY,
    github::GITHUB_ISSUE_BODY_UPDATE,
    github::GITHUB_PR,
    github::GITHUB_PR_CHECKS,
    github::GITHUB_PR_CHECKS_PROBE,
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
    cloudflare::CLOUDFLARE_ZONE_DNS_RECORD_LIST,
    cloudflare::CLOUDFLARE_ZONE_DNS_RECORD_CREATE,
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
