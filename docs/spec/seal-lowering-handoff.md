# Seal Lowering Handoff

Status: implementation handoff draft.

This document describes the boundary between the current Seal parser/Grounded
AST work and later IR/runtime work. It is intentionally not a runtime
scheduling spec. Its job is to say what lowering may rely on after Raw AST and
Grounded AST have accepted a source file, and what still belongs to runtime
design.

The source contract remains `seal-language.md`. This file is the bridge from
that source contract to the first IR design.

## Pipeline Boundary

The cold-start implementation shape is:

```text
Lexer -> Parser -> Raw AST -> Grounded AST -> IR -> Runtime
```

The stages should keep these responsibilities separate:

- Lexer owns tokens, trivia spans, comments, strings, and process-marker
  recognition.
- Parser owns source structure, recovery, Raw AST nodes, block-vs-map
  structure, process argv structure, and comment attachment.
- Raw AST preserves source-relevant information even when later stages drop it:
  spans, comments, syntactic call forms, labels, stream operators, process argv
  atoms, match arm body shape, and lambda bodies.
- Grounded AST owns source-level semantic shape checks and metadata. It may
  reject source that cannot be lowered coherently, but it should not become a
  static type system.
- IR owns canonical operation structure, stream graph nodes, frame events,
  completion values, and explicit runtime edges.
- Runtime owns scheduling, buffering/spooling, stream progress, completion
  timing, cleanup execution, and concrete built-in behavior.

## Raw AST Guarantees

Lowering may rely on Raw AST to preserve these distinctions:

- Comments exist as source nodes and attachments. Lowering may ignore them, but
  formatters and source tools do not need to reconstruct them from trivia.
- A process node preserves its program and argv atoms. Bare words, strings,
  text blocks, interpolation parts, and `*expr` spreads are distinct.
- Process nodes end at statement/effect boundaries and containing close
  delimiters. Commas inside bare argv words remain argv text.
- A map literal and a statement block are distinct after parsing. Match arms
  preserve expression bodies separately from statement-block bodies.
- Lambda bodies preserve callable-frame boundaries. They are not just nested
  statement blocks.
- Receiver calls, direct calls, `@` calls, block calls, process nodes, stream
  flow, and grouped expressions remain syntactically distinct.
- Pattern structure is preserved for wildcard, expression, array, and map
  patterns.

## Grounded AST Guarantees

Lowering may rely on Grounded AST diagnostics to reject these source shapes:

- Chained comparison operators.
- Effect blocks that do not contain exactly one stream graph.
- Duplicate labels in call arguments.
- Labeled arguments on dynamic callables.
- Labeled arguments on `@call.forward(...)`.
- `@call.forward(...)` with the wrong arity or a statically visible non-array
  argument bundle.
- `@call.stdio(...)` and `@call.completion(...)` with the wrong arity,
  statically visible non-lambda handlers, or literal handlers with the wrong
  parameter count.
- Completion-chain handlers `.ok(...)`, `.failed(...)`, `.faulted(...)`,
  `.cancelled(...)`, and `.always(...)` with the wrong arity, statically
  visible non-lambda handlers, or literal handlers with the wrong parameter
  count.
- `@call.exit(...)` with more than two arguments or a statically visible
  non-map event argument.
- Literal frame event maps with missing or invalid `type`, unknown event types,
  missing required fields, or invalid cleanup `run` handlers.
- Duplicate keys in map literals and map patterns.
- `break` and `continue` outside loop bodies, with lambda bodies starting a
  fresh non-loop control context.

Grounded AST also records method tail-output metadata:

- `Implicit` means the method has a final expression and no explicit
  current-frame `#stdout` use in that callable.
- `DisabledByStdout` means current-frame `#stdout` is explicitly referenced in
  that callable. Nested lambdas and handlers do not disable the outer callable.
- `None` means there is no expression tail to lower as implicit stdout.

## Canonical Lowering Targets

The first IR should model these source forms as canonical operation structures:

```text
foo(a, b)                 -> @call.forward(foo, [a, b])
| gh pr view {number}     -> @call.process("gh", ["pr", "view", number])
text.trim()               -> @call.self(text, @string.trim, [])
a >> b                    -> @stream.flow(a, b)
a << b                    -> @stream.flow(b, a)
long stream chain         -> @stream.pipeline([...])
@type.string(call)        -> call + stdout absorption + conversion
let x := call             -> call + stdout readonly stream view
@stream.dupe(call)        -> call + writable stream materialization
@call.exit(value, event?) -> value to #stdout + event-or-ok to #frame + stop
```

These are lowering shapes, not a promise that source must be rewritten into
literal `@` calls before IR construction. IR may build the canonical structure
directly from Raw/Grounded nodes.

## Frame Boundaries

Lowering must preserve callable-frame boundaries:

- Method bodies are operation frames.
- External process nodes instantiate operation frames.
- Tool and builtin calls instantiate operation frames unless their optimized
  lowering says otherwise.
- Lambda and handler bodies are independent callable frames.
- `match`, `if`, `for`, `while`, and `with env` blocks are same-frame control
  structure unless they contain nested callables.

Current-frame channels are lexical with respect to the callable frame:

- `#stdin`, `#stdout`, `#stderr`, and `#frame` inside a handler refer to the
  handler frame.
- Target-call streams passed to `@call.stdio(...)` and `@call.completion(...)`
  handlers are ordinary parameters.
- `#frame` is an event stream, not a frame object.

## Runtime Work Still Open

Grounded AST closure does not settle these runtime decisions:

- How stream scheduling prevents deadlock between a target call and its handler.
- When `:= call` observes or propagates the backing call completion.
- Buffering, replay, and spooling policy for stream views.
- Exact optimized execution strategy for `@type.*(...)`.
- Concrete built-in namespace implementations.
- Cleanup ordering beyond the source-visible frame event shape.
- Whether common completion-plus-stdout workflows get additional helper names.

IR work should start by consuming the guarantees above. If an IR design needs a
new source distinction that is not listed here, that is a signal to reopen the
Raw/Grounded boundary explicitly rather than smuggling runtime assumptions into
lowering.
