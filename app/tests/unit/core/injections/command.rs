use std::collections::BTreeMap;

use super::*;

struct MockEnv {
    vars: BTreeMap<String, String>,
}

impl EnvReader for MockEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

#[test]
fn parse_export_and_plain_assignment() {
    let env = MockEnv {
        vars: BTreeMap::new(),
    };
    let vars = parse_exports("export A='1'\nB=2\nignored line\n", &env);
    assert_eq!(
        vars,
        vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string())
        ]
    );
}

#[test]
fn parse_fnm_style_path_value() {
    let env = MockEnv {
        vars: BTreeMap::from([("RUNSEAL_TEST_PATH".to_string(), "/usr/bin:/bin".to_string())]),
    };
    let vars = parse_exports(
        "export PATH=\"/tmp/fnm/bin\":\"$RUNSEAL_TEST_PATH\"\n",
        &env,
    );
    assert_eq!(
        vars,
        vec![("PATH".to_string(), "/tmp/fnm/bin:/usr/bin:/bin".to_string())]
    );
}

#[test]
fn preserve_inner_quotes_when_normalizing() {
    let env = MockEnv {
        vars: BTreeMap::new(),
    };
    let vars = parse_exports("export A='x\"y\"z'\n", &env);
    assert_eq!(vars, vec![("A".to_string(), "x\"y\"z".to_string())]);
}

#[test]
fn skip_invalid_env_keys_from_command_output() {
    let env = MockEnv {
        vars: BTreeMap::new(),
    };
    let vars = parse_exports("export BAD-KEY=1\nexport _GOOD=2\n", &env);
    assert_eq!(vars, vec![("_GOOD".to_string(), "2".to_string())]);
}
