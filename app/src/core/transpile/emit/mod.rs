use super::ast::{ArgvKind, ArgvSpec, Item, OutputStream, Program, Statement};
use super::guards::{bash_required_tools, emit_bash_guards};

mod powershell;
mod powershell_support;
mod support;

pub(crate) use powershell::emit_powershell;
use support::{
    bash_predicate, bash_value, generated_header, join_values, option_name, seal_value, sh_quote,
};

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
        Statement::ExecWrite {
            stream,
            path,
            append,
            argv,
        } => {
            out.push_str(&pad);
            out.push_str(&join_values(argv, seal_value));
            out.push(' ');
            out.push_str(match (stream, append) {
                (OutputStream::Stdout, false) => ">",
                (OutputStream::Stdout, true) => ">>",
                (OutputStream::Stderr, false) => "2>",
                (OutputStream::Stderr, true) => "2>>",
            });
            out.push(' ');
            out.push_str(&seal_value(path));
            out.push('\n');
        }
        Statement::EnvExecChecked { env, argv } => {
            out.push_str(&pad);
            for item in env {
                out.push_str(&item.name);
                out.push('=');
                out.push_str(&seal_value(&item.value));
                out.push(' ');
            }
            out.push_str(&join_values(argv, seal_value));
            out.push('\n');
        }
        Statement::Shift { count } => {
            out.push_str(&pad);
            out.push_str("shift");
            if *count != 1 {
                out.push(' ');
                out.push_str(&count.to_string());
            }
            out.push('\n');
        }
        Statement::ArgvParse { specs } => {
            emit_seal_argv_parse(out, specs, indent);
        }
        Statement::CaptureChecked { name, argv } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("=$(");
            out.push_str(&join_values(argv, seal_value));
            out.push_str(")\n");
        }
        Statement::CaptureFunction {
            name,
            function,
            argv,
        } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("=$(");
            out.push_str(function);
            if !argv.is_empty() {
                out.push(' ');
                out.push_str(&join_values(argv, seal_value));
            }
            out.push_str(")\n");
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{pad}if {}; then\n", bash_predicate(predicate)));
            emit_seal_statements(out, then_body, indent + 1);
            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                emit_seal_statements(out, else_body, indent + 1);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Statement::While { predicate, body } => {
            out.push_str(&format!("{pad}while {}; do\n", bash_predicate(predicate)));
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

fn emit_seal_argv_parse(out: &mut String, specs: &[ArgvSpec], indent: usize) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{pad}__seal_argc=$#\n"));
    out.push_str(&format!("{pad}__seal_help=false\n"));
    for spec in specs {
        let value = match spec.kind {
            ArgvKind::String => spec.default.as_deref().unwrap_or(""),
            ArgvKind::Flag => "false",
        };
        out.push_str(&format!("{pad}{}={value}\n", spec.name));
    }
    out.push_str(&format!("{pad}while [ \"$#\" -gt 0 ]; do\n"));
    out.push_str(&format!("{pad}  case \"$1\" in\n"));
    for spec in specs {
        match spec.kind {
            ArgvKind::String => emit_seal_string_option(out, spec, indent),
            ArgvKind::Flag => emit_seal_flag_option(out, spec, indent),
        }
    }
    out.push_str(&format!("{pad}    --)\n"));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      break\n"));
    out.push_str(&format!("{pad}      ;;\n"));
    out.push_str(&format!("{pad}    -h|--help|help)\n"));
    out.push_str(&format!("{pad}      __seal_help=true\n"));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      ;;\n"));
    out.push_str(&format!("{pad}    *) fail \"unknown option: $1\" ;;\n"));
    out.push_str(&format!("{pad}  esac\n"));
    out.push_str(&format!("{pad}done\n"));
}

fn emit_seal_string_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "  ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}    {option})\n"));
    out.push_str(&format!(
        "{pad}      if [ \"$#\" -lt 2 ]; then fail 'missing value for {option}'; fi\n"
    ));
    out.push_str(&format!("{pad}      {}=$2\n", spec.name));
    out.push_str(&format!("{pad}      shift 2\n"));
    out.push_str(&format!("{pad}      ;;\n"));
    out.push_str(&format!("{pad}    {option}=*)\n"));
    out.push_str(&format!("{pad}      {}=${{1#{option}=}}\n", spec.name));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      ;;\n"));
}

fn emit_seal_flag_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "  ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}    {option})\n"));
    out.push_str(&format!("{pad}      {}=true\n", spec.name));
    out.push_str(&format!("{pad}      shift\n"));
    out.push_str(&format!("{pad}      ;;\n"));
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
                emit_bash_body(out, body, 1);
                out.push_str("}\n\n");
            }
            Item::Statement { statement } => emit_bash_statement(out, statement, 0),
        }
    }
}

fn emit_bash_body(out: &mut String, statements: &[Statement], indent: usize) {
    if statements.is_empty() {
        let pad = "  ".repeat(indent);
        out.push_str(&format!("{pad}:\n"));
    } else {
        emit_bash_statements(out, statements, indent);
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
        Statement::ExecWrite {
            stream,
            path,
            append,
            argv,
        } => {
            out.push_str(&pad);
            out.push_str(&join_values(argv, bash_value));
            out.push(' ');
            out.push_str(match (stream, append) {
                (OutputStream::Stdout, false) => ">",
                (OutputStream::Stdout, true) => ">>",
                (OutputStream::Stderr, false) => "2>",
                (OutputStream::Stderr, true) => "2>>",
            });
            out.push(' ');
            out.push_str(&bash_value(path));
            out.push('\n');
        }
        Statement::ExecChecked { argv } => {
            out.push_str(&pad);
            out.push_str(&join_values(argv, bash_value));
            out.push('\n');
        }
        Statement::EnvExecChecked { env, argv } => {
            out.push_str(&pad);
            for item in env {
                out.push_str(&item.name);
                out.push('=');
                out.push_str(&bash_value(&item.value));
                out.push(' ');
            }
            out.push_str(&join_values(argv, bash_value));
            out.push('\n');
        }
        Statement::Shift { count } => {
            out.push_str(&pad);
            out.push_str("shift");
            if *count != 1 {
                out.push(' ');
                out.push_str(&count.to_string());
            }
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
        Statement::CaptureFunction {
            name,
            function,
            argv,
        } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("=$(");
            out.push_str(function);
            if !argv.is_empty() {
                out.push(' ');
                out.push_str(&join_values(argv, bash_value));
            }
            out.push_str(")\n");
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{pad}if {}; then\n", bash_predicate(predicate)));
            emit_bash_body(out, then_body, indent + 1);
            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                emit_bash_body(out, else_body, indent + 1);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Statement::While { predicate, body } => {
            out.push_str(&format!("{pad}while {}; do\n", bash_predicate(predicate)));
            emit_bash_body(out, body, indent + 1);
            out.push_str(&format!("{pad}done\n"));
        }
        Statement::Case { value, arms } => {
            out.push_str(&format!("{pad}case {} in\n", bash_value(value)));
            for arm in arms {
                out.push_str(&format!("{pad}  {})\n", arm.patterns.join("|")));
                emit_bash_body(out, &arm.body, indent + 2);
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
    out.push_str(&format!("{pad}__seal_argc=$#\n"));
    out.push_str(&format!("{pad}__seal_help=false\n"));
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
    out.push_str(&format!("{pad}    -h|--help|help)\n"));
    out.push_str(&format!("{pad}      __seal_help=true\n"));
    out.push_str(&format!("{pad}      shift\n"));
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
