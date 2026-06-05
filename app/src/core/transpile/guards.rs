use std::collections::BTreeSet;

use super::ast::{Item, Predicate, Program, Statement};

pub(crate) fn bash_required_tools(program: &Program) -> BTreeSet<&'static str> {
    let mut tools = BTreeSet::new();
    for item in &program.items {
        match item {
            Item::Function { body, .. } => collect_bash_tools(body, &mut tools),
            Item::Statement { statement } => collect_bash_tool(statement, &mut tools),
        }
    }
    tools
}

pub(crate) fn emit_bash_guards(out: &mut String, tools: &BTreeSet<&'static str>) {
    for tool in tools {
        out.push_str(&format!(
            "if ! command -v {tool} >/dev/null 2>&1; then\n  seal_fail 'missing dependency: {tool}'\nfi\n\n"
        ));
    }
}

fn collect_bash_tools(statements: &[Statement], tools: &mut BTreeSet<&'static str>) {
    for statement in statements {
        collect_bash_tool(statement, tools);
    }
}

fn collect_bash_tool(statement: &Statement, tools: &mut BTreeSet<&'static str>) {
    match statement {
        Statement::StringTrim { .. } => {
            tools.insert("sed");
        }
        Statement::JsonGet { .. } => {
            tools.insert("jq");
        }
        Statement::If { predicate, .. } | Statement::While { predicate, .. }
            if predicate_requires_jq(predicate) =>
        {
            tools.insert("jq");
        }
        Statement::If {
            then_body,
            else_body,
            ..
        } => {
            collect_bash_tools(then_body, tools);
            collect_bash_tools(else_body, tools);
        }
        Statement::While { body, .. } => {
            collect_bash_tools(body, tools);
        }
        Statement::Case { arms, .. } => {
            for arm in arms {
                collect_bash_tools(&arm.body, tools);
            }
        }
        Statement::Assign { .. }
        | Statement::ArgvParse { .. }
        | Statement::ExecChecked { .. }
        | Statement::CaptureChecked { .. }
        | Statement::IntAdd { .. }
        | Statement::CallFunction { .. }
        | Statement::Print { .. }
        | Statement::Error { .. }
        | Statement::Fail { .. }
        | Statement::Exit { .. }
        | Statement::Break
        | Statement::Sleep { .. } => {}
    }
}

fn predicate_requires_jq(predicate: &Predicate) -> bool {
    matches!(
        predicate,
        Predicate::JsonEmpty { .. } | Predicate::JsonNotEmpty { .. }
    )
}
