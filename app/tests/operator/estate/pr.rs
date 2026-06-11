use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use super::{fixture, log, run_wrapper, run_wrapper_env};

#[test]
fn api_flow() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("feat/seal", true, false);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--branch", "feat/seal", "--body", "Body", "Land change"],
        &[("RUNSEAL_GITEE_API_BASE", api_base)],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert!(String::from_utf8_lossy(&output.stdout).contains("https://gitee.test/pr/42"));
    assert_eq!(
        log(&fx),
        "git|checkout|-b|feat/seal\ngit|add|-A\ngit|commit|-m|Land change\ngit|push|-u|origin|feat/seal\ngit|checkout|main\ngit|pull|--ff-only|origin|main\ngit|push|origin|--delete|feat/seal\ngit|branch|-D|feat/seal\n"
    );
}

#[test]
fn default_branch() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("auto/land-change", true, false);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["Land Change"],
        &[("RUNSEAL_GITEE_API_BASE", api_base)],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert_eq!(
        log(&fx),
        "git|checkout|-b|auto/land-change\ngit|add|-A\ngit|commit|-m|Land Change\ngit|push|-u|origin|auto/land-change\ngit|checkout|main\ngit|pull|--ff-only|origin|main\ngit|push|origin|--delete|auto/land-change\ngit|branch|-D|auto/land-change\n"
    );
}

#[test]
fn dry_run() {
    let fx = fixture();

    let output = run_wrapper(
        &fx,
        "pr",
        &["--branch", "feat/seal", "--dry-run", "Land change"],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("branch: feat/seal"));
    assert!(stdout.contains("owner: perishme"));
    assert!(stdout.contains("repo: perish.top"));
    assert_eq!(log(&fx), "");
}

#[test]
fn no_merge() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("feat/no-merge", false, false);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--branch", "feat/no-merge", "--no-merge", "No merge"],
        &[("RUNSEAL_GITEE_API_BASE", api_base)],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert_eq!(
        log(&fx),
        "git|checkout|-b|feat/no-merge\ngit|add|-A\ngit|commit|-m|No merge\ngit|push|-u|origin|feat/no-merge\ngit|checkout|main\n"
    );
}

#[test]
fn reuse_existing_pr() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("feat/reuse", false, true);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--branch", "feat/reuse", "--no-merge", "Reuse existing"],
        &[("RUNSEAL_GITEE_API_BASE", api_base)],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert!(String::from_utf8_lossy(&output.stdout).contains("https://gitee.test/pr/42"));
    assert_eq!(
        log(&fx),
        "git|checkout|-b|feat/reuse\ngit|add|-A\ngit|commit|-m|Reuse existing\ngit|push|-u|origin|feat/reuse\ngit|checkout|main\n"
    );
}

#[test]
fn resume_local() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("feat/resume", true, true);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--resume", "Resume change"],
        &[
            ("RUNSEAL_GITEE_API_BASE", api_base),
            ("RUNSEAL_TEST_BRANCH", "feat/resume".to_string()),
            ("RUNSEAL_TEST_STATUS", "".to_string()),
        ],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert_eq!(
        log(&fx),
        "git|push|-u|origin|feat/resume\ngit|checkout|main\ngit|pull|--ff-only|origin|main\ngit|push|origin|--delete|feat/resume\ngit|branch|-D|feat/resume\n"
    );
}

#[test]
fn resume_remote() {
    let fx = fixture();
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should exist");
    std::fs::write(secrets.join("gitee.env"), "GITEE_TOKEN=test-token\n")
        .expect("token file should be written");
    let (api_base, handle) = mock_gitee_sequence("feat/remote", true, true);

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--resume", "--branch", "feat/remote", "Resume remote"],
        &[
            ("RUNSEAL_GITEE_API_BASE", api_base),
            ("RUNSEAL_TEST_BRANCH", "main".to_string()),
            ("RUNSEAL_TEST_CHECKOUT_FAIL", "feat/remote".to_string()),
            ("RUNSEAL_TEST_STATUS", "".to_string()),
        ],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    handle.join().expect("mock server should finish");
    assert_eq!(
        log(&fx),
        "git|checkout|feat/remote\ngit|fetch|origin|feat/remote\ngit|checkout|-B|feat/remote|origin/feat/remote\ngit|push|-u|origin|feat/remote\ngit|checkout|main\ngit|pull|--ff-only|origin|main\ngit|push|origin|--delete|feat/remote\ngit|branch|-D|feat/remote\n"
    );
}

#[test]
fn clean_start() {
    let fx = fixture();

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--branch", "feat/empty", "Empty change"],
        &[("RUNSEAL_TEST_STATUS", "".to_string())],
    );

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("pr: no local changes to land"));
    assert_eq!(log(&fx), "");
}

#[test]
fn resume_clean() {
    let fx = fixture();

    let output = run_wrapper_env(
        &fx,
        "pr",
        &["--resume", "Resume dirty"],
        &[
            ("RUNSEAL_TEST_BRANCH", "feat/resume".to_string()),
            ("RUNSEAL_TEST_STATUS", " M docs.md".to_string()),
        ],
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("pr: --resume requires a clean topic branch")
    );
    assert_eq!(log(&fx), "");
}

fn mock_gitee_sequence(
    expected_head: &'static str,
    merge: bool,
    existing_pr: bool,
) -> (String, thread::JoinHandle<()>) {
    let server = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let address = server
        .local_addr()
        .expect("mock server address should exist");
    let handle = thread::spawn(move || {
        let mut expected = vec![("GET", "/repos/perishme/perish.top/pulls")];
        if !existing_pr {
            expected.push(("POST", "/repos/perishme/perish.top/pulls"));
        }
        if merge {
            expected.push(("POST", "/repos/perishme/perish.top/pulls/42/review"));
            expected.push(("POST", "/repos/perishme/perish.top/pulls/42/test"));
            expected.push(("PUT", "/repos/perishme/perish.top/pulls/42/merge"));
        }
        for (index, expected) in expected.into_iter().enumerate() {
            let (mut stream, _) = server.accept().expect("mock request should arrive");
            let mut request = [0_u8; 4096];
            let read = stream
                .read(&mut request)
                .expect("request should be readable");
            let request = String::from_utf8_lossy(&request[..read]);
            if expected.0 == "GET" {
                assert!(
                    request.starts_with(&format!("{} {}?", expected.0, expected.1)),
                    "unexpected request: {request}"
                );
            } else {
                assert!(
                    request.starts_with(&format!("{} {} ", expected.0, expected.1)),
                    "unexpected request: {request}"
                );
            }
            assert!(request.contains("authorization: token test-token"));
            let body = if index == 0 {
                assert!(request.contains(&format!("head={}", expected_head.replace('/', "%2F"))));
                assert!(request.contains("state=open"));
                if request.contains("base=main") {
                } else {
                    panic!("expected base=main filter in request: {request}");
                }
                if existing_pr {
                    Box::leak(
                        format!(
                            r#"[{{"number":42,"head":{{"ref":"{expected_head}"}},"base":{{"ref":"main"}},"html_url":"https://gitee.test/pr/42"}}]"#
                        )
                        .into_boxed_str(),
                    )
                } else {
                    r#"[]"#
                }
            } else if !existing_pr && index == 1 {
                assert!(request.contains("test-token"));
                assert!(request.contains(expected_head));
                r#"{"number":42,"html_url":"https://gitee.test/pr/42"}"#
            } else {
                r#"{}"#
            };
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("response should be written");
        }
    });
    (format!("http://{address}"), handle)
}
