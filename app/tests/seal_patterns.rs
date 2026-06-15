use runseal::core::seal::{
    ast::{RawExprKind, RawItemKind, RawPatternKind, RawStatementKind},
    ground, parse,
};

#[test]
fn structured_patterns() {
    let output = parse(
        r#"
match completion {
  { status: "ok" } => "ok"
  { status: "failed", exit: [code, message] } => message
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected expression statement");
    };
    let RawExprKind::Match(match_expr) = &expr.kind else {
        panic!("expected match expression");
    };
    let RawPatternKind::Map(entries) = &match_expr.arms[0].patterns[0].kind else {
        panic!("expected map pattern");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "status");
    let RawPatternKind::Map(entries) = &match_expr.arms[1].patterns[0].kind else {
        panic!("expected map pattern");
    };
    assert_eq!(entries.len(), 2);
    assert!(matches!(entries[1].pattern.kind, RawPatternKind::Array(_)));
}

#[test]
fn pattern_comparisons() {
    let output = parse(
        r#"
match value {
  { nested: a < b < c } => "bad"
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    let grounded = ground::ground(&output.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "comparison operators cannot be chained"
    );
}
