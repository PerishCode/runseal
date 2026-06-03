use super::*;

#[test]
fn invalid_export_key_is_rejected() {
    let err = to_env_map(vec![("not-valid-key".to_string(), "x".to_string())])
        .expect_err("invalid key should fail");
    assert!(err.to_string().contains("invalid exported key"));
}
