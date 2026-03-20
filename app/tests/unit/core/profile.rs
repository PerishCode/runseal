use super::*;

#[test]
fn parse_injections_with_defaults() {
    let raw = r#"
        {
          "injections": [
            { "type": "env", "vars": { "A": "1", "B": "2" } },
            { "type": "symlink", "source": "./fixtures/agents.md", "target": "~/.codex/AGENTS.md" }
          ]
        }"#;

    let profile: Profile = serde_json::from_str(raw).expect("profile should parse");
    assert_eq!(profile.injections.len(), 2);

    match &profile.injections[0] {
        InjectionProfile::Env(env) => {
            assert!(env.enabled);
            assert_eq!(env.vars.get("A"), Some(&"1".to_string()));
            assert_eq!(env.vars.get("B"), Some(&"2".to_string()));
            assert!(env.ops.is_empty());
        }
        _ => panic!("expected env injection"),
    }
    match &profile.injections[1] {
        InjectionProfile::Symlink(link) => {
            assert!(link.enabled);
            assert!(matches!(link.on_exist, SymlinkOnExist::Error));
            assert!(link.cleanup);
        }
        _ => panic!("expected symlink injection"),
    }
}

#[test]
fn reject_unknown_injection_type() {
    let raw = r#"
        {
          "injections": [
            { "type": "python" }
          ]
        }"#;

    let err = serde_json::from_str::<Profile>(raw).expect_err("unknown type should fail");
    let msg = err.to_string();
    assert!(msg.contains("unknown variant"));
}

#[test]
fn parse_env_ops() {
    let raw = r#"
        {
          "injections": [
            {
              "type": "env",
              "vars": { "A": "1" },
              "ops": [
                { "op": "prepend", "key": "PATH", "value": "/opt/bin", "separator": "os", "dedup": true },
                { "op": "set_if_absent", "key": "NPM_CONFIG_REGISTRY", "value": "https://registry.npmjs.org/" }
              ]
            }
          ]
        }"#;

    let profile: Profile = serde_json::from_str(raw).expect("profile should parse");
    match &profile.injections[0] {
        InjectionProfile::Env(env) => {
            assert_eq!(env.vars.get("A"), Some(&"1".to_string()));
            assert_eq!(env.ops.len(), 2);
        }
        _ => panic!("expected env injection"),
    }
}

#[test]
fn parse_command_injection() {
    let raw = r#"
        {
          "injections": [
            { "type": "command", "program": "fnm", "args": ["env", "--shell", "bash"] }
          ]
        }"#;

    let profile: Profile = serde_json::from_str(raw).expect("profile should parse");
    match &profile.injections[0] {
        InjectionProfile::Command(cmd) => {
            assert!(cmd.enabled);
            assert_eq!(cmd.program, "fnm");
            assert_eq!(cmd.args, vec!["env", "--shell", "bash"]);
        }
        _ => panic!("expected command injection"),
    }
}
