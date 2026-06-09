use super::support::{generated_header, option_name};
use crate::core::transpile::ast::{
    ArgvKind, ArgvSpec, EnvAssign, Item, Predicate, Program, Statement, Value,
};

pub(crate) fn emit_powershell(program: &Program, source_name: Option<&str>) -> String {
    let mut out = generated_header("powershell", source_name);
    out.push_str("$ErrorActionPreference = 'Stop'\n\n");
    let top_level = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Statement { statement } => Some(statement),
            Item::Function { .. } => None,
        })
        .collect::<Vec<_>>();
    if emit_positional_bindings(
        &mut out,
        0,
        max_positional_statements(top_level.iter().copied()),
    ) {
        out.push('\n');
    }
    emit_items(&mut out, program);
    out
}

fn emit_items(out: &mut String, program: &Program) {
    let top_level_max = max_positional_statements(program.items.iter().filter_map(|item| {
        if let Item::Statement { statement } = item {
            Some(statement)
        } else {
            None
        }
    }));
    for item in &program.items {
        match item {
            Item::Function { name, body } => {
                out.push_str(&format!("function {name} {{\n"));
                emit_positional_bindings(out, 1, max_positional_statements(body.iter()));
                emit_statements(out, body, 1);
                out.push_str("}\n\n");
            }
            Item::Statement { statement } => emit_statement(out, statement, 0, top_level_max),
        }
    }
}

fn emit_statements(out: &mut String, statements: &[Statement], indent: usize) {
    let positional_max = max_positional_statements(statements.iter());
    for statement in statements {
        emit_statement(out, statement, indent, positional_max);
    }
}

fn emit_statement(out: &mut String, statement: &Statement, indent: usize, positional_max: usize) {
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
        Statement::EnvExecChecked { env, argv } => emit_env_exec(out, &pad, env, argv),
        Statement::Shift { count } => {
            if *count == 0 {
                out.push_str(&format!("{pad}$args = @($args)\n"));
            } else {
                out.push_str(&format!(
                    "{pad}$args = if ($args.Count -gt {count}) {{ @($args[{count}..($args.Count - 1)]) }} else {{ @() }}\n"
                ));
            }
            emit_positional_bindings(out, indent, positional_max);
        }
        Statement::ArgvParse { specs } => emit_argv_parse(out, specs, indent),
        Statement::CaptureChecked { name, argv } => {
            out.push_str(&pad);
            out.push_str(&format!("${name} = & "));
            out.push_str(&join_values(argv, powershell_value));
            out.push('\n');
        }
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => emit_if(out, &pad, predicate, then_body, else_body, indent),
        Statement::While { predicate, body } => {
            emit_while(out, &pad, predicate, body, indent);
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

fn emit_positional_bindings(out: &mut String, indent: usize, max: usize) -> bool {
    if max == 0 {
        return false;
    }
    let pad = "    ".repeat(indent);
    out.push_str(&format!("{pad}$0 = $args.Count\n"));
    for index in 1..=max {
        let offset = index - 1;
        out.push_str(&format!(
            "{pad}${index} = if ($args.Count -ge {index}) {{ $args[{offset}] }} else {{ '' }}\n"
        ));
    }
    true
}

fn emit_argv_parse(out: &mut String, specs: &[ArgvSpec], indent: usize) {
    let pad = "    ".repeat(indent);
    out.push_str(&format!("{pad}$__seal_argc = $args.Count\n"));
    out.push_str(&format!("{pad}$__seal_help = 'false'\n"));
    for spec in specs {
        let value = match spec.kind {
            ArgvKind::String => powershell_quote(spec.default.as_deref().unwrap_or("")),
            ArgvKind::Flag => "'false'".to_string(),
        };
        out.push_str(&format!("{pad}${} = {value}\n", spec.name));
    }
    out.push_str(&format!("{pad}$__seal_index = 0\n"));
    out.push_str(&format!("{pad}while ($__seal_index -lt $args.Count) {{\n"));
    out.push_str(&format!("{pad}    $__seal_arg = $args[$__seal_index]\n"));
    out.push_str(&format!("{pad}    switch -Regex ($__seal_arg) {{\n"));
    for spec in specs {
        match spec.kind {
            ArgvKind::String => emit_string_option(out, spec, indent),
            ArgvKind::Flag => emit_flag_option(out, spec, indent),
        }
    }
    out.push_str(&format!("{pad}        '^--$' {{\n"));
    out.push_str(&format!("{pad}            $__seal_index = $args.Count\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    out.push_str(&format!("{pad}        '^(-h|--help|help)$' {{\n"));
    out.push_str(&format!("{pad}            $__seal_help = 'true'\n"));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    out.push_str(&format!(
        "{pad}        default {{ throw \"unknown option: $__seal_arg\" }}\n"
    ));
    out.push_str(&format!("{pad}    }}\n"));
    out.push_str(&format!("{pad}}}\n"));
}

fn emit_string_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "    ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}        '^{}$' {{\n", regex_quote(&option)));
    out.push_str(&format!(
        "{pad}            if ($__seal_index + 1 -ge $args.Count) {{ throw 'missing value for {option}' }}\n"
    ));
    out.push_str(&format!(
        "{pad}            ${} = $args[$__seal_index + 1]\n",
        spec.name
    ));
    out.push_str(&format!("{pad}            $__seal_index += 2\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
    out.push_str(&format!("{pad}        '^{}=' {{\n", regex_quote(&option)));
    out.push_str(&format!(
        "{pad}            ${} = $__seal_arg.Substring({})\n",
        spec.name,
        option.len() + 1
    ));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
}

fn emit_flag_option(out: &mut String, spec: &ArgvSpec, indent: usize) {
    let pad = "    ".repeat(indent);
    let option = option_name(&spec.name);
    out.push_str(&format!("{pad}        '^{}$' {{\n", regex_quote(&option)));
    out.push_str(&format!("{pad}            ${} = 'true'\n", spec.name));
    out.push_str(&format!("{pad}            $__seal_index += 1\n"));
    out.push_str(&format!("{pad}            break\n"));
    out.push_str(&format!("{pad}        }}\n"));
}

fn regex_quote(value: &str) -> String {
    value.replace('-', "\\-")
}

fn emit_if(
    out: &mut String,
    pad: &str,
    predicate: &Predicate,
    then_body: &[Statement],
    else_body: &[Statement],
    indent: usize,
) {
    if let Predicate::Command { argv } = predicate {
        out.push_str(&format!("{pad}& "));
        out.push_str(&join_values(argv, powershell_value));
        out.push('\n');
        out.push_str(&format!("{pad}if ($LASTEXITCODE -eq 0) {{\n"));
        emit_statements(out, then_body, indent + 1);
        if else_body.is_empty() {
            out.push_str(&format!("{pad}}}\n"));
        } else {
            out.push_str(&format!("{pad}}} else {{\n"));
            emit_statements(out, else_body, indent + 1);
            out.push_str(&format!("{pad}}}\n"));
        }
        return;
    }
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

fn emit_while(
    out: &mut String,
    pad: &str,
    predicate: &Predicate,
    body: &[Statement],
    indent: usize,
) {
    if let Predicate::Command { argv } = predicate {
        out.push_str(&format!("{pad}while ($true) {{\n"));
        let inner = "    ".repeat(indent + 1);
        out.push_str(&format!("{inner}& "));
        out.push_str(&join_values(argv, powershell_value));
        out.push('\n');
        out.push_str(&format!(
            "{inner}if ($LASTEXITCODE -ne 0) {{\n{inner}    break\n{inner}}}\n"
        ));
        emit_statements(out, body, indent + 1);
        out.push_str(&format!("{pad}}}\n"));
        return;
    }
    out.push_str(&format!("{pad}while ({}) {{\n", predicate_text(predicate)));
    emit_statements(out, body, indent + 1);
    out.push_str(&format!("{pad}}}\n"));
}

fn predicate_text(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Command { argv } => {
            format!(
                "(& {}; $LASTEXITCODE) -eq 0",
                join_values(argv, powershell_value)
            )
        }
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
                "(& 'runseal' '@tool' 'json' 'empty' {}) -eq 'true'",
                powershell_value(value)
            )
        }
        Predicate::JsonNotEmpty { value } => {
            format!(
                "(& 'runseal' '@tool' 'json' 'empty' {}) -eq 'false'",
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
        Value::Argc => "$args.Count".to_string(),
        Value::Var { name } => format!("${name}"),
        Value::Args => "@args".to_string(),
        Value::Env { name } => format!("$env:{name}"),
        Value::EnvDefault { name, default } => {
            format!(
                "$(if ($env:{name}) {{ $env:{name} }} else {{ {} }})",
                powershell_quote(default)
            )
        }
        Value::Concat { parts } => {
            if parts.is_empty() {
                return "''".to_string();
            }
            let value = parts
                .iter()
                .map(powershell_value)
                .collect::<Vec<_>>()
                .join(" + ");
            format!("({value})")
        }
    }
}

fn max_positional_statements<'a>(statements: impl IntoIterator<Item = &'a Statement>) -> usize {
    statements
        .into_iter()
        .map(max_positional_statement)
        .max()
        .unwrap_or_default()
}

fn max_positional_statement(statement: &Statement) -> usize {
    match statement {
        Statement::Assign { value, .. } => max_positional_value(value),
        Statement::ExecChecked { argv }
        | Statement::EnvExecChecked { argv, .. }
        | Statement::CaptureChecked { argv, .. }
        | Statement::CallFunction { argv, .. } => argv
            .iter()
            .map(max_positional_value)
            .max()
            .unwrap_or_default(),
        Statement::If {
            predicate,
            then_body,
            else_body,
        } => max_positional_predicate(predicate)
            .max(max_positional_statements(then_body.iter()))
            .max(max_positional_statements(else_body.iter())),
        Statement::While { predicate, body } => {
            max_positional_predicate(predicate).max(max_positional_statements(body.iter()))
        }
        Statement::Case { value, arms } => arms
            .iter()
            .map(|arm| max_positional_statements(arm.body.iter()))
            .max()
            .unwrap_or_default()
            .max(max_positional_value(value)),
        Statement::Print { value } | Statement::Error { value } | Statement::Fail { value } => {
            max_positional_value(value)
        }
        Statement::ArgvParse { .. }
        | Statement::Shift { .. }
        | Statement::Exit { .. }
        | Statement::Break
        | Statement::Sleep { .. } => 0,
    }
}

fn emit_env_exec(out: &mut String, pad: &str, env: &[EnvAssign], argv: &[Value]) {
    out.push_str(&format!("{pad}& {{\n"));
    for item in env {
        out.push_str(&format!(
            "{pad}    $__seal_old_env_{} = $env:{}\n",
            item.name, item.name
        ));
    }
    out.push_str(&format!("{pad}    try {{\n"));
    for item in env {
        out.push_str(&format!(
            "{pad}        $env:{} = {}\n",
            item.name,
            powershell_value(&item.value)
        ));
    }
    out.push_str(&format!("{pad}        & "));
    out.push_str(&join_values(argv, powershell_value));
    out.push('\n');
    out.push_str(&format!("{pad}    }} finally {{\n"));
    for item in env {
        out.push_str(&format!(
            "{pad}        $env:{} = $__seal_old_env_{}\n",
            item.name, item.name
        ));
    }
    out.push_str(&format!("{pad}    }}\n"));
    out.push_str(&format!("{pad}}}\n"));
}

fn max_positional_predicate(predicate: &Predicate) -> usize {
    match predicate {
        Predicate::Command { argv } => argv
            .iter()
            .map(max_positional_value)
            .max()
            .unwrap_or_default(),
        Predicate::Empty { value }
        | Predicate::NotEmpty { value }
        | Predicate::JsonEmpty { value }
        | Predicate::JsonNotEmpty { value } => max_positional_value(value),
        Predicate::Eq { left, right }
        | Predicate::Neq { left, right }
        | Predicate::IntLt { left, right }
        | Predicate::IntLte { left, right }
        | Predicate::IntGt { left, right }
        | Predicate::IntGte { left, right } => {
            max_positional_value(left).max(max_positional_value(right))
        }
        Predicate::FileExists { path } | Predicate::DirExists { path } => {
            max_positional_value(path)
        }
    }
}

fn max_positional_value(value: &Value) -> usize {
    match value {
        Value::Var { name } => name.parse::<usize>().unwrap_or_default(),
        Value::Concat { parts } => parts
            .iter()
            .map(max_positional_value)
            .max()
            .unwrap_or_default(),
        Value::Literal { .. }
        | Value::Argc
        | Value::Args
        | Value::Env { .. }
        | Value::EnvDefault { .. } => 0,
    }
}

fn join_values(values: &[Value], format: fn(&Value) -> String) -> String {
    values.iter().map(format).collect::<Vec<_>>().join(" ")
}

fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
