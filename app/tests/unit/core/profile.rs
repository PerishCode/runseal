use super::*;
use tempfile::TempDir;

#[test]
fn loads_toml_yaml_and_json_profiles() {
    let temp = TempDir::new().expect("temp dir should be created");
    let toml_path = temp.path().join("runseal.toml");
    let yaml_path = temp.path().join("runseal.yaml");
    let json_path = temp.path().join("runseal.json");

    std::fs::write(
        &toml_path,
        "[[injections]]\ntype = \"env\"\n[injections.vars]\nA = \"toml\"\n",
    )
    .expect("toml should be written");
    std::fs::write(
        &yaml_path,
        "injections:\n  - type: env\n    vars:\n      A: yaml\n",
    )
    .expect("yaml should be written");
    std::fs::write(
        &json_path,
        r#"{"injections":[{"type":"env","vars":{"A":"json"}}]}"#,
    )
    .expect("json should be written");

    assert_eq!(
        load(&toml_path).expect("toml should load").injections.len(),
        1
    );
    assert_eq!(
        load(&yaml_path).expect("yaml should load").injections.len(),
        1
    );
    assert_eq!(
        load(&json_path).expect("json should load").injections.len(),
        1
    );
}

#[test]
fn rejects_unknown_injection_type() {
    let temp = TempDir::new().expect("temp dir should be created");
    let profile = temp.path().join("runseal.json");
    std::fs::write(
        &profile,
        r#"{"injections":[{"type":"exec","program":"echo"}]}"#,
    )
    .expect("profile should be written");

    let err = load(&profile).expect_err("unknown injection should be rejected");
    assert!(err.to_string().contains("failed to parse JSON"));
}
