use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    thread,
};

use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

#[test]
fn gitee_pr() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");
    let token_file = cwd.join("gitee.env");
    std::fs::write(&token_file, "GITEE_TOKEN=file-token\n").expect("token file should be written");

    let (api_base, handle) = mock_gitee(
        |request| {
            assert!(request.starts_with("POST /repos/perishme/perish.top/pulls "));
            assert!(request.contains("authorization: token file-token"));
            assert!(request.contains("Land change"));
            assert!(request.contains("file-token"));
        },
        r#"{"number":42,"html_url":"https://gitee.test/pr/42"}"#,
    );
    let create = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_GITEE_API_BASE", api_base)
        .args([
            "@tool",
            "gitee",
            "pr",
            "create",
            "--owner",
            "perishme",
            "--repo",
            "perish.top",
            "--token-file",
            token_file.to_str().unwrap(),
            "--base",
            "main",
            "--head",
            "feat/seal",
            "--title",
            "Land change",
            "--body",
            "Body",
        ])
        .output()
        .expect("runseal should run");
    assert!(create.status.success());
    handle.join().expect("mock server should finish");
    let payload: serde_json::Value =
        serde_json::from_slice(&create.stdout).expect("stdout should be JSON");
    assert_eq!(payload["number"], 42);

    let (api_base, handle) = mock_gitee(
        |request| {
            assert!(request.starts_with("PUT /repos/perishme/perish.top/pulls/42/merge "));
            assert!(request.contains("authorization: token env-token"));
            assert!(request.contains("squash"));
            assert!(request.contains("env-token"));
        },
        r#"{"merged":true}"#,
    );
    let merge = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_GITEE_API_BASE", api_base)
        .env("GITEE_TOKEN", "env-token")
        .args([
            "@tool",
            "gitee",
            "pr",
            "merge",
            "--owner",
            "perishme",
            "--repo",
            "perish.top",
            "--number",
            "42",
            "--method",
            "squash",
        ])
        .output()
        .expect("runseal should run");
    assert!(merge.status.success());
    handle.join().expect("mock server should finish");
    let payload: serde_json::Value =
        serde_json::from_slice(&merge.stdout).expect("stdout should be JSON");
    assert_eq!(payload["merged"], true);

    let (api_base, handle) = mock_gitee(
        |request| {
            assert!(request.starts_with(
                "GET /repos/perishme/perish.top/pulls?head=feat%2Fseal&state=open&base=main "
            ));
            assert!(request.contains("authorization: token env-token"));
        },
        r#"[{"number":42,"head":{"ref":"feat/seal"},"base":{"ref":"main"},"html_url":"https://gitee.test/pr/42"}]"#,
    );
    let find = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_GITEE_API_BASE", api_base)
        .env("GITEE_TOKEN", "env-token")
        .args([
            "@tool",
            "gitee",
            "pr",
            "find",
            "--owner",
            "perishme",
            "--repo",
            "perish.top",
            "--head",
            "feat/seal",
            "--base",
            "main",
        ])
        .output()
        .expect("runseal should run");
    assert!(find.status.success());
    handle.join().expect("mock server should finish");
    let payload: serde_json::Value =
        serde_json::from_slice(&find.stdout).expect("stdout should be JSON");
    assert_eq!(payload["number"], 42);

    let (api_base, handle) = mock_gitee(
        |request| {
            assert!(request.starts_with(
                "GET /repos/perishme/perish.top/pulls?head=feat%2Fmissing&state=open "
            ));
            assert!(request.contains("authorization: token env-token"));
        },
        r#"[]"#,
    );
    let find_none = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_GITEE_API_BASE", api_base)
        .env("GITEE_TOKEN", "env-token")
        .args([
            "@tool",
            "gitee",
            "pr",
            "find",
            "--owner",
            "perishme",
            "--repo",
            "perish.top",
            "--head",
            "feat/missing",
        ])
        .output()
        .expect("runseal should run");
    assert!(find_none.status.success());
    handle.join().expect("mock server should finish");
    let payload: serde_json::Value =
        serde_json::from_slice(&find_none.stdout).expect("stdout should be JSON");
    assert!(payload.is_null());

    let (api_base, handle) = mock_gitee(
        |request| {
            assert!(
                request.starts_with(
                    "GET /repos/perishme/perish.top/pulls?head=feat%2Fdup&state=open "
                )
            );
            assert!(request.contains("authorization: token env-token"));
        },
        r#"[{"number":42,"head":{"ref":"feat/dup"},"base":{"ref":"main"}},{"number":43,"head":{"label":"perishme:feat/dup"},"base":{"label":"perishme:main"}}]"#,
    );
    let find_ambiguous = bin()
        .current_dir(&cwd)
        .env("RUNSEAL_HOME", temp.path().join("home"))
        .env("RUNSEAL_GITEE_API_BASE", api_base)
        .env("GITEE_TOKEN", "env-token")
        .args([
            "@tool",
            "gitee",
            "pr",
            "find",
            "--owner",
            "perishme",
            "--repo",
            "perish.top",
            "--head",
            "feat/dup",
        ])
        .output()
        .expect("runseal should run");
    assert!(!find_ambiguous.status.success());
    handle.join().expect("mock server should finish");
    assert!(
        String::from_utf8_lossy(&find_ambiguous.stderr)
            .contains("Gitee PR find is ambiguous for head `feat/dup`: found 2 matches")
    );
}

#[test]
fn gitee_origin() {
    let temp = TempDir::new().expect("temp dir should be created");
    let cwd = temp.path().join("empty");
    std::fs::create_dir_all(&cwd).expect("empty cwd should be created");

    for url in [
        "git@gitee.com:perishme/perish.top.git",
        "https://gitee.com/perishme/perish.top.git",
        "ssh://git@gitee.com/perishme/perish.top",
    ] {
        let output = bin()
            .current_dir(&cwd)
            .env("RUNSEAL_HOME", temp.path().join("home"))
            .args(["@tool", "gitee", "repo", "parse-origin", url])
            .output()
            .expect("runseal should run");
        assert!(output.status.success());
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
        assert_eq!(payload["owner"], "perishme");
        assert_eq!(payload["repo"], "perish.top");
    }
}

fn mock_gitee<F>(assert_request: F, body: &'static str) -> (String, thread::JoinHandle<()>)
where
    F: FnOnce(&str) + Send + 'static,
{
    let server = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let address = server
        .local_addr()
        .expect("mock server address should exist");
    let handle = thread::spawn(move || {
        let (mut stream, _) = server.accept().expect("mock request should arrive");
        let mut request = [0_u8; 4096];
        let read = stream
            .read(&mut request)
            .expect("request should be readable");
        let request = String::from_utf8_lossy(&request[..read]);
        assert_request(&request);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should be written");
    });
    (format!("http://{address}"), handle)
}
