use std::collections::BTreeSet;

use super::ast::{Item, Program, Statement, Value};

pub(crate) fn lower_functions(mut program: Program) -> Program {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function { name, .. } => Some(name.clone()),
            Item::Statement { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    for item in &mut program.items {
        match item {
            Item::Function { body, .. } => lower_statements(body, &functions),
            Item::Statement { statement } => lower_statement(statement, &functions),
        }
    }
    program
}

fn lower_statements(statements: &mut [Statement], functions: &BTreeSet<String>) {
    for statement in statements {
        lower_statement(statement, functions);
    }
}

fn lower_statement(statement: &mut Statement, functions: &BTreeSet<String>) {
    match statement {
        Statement::ExecChecked { argv } => {
            let Some(Value::Literal { text }) = argv.first() else {
                return;
            };
            if functions.contains(text) {
                let name = text.clone();
                let argv = argv[1..].to_vec();
                *statement = Statement::CallFunction { name, argv };
            }
        }
        Statement::If {
            then_body,
            else_body,
            ..
        } => {
            lower_statements(then_body, functions);
            lower_statements(else_body, functions);
        }
        Statement::Case { arms, .. } => {
            for arm in arms {
                lower_statements(&mut arm.body, functions);
            }
        }
        Statement::Assign { .. }
        | Statement::CaptureChecked { .. }
        | Statement::StringTrim { .. }
        | Statement::CallFunction { .. }
        | Statement::Print { .. }
        | Statement::Error { .. }
        | Statement::Fail { .. }
        | Statement::Exit { .. }
        | Statement::Sleep { .. } => {}
    }
}
