use runseal::core::seal::{ground, parse};

#[test]
fn cleanup_frame_event() {
    let output = parse(
        r#"
{
  type: "cleanup",
  run: () => {
    @file.remove(tmp)
  },
} >> #frame
"#,
    );

    assert!(output.diagnostics.is_empty());
    let grounded = ground::ground(&output.file);
    assert!(grounded.diagnostics.is_empty());
}

#[test]
fn cleanup_requires_run_lambda() {
    let missing = parse(
        r#"
{
  type: "cleanup",
} >> #frame
"#,
    );
    assert!(missing.diagnostics.is_empty());
    let grounded = ground::ground(&missing.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "cleanup frame event requires field 'run'"
    );

    let not_lambda = parse(
        r#"
{
  type: "cleanup",
  run: "tmp-cleanup",
} >> #frame
"#,
    );
    assert!(not_lambda.diagnostics.is_empty());
    let grounded = ground::ground(&not_lambda.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "cleanup frame event field 'run' must be a lambda"
    );
}

#[test]
fn frame_event_required_fields() {
    let valid = parse(
        r#"
{ type: "ok" } >> #frame

{
  type: "failed",
  exit: { code: 1, signal: null },
} >> #frame

{
  type: "fault",
  fault: { kind: "shape", message: "bad", data: {} },
} >> #frame

{
  type: "cancelled",
  source: "operator",
  signal: "interrupt",
} >> #frame
"#,
    );
    assert!(valid.diagnostics.is_empty());
    let grounded = ground::ground(&valid.file);
    assert!(grounded.diagnostics.is_empty());

    let invalid = parse(r#"{ type: "failed" } >> #frame"#);
    assert!(invalid.diagnostics.is_empty());
    let grounded = ground::ground(&invalid.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "frame event type requires field 'exit'"
    );
}

#[test]
fn frame_event_type_diagnostics() {
    let missing = parse(r#"{ exit: { code: 0, signal: null } } >> #frame"#);
    assert!(missing.diagnostics.is_empty());
    let grounded = ground::ground(&missing.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "frame event map must include field 'type'"
    );

    let non_string = parse(r#"{ type: status } >> #frame"#);
    assert!(non_string.diagnostics.is_empty());
    let grounded = ground::ground(&non_string.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "frame event field 'type' must be a string literal"
    );

    let unknown = parse(r#"{ type: "later" } >> #frame"#);
    assert!(unknown.diagnostics.is_empty());
    let grounded = ground::ground(&unknown.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "unknown frame event type 'later'"
    );
}

#[test]
fn dynamic_frame_event() {
    let output = parse("event >> #frame");

    assert!(output.diagnostics.is_empty());
    let grounded = ground::ground(&output.file);
    assert!(grounded.diagnostics.is_empty());
}
