use runseal::core::seal::{
    ast::{LetBinding, RawExprKind, RawItemKind, RawLiteral, RawProcessArgKind, RawStatementKind},
    parse,
};

#[test]
fn string_values_decode() {
    let output = parse("\"line\\nvalue\"\n| printf \"hello\\tthere\"\n`raw\\ntext`\n");

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected string statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected string expression");
    };
    assert!(matches!(
        &expr.kind,
        RawExprKind::Literal(RawLiteral::String(value)) if value == "line\nvalue"
    ));

    let RawItemKind::Statement(statement) = &output.file.items[1].kind else {
        panic!("expected process statement");
    };
    let RawStatementKind::Effect(expr) = &statement.kind else {
        panic!("expected process effect");
    };
    let RawExprKind::Process(process) = &expr.kind else {
        panic!("expected process");
    };
    assert!(matches!(
        &process.args[0].kind,
        RawProcessArgKind::String(value) if value == "hello\tthere"
    ));

    let RawItemKind::Statement(statement) = &output.file.items[2].kind else {
        panic!("expected text block statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected text block expression");
    };
    assert!(matches!(
        &expr.kind,
        RawExprKind::Literal(RawLiteral::TextBlock(value)) if value == "raw\\ntext"
    ));
}

#[test]
fn recovery_braced_binding() {
    let output = parse(
        r#"
let logs := {
  | git status
}
let ok = 1
"#,
    );

    assert!(!output.diagnostics.is_empty());
    let last = output.file.items.last().expect("expected recovered item");
    let RawItemKind::Statement(statement) = &last.kind else {
        panic!("expected recovered statement");
    };
    assert!(matches!(
        statement.kind,
        RawStatementKind::Let {
            ref name,
            binding: LetBinding::Value,
            ..
        } if name == "ok"
    ));
}
