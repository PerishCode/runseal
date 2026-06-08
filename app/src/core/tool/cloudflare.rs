use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

mod dns_record;

#[derive(Debug, Clone)]
struct Config {
    account_id: String,
    api_token: String,
    zone_name: String,
    manage_host: String,
    manage_origin_host: String,
    manage_redirect_prefix: String,
}

struct Options {
    query: Vec<(String, String)>,
    body: Option<JsonValue>,
}

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match command {
        "config" => config(args),
        "api" => api(args),
        "zone" => zone(args),
        "account" => account(args),
        "redirect-rule" => redirect_rule(args),
        _ => bail!("unknown tool command: cloudflare {command}"),
    }
}

fn config(args: &[String]) -> Result<Option<String>> {
    match args {
        [command, key] if command == "get" => {
            let config = load_config()?;
            Ok(Some(match key.as_str() {
                "account_id" => config.account_id,
                "zone_name" => config.zone_name,
                "manage_host" => config.manage_host,
                "manage_origin_host" => config.manage_origin_host,
                "manage_redirect_prefix" => config.manage_redirect_prefix,
                other => bail!("unknown Cloudflare config key: {other}"),
            }))
        }
        [command] if command == "json" => {
            let config = load_config()?;
            Ok(Some(serde_json::to_string(&serde_json::json!({
                "account_id": config.account_id,
                "zone_name": config.zone_name,
                "manage_host": config.manage_host,
                "manage_origin_host": config.manage_origin_host,
                "manage_redirect_prefix": config.manage_redirect_prefix,
            }))?))
        }
        _ => bail!("usage: runseal @tool cloudflare config get <key>|json"),
    }
}

fn api(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!(
            "usage: runseal @tool cloudflare api request <method> <path> [--query k=v]... [--json <json>]"
        );
    };
    if command != "request" {
        bail!(
            "usage: runseal @tool cloudflare api request <method> <path> [--query k=v]... [--json <json>]"
        );
    }
    let [method, path, options @ ..] = rest else {
        bail!(
            "usage: runseal @tool cloudflare api request <method> <path> [--query k=v]... [--json <json>]"
        );
    };
    let parsed = parse_options(options)?;
    let config = load_config()?;
    request(&config, method, path, parsed.query, parsed.body)
}

fn zone(args: &[String]) -> Result<Option<String>> {
    match args {
        [command, rest @ ..] if command == "get" => zone_get(rest),
        [ruleset, rest @ ..] if ruleset == "ruleset" => zone_ruleset(rest),
        [dns, command, rest @ ..] if dns == "dns-record" => dns_record::eval(command, rest),
        _ => bail!("usage: runseal @tool cloudflare zone get|ruleset|dns-record ..."),
    }
}

fn zone_get(args: &[String]) -> Result<Option<String>> {
    let name = required_option(args, "--name")?;
    let config = load_config()?;
    let payload = request(
        &config,
        "GET",
        "/zones",
        vec![("name".to_string(), name.clone())],
        None,
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    let result = value
        .get("result")
        .and_then(JsonValue::as_array)
        .context("Cloudflare zone response missing result array")?;
    if result.is_empty() {
        bail!("zone not found for name: {name}");
    }
    if result.len() != 1 {
        bail!("expected one zone for {name}, found {}", result.len());
    }
    Ok(Some(serde_json::to_string(&result[0])?))
}

fn zone_ruleset(args: &[String]) -> Result<Option<String>> {
    match args {
        [command, rest @ ..] if command == "list" => zone_ruleset_list(rest),
        [command, rest @ ..] if command == "get" => zone_ruleset_get(rest),
        [command, rest @ ..] if command == "create" => zone_ruleset_create(rest),
        [rule, rest @ ..] if rule == "rule" => zone_ruleset_rule(rest),
        _ => bail!("usage: runseal @tool cloudflare zone ruleset list|get|create|rule ..."),
    }
}

fn zone_ruleset_list(args: &[String]) -> Result<Option<String>> {
    let zone_id = required_option(args, "--zone-id")?;
    let config = load_config()?;
    let payload = request(
        &config,
        "GET",
        &format!("/zones/{zone_id}/rulesets"),
        Vec::new(),
        None,
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    Ok(Some(serde_json::to_string(
        value.get("result").unwrap_or(&JsonValue::Array(Vec::new())),
    )?))
}

fn zone_ruleset_get(args: &[String]) -> Result<Option<String>> {
    let zone_id = required_option(args, "--zone-id")?;
    let ruleset_id = required_option(args, "--ruleset-id")?;
    let config = load_config()?;
    let payload = request(
        &config,
        "GET",
        &format!("/zones/{zone_id}/rulesets/{ruleset_id}"),
        Vec::new(),
        None,
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    Ok(Some(serde_json::to_string(
        value.get("result").unwrap_or(&JsonValue::Null),
    )?))
}

fn zone_ruleset_create(args: &[String]) -> Result<Option<String>> {
    let zone_id = required_option(args, "--zone-id")?;
    let phase = required_option(args, "--phase")?;
    let name = required_option(args, "--name")?;
    let body = serde_json::json!({
        "kind": "zone",
        "name": name,
        "phase": phase,
        "rules": [],
    });
    let config = load_config()?;
    let payload = request(
        &config,
        "POST",
        &format!("/zones/{zone_id}/rulesets"),
        Vec::new(),
        Some(body),
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    Ok(Some(serde_json::to_string(
        value.get("result").unwrap_or(&JsonValue::Null),
    )?))
}

fn zone_ruleset_rule(args: &[String]) -> Result<Option<String>> {
    match args {
        [command, rest @ ..] if command == "add" || command == "update" => {
            let zone_id = required_option(rest, "--zone-id")?;
            let ruleset_id = required_option(rest, "--ruleset-id")?;
            let rule_json = required_option(rest, "--json")?;
            let body: JsonValue = serde_json::from_str(&rule_json).context("invalid rule JSON")?;
            let (method, path) = if command == "add" {
                (
                    "POST",
                    format!("/zones/{zone_id}/rulesets/{ruleset_id}/rules"),
                )
            } else {
                let rule_id = required_option(rest, "--rule-id")?;
                (
                    "PATCH",
                    format!("/zones/{zone_id}/rulesets/{ruleset_id}/rules/{rule_id}"),
                )
            };
            let config = load_config()?;
            let payload = request(&config, method, &path, Vec::new(), Some(body))?;
            let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
            Ok(Some(serde_json::to_string(
                value.get("result").unwrap_or(&JsonValue::Null),
            )?))
        }
        _ => bail!("usage: runseal @tool cloudflare zone ruleset rule add|update ..."),
    }
}

fn account(args: &[String]) -> Result<Option<String>> {
    match args {
        [command, rest @ ..] if command == "get" => account_get(rest),
        [r2, bucket, command, rest @ ..]
            if r2 == "r2" && bucket == "bucket" && command == "list" =>
        {
            r2_bucket_list(rest)
        }
        _ => bail!("usage: runseal @tool cloudflare account get|r2 bucket list ..."),
    }
}

fn account_get(args: &[String]) -> Result<Option<String>> {
    let account_id = required_option(args, "--account-id")?;
    let config = load_config()?;
    let payload = request(
        &config,
        "GET",
        &format!("/accounts/{account_id}"),
        Vec::new(),
        None,
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    Ok(Some(serde_json::to_string(
        value.get("result").unwrap_or(&JsonValue::Null),
    )?))
}

fn r2_bucket_list(args: &[String]) -> Result<Option<String>> {
    let account_id = required_option(args, "--account-id")?;
    let config = load_config()?;
    let payload = request(
        &config,
        "GET",
        &format!("/accounts/{account_id}/r2/buckets"),
        Vec::new(),
        None,
    )?;
    let value: JsonValue = serde_json::from_str(&payload.unwrap_or_default())?;
    Ok(Some(serde_json::to_string(
        value
            .get("result")
            .and_then(|result| result.get("buckets"))
            .unwrap_or(&JsonValue::Array(Vec::new())),
    )?))
}

fn redirect_rule(args: &[String]) -> Result<Option<String>> {
    let [command, rest @ ..] = args else {
        bail!("usage: runseal @tool cloudflare redirect-rule exact ...");
    };
    if command != "exact" {
        bail!("usage: runseal @tool cloudflare redirect-rule exact ...");
    }
    let ref_name = required_option(rest, "--ref")?;
    let description = required_option(rest, "--description")?;
    let host = required_option(rest, "--host")?;
    let path = required_option(rest, "--path")?;
    let target_url = required_option(rest, "--target-url")?;
    let status_code = optional_option(rest, "--status-code")
        .unwrap_or_else(|| "302".to_string())
        .parse::<u16>()
        .context("invalid redirect status code")?;
    Ok(Some(serde_json::to_string(&serde_json::json!({
        "ref": ref_name,
        "description": description,
        "expression": format!("(http.host eq \"{host}\" and http.request.uri.path eq \"{path}\")"),
        "action": "redirect",
        "enabled": true,
        "action_parameters": {
            "from_value": {
                "target_url": {
                    "value": target_url,
                },
                "status_code": status_code,
                "preserve_query_string": false,
            },
        },
    }))?))
}

fn parse_options(args: &[String]) -> Result<Options> {
    let mut query = Vec::new();
    let mut body = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--query" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("--query requires a value");
                };
                let Some((key, value)) = value.split_once('=') else {
                    bail!("invalid --query value: {value}; expected key=value");
                };
                query.push((key.to_string(), value.to_string()));
                index += 2;
            }
            "--json" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("--json requires a value");
                };
                body = Some(serde_json::from_str(value).context("invalid --json payload")?);
                index += 2;
            }
            other => bail!("unknown Cloudflare option: {other}"),
        }
    }
    Ok(Options { query, body })
}

fn load_config() -> Result<Config> {
    let values = parse_env_file(&token_file()?)?;
    let account_id = required_config_value(&values, "CLOUDFLARE_ACCOUNT_ID")?;
    let api_token = required_config_value(&values, "CLOUDFLARE_API_TOKEN")?;
    Ok(Config {
        account_id,
        api_token,
        zone_name: values
            .get("CLOUDFLARE_ZONE_NAME")
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "perish.uk".to_string()),
        manage_host: values
            .get("CLOUDFLARE_MANAGE_HOST")
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "runseal.perish.uk".to_string()),
        manage_origin_host: values
            .get("CLOUDFLARE_MANAGE_ORIGIN_HOST")
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "releases.runseal.perish.uk".to_string()),
        manage_redirect_prefix: values
            .get("CLOUDFLARE_MANAGE_REDIRECT_PREFIX")
            .map(|value| value.trim_matches('/').to_string())
            .unwrap_or_default(),
    })
}

fn token_file() -> Result<PathBuf> {
    let secrets = std::env::var_os("RUNSEAL_REPO_SECRETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".local").join("secrets"));
    Ok(secrets.join("cloudflare.env"))
}

fn parse_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("missing secrets file: {}", path.display()))?;
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

fn required_config_value(values: &BTreeMap<String, String>, key: &str) -> Result<String> {
    let Some(value) = values.get(key).filter(|value| !value.is_empty()) else {
        bail!(
            "missing required key(s) in {}: {key}",
            token_file()?.display()
        );
    };
    Ok(value.clone())
}

fn request(
    config: &Config,
    method: &str,
    path: &str,
    query: Vec<(String, String)>,
    body: Option<JsonValue>,
) -> Result<Option<String>> {
    let base = std::env::var("RUNSEAL_CLOUDFLARE_API_BASE")
        .unwrap_or_else(|_| "https://api.cloudflare.com/client/v4".to_string());
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
    let mut request = client
        .request(method.clone(), &url)
        .bearer_auth(&config.api_token)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::CONTENT_TYPE, "application/json");
    if !query.is_empty() {
        request = request.query(&query);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request
        .send()
        .with_context(|| format!("Cloudflare API {method} {path} unreachable"))?;
    let status = response.status();
    let raw = response
        .text()
        .with_context(|| format!("Cloudflare API {method} {path} returned unreadable body"))?;
    if !status.is_success() {
        bail!(
            "Cloudflare API {method} {path} -> {}: {raw}",
            status.as_u16()
        );
    }
    let payload: JsonValue = if raw.trim().is_empty() {
        JsonValue::Object(Default::default())
    } else {
        serde_json::from_str(&raw)
            .with_context(|| format!("Cloudflare API returned invalid JSON for {path}"))?
    };
    if payload
        .get("success")
        .and_then(JsonValue::as_bool)
        .is_some_and(|success| !success)
    {
        bail!("Cloudflare API {method} {path} failed: {payload}");
    }
    Ok(Some(serde_json::to_string(&payload)?))
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
