# Seal Grammar Draft

This is a first-pass grammar draft for the isolated Seal source syntax. It is
meant to drive parser design, not to freeze every surface detail.

The grammar is intentionally centered on the current syntax spine:

```text
Seal call          foo(a, b)
tool/builtin call  @github.pr.view(...)
process node       | gh pr view ...
stream flow        a >> b
mirror flow        b << a
```

## Notation

This file uses informal EBNF:

```text
x?      optional x
x*      zero or more x
x+      one or more x
x | y   x or y
"x"     literal token
```

`SEP` means a statement separator. Newline and semicolon are equivalent
separators, except where a surrounding construct owns the newline.

```text
SEP = NEWLINE | ";"
```

## Comments

Line comments use `//`. Block comments use `/* ... */`.

```seal
// line comment

/*
  block comment
*/
```

First-pass block comments do not need to nest.

Comments are trivia. For process argv, `//` should be recognized as a comment
only at a trivia boundary, such as the start of a line or after whitespace, so a
bare URL-like argv token is not split accidentally.

```seal
| curl https://example.com       // ok: URL is one argv token
| curl https://example.com // ok: comment starts after whitespace
```

## Lexical Structure

```text
identifier
  = ASCII_ALPHA (ASCII_ALNUM | "_")*

at_name
  = "@" identifier ("." identifier)*

env_name
  = "$" identifier

frame_channel
  = "#" identifier

string
  = double_quoted_string | text_block
```

String literals are the only lexical escape hatch. Double-quoted strings cover
ordinary inline text and support `{expr}` interpolation plus string escape
sequences. Backtick text blocks cover multiline strings. Seal does not provide
backtick identifiers, raw identifiers, shell-style backslash escaping in process
argv, or alternate quote forms for escaping syntax.

```seal
"ref={ref}"
"literal {ref}"
`
multi-line text
`
```

Reserved words:

```text
method let if else match for in while break continue with env
true false null
```

Reserved words and syntax symbols keep their grammar meaning outside strings.
If source needs one as literal data, put it in a string literal.

```seal
| some-tool ";"
| some-tool ">>"
let word = "method"
```

Tokenization should prefer the longest valid token. In particular:

```text
">>" before ">"
"<<" before "<"
"||" before "|"
"=>" before "="
":=" before ":"
"??" before "?"
```

`| WHITESPACE` is recognized as a process-node marker only where the parser
expects an effect atom or effect statement. In expression grammar, `||` remains
boolean OR. In match pattern grammar, `|` remains a pattern alternative
separator.

## Program

```text
program
  = separator* item* EOF

item
  = method_decl
  | statement

separator
  = SEP+
```

Separators are ignored inside strings and inside parenthesized, bracketed, or
braced expression forms unless that construct explicitly parses statements.

## Blocks

```text
block
  = "{" separator* statement_list? separator* "}"

statement_list
  = statement (separator+ statement)* separator*
```

Blocks own their internal separators. A block used as a map literal is parsed in
expression context; a block used as a method body, control-flow body, or effect
body is parsed in statement context.

## Methods

```text
method_decl
  = "method" identifier "(" parameter_list? ")" block

parameter_list
  = parameter ("," parameter)* ","?

parameter
  = identifier ("=" expression)?
```

```seal
method release(channel, ref = "main") {
  | gh workflow run {channel} --ref {ref}
}
```

## Callable Tail And Completion

Callable bodies are operation frames. They do not need a hidden return-value
slot. Output and completion are modeled through the current frame streams:

```text
value output  -> value >> #stdout
normal end    -> { type: "ok" } >> #frame
```

At the end of a callable body, fallthrough emits a normal ok completion event.
If the callable body has a tail value expression and does not explicitly use the
current frame's `#stdout`, that tail value is implicitly written to `#stdout`.

```seal
method workflow_for(channel) {
  match channel {
    "stable" => "release-stable.yml"
    "beta" => "release-beta.yml"
    _ => fail("invalid channel: {channel}")
  }
}
```

Conceptual lowering:

```seal
method workflow_for(channel) {
  match channel {
    "stable" => "release-stable.yml"
    "beta" => "release-beta.yml"
    _ => fail("invalid channel: {channel}")
  } >> #stdout

  { type: "ok" } >> #frame
}
```

If the callable body explicitly references the current frame's `#stdout`, this
implicit tail-output sugar is disabled for that callable. Nested callables,
lambdas, and handlers have their own `#stdout` and do not affect the outer
callable's tail-output decision.

```seal
method report() {
  "starting" >> #stdout
  make_summary() // not implicitly written to #stdout
}
```

Early normal completion uses an explicit call-domain helper:

```seal
@call.exit(value, event?)
```

Conceptual lowering:

```seal
value >> #stdout
(event ?? { type: "ok" }) >> #frame
// stop current callable frame
```

`@call.exit()` is equivalent to `@call.exit(null)`. `@call.exit(value)` is the
canonical form for early return-like behavior. A future concise return sugar can
lower to this helper, but the explicit `#stdout`/`#frame` path must remain
adjacent and usable.

## Statements

```text
statement
  = let_statement
  | assign_statement
  | if_statement
  | match_statement
  | for_statement
  | while_statement
  | with_env_statement
  | break_statement
  | continue_statement
  | effect_statement
  | expression_statement
```

```text
let_statement
  = "let" identifier "=" expression
  | "let" identifier ":=" effect_block_or_expr

assign_statement
  = lvalue "=" expression

break_statement
  = "break"

continue_statement
  = "continue"

effect_statement
  = effect_expression

expression_statement
  = expression
```

```text
lvalue
  = identifier lvalue_suffix*

lvalue_suffix
  = "." identifier
  | "[" expression "]"
```

`let x := ...` binds stdout as a readonly stream view. `let x = ...` binds a
Seal value.

```seal
let text = "hello"

let logs := {
  | kubectl logs deploy/app --follow
}
```

## Control Flow

```text
if_statement
  = "if" expression block ("else" "if" expression block)* ("else" block)?

while_statement
  = "while" expression block

for_statement
  = "for" identifier "in" expression block
```

```seal
if branch in ["main", "master"] {
  fail("protected branch: {branch}")
} else if branch == "" {
  fail("not on a branch")
}
```

`match` is both an expression and a statement form. Arms can return expressions
or run blocks.

```text
match_expression
  = "match" expression "{" match_arm* "}"

match_statement
  = match_expression

match_arm
  = pattern_list "=>" (expression | block) separator*

pattern_list
  = pattern ("|" pattern)*

pattern
  = "_"
  | literal
  | identifier
  | map_pattern
  | array_pattern

map_pattern
  = "{" (map_pattern_entry ("," map_pattern_entry)* ","?)? "}"

map_pattern_entry
  = identifier ":" pattern

array_pattern
  = "[" (pattern ("," pattern)* ","?)? "]"
```

```seal
let workflow = match channel {
  "stable" => "release-stable.yml"
  "beta" => "release-beta.yml"
  _ => fail("invalid channel: {channel}")
}
```

Pattern alternatives use `|` only inside match-arm pattern context. That does
not conflict with process nodes because process nodes are recognized only at
statement or effect-expression starts followed by whitespace.

## Environment Scope

```text
with_env_statement
  = "with" "env" env_block block

env_block
  = "{" separator* env_binding* separator* "}"

env_binding
  = identifier "=" expression separator*
```

The first block is pure environment projection. It does not accept commands,
tool calls, or control flow.

```seal
with env {
  RUST_LOG = "debug"
  RUNSEAL_CHANNEL = channel
} {
  | cargo test --locked --workspace
}
```

## Expressions

The expression grammar is ordinary and deliberately separate from process argv
mode.

```text
expression
  = match_expression
  | lambda_expression
  | null_coalesce

lambda_expression
  = "(" parameter_list? ")" "=>" block

null_coalesce
  = boolean_or ("??" boolean_or)*

boolean_or
  = boolean_and ("||" boolean_and)*

boolean_and
  = equality ("&&" equality)*

equality
  = comparison (("==" | "!=") comparison)*

comparison
  = additive (("<" | "<=" | ">" | ">=" | "in") additive)*

additive
  = multiplicative (("+" | "-") multiplicative)*

multiplicative
  = unary (("*" | "/" | "%") unary)*

unary
  = ("!" | "-") unary
  | postfix

postfix
  = primary postfix_suffix*

postfix_suffix
  = call_suffix
  | block_argument
  | "." identifier
  | "[" expression "]"

call_suffix
  = "(" argument_list? ")"

argument_list
  = argument ("," argument)* ","?

argument
  = expression
  | identifier ":" expression

block_argument
  = effect_block
```

Labeled call arguments are source sugar. They should not become a separate
runtime forwarding system; lowering can turn them into ordinary structured
values for tool and helper APIs.

Primary expressions:

```text
primary
  = literal
  | identifier
  | at_name
  | env_name
  | frame_channel
  | array_literal
  | map_literal
  | "(" expression ")"

literal
  = string | integer | "true" | "false" | "null"

array_literal
  = "[" (expression ("," expression)* ","?)? "]"

map_literal
  = "{" (map_entry ("," map_entry)* ","?)? "}"

map_entry
  = identifier ":" expression
  | string ":" expression
```

Examples:

```seal
let port = @type.int($PORT ?? "8080")
let run_id = require(runs[0].databaseId, "missing run id")

@github.issue.comment.create(
  repo: "PerishCode/runseal",
  number: 49,
  body_file: "body.md",
)
```

## Receiver Calls

Postfix member calls can be receiver-style calls. This is source sugar over a
canonical self-call operation.

```seal
let trimmed = text.trim()
let upper = name.upper()
```

Candidate lowering:

```seal
@call.self(text, @string.trim, [])
@call.self(name, @string.upper, [])
```

Primitive/runtime values may be boxed and unboxed by the runtime to call
built-in receiver methods. This keeps convenient method syntax adjacent to an
explicit metaprogramming form, similar to the relationship between `foo(a)` and
`@call.forward(foo, [a])`.

Field access and receiver calls remain distinct at the syntax edge:

```seal
release.version      // field access
release.version()    // receiver-style call
```

## Effect Expressions

Effect expressions are where process nodes and stream flow live.

```text
effect_block_or_expr
  = effect_block
  | effect_expression

effect_block
  = "{" separator* effect_expression separator* "}"

effect_expression
  = stream_expression

stream_expression
  = effect_atom (stream_operator effect_atom)*

stream_operator
  = ">>" | "<<"

effect_atom
  = process_node
  | expression
```

`>>` connects the left atom's stdout to the right atom's stdin. `<<` is the
mirror spelling.

```seal
| gh api repos/PerishCode/runseal/actions/runs >> @json.pretty.stdin()

@file.write("out.json") << {
  | gh pr view {number} --json number,url
}
```

Long stream chains can lower to an array-shaped pipeline helper.

```seal
| git branch --format "%(refname:short)" >>
| grep "^feat/" >>
| head -n 1
```

Conceptual lowering:

```seal
@stream.pipeline([
  @call.process("git", ["branch", "--format", "%(refname:short)"]),
  @call.process("grep", ["^feat/"]),
  @call.process("head", ["-n", "1"]),
])
```

## Process Nodes

A process node starts with `|` followed by whitespace. A lone `|` is not an
infix pipeline operator.

```text
process_node
  = "|" WHITESPACE process_program process_arg*

process_program
  = process_word
  | process_interpolation

process_arg
  = process_word
  | process_string
  | process_interpolation
  | process_spread

process_string
  = string

process_interpolation
  = "{" expression "}"

process_spread
  = "*" identifier
  | "*{" expression "}"
```

`process_word` is a bare argv token. It ends at whitespace, `SEP`, `>>`, `<<`,
or the closing delimiter of the surrounding effect block. It does not perform
shell expansion or backslash escaping. If an argv value needs whitespace,
reserved syntax, or a token boundary character, write it as a double-quoted
string.

```seal
| gh pr view {number} --json number,url
| gh *args
| {program} *{args}
| some-tool ";"
```

Lowering:

```seal
| gh pr view {number}

@call.process("gh", ["pr", "view", number])
```

`*args` spreads an array into process argv. `{expr}` inserts one argv value.
String conversion should be explicit and strict at runtime; non-stringable
values fail fast unless spread syntax is used intentionally.

## Separators And Continuation

Newline and semicolon are equivalent statement separators.

```seal
| gh --version
| gh auth status

| gh --version; | gh auth status
```

A separator is suppressed when the parser is inside `()`, `[]`, `{}` expression
forms, method/control blocks, environment blocks, or when a stream operator is
waiting for its right-hand side.

```seal
let branch = @type.string {
  | git branch --format "%(refname:short)" >>
  | grep "^feat/" >>
  | head -n 1
}
```

A process node ends at the next statement separator, stream operator, or closing
delimiter owned by the containing construct.

```seal
| gh pr view {number}; print("done")
| gh api repos/... >> @json.pretty.stdin()
```

If `;` or whitespace-sensitive text should be passed as argv, quote it.

```seal
| some-tool ";"
```

## Parser Modes

The first parser should keep two clear modes:

- **Expression mode** parses Seal values, calls, arrays, maps, lambdas, and
  control expressions.
- **Process argv mode** starts after `| <whitespace>` and parses raw argv tokens
  plus `{expr}` interpolation and `*` spread.

This avoids the old ambiguous shape where a bare command had to be parsed inside
ordinary function-call parentheses.

```seal
let pr = @type.map {
  | gh pr view {number} --json number,url
}
```

The block gives the effect boundary. The `|` marker gives the process-node
boundary. The ordinary expression parser never has to guess where `gh ...`
ends.

## Open Items

This draft intentionally leaves a few details open:

- Whether `/* ... */` block comments nest.
- Whether backtick text blocks support interpolation, how they handle indentation
  trimming, and whether the final newline is preserved.
- The exact character set accepted by `process_word`.
- Whether labeled call arguments are allowed everywhere or restricted to
  `@` tool/helper calls.
- Whether comparison chaining such as `a < b < c` is rejected syntactically or
  by runtime shape checks.
- Whether a later syntax should add multi-statement effect blocks. First-pass
  `effect_block` is exactly one stream graph.
- The exact v0 shape of the normal ok frame event used by `@call.exit(...)` and
  fallthrough completion.
- Whether `@call.exit(...)` needs a concise source sugar, or whether the
  canonical helper is enough for cold-start Seal.
- The final lowering shape for long `>>` chains: nested `@stream.flow(...)` or
  array-shaped `@stream.pipeline(...)`.
