# Seal Calls

Seal has three source call forms. They should be visually distinct because they
come from different providers, but they all lower toward function-valued
callables that instantiate operation frames.

## Method calls

Methods are Seal-owned operations. Calls always use parentheses, even when no
arguments are passed. This keeps method calls distinct from external binaries.

```seal
method release(channel, watch = false) {
  validate_release_channel(channel)
  build(channel)
  publish(channel, watch)
}

method validate_release_channel(channel) {
  if channel != "stable" && channel != "beta" {
    fail("invalid release channel: {channel}", code: 2)
  }
}

release("beta", watch: true)
```

## External Process Calls

External programs use a leading `|` marker. The marker reads as an external
process node, not as a pipeline operator. Stream flow uses `>>` and `<<`.

```seal
| git status
| cargo test --locked --workspace
| kubectl apply -f "deploy.yaml"
| ssh deploy "systemctl restart runseal"
```

Variables in argv position use `{expr}` interpolation. Bare command words remain
literal argv tokens.

```seal
let ref = "main"
let workflow = "release-beta.yml"

| gh workflow run {workflow} --ref {ref} -f "ref={ref}"
```

The explicit metaprogramming/debug form is `@call.process(...)`.

```seal
| gh pr view {number} --json number,url

@call.process("gh", ["pr", "view", number, "--json", "number,url"])
```

## Tool calls

Runseal tools are first-class `@` calls, not disguised external binaries. The
current CLI path `runseal @tool github issue comment create ...` maps to a
structured Seal call.

```seal
@github.issue.comment.create(
  repo: "PerishCode/runseal",
  number: 49,
  body_file: "body.md",
  body_max: 0,
  prefix_enable: true,
)
```

The `@` namespace is function-call shaped only. It has two valid categories:
regular methods whose behavior can be modeled by composing runtime primitives,
and optimized built-ins that the runtime may implement directly. It should not
grow semantic glue such as `@frame.*`; frame structure belongs to `#` streams.

Tool and built-in calls can be used as statements when the effect matters, or as
expressions when their return value is needed.

```seal
let exists = @process.exists("git")

if !exists {
  fail("missing required tool: git")
}

@fs.mkdir(".git/hooks", mode: 700)
```

## Function-valued callables

Methods, external process calls, and `@` built-ins all become callable values at
the runtime model layer. A direct call is the surface form of raw execution:
evaluate the callable with its actual arguments, create the operation frame, and
apply the default completion policy.

```seal
method current_branch() {
  | git branch --show-current
}

let branch_reader = current_branch
let branch = @type.string(branch_reader())
```

The equivalent metaprogramming/debug form is deliberately more explicit.

```seal
@call.forward(branch_reader, [])
```

`@call.forward(branch_reader, [])` is semantically equivalent to
`branch_reader()`. `@call.process("git", ["branch", "--show-current"])` is
semantically equivalent to `| git branch --show-current`. The second argument in
both explicit forms is an ordinary argument bundle array, not a frame variable.
Named arguments can be carried as ordinary map values when needed; the
cold-start model does not need separate named-argument forwarding syntax.

The cold-start surface does not need heavy function syntax, but the model should
allow function values because guards, cleanup callbacks, and callable frame
expansion all depend on the same idea.

## Complete shape

```seal
method main() {
  let channel = $RUNSEAL_CHANNEL ?? "beta"
  let workflow = match channel {
    "stable" => "release-stable.yml"
    "beta" => "release-beta.yml"
    _ => fail("invalid release channel: {channel}", code: 2)
  }

  | git --version
  | gh auth status

  @github.workflow.run(
    workflow: workflow,
    ref: "main",
    fields: {
      ref: "main",
      version_override: "",
    },
  )
}
```
