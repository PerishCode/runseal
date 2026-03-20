use super::*;

#[test]
fn escape_single_quotes_for_shell() {
    assert_eq!(shell_single_quote_escape("a'b"), "a'\"'\"'b");
}

#[test]
fn env_map_keeps_last_value_for_duplicate_keys() {
    let map = to_env_map(
        vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
            ("A".to_string(), "3".to_string()),
        ],
        false,
    )
    .expect("non-strict mode should allow duplicate keys");
    assert_eq!(map.get("A"), Some(&"3".to_string()));
    assert_eq!(map.get("B"), Some(&"2".to_string()));
}

#[test]
fn env_map_rejects_duplicate_keys_in_strict_mode() {
    let err = to_env_map(
        vec![
            ("A".to_string(), "1".to_string()),
            ("A".to_string(), "2".to_string()),
        ],
        true,
    )
    .expect_err("strict mode should reject duplicate keys");
    assert!(err.to_string().contains("duplicate exported key"));
}

#[test]
fn env_map_rejects_invalid_key() {
    let err = to_env_map(vec![("BAD-KEY".to_string(), "1".to_string())], false)
        .expect_err("invalid env key should fail");
    assert!(err.to_string().contains("invalid exported key"));
}
