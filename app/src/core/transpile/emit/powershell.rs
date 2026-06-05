use super::generated_header;
use crate::core::transpile::ast::{Item, Predicate, Program, Statement, Value};
use crate::core::transpile::json_path::powershell_json_get;

pub(crate) fn emit_powershell(program: &Program, source_name: Option<&str>) -> String {
    let mut out = generated_header("powershell", source_name);
    out.push_str("$ErrorActionPreference = 'Stop'\n\n");
    emit_items(&mut out, program);
    out
}

fn emit_items(out: &mut String, program: &Program) {
    for item in &program.items {
        match item {
            Item::Function { name, body } => {
                out.push_str(&format!("function {name} {{\n"));
                emit_statements(out, body, 1);
                out.push_str("}\n\n");
            }
            Item::Statement { statement } => emit_statement(out, statement, 0),
        }
    }
}

fn emit_statements(out: &mut String, statements: &[Statement], indent: usize) {
    for statement in statements {
        emit_statement(out, statement, indent);
    }
}

fn emit_statement(out: &mut String, statement: &Statement, indent: usize) {
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
        Statement::JsonGet { name, json, path } => {
            out.push_str(&format!(
                "{pad}${name} = [string]({})\n",
                powershell_json_get(&powershell_value(json), path)
            ));
        }
        Statement::IntAdd { name, left, right } => {
            out.push_str(&format!(
                "{pad}${name} = [int]{} + {}\n",
                powershell_value(left),
                powershell_value(right)
            ));
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => emit_if(out, &pad, predicate, then_body, else_body, indent),
        Statement::While { predicate, body } => {
            out.push_str(&format!("{pad}while ({}) {{\n", predicate_text(predicate)));
            emit_statements(out, body, indent + 1);
            out.push_str(&format!("{pad}}}\n"));
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
                    emit_statements(out, &arm.body, indent + 2);
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
        Statement::Break => out.push_str(&format!("{pad}break\n")),
        Statement::Sleep { seconds } => {
            out.push_str(&format!("{pad}Start-Sleep -Seconds {seconds}\n"));
        }
    }
}

fn emit_if(
    out: &mut String,
    pad: &str,
    predicate: &Predicate,
    then_body: &[Statement],
    else_body: &[Statement],
    indent: usize,
) {
    out.push_str(&format!("{pad}if ({}) {{\n", predicate_text(predicate)));
    emit_statements(out, then_body, indent + 1);
    if else_body.is_empty() {
        out.push_str(&format!("{pad}}}\n"));
    } else {
        out.push_str(&format!("{pad}}} else {{\n"));
        emit_statements(out, else_body, indent + 1);
        out.push_str(&format!("{pad}}}\n"));
    }
}

fn predicate_text(predicate: &Predicate) -> String {
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
        Predicate::IntLt { left, right } => int_compare(left, "-lt", right),
        Predicate::IntLte { left, right } => int_compare(left, "-le", right),
        Predicate::IntGt { left, right } => int_compare(left, "-gt", right),
        Predicate::IntGte { left, right } => int_compare(left, "-ge", right),
        Predicate::JsonEmpty { value } => {
            format!(
                "(({} | ConvertFrom-Json).Count -eq 0)",
                powershell_value(value)
            )
        }
        Predicate::JsonNotEmpty { value } => {
            format!(
                "(({} | ConvertFrom-Json).Count -gt 0)",
                powershell_value(value)
            )
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

fn int_compare(left: &Value, operator: &str, right: &Value) -> String {
    format!(
        "[int]{} {operator} {}",
        powershell_value(left),
        powershell_value(right)
    )
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

fn join_values(values: &[Value], format: fn(&Value) -> String) -> String {
    values.iter().map(format).collect::<Vec<_>>().join(" ")
}

fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
