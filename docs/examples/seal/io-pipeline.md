# Seal IO And Pipelines

Seal keeps process IO visible without turning the source language back into
general shell scripting.

## Stream routing

```seal
@call.stdio(
  @call.process("cargo", ["test", "--locked", "--workspace"]),
  (stdin, stdout, stderr) => {
    stdout >> @file.write("target/runseal-test.log")
    stderr >> @file.write("target/runseal-test.err")
  },
)
```

Stdio routing uses scope-local streams, not shell file descriptor syntax.

```seal
@call.stdio(
  @call.process("gh", ["pr", "checks", number, "--watch", "--interval", "10"]),
  (stdin, stdout, stderr) => {
    stdout >> @file.write("target/pr-checks.log")
    stderr >> @file.write("target/pr-checks.err")
  },
)
```

## Pipelines

Pipelines connect effect scopes: the left stage `#stdout` feeds the right stage
`#stdin`. They do not produce Seal values by themselves.

```seal
| git branch --format "%(refname:short)" >> | grep "^feat/" >> | head -n 1
```

Tool calls can participate when the tool has a stream mode.

```seal
| gh api repos/PerishCode/runseal/actions/runs >> @json.pretty.stdin()
```

## Capturing stdout

There are three stdout entrypoints.

Use optimized `@type.*` helpers when process stdout becomes a Seal value. When
the argument is a process node or stream graph, the helper runs it, absorbs
stdout, waits for completion, quick-fails on non-ok completion, converts stdout,
and returns the value.

```seal
let branch = @type.string {
  | git branch --show-current
}

let runs = @type.array {
  | gh run list --workflow {workflow} --branch {branch} --limit 1 --json databaseId
}

let run_id = require(runs[0].databaseId, "missing run id")
```

Use `:=` when process stdout should be bound as a readonly stream view. This
is copy/view sugar, not a writable clone.

```seal
let raw := {
  | gh run list --workflow {workflow} --branch {branch} --limit 1 --json databaseId
}
let text = @type.string(raw)
```

Use `@stream.dupe(...)` when the workflow needs a new writable stream with the
process stdout fully materialized into it.

```seal
let mutable = @stream.dupe {
  | gh run list --workflow {workflow} --branch {branch} --limit 1 --json databaseId
}
```

Stderr remains independent. When needed, declare stderr routing on the process
call with `@call.stdio(...)`. In the full model, convert the explicit `stdout`
parameter yourself.

```seal
let pr = null

@call.stdio(
  @call.process("gh", ["pr", "view", number, "--json", "number,url,isDraft"]),
  (stdin, stdout, stderr) => {
    pr = @type.map(stdout)
    stderr >> @file.write("target/gh-pr-view.err")
  },
)
```
