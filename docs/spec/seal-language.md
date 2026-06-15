# Seal Language Specification

Status: v0 draft.

This document is the normative draft for the isolated Seal source language. The
files under `docs/examples/seal/` remain design examples and explanatory
sketches; this file is the contract a first parser and interpreter should target
for v0 source syntax.

Seal is an operations language. It exposes the process model as the callable
model: a callable creates an operation frame with `#stdin`, `#stdout`, `#stderr`,
`#frame`, cleanup, and completion. Syntax may be concise, but it must not erase
the stdout/completion model.

## Design Rules

Seal v0 follows these rules:

- Method calls, tool/builtin calls, and external process nodes all lower to call
  expressions that create operation frames.
- External process nodes use `| <program> ...`; bare command statements are not
  Seal syntax.
- Stream flow uses `>>` and `<<`; `|` is not an infix pipeline operator.
- Strings are the only lexical escape hatch.
- `@` exposes canonical operations behind source sugar, similar in spirit to a
  reflection surface.
- Return-like behavior is modeled as stdout output plus frame completion, not as
  a hidden return-value slot.

## Source Forms

Seal has four primary callable/source forms:

```seal
deploy("prod")       // Seal callable
@github.pr.view(...) // tool or builtin callable
| gh pr view ...     // external process node
text.trim()          // receiver-style call
```

Canonical lowering examples:

```seal
foo(a, b)
@call.forward(foo, [a, b])

| gh pr view {number}
@call.process("gh", ["pr", "view", number])

text.trim()
@call.self(text, @string.trim, [])
```

Stream flow:

```seal
| gh api repos/PerishCode/runseal/actions/runs >> @json.pretty.stdin()
@file.write("out.json") << { | gh pr view {number} --json number,url }
```

## Lexical Rules

Identifiers:

```text
identifier = ASCII_ALPHA (ASCII_ALNUM | "_")*
```

Names:

```text
at_name       = "@" identifier ("." identifier)*
env_name      = "$" identifier
frame_channel = "#" identifier
```

Reserved words:

```text
method let if else match for in while break continue with env
true false null
```

Strings:

- Double-quoted strings are ordinary inline strings.
- Backtick text blocks are multiline strings.
- Strings are the only lexical escape hatch.
- Seal v0 does not have backtick identifiers, raw identifiers, shell-style argv
  backslash escaping, or alternate quote forms.

```seal
"ref={ref}"
`
multi-line text
`
```

Double-quoted strings support `{expr}` interpolation and ordinary string escape
sequences. Backtick text blocks are raw: they do not interpolate, do not process
escape sequences, do not trim indentation, and preserve every character between
the opening and closing backtick, including final newlines when present. A raw
backtick cannot appear inside a backtick text block in v0; use a double-quoted
string or concatenate values when a literal backtick is needed.

Comments:

```seal
// line comment

/*
  block comment
*/
```

Block comments do not nest in v0.

Tokenization uses longest-match behavior. In particular:

```text
">>" before ">"
"<<" before "<"
"||" before "|"
"=>" before "="
":=" before ":"
"??" before "?"
```

`| WHITESPACE` is a process-node marker only where the parser expects an effect
atom or effect statement.

## Separators

Newline and semicolon are equivalent statement separators.

```seal
| gh --version
| gh auth status

| gh --version; | gh auth status
```

Separators are suppressed inside `()`, `[]`, `{}` expression forms, statement
blocks, environment blocks, and while a stream operator is waiting for its
right-hand side.

## Program Structure

```text
program = item* EOF

item
  = method_decl
  | statement
```

Methods:

```seal
method release(channel, ref = "main") {
  | gh workflow run {channel} --ref {ref}
}
```

Parameters may have default expressions:

```text
method_decl = "method" identifier "(" parameter_list? ")" block
parameter   = identifier ("=" expression)?
```

## Statements

Seal v0 statements:

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

Variable binding:

```seal
let value = expression

let raw := {
  | kubectl logs deploy/app --follow
}
```

`let x = ...` binds a Seal value. `let x := ...` binds stdout as a readonly
stream view.

Environment scope:

```seal
with env {
  RUST_LOG = "debug"
  RUNSEAL_CHANNEL = channel
} {
  | cargo test --locked --workspace
}
```

The first `with env` block is a dedicated environment binding block. It accepts
only `NAME = expression` entries.

Control flow:

```seal
if branch in ["main", "master"] {
  fail("protected branch: {branch}")
} else if branch == "" {
  fail("not on a branch")
}

for tool in tools {
  check_tool(tool)
}

while attempt < 6 {
  attempt = attempt + 1
}
```

`match` is both an expression and statement form:

```seal
let workflow = match channel {
  "stable" => "release-stable.yml"
  "beta" => "release-beta.yml"
  _ => fail("invalid channel: {channel}")
}
```

Pattern alternatives use `|` only inside match patterns.

## Expressions

Expression mode is separate from process argv mode.

Operator precedence, high to low:

```text
postfix:     call, block argument, field, index
unary:       ! -
multiply:    * / %
add:         + -
compare:     < <= > >= in
equality:    == !=
boolean and: &&
boolean or:  ||
null coalesce: ??
```

Primary expressions include literals, identifiers, `@` names, `$` env reads,
`#` frame channels, arrays, maps, and grouped expressions.

Arrays and maps:

```seal
let tools = ["git", "gh", "cargo"]
let release = {
  channel: "beta",
  ref: "main",
}
```

Labeled call arguments are source sugar for structured helper/tool calls:

```seal
@fs.mkdir(tmp_dir, mode: 700)
```

They must not become a separate runtime named-argument forwarding system.
The parser accepts labeled arguments in call syntax. Semantic lowering then
allows them only when the callee is statically known to accept labels, such as a
method with named parameters or an `@` helper/tool. Dynamic callable values and
explicit `@call.forward(...)` argument bundles are positional-only.

Comparison operators are non-associative. Chaining is a syntax error:

```seal
a < b < c      // invalid
a < b && b < c // valid
```

## Process Nodes

A process node starts with `|` followed by whitespace:

```seal
| gh pr view {number} --json number,url
| gh *args
| {program} *{args}
```

Lowering:

```seal
| gh pr view {number}
@call.process("gh", ["pr", "view", number])
```

Process argv mode supports:

- bare argv words
- double-quoted strings and backtick text blocks
- `{expr}` interpolation for one argv value
- `*args` and `*{expr}` array spread

Bare argv words do not perform shell expansion or backslash escaping. A bare
word ends at whitespace, statement separator, `>>`, `<<`, or the closing
delimiter of the containing effect block. It also ends before `{`, `}`, `(`,
`)`, `[`, `]`, `"`, or `` ` ``. A `//` comment starts only at a trivia boundary,
such as the start of a line or after whitespace, so `https://example.com` remains
one argv word. If an argv value needs syntax characters, delimiters, comments,
or whitespace, use a string literal.

```seal
| some-tool ";"
| some-tool ">>"
```

## Stream Flow

`>>` connects the left effect atom's stdout to the right effect atom's stdin.
`<<` is the mirror spelling.

```seal
| git branch --format "%(refname:short)" >>
| grep "^feat/" >>
| head -n 1
```

`a >> b` lowers to `@stream.flow(a, b)`. Longer left-to-right chains lower to
`@stream.pipeline([...])` in source order. `a << b` lowers as `@stream.flow(b,
a)`. Source semantics are the same in all forms: each stage is an operation
frame, and edges connect stdout to stdin.

Effect blocks contain exactly one stream graph in v0:

```seal
let branch = @type.string {
  | git branch --show-current
}
```

## Callable Output And Completion

A callable body is an operation frame. It has no hidden return-value slot.
Output and completion are modeled through current-frame streams:

```text
value output -> value >> #stdout
normal end   -> { type: "ok" } >> #frame
```

At callable fallthrough, the runtime emits normal ok completion. If a callable
body has a tail value expression and does not explicitly reference current-frame
`#stdout`, that tail value is implicitly written to `#stdout`.

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

Explicit current-frame `#stdout` use disables implicit tail output for that
callable only. Nested callables, lambdas, and handlers have independent
`#stdout`.

Early normal completion uses:

```seal
@call.exit(value, event?)
```

Conceptual lowering:

```seal
value >> #stdout
(event ?? { type: "ok" }) >> #frame
// stop current callable frame
```

`@call.exit()` is equivalent to `@call.exit(null)`.

Seal v0 does not define a shorter return keyword or operator. Any future concise
return-like syntax must lower to `@call.exit(...)` and must preserve the
explicit `#stdout`/`#frame` escape path.

## Frame Channels

The predefined current-frame channels are:

```seal
#stdin
#stdout
#stderr
#frame
```

`#frame` is a control/event stream, not a capability object. Frame lifecycle
events are ordinary structured values written to `#frame`.

Frame event shapes recognized by the v0 runtime are:

```seal
{ type: "ok" }

{
  type: "failed",
  exit: {
    code: 1,
    signal: null,
  },
  message: null,
  data: {},
}

{
  type: "fault",
  fault: {
    kind: "shape",
    message: "expected string",
    data: {},
  },
}

{
  type: "cancelled",
  source: "operator",
  signal: "interrupt",
}

{
  type: "cleanup",
  run: () => {
    @file.remove(tmp)
  },
}
```

These are control/event stream values, not `@frame.*` API calls. Additional
fields are allowed and ignored by runtimes that do not recognize them; missing
required fields are faults.

## Runtime Values

Seal v0 values:

- string
- int
- boolean
- byte
- bytes
- array
- map
- null
- stream
- function

`null` is the only missing value. `??` only handles `null`. Equality is strict;
there is no implicit type conversion.

Streams are process IO resources. Convert them explicitly with helpers such as:

```seal
@type.string(...)
@type.bytes(...)
@type.array(...)
@type.map(...)
```

`@type.array(...)` and `@type.map(...)` decode JSON bytes and require the
decoded top-level shape to match.

## Receiver Calls

Receiver calls are native sugar:

```seal
let trimmed = text.trim()
```

Candidate lowering:

```seal
@call.self(text, @string.trim, [])
```

The runtime may box and unbox primitive/runtime values to call built-in receiver
methods. Field access and receiver calls remain syntactically distinct:

```seal
release.version
release.version()
```

## Canonical Operation Namespace

The `@` namespace exposes canonical operations behind source sugar:

```seal
foo(a, b)      -> @call.forward(foo, [a, b])
| gh ...       -> @call.process("gh", [...])
text.trim()    -> @call.self(text, @string.trim, [])
a >> b         -> @stream.flow(a, b)
```

This namespace is not a dumping ground for arbitrary language magic. It exists
to make the core operation model visible, testable, and available for
metaprogramming.

## Lexer And Parser Closure

The v0 lexer and parser should be able to proceed from this spec without
unresolved source-syntax decisions. Remaining work is runtime design: stream
scheduling, completion-timing details for `@type.*` and `:=`,
buffering/spooling policy, and the concrete implementation of built-in helper
namespaces.
