use super::ast::{ArgvKind, ArgvSpec, Item, Predicate, Program, Statement, Value};
use super::guards::{bash_required_tools, emit_bash_guards};
use super::json_path::json_path;

mod powershell;

pub(crate) use powershell::emit_powershell;

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
        Statement::ArgvParse { specs } => {
            out.push_str(&pad);
            out.push_str("seal argv parse");
            for spec in specs {
                out.push(' ');
                out.push_str(match spec.kind {
                    ArgvKind::String => "--string",
                    ArgvKind::Flag => "--flag",
                });
                out.push(' ');
                out.push_str(&argv_spec_name(spec));
            }
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
        Statement::JsonGet { name, json, path } => {
            out.push_str(&format!(
                "{pad}{name}=$(seal json get {} {})\n",
                seal_value(json),
                sh_quote(&json_path(path))
            ));
        }
        Statement::IntAdd { name, left, right } => {
            out.push_str(&format!(
                "{pad}{name}=$(seal int add {} {})\n",
                seal_value(left),
                seal_value(right)
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
        Statement::While { predicate, body } => {
            out.push_str(&format!("{pad}while {}; do\n", seal_predicate(predicate)));
            emit_seal_statements(out, body, indent + 1);
            out.push_str(&format!("{pad}done\n"));
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
        Statement::Break => out.push_str(&format!("{pad}break\n")),
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
        Statement::ArgvParse { specs } => emit_bash_argv_parse(out, specs, indent),
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
        Statement::JsonGet { name, json, path } => {
            out.push_str(&format!(
                "{pad}{name}=$(printf '%s' {} | jq -r {})\n",
                bash_value(json),
                sh_quote(&json_path(path))
            ));
        }
        Statement::IntAdd { name, left, right } => {
            out.push_str(&format!(
                "{pad}{name}=$(({} + {}))\n",
                bash_int_value(left),
                bash_int_value(right)
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
        Statement::While { predicate, body } => {
            out.push_str(&format!("{pad}while {}; do\n", bash_predicate(predicate)));
            emit_bash_statements(out, body, indent + 1);
            out.push_str(&format!("{pad}done\n"));
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
        Statement::Break => out.push_str(&format!("{pad}break\n")),
        Statement::Sleep { seconds } => out.push_str(&format!("{pad}sleep {seconds}\n")),
    }
}

fn emit_bash_argv_parse(out: &mut String, specs: &[ArgvSpec], indent: usize) {
    let pad = "  ".repeat(indent);
    for spec in specs {
        let value = match spec.kind {
            ArgvKind::String => sh_quote(spec.default.as_deref().unwrap_or("")),
            ArgvKind::Flag => "false".to_string(),
        };
        out.push_str(&format!("{pad}{}={value}\n", spec.name));
    }
    out.push_str(&format!("{pad}while [ \"$#\" -gt 0 ]; do\n"));
    out.push_str(&format!("{pad}  case \"$1\" in\n"));
    for spec in specs {
        match spec.kind {
            ArgvKind::String => emit_bash_string_option(out, spec, indent),
            ArgvKind::Flag => emit_bash_flag_option(out, spec, indent),
        }
    }
    out.push_str(&format!("{pad}    --)\n"));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      break\n"));
    out.push_str(&format!("{pad}      ;;\n"));
    out.push_str(&format!(
        "{pad}    *) seal_fail \"unknown option: $1\" ;;\n"
    ));
    out.push_str(&format!("{pad}  esac\n"));
    out.push_str(&format!("{pad}done\n"));
}

fn emit_bash_string_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "  ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}    {option})\n"));
    out.push_str(&format!(
        "{pad}      if [ \"$#\" -lt 2 ]; then seal_fail 'missing value for {option}'; fi\n"
    ));
    out.push_str(&format!("{pad}      {}=$2\n", spec.name));
    out.push_str(&format!("{pad}      shift 2\n"));
    out.push_str(&format!("{pad}      ;;\n"));
    out.push_str(&format!("{pad}    {option}=*)\n"));
    out.push_str(&format!("{pad}      {}=${{1#{option}=}}\n", spec.name));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      ;;\n"));
}

fn emit_bash_flag_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "  ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}    {option})\n"));
    out.push_str(&format!("{pad}      {}=true\n", spec.name));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      ;;\n"));
}

fn argv_spec_name(spec: &ArgvSpec) -> String {
    match &spec.default {
        Some(default) => format!("{}={default}", spec.name),
        None => spec.name.clone(),
    }
}

pub(super) fn option_name(name: &str) -> String {
    format!("--{}", name.replace('_', "-"))
}

fn join_values(values: &[Value], format: fn(&Value) -> String) -> String {
    values.iter().map(format).collect::<Vec<_>>().join(" ")
}

pub(super) fn generated_header(target: &str, source_name: Option<&str>) -> String {
    let source = source_name.unwrap_or("<memory>");
    format!("# Generated by runseal @transpile from {source} for {target}.\n")
}

fn seal_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Empty { value } => format!("empty {}", seal_value(value)),
        Predicate::NotEmpty { value } => format!("not_empty {}", seal_value(value)),
        Predicate::Eq { left, right } => format!("eq {} {}", seal_value(left), seal_value(right)),
        Predicate::Neq { left, right } => format!("neq {} {}", seal_value(left), seal_value(right)),
        Predicate::IntLt { left, right } => {
            format!("lt {} {}", seal_value(left), seal_value(right))
        }
        Predicate::IntLte { left, right } => {
            format!("lte {} {}", seal_value(left), seal_value(right))
        }
        Predicate::IntGt { left, right } => {
            format!("gt {} {}", seal_value(left), seal_value(right))
        }
        Predicate::IntGte { left, right } => {
            format!("gte {} {}", seal_value(left), seal_value(right))
        }
        Predicate::JsonEmpty { value } => format!("json_empty {}", seal_value(value)),
        Predicate::JsonNotEmpty { value } => format!("json_not_empty {}", seal_value(value)),
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
        Predicate::IntLt { left, right } => {
            format!("[ {} -lt {} ]", bash_int_value(left), bash_int_value(right))
        }
        Predicate::IntLte { left, right } => {
            format!("[ {} -le {} ]", bash_int_value(left), bash_int_value(right))
        }
        Predicate::IntGt { left, right } => {
            format!("[ {} -gt {} ]", bash_int_value(left), bash_int_value(right))
        }
        Predicate::IntGte { left, right } => {
            format!("[ {} -ge {} ]", bash_int_value(left), bash_int_value(right))
        }
        Predicate::JsonEmpty { value } => {
            format!(
                "[ \"$(printf '%s' {} | jq 'length')\" -eq 0 ]",
                bash_value(value)
            )
        }
        Predicate::JsonNotEmpty { value } => {
            format!(
                "[ \"$(printf '%s' {} | jq 'length')\" -gt 0 ]",
                bash_value(value)
            )
        }
        Predicate::FileExists { path } => format!("[ -f {} ]", bash_value(path)),
        Predicate::DirExists { path } => format!("[ -d {} ]", bash_value(path)),
        Predicate::ToolExists { name } => format!("command -v {} >/dev/null 2>&1", sh_quote(name)),
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

fn bash_int_value(value: &Value) -> String {
    match value {
        Value::Var { name } => format!("${name}"),
        Value::Env { name } => format!("${{{name}}}"),
        Value::EnvDefault { name, default } => format!("${{{name}:-{default}}}"),
        Value::Literal { text } => sh_quote(text),
        Value::Concat { .. } => bash_value(value),
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
