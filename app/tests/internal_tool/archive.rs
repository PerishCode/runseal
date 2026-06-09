use std::process::Command;

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn archive_roundtrip() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let local = cwd.join(".local");
    let ssh = local.join("ssh");
    let kube = local.join("kube");
    std::fs::create_dir_all(&ssh).expect("ssh dir should exist");
    std::fs::create_dir_all(&kube).expect("kube dir should exist");
    std::fs::write(ssh.join("config"), "Host test\n").expect("ssh config should be written");
    std::fs::write(kube.join("test.yaml"), "kube").expect("kube config should be written");
    let archive = cwd.join("local.enc");

    let export = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_TEST_PASSWORD", "secret")
        .args([
            "@tool",
            "archive",
            "local",
            "export",
            "--source",
            local.to_str().unwrap(),
            "--archive",
            archive.to_str().unwrap(),
            "--password-env",
            "RUNSEAL_TEST_PASSWORD",
        ])
        .output()
        .expect("runseal should run");
    assert!(
        export.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(archive.is_file());

    let overwrite = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_TEST_PASSWORD", "secret")
        .args([
            "@tool",
            "archive",
            "local",
            "export",
            "--source",
            local.to_str().unwrap(),
            "--archive",
            archive.to_str().unwrap(),
            "--password-env",
            "RUNSEAL_TEST_PASSWORD",
        ])
        .output()
        .expect("runseal should run");
    assert!(!overwrite.status.success());

    std::fs::remove_dir_all(&local).expect("local dir should be removable");
    let import = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_TEST_PASSWORD", "secret")
        .args([
            "@tool",
            "archive",
            "local",
            "import",
            "--source",
            local.to_str().unwrap(),
            "--archive",
            archive.to_str().unwrap(),
            "--password-env",
            "RUNSEAL_TEST_PASSWORD",
            "--force",
        ])
        .output()
        .expect("runseal should run");
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(ssh.join("config")).expect("ssh config should be restored"),
        "Host test\n"
    );
    assert_eq!(
        std::fs::read_to_string(kube.join("test.yaml")).expect("kube config should be restored"),
        "kube"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&local)
                .expect("local metadata should exist")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(ssh.join("config"))
                .expect("ssh config metadata should exist")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
