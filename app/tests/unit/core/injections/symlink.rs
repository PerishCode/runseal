use super::*;
use tempfile::TempDir;

#[test]
fn creates_and_cleans_symlink() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    std::fs::write(&source, "x").expect("source should be written");

    let mut injection = SymlinkInjection::new(SymlinkProfile {
        enabled: true,
        source: source.clone(),
        target: target.clone(),
        on_exist: SymlinkOnExist::Error,
        cleanup: true,
    });

    injection.validate().expect("validation should pass");
    injection.register().expect("register should pass");
    assert!(target.exists());
    injection.shutdown().expect("shutdown should pass");
    assert!(!target.exists());
}
