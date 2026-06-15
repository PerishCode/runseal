use runseal::core::seal::{
    ast::{
        LetBinding, RawExprKind, RawItemKind, RawMatchArmBody, RawProcessArgKind, RawProcessPart,
        RawStatement, RawStatementKind,
    },
    ground::{self, GroundNode, TailOutput},
    lex, parse,
    token::{Keyword, TokenKind},
};

#[test]
fn lexer_comment_newline() {
    let output = lex("// hi\nlet x = 1");

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.tokens[0].kind, TokenKind::Newline);
    assert_eq!(output.tokens[0].leading_comments().count(), 1);
    assert_eq!(output.tokens[1].kind, TokenKind::Keyword(Keyword::Let));
}

#[test]
fn lexer_url_comment() {
    let output = lex("| curl https://example.com // comment\n");
    let slash_count = output
        .tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Slash)
        .count();

    assert_eq!(slash_count, 2);
    assert!(
        output
            .tokens
            .iter()
            .any(|token| token.leading_comments().count() == 1)
    );
}

#[test]
fn raw_comments() {
    let output = parse("// file\nlet x = 1 // x\nlet y = 2");

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.file.comments.len(), 2);
    assert!(matches!(output.file.items[0].kind, RawItemKind::Comment(0)));
    assert_eq!(output.file.items[1].trailing_comments, vec![1]);
}

#[test]
fn process_argv() {
    let output = parse("| gh pr view {number} --json number,url *extra\n");

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Effect(expr) = &statement.kind else {
        panic!("expected effect");
    };
    let RawExprKind::Process(process) = &expr.kind else {
        panic!("expected process");
    };
    assert!(matches!(
        process.program.as_ref().map(|arg| &arg.kind),
        Some(RawProcessArgKind::Word(parts)) if parts == &vec![RawProcessPart::Text("gh".to_string())]
    ));
    assert_eq!(process.args.len(), 6);
    assert!(matches!(
        process.args[2].kind,
        RawProcessArgKind::Word(ref parts)
            if parts.iter().any(|part| matches!(part, RawProcessPart::Interpolation(_)))
    ));
    assert!(matches!(process.args[5].kind, RawProcessArgKind::Spread(_)));
}

#[test]
fn process_whitespace() {
    let output = parse("|gh\n");

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].message,
        "expected whitespace after process marker '|'"
    );
}

#[test]
fn process_call_arg_boundary() {
    let output = parse(r#"let branch = @type.string(| git branch --show-current)"#);

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Let { value, .. } = &statement.kind else {
        panic!("expected let");
    };
    let RawExprKind::Call { args, .. } = &value.kind else {
        panic!("expected call");
    };
    assert_eq!(args.len(), 1);
    let RawExprKind::Process(process) = &args[0].value.kind else {
        panic!("expected process arg");
    };
    assert_eq!(process.args.len(), 2);
}

#[test]
fn with_env_scope() {
    let output = parse(
        r#"
with env {
  RUST_LOG = "debug"
  RUNSEAL_CHANNEL = channel
} {
  | cargo test
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::WithEnv { bindings, body } = &statement.kind else {
        panic!("expected with env");
    };
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].name, "RUST_LOG");
    assert_eq!(body.items.len(), 1);
}

#[test]
fn control_flow() {
    let output = parse(
        r#"
if branch == "main" {
  attempt = 1
} else if branch == "beta" {
  attempt = 2
} else {
  attempt = 3
}

while attempt < 6 {
  attempt = attempt + 1
}

for tool in tools {
  | which {tool}
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.file.items.len(), 3);
    assert!(matches!(
        output.file.items[0].kind,
        RawItemKind::Statement(RawStatement {
            kind: RawStatementKind::If { .. },
            ..
        })
    ));
    assert!(matches!(
        output.file.items[1].kind,
        RawItemKind::Statement(RawStatement {
            kind: RawStatementKind::While { .. },
            ..
        })
    ));
    assert!(matches!(
        output.file.items[2].kind,
        RawItemKind::Statement(RawStatement {
            kind: RawStatementKind::For { .. },
            ..
        })
    ));
}

#[test]
fn match_expr() {
    let output = parse(
        r#"
let workflow = match channel {
  "stable" | "prod" => "release-stable.yml"
  "beta" => "release-beta.yml"
  _ => fail("invalid channel")
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Let { value, .. } = &statement.kind else {
        panic!("expected let");
    };
    let RawExprKind::Match(match_expr) = &value.kind else {
        panic!("expected match expression");
    };
    assert_eq!(match_expr.arms.len(), 3);
    assert_eq!(match_expr.arms[0].patterns.len(), 2);
}

#[test]
fn match_arm_body_shapes() {
    let block = parse(
        r#"
match target {
  "macos" => {
    | sw_vers
  }
}
"#,
    );

    assert!(block.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &block.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected expression statement");
    };
    let RawExprKind::Match(match_expr) = &expr.kind else {
        panic!("expected match expression");
    };
    assert!(matches!(match_expr.arms[0].body, RawMatchArmBody::Block(_)));

    let map = parse(
        r#"
let result = match status {
  "ok" => { status: "ok" }
}
"#,
    );

    assert!(map.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &map.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Let { value, .. } = &statement.kind else {
        panic!("expected let");
    };
    let RawExprKind::Match(match_expr) = &value.kind else {
        panic!("expected match expression");
    };
    assert!(matches!(match_expr.arms[0].body, RawMatchArmBody::Expr(_)));
}

#[test]
fn block_call() {
    let output = parse(
        r#"
let branch = @type.string {
  | git branch --show-current
}
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Let { value, .. } = &statement.kind else {
        panic!("expected let");
    };
    let RawExprKind::BlockCall { block, .. } = &value.kind else {
        panic!("expected block call");
    };
    assert_eq!(block.items.len(), 1);
}

#[test]
fn lambda_handler_call_arg() {
    let output = parse(
        r#"
@call.stdio(call, (stdin, stdout, stderr) => {
  stdout >> #stdout
})
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected expression statement");
    };
    let RawExprKind::Call { args, .. } = &expr.kind else {
        panic!("expected call expression");
    };
    assert_eq!(args.len(), 2);
    let RawExprKind::Lambda(lambda) = &args[1].value.kind else {
        panic!("expected handler lambda");
    };
    assert_eq!(lambda.params.len(), 3);
    assert_eq!(lambda.params[0].name, "stdin");
    assert_eq!(lambda.params[1].name, "stdout");
    assert_eq!(lambda.params[2].name, "stderr");
    assert_eq!(lambda.body.items.len(), 1);
}

#[test]
fn lambda_completion_chain_arg() {
    let output = parse(
        r#"
@call.completion(call, (stdin, stdout, stderr, frame) => {})
  .ok((completion) => {
    "ok" >> #stdout
  })
"#,
    );

    assert!(output.diagnostics.is_empty());
    let RawItemKind::Statement(statement) = &output.file.items[0].kind else {
        panic!("expected statement");
    };
    let RawStatementKind::Expr(expr) = &statement.kind else {
        panic!("expected expression statement");
    };
    let RawExprKind::ReceiverCall { method, args, .. } = &expr.kind else {
        panic!("expected receiver call");
    };
    assert_eq!(method, "ok");
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0].value.kind, RawExprKind::Lambda(_)));
}

#[test]
fn parse_recovery() {
    let output = parse("let x =\nlet y = 1\n");

    assert!(!output.diagnostics.is_empty());
    assert_eq!(output.file.items.len(), 2);
    let RawItemKind::Statement(statement) = &output.file.items[1].kind else {
        panic!("expected second statement");
    };
    assert!(matches!(
        statement.kind,
        RawStatementKind::Let {
            ref name,
            binding: LetBinding::Value,
            ..
        } if name == "y"
    ));
}

#[test]
fn ground_comparison_chain() {
    let output = parse("// file\nlet x = a < b < c\n");
    assert!(output.diagnostics.is_empty());

    let grounded = ground::ground(&output.file);

    assert_eq!(grounded.file.nodes.len(), 1);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "comparison operators cannot be chained"
    );
}

#[test]
fn ground_effect_block() {
    let valid = parse(
        r#"
let branch = @type.string {
  | git branch --show-current
}
"#,
    );
    assert!(valid.diagnostics.is_empty());
    assert!(ground::ground(&valid.file).diagnostics.is_empty());

    let invalid = parse(
        r#"
let branch = @type.string {
  let x = 1
}
"#,
    );
    assert!(invalid.diagnostics.is_empty());
    let grounded = ground::ground(&invalid.file);
    assert_eq!(grounded.diagnostics.len(), 1);
    assert_eq!(
        grounded.diagnostics[0].message,
        "effect block must contain exactly one stream graph"
    );
}

#[test]
fn ground_method_tail() {
    let implicit = parse(
        r#"
method workflow_for(channel) {
  match channel {
    "stable" => "release-stable.yml"
    _ => fail("invalid channel")
  }
}
"#,
    );
    assert!(implicit.diagnostics.is_empty());
    let grounded = ground::ground(&implicit.file);
    assert!(grounded.diagnostics.is_empty());
    assert!(matches!(
        grounded.file.nodes[0],
        GroundNode::Method {
            tail: TailOutput::Implicit { .. },
            ..
        }
    ));

    let explicit = parse(
        r###"
method status() {
  "starting" >> #stdout
  make_summary()
}
"###,
    );
    assert!(explicit.diagnostics.is_empty());
    let grounded = ground::ground(&explicit.file);
    assert!(grounded.diagnostics.is_empty());
    assert!(matches!(
        grounded.file.nodes[0],
        GroundNode::Method {
            tail: TailOutput::DisabledByStdout { .. },
            ..
        }
    ));

    let handler_stdout = parse(
        r###"
method routed(call) {
  @call.stdio(call, (stdin, stdout, stderr) => {
    stdout >> #stdout
  })

  "done"
}
"###,
    );
    assert!(handler_stdout.diagnostics.is_empty());
    let grounded = ground::ground(&handler_stdout.file);
    assert!(grounded.diagnostics.is_empty());
    assert!(matches!(
        grounded.file.nodes[0],
        GroundNode::Method {
            tail: TailOutput::Implicit { .. },
            ..
        }
    ));

    let match_arm_stdout = parse(
        r###"
method routed(target) {
  match target {
    "local" => {
      "local" >> #stdout
    }
  }

  "done"
}
"###,
    );
    assert!(match_arm_stdout.diagnostics.is_empty());
    let grounded = ground::ground(&match_arm_stdout.file);
    assert!(grounded.diagnostics.is_empty());
    assert!(matches!(
        grounded.file.nodes[0],
        GroundNode::Method {
            tail: TailOutput::DisabledByStdout { .. },
            ..
        }
    ));
}
