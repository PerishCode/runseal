#![cfg(unix)]

use std::{
    ffi::OsString,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    thread,
};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    project: PathBuf,
}

fn fixture() -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let project = temp.path().join("project");
    std::fs::create_dir_all(project.join(".runseal/wrappers"))
        .expect("wrapper dir should be created");
    std::fs::write(
        project.join("runseal.toml"),
        r#"
[resources]
root = ".local"

[[injections]]
type = "env"

[injections.vars]
RUNSEAL_REPO_LOCAL_DIR = "resource://"
RUNSEAL_REPO_SECRETS_DIR = "resource://secrets"
RUNSEAL_REPO_TMP_DIR = "resource://tmp"
"#,
    )
    .expect("profile should be written");
    std::fs::write(
        project.join(".runseal/wrappers/cloudflare.seal"),
        std::fs::read_to_string(repo_root().join(".runseal/wrappers/cloudflare.seal"))
            .expect("repo cloudflare seal should be readable"),
    )
    .expect("cloudflare seal should be copied");
    Fixture {
        _temp: temp,
        project,
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

fn run_cloudflare(fx: &Fixture, args: &[&str]) -> std::process::Output {
    run_cloudflare_with_env(fx, args, &[])
}

fn run_cloudflare_with_env(
    fx: &Fixture,
    args: &[&str],
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command
        .current_dir(&fx.project)
        .env("PATH", prepend_path())
        .arg("-p")
        .arg(fx.project.join("runseal.toml"))
        .arg(":cloudflare")
        .args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("cloudflare wrapper should run")
}

fn prepend_path() -> OsString {
    let mut paths = Vec::new();
    if let Some(runseal_dir) = Path::new(env!("CARGO_BIN_EXE_runseal")).parent() {
        paths.push(runseal_dir.to_path_buf());
    }
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(paths).expect("PATH should be joinable")
}

fn run_cloudflare_tool(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_runseal"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("cloudflare tool should run")
}

fn write_credentials(fx: &Fixture) {
    let secrets = fx.project.join(".local/secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should be created");
    std::fs::write(
        secrets.join("cloudflare.env"),
        "\
CLOUDFLARE_ACCOUNT_ID=account-123
CLOUDFLARE_API_TOKEN=token-456
CLOUDFLARE_ZONE_NAME=perish.uk
CLOUDFLARE_MANAGE_HOST=runseal.perish.uk
CLOUDFLARE_MANAGE_ORIGIN_HOST=releases.runseal.perish.uk
CLOUDFLARE_MANAGE_REDIRECT_PREFIX=
",
    )
    .expect("credentials should be written");
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn cloudflare_init_writes_template() {
    let fx = fixture();

    let output = run_cloudflare(&fx, &["init"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("created"));
    let token_file = fx.project.join(".local/secrets/cloudflare.env");
    let text = std::fs::read_to_string(token_file).expect("token template should exist");
    assert!(text.contains("CLOUDFLARE_ACCOUNT_ID="));
    assert!(text.contains("CLOUDFLARE_ZONE_NAME=perish.uk"));
}

#[test]
fn manage_plan_uses_tool() {
    let fx = fixture();
    write_credentials(&fx);

    let output = run_cloudflare(&fx, &["manage-plan"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("manage redirect plan"));
    assert!(stdout.contains("runseal_manage_sh_redirect"));
    assert!(stdout.contains("https://releases.runseal.perish.uk/manage.sh"));
    assert!(stdout.contains("runseal_manage_ps1_redirect"));
}

#[test]
fn zone_get_uses_tool() {
    let temp = TempDir::new().expect("temp dir should be created");
    let secrets = temp.path().join("secrets");
    std::fs::create_dir_all(&secrets).expect("secrets dir should be created");
    std::fs::write(
        secrets.join("cloudflare.env"),
        "\
CLOUDFLARE_ACCOUNT_ID=account-123
CLOUDFLARE_API_TOKEN=token-456
",
    )
    .expect("credentials should be written");
    let server = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let address = server
        .local_addr()
        .expect("mock server address should exist");
    let handle = thread::spawn(move || {
        let (mut stream, _) = server.accept().expect("mock request should arrive");
        let mut request = [0_u8; 2048];
        let read = stream
            .read(&mut request)
            .expect("request should be readable");
        let request = String::from_utf8_lossy(&request[..read]);
        assert!(request.starts_with("GET /zones?name=perish.uk "));
        assert!(request.contains("authorization: Bearer token-456"));
        let body =
            r#"{"success":true,"result":[{"id":"zone-123","name":"perish.uk","status":"active"}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should be written");
    });

    let output = run_cloudflare_tool(
        &["@tool", "cloudflare", "zone", "get", "--name", "perish.uk"],
        &[
            (
                "RUNSEAL_REPO_SECRETS_DIR",
                secrets.to_string_lossy().into_owned(),
            ),
            ("RUNSEAL_CLOUDFLARE_API_BASE", format!("http://{address}")),
        ],
    );

    handle.join().expect("mock server should finish");
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        r#"{"id":"zone-123","name":"perish.uk","status":"active"}"#.to_string() + "\n"
    );
}

#[test]
fn api_passthrough_uses_tool() {
    let fx = fixture();
    write_credentials(&fx);
    let server = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let address = server
        .local_addr()
        .expect("mock server address should exist");
    let handle = thread::spawn(move || {
        let (mut stream, _) = server.accept().expect("mock request should arrive");
        let mut request = [0_u8; 2048];
        let read = stream
            .read(&mut request)
            .expect("request should be readable");
        let request = String::from_utf8_lossy(&request[..read]);
        assert!(request.starts_with("GET /zones?name=perish.uk "));
        let body = r#"{"success":true,"result":[{"id":"zone-123"}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should be written");
    });

    let output = run_cloudflare_with_env(
        &fx,
        &["api", "GET", "/zones", "--query", "name=perish.uk"],
        &[("RUNSEAL_CLOUDFLARE_API_BASE", format!("http://{address}"))],
    );

    handle.join().expect("mock server should finish");
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains(r#""id":"zone-123""#));
}
