use super::*;
use tempfile::TempDir;

#[test]
fn append_and_persist_alias() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut store = AliasStore::default();
    store
        .append("work".to_string(), "profiles/work.json".to_string())
        .expect("append should succeed");
    let path = store.save(temp.path()).expect("save should succeed");
    assert!(path.exists());

    let loaded = AliasStore::load(temp.path()).expect("load should succeed");
    assert_eq!(
        loaded.get("work").map(|entry| entry.profile.as_str()),
        Some("profiles/work.json")
    );
}

#[test]
fn append_rejects_duplicate_name() {
    let mut store = AliasStore::default();
    store
        .append("work".to_string(), "profiles/work.json".to_string())
        .expect("first append should succeed");
    let err = store
        .append("work".to_string(), "profiles/other.json".to_string())
        .expect_err("duplicate append should fail");
    assert!(err.to_string().contains("alias already exists"));
}
