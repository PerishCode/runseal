use super::ast::{Item, Predicate, Program, Statement, Value};
use super::guards::{bash_required_tools, emit_bash_guards};

pub(crate) fn emit_seal(program: &Program) -> String {
    let mut out = String::new();
    for item in &program.items {
        match item {
            Item::Function { name, body } => {
                out.push_str(name);
                out.push_str("() {\n");
                emit_seal_statements(&mut out, body, 1);
                out.push_str("}\n");
            }
            Item::Statement { statement } => emit_seal_statement(&mut out, statement, 0),
        }
    }
    out
}

fn emit_seal_statements(out: &mut String, statements: &[Statement], indent: usize) {
    for statement in statements {
        emit_seal_statement(out, statement, indent);
    }
}

fn emit_seal_statement(out: &mut String, statement: &Statement, indent: usize) {
    let pad = "  ".repeat(indent);
    match statement {
        Statement::Assign { name, value } => {
            out.push_str(&format!("{pad}{name}={}\n", seal_value(value)));
        }
        Statement::ExecChecked { argv } => {
            out.push_str(&pad);
            out.push_str(&join_values(argv, seal_value));
            out.push('\n');
        }
        Statement::CaptureChecked { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("=$(");
            out.push_str(&join_values(argv, seal_value));
            out.push_str(")\n");
        }
        Statement::StringTrim { name, value } => {
            out.push_str(&format!(
                "{pad}{name}=$(seal string trim {})\n",
                seal_value(value)
            ));
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{pad}if {}; then\n", seal_predicate(predicate)));
            emit_seal_statements(out, then_body, indent + 1);
            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                emit_seal_statements(out, else_body, indent + 1);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Statement::Case { value, arms } => {
            out.push_str(&format!("{pad}case {} in\n", seal_value(value)));
            for arm in arms {
                out.push_str(&format!("{pad}  {})\n", arm.patterns.join("|")));
                emit_seal_statements(out, &arm.body, indent + 2);
                out.push_str(&format!("{pad}    ;;\n"));
            }
            out.push_str(&format!("{pad}esac\n"));
        }
        Statement::CallFunction { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            if !argv.is_empty() {
                out.push(' ');
                out.push_str(&join_values(argv, seal_value));
            }
            out.push('\n');
        }
        Statement::Print { value } => out.push_str(&format!("{pad}print {}\n", seal_value(value))),
        Statement::Error { value } => out.push_str(&format!("{pad}error {}\n", seal_value(value))),
        Statement::Fail { value } => out.push_str(&format!("{pad}fail {}\n", seal_value(value))),
        Statement::Exit { code } => out.push_str(&format!("{pad}exit {code}\n")),
        Statement::Sleep { seconds } => out.push_str(&format!("{pad}sleep {seconds}\n")),
    }
}

pub(crate) fn emit_bash(program: &Program, source_name: Option<&str>) -> String {
    let mut out = generated_header("bash", source_name);
    out.push_str("set -euo pipefail\n\n");
    out.push_str("seal_fail() {\n  printf '%s\\n' \"$1\" >&2\n  exit 1\n}\n\n");
    emit_bash_guards(&mut out, &bash_required_tools(program));
    emit_bash_items(&mut out, program);
    out
}

fn emit_bash_items(out: &mut String, program: &Program) {
    for item in &program.items {
        match item {
            Item::Function { name, body } => {
                out.push_str(name);
                out.push_str("() {\n");
                emit_bash_statements(out, body, 1);
                out.push_str("}\n\n");
            }
            Item::Statement { statement } => emit_bash_statement(out, statement, 0),
        }
    }
}

fn emit_bash_statements(out: &mut String, statements: &[Statement], indent: usize) {
    for statement in statements {
        emit_bash_statement(out, statement, indent);
    }
}

fn emit_bash_statement(out: &mut String, statement: &Statement, indent: usize) {
    let pad = "  ".repeat(indent);
    match statement {
        Statement::Assign { name, value } => {
            out.push_str(&format!("{pad}{name}={}\n", bash_value(value)));
        }
        Statement::ExecChecked { argv } => {
            out.push_str(&pad);
            out.push_str(&join_values(argv, bash_value));
            out.push('\n');
        }
        Statement::CaptureChecked { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("=$(");
            out.push_str(&join_values(argv, bash_value));
            out.push_str(")\n");
        }
        Statement::StringTrim { name, value } => {
            out.push_str(&format!(
                "{pad}{name}=$(printf '%s' {} | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')\n",
                bash_value(value)
            ));
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{pad}if {}; then\n", bash_predicate(predicate)));
            emit_bash_statements(out, then_body, indent + 1);
            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                emit_bash_statements(out, else_body, indent + 1);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Statement::Case { value, arms } => {
            out.push_str(&format!("{pad}case {} in\n", bash_value(value)));
            for arm in arms {
                out.push_str(&format!("{pad}  {})\n", arm.patterns.join("|")));
                emit_bash_statements(out, &arm.body, indent + 2);
                out.push_str(&format!("{pad}    ;;\n"));
            }
            out.push_str(&format!("{pad}esac\n"));
        }
        Statement::CallFunction { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            if !argv.is_empty() {
                out.push(' ');
                out.push_str(&join_values(argv, bash_value));
            }
            out.push('\n');
        }
        Statement::Print { value } => {
            out.push_str(&format!("{pad}printf '%s\\n' {}\n", bash_value(value)));
        }
        Statement::Error { value } => {
            out.push_str(&format!("{pad}printf '%s\\n' {} >&2\n", bash_value(value)));
        }
        Statement::Fail { value } => {
            out.push_str(&format!("{pad}seal_fail {}\n", bash_value(value)));
        }
        Statement::Exit { code } => out.push_str(&format!("{pad}exit {code}\n")),
        Statement::Sleep { seconds } => out.push_str(&format!("{pad}sleep {seconds}\n")),
    }
}

pub(crate) fn emit_powershell(program: &Program, source_name: Option<&str>) -> String {
    let mut out = generated_header("powershell", source_name);
    out.push_str("$ErrorActionPreference = 'Stop'\n\n");
    emit_powershell_items(&mut out, program);
    out
}

fn emit_powershell_items(out: &mut String, program: &Program) {
    for item in &program.items {
        match item {
            Item::Function { name, body } => {
                out.push_str(&format!("function {name} {{\n"));
                emit_powershell_statements(out, body, 1);
                out.push_str("}\n\n");
            }
            Item::Statement { statement } => emit_powershell_statement(out, statement, 0),
        }
    }
}

fn emit_powershell_statements(out: &mut String, statements: &[Statement], indent: usize) {
    for statement in statements {
        emit_powershell_statement(out, statement, indent);
    }
}

fn emit_powershell_statement(out: &mut String, statement: &Statement, indent: usize) {
    let pad = "    ".repeat(indent);
    match statement {
        Statement::Assign { name, value } => {
            out.push_str(&format!("{pad}${name} = {}\n", powershell_value(value)));
        }
        Statement::ExecChecked { argv } => {
            out.push_str(&pad);
            out.push_str("& ");
            out.push_str(&join_values(argv, powershell_value));
            out.push('\n');
        }
        Statement::CaptureChecked { name, argv } => {
            out.push_str(&pad);
            out.push_str(&format!("${name} = & "));
            out.push_str(&join_values(argv, powershell_value));
            out.push('\n');
        }
        Statement::StringTrim { name, value } => {
            out.push_str(&format!(
                "{pad}${name} = ({}).Trim()\n",
                powershell_value(value)
            ));
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => {
            out.push_str(&format!(
                "{pad}if ({}) {{\n",
                powershell_predicate(predicate)
            ));
            emit_powershell_statements(out, then_body, indent + 1);
            if else_body.is_empty() {
                out.push_str(&format!("{pad}}}\n"));
            } else {
                out.push_str(&format!("{pad}}} else {{\n"));
                emit_powershell_statements(out, else_body, indent + 1);
                out.push_str(&format!("{pad}}}\n"));
            }
        }
        Statement::Case { value, arms } => {
            out.push_str(&format!("{pad}switch ({}) {{\n", powershell_value(value)));
            for arm in arms {
                for pattern in &arm.patterns {
                    let pattern = if pattern == "*" {
                        "Default".to_string()
                    } else {
                        powershell_quote(pattern)
                    };
                    out.push_str(&format!("{pad}    {pattern} {{\n"));
                    emit_powershell_statements(out, &arm.body, indent + 2);
                    out.push_str(&format!("{pad}        break\n"));
                    out.push_str(&format!("{pad}    }}\n"));
                }
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Statement::CallFunction { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            if !argv.is_empty() {
                out.push(' ');
                out.push_str(&join_values(argv, powershell_value));
            }
            out.push('\n');
        }
        Statement::Print { value } => {
            out.push_str(&format!("{pad}Write-Output {}\n", powershell_value(value)));
        }
        Statement::Error { value } => {
            out.push_str(&format!(
                "{pad}[Console]::Error.WriteLine({})\n",
                powershell_value(value)
            ));
        }
        Statement::Fail { value } => {
            out.push_str(&format!("{pad}throw {}\n", powershell_value(value)));
        }
        Statement::Exit { code } => out.push_str(&format!("{pad}exit {code}\n")),
        Statement::Sleep { seconds } => {
            out.push_str(&format!("{pad}Start-Sleep -Seconds {seconds}\n"));
        }
    }
}

fn join_values(values: &[Value], format: fn(&Value) -> String) -> String {
    values.iter().map(format).collect::<Vec<_>>().join(" ")
}

fn generated_header(target: &str, source_name: Option<&str>) -> String {
    let source = source_name.unwrap_or("<memory>");
    format!("# Generated by runseal @transpile from {source} for {target}.\n")
}

fn seal_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Empty { value } => format!("empty {}", seal_value(value)),
        Predicate::NotEmpty { value } => format!("not_empty {}", seal_value(value)),
        Predicate::Eq { left, right } => format!("eq {} {}", seal_value(left), seal_value(right)),
        Predicate::Neq { left, right } => format!("neq {} {}", seal_value(left), seal_value(right)),
        Predicate::FileExists { path } => format!("file_exists {}", seal_value(path)),
        Predicate::DirExists { path } => format!("dir_exists {}", seal_value(path)),
        Predicate::ToolExists { name } => format!("tool_exists {name}"),
    }
}

fn bash_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Empty { value } => format!("[ -z {} ]", bash_value(value)),
        Predicate::NotEmpty { value } => format!("[ -n {} ]", bash_value(value)),
        Predicate::Eq { left, right } => {
            format!("[ {} = {} ]", bash_value(left), bash_value(right))
        }
        Predicate::Neq { left, right } => {
            format!("[ {} != {} ]", bash_value(left), bash_value(right))
        }
        Predicate::FileExists { path } => format!("[ -f {} ]", bash_value(path)),
        Predicate::DirExists { path } => format!("[ -d {} ]", bash_value(path)),
        Predicate::ToolExists { name } => format!("command -v {} >/dev/null 2>&1", sh_quote(name)),
    }
}

fn powershell_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Empty { value } => {
            format!("[string]::IsNullOrEmpty({})", powershell_value(value))
        }
        Predicate::NotEmpty { value } => {
            format!("![string]::IsNullOrEmpty({})", powershell_value(value))
        }
        Predicate::Eq { left, right } => {
            format!("{} -eq {}", powershell_value(left), powershell_value(right))
        }
        Predicate::Neq { left, right } => {
            format!("{} -ne {}", powershell_value(left), powershell_value(right))
        }
        Predicate::FileExists { path } => {
            format!(
                "Test-Path -LiteralPath {} -PathType Leaf",
                powershell_value(path)
            )
        }
        Predicate::DirExists { path } => {
            format!(
                "Test-Path -LiteralPath {} -PathType Container",
                powershell_value(path)
            )
        }
        Predicate::ToolExists { name } => {
            format!(
                "$null -ne (Get-Command {} -ErrorAction SilentlyContinue)",
                powershell_quote(name)
            )
        }
    }
}

fn seal_value(value: &Value) -> String {
    match value {
        Value::Literal { text } => sh_quote(text),
        Value::Var { name } => format!("${name}"),
        Value::Env { name } => format!("${{{name}}}"),
        Value::EnvDefault { name, default } => format!("${{{name}:-{default}}}"),
        Value::Concat { parts } => {
            let inner = parts
                .iter()
                .map(|part| match part {
                    Value::Literal { text } => text.clone(),
                    _ => seal_value(part),
                })
                .collect::<String>();
            double_quote(&inner)
        }
    }
}

fn bash_value(value: &Value) -> String {
    match value {
        Value::Literal { text } => sh_quote(text),
        Value::Var { name } => format!("\"${name}\""),
        Value::Env { name } => format!("\"${{{name}}}\""),
        Value::EnvDefault { name, default } => format!("\"${{{name}:-{default}}}\""),
        Value::Concat { parts } => double_quote(
            &parts
                .iter()
                .map(|part| match part {
                    Value::Literal { text } => text.clone(),
                    Value::Var { name } => format!("${name}"),
                    Value::Env { name } => format!("${{{name}}}"),
                    Value::EnvDefault { name, default } => format!("${{{name}:-{default}}}"),
                    Value::Concat { .. } => bash_value(part),
                })
                .collect::<String>(),
        ),
    }
}

fn powershell_value(value: &Value) -> String {
    match value {
        Value::Literal { text } => powershell_quote(text),
        Value::Var { name } => format!("${name}"),
        Value::Env { name } => format!("$env:{name}"),
        Value::EnvDefault { name, default } => {
            format!(
                "$(if ($env:{name}) {{ $env:{name} }} else {{ {} }})",
                powershell_quote(default)
            )
        }
        Value::Concat { parts } => {
            let value = parts
                .iter()
                .map(powershell_value)
                .collect::<Vec<_>>()
                .join(" + ");
            format!("({value})")
        }
    }
}

fn sh_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-' | b':')
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn double_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
}

fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
