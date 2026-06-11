use std::{collections::BTreeMap, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "pr" => pr(args),
        "repo" => repo(args),
        _ => bail!("unknown tool command: gitee {command}"),
    }
}

fn repo(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!("usage: runseal @tool gitee repo parse-origin <url>");
    };
    match command.as_str() {
        "parse-origin" => repo_parse_origin(rest),
        _ => bail!("usage: runseal @tool gitee repo parse-origin <url>"),
    }
}

fn repo_parse_origin(args: &[String]) -> Result<Option<String>> {
    let [url] = args else {
        bail!("usage: runseal @tool gitee repo parse-origin <url>");
    };
    let (owner, repo) = parse_origin(url)?;
    Ok(Some(serde_json::to_string(&serde_json::json!({
        "owner": owner,
        "repo": repo,
    }))?))
}

fn pr(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!("usage: runseal @tool gitee pr find|create|pass-gates|merge ...");
    };
    match command.as_str() {
        "find" => pr_find(rest),
        "create" => pr_create(rest),
        "pass-gates" => pr_pass_gates(rest),
        "merge" => pr_merge(rest),
        _ => bail!("usage: runseal @tool gitee pr find|create|pass-gates|merge ..."),
    }
}

fn pr_find(args: &[String]) -> Result<Option<String>> {
    let owner = required_option(args, "--owner")?;
    let repo = required_option(args, "--repo")?;
    let token = token(args)?;
    let head = required_option(args, "--head")?;
    let base = optional_option(args, "--base");
    let state = optional_option(args, "--state").unwrap_or_else(|| "open".to_string());
    if !matches!(state.as_str(), "open" | "merged" | "closed" | "all") {
        bail!("--state must be one of: open, merged, closed, all");
    }

    let mut query = vec![
        ("head".to_string(), head.clone()),
        ("state".to_string(), state),
    ];
    if let Some(base) = &base {
        query.push(("base".to_string(), base.clone()));
    }

    let raw = request_json(
        "GET",
        &format!("/repos/{owner}/{repo}/pulls"),
        &token,
        None,
        &query,
    )?;
    let Some(items) = raw.as_array() else {
        bail!("Gitee API returned non-array JSON for pull request listing");
    };

    let matches = items
        .iter()
        .filter(|item| pr_matches(item, &head, base.as_deref()))
        .cloned()
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Ok(Some("null".to_string())),
        1 => Ok(Some(serde_json::to_string(&matches[0])?)),
        count => bail!(
            "Gitee PR find is ambiguous for head `{head}`{}: found {count} matches",
            base.as_deref()
                .map(|value| format!(" and base `{value}`"))
                .unwrap_or_default()
        ),
    }
}

fn pr_create(args: &[String]) -> Result<Option<String>> {
    let owner = required_option(args, "--owner")?;
    let repo = required_option(args, "--repo")?;
    let token = token(args)?;
    let base = required_option(args, "--base")?;
    let head = required_option(args, "--head")?;
    let title = required_option(args, "--title")?;
    let body = required_option(args, "--body")?;
    request(
        "POST",
        &format!("/repos/{owner}/{repo}/pulls"),
        &token,
        serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
            "body": body,
        }),
    )
}

fn pr_pass_gates(args: &[String]) -> Result<Option<String>> {
    let owner = required_option(args, "--owner")?;
    let repo = required_option(args, "--repo")?;
    let token = token(args)?;
    let number = required_option(args, "--number")?;
    let mut result = serde_json::Map::new();
    for op in ["review", "test"] {
        let passed = request(
            "POST",
            &format!("/repos/{owner}/{repo}/pulls/{number}/{op}"),
            &token,
            serde_json::json!({}),
        )
        .is_ok();
        result.insert(op.to_string(), JsonValue::Bool(passed));
    }
    Ok(Some(serde_json::to_string(&JsonValue::Object(result))?))
}

fn pr_merge(args: &[String]) -> Result<Option<String>> {
    let owner = required_option(args, "--owner")?;
    let repo = required_option(args, "--repo")?;
    let token = token(args)?;
    let number = required_option(args, "--number")?;
    let method = optional_option(args, "--method").unwrap_or_else(|| "squash".to_string());
    request(
        "PUT",
        &format!("/repos/{owner}/{repo}/pulls/{number}/merge"),
        &token,
        serde_json::json!({
            "merge_method": method,
        }),
    )
}

fn request(method: &str, path: &str, token: &str, body: JsonValue) -> Result<Option<String>> {
    let payload = request_json(method, path, token, Some(body), &[])?;
    Ok(Some(serde_json::to_string(&payload)?))
}

fn request_json(
    method: &str,
    path: &str,
    token: &str,
    body: Option<JsonValue>,
    query: &[(String, String)],
) -> Result<JsonValue> {
    let base = std::env::var("RUNSEAL_GITEE_API_BASE")
        .unwrap_or_else(|_| "https://gitee.com/api/v5".to_string());
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let url = format!("{base}{path}");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let method = method
        .parse::<reqwest::Method>()
        .with_context(|| format!("invalid HTTP method: {method}"))?;
    let response = if let Some(mut body) = body {
        let Some(object) = body.as_object_mut() else {
            bail!("Gitee request body must be a JSON object");
        };
        object.insert(
            "access_token".to_string(),
            JsonValue::String(token.to_string()),
        );
        client
            .request(method.clone(), &url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
            .query(query)
            .json(&body)
            .send()
            .with_context(|| format!("Gitee API {method} {path} unreachable"))?
    } else {
        client
            .request(method.clone(), &url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
            .query(query)
            .send()
            .with_context(|| format!("Gitee API {method} {path} unreachable"))?
    };
    let status = response.status();
    let raw = response
        .text()
        .with_context(|| format!("Gitee API {method} {path} returned unreadable body"))?;
    if !status.is_success() {
        bail!("Gitee API {method} {path} -> {}: {raw}", status.as_u16());
    }
    if raw.trim().is_empty() {
        return Ok(JsonValue::Object(Default::default()));
    }
    serde_json::from_str(&raw)
        .with_context(|| format!("Gitee API returned invalid JSON for {path}"))
}

fn pr_matches(item: &JsonValue, head: &str, base: Option<&str>) -> bool {
    let Some(item_head) = pr_head_branch(item) else {
        return false;
    };
    if item_head != head {
        return false;
    }
    match base {
        Some(expected) => pr_base_branch(item).is_some_and(|value| value == expected),
        None => true,
    }
}

fn pr_head_branch(item: &JsonValue) -> Option<&str> {
    item.get("head")
        .and_then(|value| value.get("ref").or_else(|| value.get("label")))
        .and_then(JsonValue::as_str)
        .or_else(|| item.get("head").and_then(JsonValue::as_str))
        .map(|value| value.rsplit(':').next().unwrap_or(value))
}

fn pr_base_branch(item: &JsonValue) -> Option<&str> {
    item.get("base")
        .and_then(|value| value.get("ref").or_else(|| value.get("label")))
        .and_then(JsonValue::as_str)
        .or_else(|| item.get("base").and_then(JsonValue::as_str))
        .map(|value| value.rsplit(':').next().unwrap_or(value))
}

fn token(args: &[String]) -> Result<String> {
    if let Some(token) = optional_option(args, "--token")
        && !token.is_empty()
    {
        return Ok(token);
    }
    if let Some(path) = optional_option(args, "--token-file") {
        let values = parse_env_file(Path::new(&path))?;
        if let Some(token) = values.get("GITEE_TOKEN").filter(|value| !value.is_empty()) {
            return Ok(token.clone());
        }
        bail!("GITEE_TOKEN not set in {path}");
    }
    if let Ok(token) = std::env::var("GITEE_TOKEN")
        && !token.is_empty()
    {
        return Ok(token);
    }
    bail!("missing Gitee token: pass --token, --token-file, or set GITEE_TOKEN")
}

fn parse_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read token file: {}", path.display()))?;
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            bail!("invalid line in {}: {line}", path.display());
        };
        values.insert(
            key.trim().to_string(),
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string(),
        );
    }
    Ok(values)
}

fn required_option(args: &[String], name: &str) -> Result<String> {
    optional_option(args, name).ok_or_else(|| anyhow::anyhow!("{name} is required"))
}

fn optional_option(args: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == name {
            return args.get(index + 1).cloned();
        }
        if let Some(value) = arg.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
        index += 1;
    }
    None
}

fn parse_origin(url: &str) -> Result<(String, String)> {
    let Some(after_host) = url
        .strip_prefix("git@gitee.com:")
        .or_else(|| url.strip_prefix("ssh://git@gitee.com/"))
        .or_else(|| url.strip_prefix("https://gitee.com/"))
        .or_else(|| url.strip_prefix("http://gitee.com/"))
    else {
        bail!("cannot parse Gitee owner/repo from origin url: {url}");
    };
    let path = after_host.trim_end_matches(".git");
    let mut parts = path.split('/');
    let Some(owner) = parts.next().filter(|value| !value.is_empty()) else {
        bail!("cannot parse Gitee owner/repo from origin url: {url}");
    };
    let Some(repo) = parts.next().filter(|value| !value.is_empty()) else {
        bail!("cannot parse Gitee owner/repo from origin url: {url}");
    };
    if parts.next().is_some() {
        bail!("cannot parse Gitee owner/repo from origin url: {url}");
    }
    Ok((owner.to_string(), repo.to_string()))
}
