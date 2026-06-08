use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn eval(command: &str, args: &[String]) -> Result<Option<String>> {
    match (command, args) {
        ("pr", [checks, probe, number]) if checks == "checks" && probe == "probe" => {
            pr_checks_probe(number)
        }
        _ => bail!("unknown tool command: github {command}"),
    }
}

fn pr_checks_probe(number: &str) -> Result<Option<String>> {
    let output = Command::new("gh")
        .args(["pr", "checks", number])
        .output()
        .with_context(|| "failed to execute command: gh")?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if combined.contains("no checks reported") {
        return Ok(Some("false".to_string()));
    }
    if output.status.success() {
        return Ok(Some("true".to_string()));
    }
    bail!(
        "gh pr checks failed with status {}: {}",
        output.status.code().unwrap_or(1),
        combined.trim()
    );
}
