# Seal Terminology And Lowering Model

This file defines the words used by the target syntax examples. It is a
normalization layer for discussion, not a parser or runtime specification.

The goal is to keep syntax design grounded in one model:

```text
callable -> call expression -> operation frame -> completion value
```

## Callable

A callable is any value that can be called.

Examples:

- Seal method
- external binary adapter
- optimized `@` built-in
- lambda / function value

Use `callable` for this layer. Avoid using `command`, `process`, or `function`
when the point is simply "something that can be called".

```seal
method current_branch() {
  | git branch --show-current
}

let reader = current_branch
let branch = @type.string(reader())
```

## Call Expression

A call expression is a call intent: callable plus argument bundle.

Daily syntax:

```seal
foo(a, b, c)
```

Explicit metaprogramming/debug form:

```seal
@call.forward(foo, [a, b, c])
```

These are semantically equivalent. `@call.forward(...)` does not add completion
handling, stream routing, retry, recovery, or any special failure policy.

The second argument is an ordinary argument bundle array. Named arguments are
not part of the call model; use map values when structured options are needed.

```seal
@call.forward(deploy, ["prod", { dry_run: true }])
```

External process calls use a leading `|` source marker. The marker lowers to
`@call.process(...)`.

```seal
| gh pr view {number} --json number,url

@call.process("gh", ["pr", "view", number, "--json", "number,url"])
```

Array spread remains available in process argv position.

```seal
let args = ["workflow", "run", workflow, "--ref", ref]
| gh *args
```

`@call.forward(foo, args)` consumes the whole argument bundle directly, so it
does not need spread syntax. `@call.process(program, args)` also consumes the
whole process argv bundle directly.

## Interpolation

Seal keeps interpolation narrow. Process argv interpolation uses `{expr}` only.

```seal
| gh workflow run {workflow} --ref {ref} -f "ref={ref}"
```

This keeps call expression parsing bounded:

- outside `{...}`, command words are argv tokens
- inside `{...}`, Seal parses an expression
- `$NAME` remains environment access, not local variable interpolation
- no additional shell-style expansion is implied

## Operation Frame

An operation frame is the runtime instance created when a call expression is
executed.

The frame owns streams and lifecycle state:

```text
operation frame {
  stdin
  stdout
  stderr
  frame
  completion
  cleanup
}
```

Inside ordinary code, `#<word>` refers to the current operation frame's stream or
channel.

```seal
#stdin
#stdout
#stderr
#frame
```

`#frame` is the current frame's control/event stream. It is not an object handle,
capability object, or `@frame.*` namespace.

## Stdio Scope

`@call.stdio(...)` executes a call expression and exposes the target call's
standard IO streams to a handler.

```seal
@call.stdio(
  @call.process("cargo", ["test", "--locked", "--workspace"]),
  (stdin, stdout, stderr) => {
    stdout >> @file.write("target/test.log")
    stderr >> @file.write("target/test.err")
  },
)
```

The handler receives the target call's streams as ordinary parameters. The
handler's own `#stdin`, `#stdout`, `#stderr`, and `#frame` still refer to the
handler frame.

`@call.stdio(...)` uses the default completion policy: non-ok completion
propagates according to normal wrapper rules.

## Completion Scope

`@call.completion(...)` executes a call expression and returns its completion as
ordinary data.

```seal
let completion = @call.completion(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr, frame) => {
    stderr >> #stderr
  },
)
```

The handler receives the target call's `stdin`, `stdout`, `stderr`, and `frame`
streams as ordinary parameters. Its own `#frame` still belongs to the handler
frame.

Use this when failed/faulted/cancelled completion is part of the workflow's
decision logic.

```seal
match completion {
  { status: "ok" } => {
    use_existing_pr()
  }
  { status: "failed", exit: exit } => {
    create_pr()
  }
  { status: "faulted", faults: faults } => {
    @exception.raise({
      kind: "child-fault",
      cause: faults,
    })
  }
}
```

## Completion Value

A completion value records how an operation frame ended. It is ordinary Seal
data, not a stream.

First-pass shape:

```seal
{
  status: "ok" | "failed" | "faulted" | "cancelled",
  exit: null | {
    code: int,
    signal: string | null,
  },
  faults: [],
  cancelled: null | {
    source: string,
    signal: string | null,
  },
}
```

Completion values intentionally do not contain stdout or stderr. Route IO
through `@call.stdio(...)` or `@call.completion(...)` callback parameters.

## Handler

A handler is the callback passed to `@call.stdio(...)` or
`@call.completion(...)`.

It should be read as a routing/effect setup scope, not as a normal business
callback. Runtime scheduling must allow the target call and stream handling to
make progress without deadlocking.

```seal
(stdin, stdout, stderr) => {
  stdout >> @file.write("target/out.log")
  stderr >> @file.write("target/err.log")
}
```

The stream parameters are capability-like values. They should not be compared,
serialized, or implicitly converted to strings.

## Stdout Entrypoints

Seal has three stdout entrypoints.

Value conversion:

```seal
let value = @type.map(call)
```

Daily source can pass a process node or stream graph as a block argument. This
keeps effect boundaries explicit without forcing every example into
`@call.process(...)`.

```seal
let value = @type.map {
  | gh pr view {number} --json number,url
}
```

This lowers to the same value-conversion path as `@type.map(call)`.

When the argument is a call expression, process node, or stream graph, optimized
`@type.*` helpers run it, absorb stdout, wait for completion, quick-fail on
non-ok completion, convert stdout, and return the value.

Readonly stream view:

```seal
let raw := call
```

This binds the call's stdout as a readonly stream view. It is not a writable
clone.

Writable stream materialization:

```seal
let mutable = @stream.dupe(call)
```

This fully reads stdout and writes the bytes into a new writable stream.

## Stream Views

Stream copies are readonly views over underlying stream data. Think slice-like
read handles, not writable clones.

If a workflow needs rewritten content, create a new stream and write the
transformed content into that stream explicitly.

## Call Lowering Summary

```text
foo(a, b)
  -> call expression

@call.forward(foo, [a, b])
  -> same call expression, explicit form

| gh pr view {number}
  -> external process call expression

@call.process("gh", ["pr", "view", number])
  -> same external process call expression, explicit form

a >> b
  -> stream flow from a.#stdout to b.#stdin

b << a
  -> stream flow from a.#stdout to b.#stdin, mirror spelling

@call.stdio(call, handler)
  -> call expression + stdio routing + default completion propagation

@call.completion(call, handler)
  -> call expression + stdio/frame routing + completion value

@type.map(call)
  -> call expression + stdout absorption + quick-fail + conversion

let x := call
  -> call expression + stdout readonly view

@stream.dupe(call)
  -> call expression + stdout materialization into a new writable stream
```

## Chaining Sugar

Completion chaining is built-in sugar over the same completion value.

```seal
@call.completion(@call.forward(foo, args), (stdin, stdout, stderr, frame) => {
  stderr >> #stderr
})
  .ok((completion) => {
    use_result()
  })
  .failed((exit, completion) => {
    handle_failed_exit(exit)
  })
  .faulted((faults, completion) => {
    handle_faults(faults)
  })
  .cancelled((cancelled, completion) => {
    handle_cancel(cancelled)
  })
  .always((completion) => {
    cleanup()
  })
```

This lowers to `match completion { ... }`. `.failed(...)` handles
completed-but-failed operations. `.faulted(...)` handles Seal/runtime faults.
They should not collapse into one catch-all branch.
