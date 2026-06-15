use runseal::core::seal::{ground, ir, parse};

#[test]
fn payload_lowering() {
    let output = parse(
        r#"
method named() {
  "value"
}

let answer = 42
let logs := | git status
let branch = @type.string(| git branch --show-current)
let commit = @type.string {
  | git rev-parse HEAD
}
"done" >> #stdout
"#,
    );

    assert!(output.diagnostics.is_empty());
    let grounded = ground::ground(&output.file);
    assert!(grounded.diagnostics.is_empty());

    let program = ir::lower(&grounded.file);
    assert_eq!(program.items.len(), 6);
    let ir::IrItem::Method(method) = &program.items[0] else {
        panic!("expected method");
    };
    assert_eq!(method.name, "named");
    assert!(matches!(method.tail, ir::IrTailOutput::Implicit { .. }));

    let ir::IrItem::Statement(ir::IrStatement::Let {
        name,
        binding,
        source,
        ..
    }) = &program.items[1]
    else {
        panic!("expected value let");
    };
    assert_eq!(name, "answer");
    assert!(matches!(binding, ir::IrBinding::Value));
    assert!(matches!(
        source,
        Some(ir::IrValueSource::Pure(ir::IrExpr::Literal {
            value: ir::IrLiteral::Int(value),
            ..
        })) if value == "42"
    ));

    let ir::IrItem::Statement(ir::IrStatement::Let {
        binding, source, ..
    }) = &program.items[2]
    else {
        panic!("expected stream let");
    };
    assert!(matches!(binding, ir::IrBinding::StreamView));
    let Some(ir::IrValueSource::StreamView(ir::IrExpr::Call(call))) = source else {
        panic!("expected process call value");
    };
    let ir::IrCallKind::Process {
        program: argv_program,
        args,
    } = &call.kind
    else {
        panic!("expected process call");
    };
    assert!(matches!(
        argv_program,
        ir::IrArgv::Text { value, .. } if value == "git"
    ));
    assert_eq!(args.len(), 1);
    assert!(matches!(
        &args[0],
        ir::IrArgv::Text { value, .. } if value == "status"
    ));

    let ir::IrItem::Statement(ir::IrStatement::Let {
        binding, source, ..
    }) = &program.items[3]
    else {
        panic!("expected type absorb let");
    };
    assert!(matches!(binding, ir::IrBinding::Value));
    let Some(ir::IrValueSource::TypeAbsorb { kind, call, .. }) = source else {
        panic!("expected type absorb source");
    };
    assert!(matches!(kind, ir::IrTypeKind::String));
    let ir::IrCallKind::Process {
        program: argv_program,
        args,
    } = &call.kind
    else {
        panic!("expected type absorb process call");
    };
    assert!(matches!(
        argv_program,
        ir::IrArgv::Text { value, .. } if value == "git"
    ));
    assert_eq!(args.len(), 2);

    let ir::IrItem::Statement(ir::IrStatement::Let { source, .. }) = &program.items[4] else {
        panic!("expected block type absorb let");
    };
    let Some(ir::IrValueSource::TypeAbsorb { kind, call, .. }) = source else {
        panic!("expected block type absorb source");
    };
    assert!(matches!(kind, ir::IrTypeKind::String));
    let ir::IrCallKind::Process {
        program: argv_program,
        args,
    } = &call.kind
    else {
        panic!("expected block type absorb process call");
    };
    assert!(matches!(
        argv_program,
        ir::IrArgv::Text { value, .. } if value == "git"
    ));
    assert_eq!(args.len(), 2);

    let ir::IrItem::Statement(ir::IrStatement::Effect { effect, .. }) = &program.items[5] else {
        panic!("expected effect");
    };
    let Some(ir::IrEffect::Flow {
        op, left, right, ..
    }) = effect
    else {
        panic!("expected stream flow");
    };
    assert!(matches!(op, ir::IrStreamOp::To));
    assert!(matches!(
        left.as_ref(),
        ir::IrExpr::Literal {
            value: ir::IrLiteral::String(value),
            ..
        } if value == "done"
    ));
    assert!(matches!(
        right.as_ref(),
        ir::IrExpr::Channel { name, .. } if name == "stdout"
    ));
}

#[test]
fn canonical_call_shapes() {
    let span = runseal::core::seal::span::Span::new(0, 12);
    let call = ir::IrCall::forward(
        ir::IrExpr::local("deploy", span),
        vec![ir::IrExpr::local("prod", span)],
        span,
    );
    assert!(matches!(call.kind, ir::IrCallKind::Forward { .. }));

    let call = ir::IrCall::process(
        ir::IrArgv::Text {
            value: "gh".to_string(),
            span,
        },
        Vec::new(),
        span,
    );
    assert!(matches!(call.kind, ir::IrCallKind::Process { .. }));

    let call = ir::IrCall::receiver(ir::IrExpr::local("text", span), "trim", Vec::new(), span);
    assert!(matches!(call.kind, ir::IrCallKind::Receiver { .. }));
}

#[test]
fn unsupported_type_absorb() {
    let output = parse(r#"let text = @type.string("literal")"#);

    assert!(output.diagnostics.is_empty());
    let grounded = ground::ground(&output.file);
    assert!(grounded.diagnostics.is_empty());

    let program = ir::lower(&grounded.file);
    let ir::IrItem::Statement(ir::IrStatement::Let { source, .. }) = &program.items[0] else {
        panic!("expected let statement");
    };
    assert!(source.is_none());
}
