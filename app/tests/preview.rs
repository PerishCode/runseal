use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

#[test]
fn preview_text_only_exposes_keys_and_metadata() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("agents.md");
    std::fs::write(&source, "agent file").expect("source file should be written");

    let profile = temp.path().join("preview.json");
    std::fs::write(
        &profile,
        r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "RUNSEAL_TOKEN": "super-secret-token"
      },
      "ops": [
        {
          "op": "set",
          "key": "API_KEY",
          "value": "super-secret-api-key"
        }
      ]
    },
    {
      "type": "command",
      "program": "fnm",
      "args": ["env", "--shell", "bash", "secret-arg"]
    },
    {
      "type": "symlink",
      "source": "./agents.md",
      "target": "~/.codex/AGENTS.md"
    }
  ]
}"#,
    )
    .expect("profile file should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "preview",
            "--profile",
            profile.to_str().expect("profile path should be UTF-8"),
        ])
        .output()
        .expect("preview command should run");

    assert!(
        output.status.success(),
        "preview failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("[env]"));
    assert!(stdout.contains("RUNSEAL_TOKEN"));
    assert!(stdout.contains("API_KEY"));
    assert!(stdout.contains("[command]"));
    assert!(stdout.contains("program=fnm"));
    assert!(stdout.contains("arg_count=4"));
    assert!(stdout.contains("[symlink]"));

    assert!(!stdout.contains("super-secret-token"));
    assert!(!stdout.contains("super-secret-api-key"));
    assert!(!stdout.contains("secret-arg"));
}

#[test]
fn preview_json_has_stable_shape_without_sensitive_values() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("preview-json.json");
    std::fs::write(
        &profile,
        r#"{
  "injections": [
    {
      "type": "env",
      "vars": {
        "A": "1",
        "B": "2"
      }
    }
  ]
}"#,
    )
    .expect("profile file should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "preview",
            "--profile",
            profile.to_str().expect("profile path should be UTF-8"),
            "--output",
            "json",
        ])
        .output()
        .expect("preview command should run");

    assert!(
        output.status.success(),
        "preview failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("preview output should be valid JSON");
    let injections = json["injections"]
        .as_array()
        .expect("injections should be array");
    assert_eq!(injections.len(), 1);
    assert_eq!(injections[0]["type"], "env");
    assert_eq!(injections[0]["keys"][0], "A");
    assert_eq!(injections[0]["keys"][1], "B");
    assert!(stdout.contains("\"keys\""));
    assert!(!stdout.contains("\"A\": \"1\""));
    assert!(!stdout.contains("\"B\": \"2\""));
}
