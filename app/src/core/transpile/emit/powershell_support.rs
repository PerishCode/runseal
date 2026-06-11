use crate::core::transpile::ast::{ExpansionOp, Predicate, Statement, Value, ValueSource};

pub(super) fn emit_positional_bindings(out: &mut String, indent: usize, max: usize) -> bool {
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

pub(super) fn max_positional_statements<'a>(
    statements: impl IntoIterator<Item = &'a Statement>,
) -> usize {
    statements
        .into_iter()
        .map(max_positional_statement)
        .max()
        .unwrap_or_default()
}

fn max_positional_statement(statement: &Statement) -> usize {
    match statement {
        Statement::Assign { value, .. } => max_positional_value(value),
        Statement::ExecWrite { argv, path, .. } => argv
            .iter()
            .map(max_positional_value)
            .max()
            .unwrap_or_default()
            .max(max_positional_value(path)),
        Statement::ExecChecked { argv }
        | Statement::EnvExecChecked { argv, .. }
        | Statement::CaptureChecked { argv, .. }
        | Statement::CaptureFunction { argv, .. }
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
        Value::Expand { source, op } => max_positional_expand(source, op),
        Value::Concat { parts } => parts
            .iter()
            .map(max_positional_value)
            .max()
            .unwrap_or_default(),
        Value::Literal { .. } | Value::Argc | Value::Args => 0,
    }
}

fn max_positional_expand(source: &ValueSource, op: &ExpansionOp) -> usize {
    let source_max = match source {
        ValueSource::Var { name } => name.parse::<usize>().unwrap_or_default(),
        ValueSource::Env { .. } => 0,
    };
    match op {
        ExpansionOp::Plain
        | ExpansionOp::DefaultIfUnsetOrEmpty { .. }
        | ExpansionOp::RequireNonEmpty { .. } => source_max,
    }
}
