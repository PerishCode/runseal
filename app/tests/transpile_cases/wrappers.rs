use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

use super::syntax;

const WRAPPERS: [&str; 4] = [
    ".runseal/wrappers/cloudflare.seal",
    ".runseal/wrappers/init.seal",
    ".runseal/wrappers/pr.seal",
    ".runseal/wrappers/release.seal",
];

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_runseal"))
}

struct Fixture {
    _temp: TempDir,
    dir: PathBuf,
    source: PathBuf,
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

fn run_transpile(fx: &Fixture, output_lang: &str) -> std::process::Output {
    bin()
        .current_dir(&fx.dir)
        .arg("@transpile")
        .arg("--input-lang=seal")
        .arg("--output-lang")
        .arg(output_lang)
        .arg(&fx.source)
        .output()
        .expect("runseal should run")
}

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app dir should have repo parent")
        .to_path_buf()
}

#[test]
fn repo_wrapper_syntax() {
    let root = repo_root();
    for wrapper in WRAPPERS {
        let source = root.join(wrapper);

        let bash = bin()
            .current_dir(&root)
            .arg("@transpile")
            .arg("--input-lang=seal")
            .arg("--output-lang=bash")
            .arg(&source)
            .output()
            .expect("runseal should run");
        assert!(
            bash.status.success(),
            "{wrapper} bash stderr: {}",
            String::from_utf8_lossy(&bash.stderr)
        );
        let bash = String::from_utf8(bash.stdout).expect("bash output should be UTF-8");
        syntax::assert_bash(&bash);

        let powershell = bin()
            .current_dir(&root)
            .arg("@transpile")
            .arg("--input-lang=seal")
            .arg("--output-lang=powershell")
            .arg(&source)
            .output()
            .expect("runseal should run");
        assert!(
            powershell.status.success(),
            "{wrapper} powershell stderr: {}",
            String::from_utf8_lossy(&powershell.stderr)
        );
        let powershell =
            String::from_utf8(powershell.stdout).expect("powershell output should be UTF-8");
        syntax::assert_pwsh(&powershell);
    }
}

#[test]
fn wrappers_use_tool_cli() {
    for wrapper in WRAPPERS {
        let source = wrapper_source(wrapper);
        for namespace in [
            "cloudflare",
            "fs",
            "github",
            "int",
            "json",
            "process",
            "regex",
            "string",
        ] {
            assert!(
                !source.contains(&format!("seal {namespace}")),
                "{wrapper} should use `runseal @tool {namespace}`, not `seal {namespace}`"
            );
        }
    }
}

#[test]
fn wrappers_use_tests() {
    for wrapper in WRAPPERS {
        let source = wrapper_source(wrapper);
        for predicate in [
            "if empty ",
            "if not_empty ",
            "if eq ",
            "if neq ",
            "if file_exists ",
            "if dir_exists ",
            "if json_empty ",
            "if json_not_empty ",
            "while lt ",
        ] {
            assert!(
                !source.contains(predicate),
                "{wrapper} should use bash test predicates, not `{predicate}`"
            );
        }
    }
}

#[test]
fn wrappers_use_shift() {
    for wrapper in WRAPPERS {
        let source = wrapper_source(wrapper);
        assert!(
            !source.contains("seal passthrough"),
            "{wrapper} should use bash shift plus `\"$@\"`, not `seal passthrough`"
        );
    }
}

#[test]
fn wrappers_use_argv_blocks() {
    for wrapper in WRAPPERS {
        let source = wrapper_source(wrapper);
        assert!(
            !source.contains("seal argv parse"),
            "{wrapper} should use a bash while/case argv parser block, not `seal argv parse`"
        );
    }
}

#[test]
fn wrappers_use_check_tool() {
    for wrapper in WRAPPERS {
        let source = wrapper_source(wrapper);
        assert!(
            !source.contains("seal capture optional"),
            "{wrapper} should use focused `runseal @tool` glue, not `seal capture optional`"
        );
    }
}

#[test]
fn shift_args_targets() {
    let fx = fixture(
        r#"
shift
runseal @tool cloudflare api request "$@"
"#,
    );

    let bash = run_transpile(&fx, "bash");
    let powershell = run_transpile(&fx, "powershell");

    assert!(bash.status.success());
    assert!(powershell.status.success());
    let bash = String::from_utf8(bash.stdout).expect("stdout should be UTF-8");
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(bash.contains("shift"));
    assert!(bash.contains("runseal @tool cloudflare api request \"$@\""));
    assert!(powershell.contains("$args = if ($args.Count -gt 1)"));
    assert!(powershell.contains("& 'runseal' '@tool' 'cloudflare' 'api' 'request' @args"));
    syntax::assert_bash(&bash);
    syntax::assert_pwsh(&powershell);
}

#[test]
fn powershell_binds_positionals() {
    let fx = fixture(
        r#"
echo_first() {
  print "$1"
}

echo_first "$2"
"#,
    );

    let powershell = run_transpile(&fx, "powershell");

    assert!(powershell.status.success());
    let powershell = String::from_utf8(powershell.stdout).expect("stdout should be UTF-8");
    assert!(powershell.contains("$1 = if ($args.Count -ge 1) { $args[0] } else { '' }"));
    assert!(powershell.contains("$2 = if ($args.Count -ge 2) { $args[1] } else { '' }"));
    assert!(powershell.contains("function echo_first {\n    $0 = $args.Count\n    $1 = if"));
    assert!(!powershell.contains("$3 = if ($args.Count -ge 3)"));
    syntax::assert_pwsh(&powershell);
}

fn wrapper_source(wrapper: &str) -> String {
    std::fs::read_to_string(repo_root().join(wrapper)).expect("wrapper should be readable")
}
