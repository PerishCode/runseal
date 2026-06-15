# Seal Full Stream Model Sketch

This file intentionally sketches a maximal stream model before surface syntax is
compressed. It is not a final author-facing style.

The core model is:

- every effectful scope owns predefined `#stdin`, `#stdout`, `#stderr`, and
  `#frame` streams
- method bodies, external process nodes, tool calls, stream graphs, and stream-scope blocks
  are all effectful scopes
- methods, external process invocations, and optimized built-ins are all callable
  function values that instantiate operation frames
- stream is a replayable FIFO byte buffer in the runtime/IR layer
- stream copies are readonly views over underlying stream data, not writable
  clones
- `#<word>` consistently denotes a current-frame stream or channel
- stream modeling is expected to be common in IR and sparse in source
- source syntax should expose intent; IR should preserve stream mechanics

## Scope-local stdio

Every effectful scope has its own stream context:

```text
scope {
  #stdin: stream
  #stdout: stream
  #stderr: stream
  #frame: stream
}
```

A nested callable inherits the current scope streams unless it declares a
different policy.

```seal
method test() {
  | cargo test --locked --workspace
}
```

Expanded model:

```text
test.#stdin  <- caller.#stdin
test.#stdout -> caller.#stdout
test.#stderr -> caller.#stderr
test.#frame  -> test executor

cargo.#stdin  <- test.#stdin
cargo.#stdout -> test.#stdout
cargo.#stderr -> test.#stderr
cargo.#frame  -> cargo executor
```

`#frame` is the current operation frame's control/event stream. The executor
owns the policy for events written to that stream, such as an exit event or a
structured fault event. This keeps frame lifecycle mechanics in the same `#`
stream family as stdio instead of introducing an `@frame.*` tool namespace.

## Full stdio scope

The maximal source sketch uses `@call.stdio(call, handler)` to expose a call's
standard IO streams explicitly. `#<word>` remains only the current frame's
stream/channel form; the target call's streams are passed to the handler as
ordinary parameters.

```seal
let outer_out = #stdout
let outer_err = #stderr

@call.stdio(
  @call.process("gh", ["api", "repos/PerishCode/runseal/issues"]),
  (stdin, stdout, stderr) => {
    @text.stream(payload) >> stdin

    stdout >> @file.write("target/issue.json")
    stdout >> outer_out
    stderr >> @file.write("target/issue.err")
    stderr >> outer_err
  },
)
```

Interpretation:

- `@call.stdio(...)` creates the callable's operation frame with default
  completion propagation
- `| gh ...` lowers to `@call.process("gh", [...])`, whose second argument is
  an ordinary array of actual arguments
- the handler receives the target call's `stdin`, `stdout`, and `stderr` streams
  as ordinary parameters
- `@text.stream(payload) >> stdin` provides the callable's stdin stream
- omitting writes to `stdin` provides an empty stdin stream
- inside the handler, `#stdin`, `#stdout`, `#stderr`, and `#frame` still refer to
  the handler lambda's own frame, not the target call frame
- stream values are replayable, so multiple consumers can read independent
  readonly views of `stdout` or `stderr`

Forwarding to the caller requires capturing the outer streams before entering
the stdio routing scope.

## Outer stream capture

This example isolates the forwarding pattern:

```seal
let outer_out = #stdout
let outer_err = #stderr

@call.stdio(
  @call.process("gh", ["api", "repos/PerishCode/runseal/issues"]),
  (stdin, stdout, stderr) => {
    @text.stream(payload) >> stdin

    stdout >> @file.write("target/issue.json")
    stdout >> outer_out
    stderr >> @file.write("target/issue.err")
    stderr >> outer_err
  },
)
```

This is intentionally explicit. Final syntax can shrink the common inherit case
without changing the model.

## Empty stdin for ordinary commands

Traditional stdout/stderr splitting becomes a stream scope with empty stdin.

```seal
@call.stdio(
  @call.process("cargo", ["test", "--locked", "--workspace"]),
  (stdin, stdout, stderr) => {
    stdout >> @file.write("target/test.log")
    stderr >> @file.write("target/test.err")
  },
)
```

Omitting writes to `stdin` means the callable receives no stdin input.

## Capturing stdout with quick-fail sugar

The full stream model can convert stdout through an explicit type shell:

```seal
@call.stdio(
  @call.process("gh", [
    "run", "list",
    "--workflow", workflow,
    "--branch", ref,
    "--limit", "1",
    "--json", "databaseId",
  ]),
  (stdin, stdout, stderr) => {
    let runs = @type.array(stdout)
    stderr >> @file.write("target/gh-run-list.err")
  },
)
```

`@type.array(stdout)` converts the stdout stream into a Seal array. It decodes
JSON bytes and requires the top-level decoded value to be an array.

The daily value syntax is an optimized `@type.*` call. When its argument is a
process node or stream graph, the built-in runs it, absorbs stdout, waits for
completion, quick-fails on non-ok completion, converts stdout, and returns the
Seal value.

```seal
let runs = @type.array {
  | gh run list --workflow {workflow} --branch {ref} --limit 1 --json databaseId
}
```

When stderr routing is needed, use the explicit `@call.stdio(...)` form above
until a smaller routing sugar is chosen.

The value sugar means:

- create a callable scope for the right side
- provide empty stdin unless a fuller call form says otherwise
- capture the callable stdout stream
- wait for stdout EOF
- wait for the callable frame completion after the stream scope leaves
- fail if the callable completion is not successful
- fail if conversion fails
- bind the converted value

Use `:=` when process stdout should be bound as a readonly stream view
instead of converted immediately.

```seal
let raw := {
  | gh run list --workflow {workflow} --branch {ref} --limit 1 --json databaseId
}
```

Use `@stream.dupe(...)` when the workflow needs a new writable stream with the
process stdout fully materialized into it.

```seal
let mutable = @stream.dupe {
  | gh run list --workflow {workflow} --branch {ref} --limit 1 --json databaseId
}
```

## Method stdio contract

Seal methods can also be captured because a method call is an effectful scope.

```seal
method current_branch() {
  | git branch --show-current
}

let branch = @type.string(current_branch())
```

Expanded model:

```text
current_branch.#stdout is fed by git.#stdout
@type.string consumes a replayable readonly view of current_branch.#stdout
branch is bound after EOF and successful conversion
```

## Forwarding streams through a method

```seal
method pretty_json() {
  let outer_in = #stdin
  let outer_out = #stdout
  let outer_err = #stderr

  @call.stdio(
    @call.forward(@json.pretty, []),
    (stdin, stdout, stderr) => {
      outer_in >> stdin
      stdout >> outer_out
      stderr >> outer_err
    },
  )
}
```

This sketch is intentionally mechanical. It means the nested tool is connected
to the method's stream context. A final surface form should be smaller.

## Pipeline plus capture

```seal
let branch = @type.string {
  | git branch --format "%(refname:short)" >>
  | grep "^feat/" >>
  | head -n 1
}
```

The body of the type block is the whole pipeline. The pipeline is a stream
graph, not a value expression chain. Each `|` command is an external process
node. `>>` connects the left node's stdout to the right node's stdin. Each stage
has local `#stdin`, `#stdout`, and `#stderr`; each stage also has its own
`#frame` control/event stream.

## Replayable stream consumption

```seal
@call.stdio(
  @call.process("gh", [
    "run", "list",
    "--workflow", workflow,
    "--branch", ref,
    "--limit", "1",
    "--json", "databaseId",
  ]),
  (stdin, stdout, stderr) => {
    stdout >> @file.write("target/runs.json")

    let runs = @type.array(stdout)
    let raw = @type.string(stdout)
  },
)
```

Each consumer reads its own readonly view of the same FIFO byte stream. Runtime
may buffer in memory, spool to disk, and release segments when all consumers are
done. These views are slice-like read handles, not writable clones. A workflow
that needs rewritten content should create a new stream and write the transformed
content into that stream explicitly.

## Infinite streams

```seal
let logs := {
  | kubectl logs deploy/app --follow
}
```

This binds a readonly stream view and does not wait for EOF by itself.
Converting it with `@type.string(logs)` would wait for EOF. Long-running streams
are an operator concern: interrupting the wrapper should propagate cancellation
to child calls and clean up stream resources. The cold-start syntax does not try
to make infinite streams safe.
