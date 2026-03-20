use super::*;
use tempfile::TempDir;

#[test]
fn register_fails_when_target_exists_in_error_mode() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source.md");
    let target = temp.path().join("AGENTS.md");
    std::fs::write(&source, "content").expect("source file should be created");
    std::fs::write(&target, "existing").expect("target file should be created");

    let injection = SymlinkInjection::new(SymlinkProfile {
        enabled: true,
        source: source.clone(),
        target: target.clone(),
        on_exist: SymlinkOnExist::Error,
        cleanup: true,
    });

    let err = injection
        .register_at(&source, &target, SymlinkOnExist::Error)
        .expect_err("existing target should fail");
    assert!(
        err.to_string()
            .contains("refusing to overwrite existing file")
    );
}

#[test]
fn register_and_shutdown_manage_symlink() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source.md");
    let target = temp.path().join(".codex/AGENTS.md");
    std::fs::write(&source, "content").expect("source file should be created");

    let mut injection = SymlinkInjection::new(SymlinkProfile {
        enabled: true,
        source: source.clone(),
        target: target.clone(),
        on_exist: SymlinkOnExist::Error,
        cleanup: true,
    });

    injection
        .register()
        .expect("register should create symlink");

    let metadata = std::fs::symlink_metadata(&target).expect("symlink should exist");
    assert!(metadata.file_type().is_symlink());

    injection
        .shutdown()
        .expect("shutdown should remove symlink");
    assert!(
        std::fs::symlink_metadata(&target).is_err(),
        "symlink should be removed"
    );
}

#[test]
fn replace_mode_replaces_existing_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source.md");
    let target = temp.path().join("AGENTS.md");
    std::fs::write(&source, "content").expect("source file should be created");
    std::fs::write(&target, "existing").expect("target file should be created");

    let mut injection = SymlinkInjection::new(SymlinkProfile {
        enabled: true,
        source: source.clone(),
        target: target.clone(),
        on_exist: SymlinkOnExist::Replace,
        cleanup: true,
    });

    injection.register().expect("replace mode should succeed");
    let metadata = std::fs::symlink_metadata(&target).expect("target should exist");
    assert!(metadata.file_type().is_symlink());
}
