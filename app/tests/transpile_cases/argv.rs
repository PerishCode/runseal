use std::process::Command;

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    dir: std::path::PathBuf,
    source: std::path::PathBuf,
}

fn fixture(source: &str) -> Fixture {
    let temp = TempDir::new().expect("temp dir should be created");
    let dir = temp.path().join("project-without-profile");
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    let source_path = dir.join("operator.seal");
    std::fs::write(&source_path, source).expect("source should be written");
    Fixture {
        _temp: temp,
        dir,
        source: source_path,
    }
}

fn run_transpile(fx: &Fixture, input_lang: &str, output_lang: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--input-lang")
        .arg(input_lang)
        .arg("--output-lang")
        .arg(output_lang)
        .arg(&fx.source)
        .output()
        .expect("runseal should run")
}

fn argv_source() -> &'static str {
    r#"
__seal_argc=$#
__seal_help=false
channel=stable
ref=main
body_file=
dry_run=false
no_merge=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --channel)
      if [ "$#" -lt 2 ]; then fail "missing value for --channel"; fi
      channel=$2
      shift 2
      ;;
    --channel=*)
      channel=${1#--channel=}
      shift
      ;;
    --ref)
      if [ "$#" -lt 2 ]; then fail "missing value for --ref"; fi
      ref=$2
      shift 2
      ;;
    --ref=*)
      ref=${1#--ref=}
      shift
      ;;
    --body-file)
      if [ "$#" -lt 2 ]; then fail "missing value for --body-file"; fi
      body_file=$2
      shift 2
      ;;
    --body-file=*)
      body_file=${1#--body-file=}
      shift
      ;;
    --dry-run)
      dry_run=true
      shift
      ;;
    --no-merge)
      no_merge=true
      shift
      ;;
    --)
      shift
      break
      ;;
    -h|--help|help)
      __seal_help=true
      shift
      ;;
    *) fail "unknown option: $1" ;;
  esac
done
if [ -z "$channel" ]; then
  fail "channel missing"
fi
print "$body_file"
"#
}

fn argv_positional_source() -> &'static str {
    r#"
__seal_argc=$#
__seal_help=false
body=
message=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --body)
      if [ "$#" -lt 2 ]; then fail "missing value for --body"; fi
      body=$2
      shift 2
      ;;
    --body=*)
      body=${1#--body=}
      shift
      ;;
    --)
      shift
      break
      ;;
    -h|--help|help)
      __seal_help=true
      shift
      ;;
    *)
      if [ -z "$message" ]; then
        message=$1
        shift
      else
        fail "unexpected argument: $1"
      fi
      ;;
  esac
done
print "$body"
print "$message"
"#
}

#[test]
fn argv_parse_roundtrip() {
    for input_lang in ["seal", "bash"] {
        let fx = fixture(argv_source());
        let output = run_transpile(&fx, input_lang, "sealir");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(stdout.contains("argv_parse"));
        assert!(stdout.contains("body_file"));
        assert!(stdout.contains("dry_run"));
    }
}

#[test]
fn argv_parse_emits_targets() {
    let fx = fixture(argv_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("body_file=${1#--body-file=}"));
    assert!(bash.contains("dry_run=true"));
    assert!(powershell.contains("$body_file = $__seal_arg.Substring(12)"));
    assert!(powershell.contains("$dry_run = 'false'"));
    assert!(powershell.contains("$dry_run = 'true'"));
    assert!(!powershell.contains("$dry_run = $false"));
    assert!(!powershell.contains("$dry_run = $true"));
    super::syntax::assert_bash(&bash);
    super::syntax::assert_pwsh(&powershell);
}

#[test]
fn argv_positional_roundtrip() {
    let fx = fixture(argv_positional_source());
    let output = run_transpile(&fx, "seal", "sealir");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"positional\""));
    assert!(stdout.contains("\"name\": \"message\""));
    assert!(stdout.contains("\"extra_error\": \"unexpected argument: $1\""));
}

#[test]
fn argv_positional_targets() {
    let fx = fixture(argv_positional_source());

    let bash = run_transpile(&fx, "seal", "bash");
    let powershell = run_transpile(&fx, "seal", "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("body=${1#--body=}"));
    assert!(bash.contains("if [ -z \"$message\" ]; then"));
    assert!(powershell.contains("$body = $__seal_arg.Substring(7)"));
    assert!(powershell.contains("if ([string]::IsNullOrEmpty($message)) {"));
    assert!(powershell.contains("$message = $__seal_arg"));
    super::syntax::assert_bash(&bash);
    super::syntax::assert_pwsh(&powershell);
}
