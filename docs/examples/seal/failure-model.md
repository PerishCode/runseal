# Seal Full Failure Model Sketch

This file intentionally sketches a maximal failure model before surface syntax
is compressed. It is not a final author-facing style.

The current direction is to model failure as part of operation frame lifecycle,
not as a separate `try/catch/finally` language family. The same frame model must
cover methods, external binaries, and optimized `@` built-ins.

The goal is to separate concepts that shell and common exception syntax often
mix together:

- operation frame lifecycle
- frame control events
- effect completion
- command exit code
- stream conversion failure
- Seal semantic exception
- operator cancellation
- cleanup / scope finalization

## Frame streams

`#<word>` consistently denotes a current-frame stream or channel.

```text
frame {
  #stdin: stream
  #stdout: stream
  #stderr: stream
  #frame: stream
}
```

`#frame` is the current operation frame's control/event stream. The executor
consumes this stream and owns the lifecycle policy for events written to it.

```seal
{
  type: "exit",
  code: 0,
} >> #frame
```

A structured fault event can also be written to `#frame`.

```seal
{
  type: "fault",
  fault: {
    kind: "shape",
    message: "expected release.version to be a string",
    data: {},
  },
} >> #frame
```

This keeps lifecycle control inside the `#` stream family. Do not introduce
`@frame.*`; the `@` namespace remains function-call shaped.

## Function calls and call-domain helpers

Methods, binaries, and optimized built-ins are all callable values at the model
layer. Direct calls are the daily surface. `@call.forward(...)` is the equivalent
metaprogramming/debug form.

```seal
method current_branch() {
  | git branch --show-current
}

let reader = current_branch
let branch = @type.string(reader())
```

The explicit call-forwarding form accepts a callable and an ordinary array of
actual arguments.

```seal
@call.forward(reader, [])
```

`@call.forward(reader, [])` is semantically equivalent to `reader()`. The second
argument is an ordinary argument bundle array, not a built-in frame variable.
Named arguments can be represented with map values instead of a separate
forwarding syntax.

External process syntax has the same explicit/debug relationship:

```seal
| gh pr view {number}

@call.process("gh", ["pr", "view", number])
```

The callable's operation frame owns `#stdin`, `#stdout`, `#stderr`, and
`#frame`. Ordinary source syntax can hide most of this, but the full model
should not.

## Failure domains

```text
completed ok
  the frame finished successfully

completed failed
  the frame finished, but the operation failed as a domain/process result

faulted
  the frame could not continue under Seal semantics, or a structured exception
  event was written to #frame

cancelled
  the frame was interrupted by the operator or a parent runtime cancellation

cleanup
  functions registered with the frame run because the scope is leaving,
  regardless of completed/failed/faulted/cancelled
```

These domains should not collapse into `try/catch/finally`.

## Frame event protocol v0

Executor-recognized writes to `#frame` use map events.

```seal
{
  type: "exit",
  code: 0,
} >> #frame

{
  type: "failed",
  code: 2,
  message: "invalid release channel",
  data: {},
} >> #frame

{
  type: "fault",
  fault: {
    kind: "shape",
    message: "expected release.version to be a string",
    data: {},
  },
} >> #frame

{
  type: "cancelled",
  source: "operator",
  signal: "interrupt",
} >> #frame

{
  type: "cleanup",
  run: () => {
    @file.remove(tmp)
  },
} >> #frame
```

The executor owns the lifecycle effect of these events. User code writes events;
it does not call an `@frame.*` control API.

## Completion value v0

`@call.completion(...)` returns a structured completion value. The value is
ordinary data and can be matched directly.

```seal
{
  status: "ok",
  exit: {
    code: 0,
    signal: null,
  },
  faults: [],
  cancelled: null,
}
```

```seal
{
  status: "failed",
  exit: {
    code: 1,
    signal: null,
  },
  faults: [],
  cancelled: null,
}
```

```seal
{
  status: "faulted",
  exit: null,
  faults: [
    {
      kind: "shape",
      message: "expected release.version to be a string",
      data: {},
    },
  ],
  cancelled: null,
}
```

```seal
{
  status: "cancelled",
  exit: null,
  faults: [],
  cancelled: {
    source: "operator",
    signal: "interrupt",
  },
}
```

This shape intentionally keeps stdout and stderr out of the completion value.
Call IO is routed through the callback parameters. Completion records how the
operation ended.

## Completion is after-frame state

Completion is not a live stream inside the child frame body. The callback passed
to `@call.completion(...)` can route the callable's live streams through
parameters, but the returned completion value only exists after that frame
leaves.

```seal
let outer_err = #stderr

let completion = @call.completion(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr, frame) => {
    stderr >> outer_err
  },
)

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
  { status: "cancelled" } => {
    @exception.raise({
      kind: "cancelled",
      message: "operator cancelled child frame",
    })
  }
}
```

This is the full-model shape for catch-like behavior: enter a child operation
frame, route its streams, then observe its completion from the parent layer.

## Completion chaining sugar

Promise-like chaining is allowed as built-in sugar over the same completion
value. It is not a separate exception system.

```seal
@call.completion(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr, frame) => {
    stderr >> outer_err
  },
)
  .ok((completion) => {
    use_existing_pr()
  })
  .failed((exit, completion) => {
    create_pr()
  })
  .faulted((faults, completion) => {
    @exception.raise({
      kind: "child-fault",
      cause: faults,
    })
  })
  .cancelled((cancelled, completion) => {
    @exception.raise({
      kind: "cancelled",
      cause: cancelled,
    })
  })
  .always((completion) => {
    @file.remove(tmp)
  })
```

Chaining lowers to `match completion { ... }`. `.failed(...)` handles
completed-but-failed operations; `.faulted(...)` handles Seal/runtime faults.
The two should not collapse into one catch-all branch.

## Quick-fail default

Ordinary effect execution quick-fails when the child frame does not complete
successfully.

```seal
| gh auth status
| git push -u origin {branch}
```

Expanded model:

```text
create child operation frame
inherit or route child stdio
wait for child frame completion
if completion is completed ok:
  continue
if completion is completed failed:
  write diagnostic/control event to current #frame
if completion is faulted:
  write structured fault event to current #frame
if completion is cancelled:
  write cancellation event to current #frame
```

When a workflow expects failure as data, it should observe completion instead of
letting the default quick-fail policy run.

## Operational fail as a helper

Many ordinary control-flow examples use `fail(...)` for operator-facing workflow
failure.

```seal
fail("invalid release channel: {channel}", code: 2)
```

This should be treated as a normal helper function, not as a special `@frame`
namespace and not as a Seal semantic exception. Conceptually it writes a failed
completion event to the helper frame's current `#frame` and then exits the
helper frame. The caller then sees the helper's failed completion through the
normal frame propagation policy.

```seal
{
  type: "failed",
  code: 2,
  message: "invalid release channel: {channel}",
  data: {},
} >> #frame

{
  type: "exit",
  code: 2,
} >> #frame

return null // unreachable
```

This is distinct from `@exception.raise(...)`, which produces a fault event.

## Stream conversion failure

Optimized `@type.*` helpers combine effect completion, stdout EOF, conversion,
and binding when their argument is a process node or stream graph.

```seal
let pr = @type.map {
  | gh pr view {number} --json number,url,isDraft
}
```

Expanded model:

```text
create child operation frame with empty stdin
capture child #stdout
inherit or route child #stderr
wait for stdout EOF
wait for child frame completion
if completion is not completed ok:
  write diagnostic/control event to current #frame
convert stdout bytes with @type.map
if conversion fails:
  write structured fault event to current #frame
bind pr
```

Conversion failure is not the same thing as command exit failure. Both can
become current-frame control events, but they come from different model layers.
Use `:=` only when stdout should be bound as a readonly stream view.

## Exception raise as a function call

`@exception.raise(...)` can remain valid because it is function-call shaped. It
is not an `@frame.*` control namespace. Its behavior can be modeled as a regular
method or optimized built-in that writes to the current `#frame` stream and then
quick-returns through its own frame.

```seal
@exception.raise({
  kind: "shape",
  message: "expected release.version to be a string",
})
```

Conceptual expansion:

```seal
{
  type: "fault",
  fault: {
    kind: "shape",
    message: "expected release.version to be a string",
    data: {},
  },
} >> #frame

{
  type: "exit",
  code: 1,
} >> #frame

return null // unreachable
```

The exact event shape is not final. The important boundary is that raising an
exception is a function call whose implementation composes frame streams; frame
itself is not exposed as an `@frame` function namespace.

## Guarding a stream

Catch-like behavior can also be modeled as guarding a stream and writing
recognized fault events to a target frame stream.

```seal
let outer_frame = #frame

@call.completion(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr, frame) => {
    @exception.guard(outer_frame, stderr)
  },
)
```

`outer_frame` is not a frame object. It is the outer `#frame` stream captured as
a value. A guard can be modeled as an ordinary function that receives a target
frame stream and a source stream.

```seal
method guard(target_frame, source) {
  while true {
    let event = @stream.read(source)

    if @exception.matches(event) {
      {
        type: "fault",
        fault: {
          kind: "guarded-stream",
          message: "guarded stream emitted an exception event",
          data: {
            cause: event,
          },
        },
      } >> target_frame

      {
        type: "exit",
        code: 1,
      } >> #frame
    }
  }
}
```

This sketch intentionally ignores EOF and scheduling details. The point is that
guarding is stream composition plus frame control events, not a special
`try/catch` primitive.

## Cleanup as frame data

Cleanup belongs to frame lifecycle, not to exception handling. Once function is
a runtime value, cleanup can be modeled as a function value registered with the
current frame.

```seal
let tmp = @file.temp()

{
  type: "cleanup",
  run: () => {
    @file.remove(tmp)
  },
} >> #frame
```

This is a deliberately verbose model shape. Surface syntax can later shrink
cleanup registration, but the executor-level behavior remains frame lifecycle:
cleanup runs when the frame leaves, regardless of success, failure, fault, or
cancellation.

## Full ugly shape

This deliberately verbose sketch shows all pieces at once.

```seal
let tmp = @file.temp()
let outer_err = #stderr
let outer_frame = #frame

{
  type: "cleanup",
  run: () => {
    @file.remove(tmp)
  },
} >> #frame

let completion = @call.completion(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr, frame) => {
    stdout >> @file.write(tmp)
    stderr >> outer_err

    @exception.guard(outer_frame, stderr)
  },
)

match completion {
  { status: "ok" } => {
    let pr = @type.map {
      | cat {tmp}
    }

    if pr.isDraft {
      @exception.raise({
        kind: "draft-pr",
        message: "PR is still draft",
      })
    }
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

  { status: "cancelled" } => {
    @exception.raise({
      kind: "cancelled",
      message: "operator cancelled child frame",
    })
  }
}
```

This is not the desired surface. It is a pressure test for the model. Later
syntax should only hide defaults and common routing patterns; it should not
change the frame/stream lifecycle underneath.
