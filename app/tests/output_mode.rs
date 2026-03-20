use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

#[test]
fn output_json_mode_prints_json_object() {
    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args([
            "-p",
            "examples/runseal.sample.json",
            "--output",
            "json",
            "--log-level",
            "error",
        ])
        .env_remove("RUST_LOG")
        .output()
        .expect("runseal command should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON object");
    let obj = json.as_object().expect("JSON output should be an object");

    let keys: BTreeSet<&str> = obj.keys().map(|k| k.as_str()).collect();
    let expected: BTreeSet<&str> = BTreeSet::from([
        "RUNSEAL_PROFILE",
        "RUNSEAL_NODE_VERSION",
        "NPM_CONFIG_REGISTRY",
        "KUBECONFIG_CONTEXT",
        "KUBECONFIG_NAMESPACE",
    ]);
    assert_eq!(
        keys, expected,
        "JSON output keys should be fully enumerable"
    );
}

#[test]
fn shell_output_can_be_evaluated_and_consumed() {
    let output = Command::new(env!("CARGO_BIN_EXE_runseal"))
        .args(["-p", "examples/runseal.sample.json", "--log-level", "error"])
        .env_remove("RUST_LOG")
        .output()
        .expect("runseal command should run");
    assert!(output.status.success());

    let shell_exports = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let mut child = Command::new("bash")
        .args([
            "-lc",
            "set -e; eval \"$(cat)\"; printf '%s|%s|%s\\n' \"$RUNSEAL_PROFILE\" \"$RUNSEAL_NODE_VERSION\" \"$KUBECONFIG_CONTEXT\"",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("bash should run");

    child
        .stdin
        .as_mut()
        .expect("stdin should be available")
        .write_all(shell_exports.as_bytes())
        .expect("should write exports to stdin");

    let eval_output = child.wait_with_output().expect("should wait for bash");
    assert!(
        eval_output.status.success(),
        "shell eval should succeed: {}",
        String::from_utf8_lossy(&eval_output.stderr)
    );

    let result = String::from_utf8(eval_output.stdout).expect("eval stdout should be UTF-8");
    assert_eq!(result.trim(), "dev|22.11.0|dev-cluster");
}
