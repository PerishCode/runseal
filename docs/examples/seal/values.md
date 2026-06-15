# Seal Values And Collections

Cold-start Seal does not use author-facing type annotations. Values still have
runtime types, and invalid operations fail fast.

## Runtime values

Seal starts with these value kinds:

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

`null` is the only missing or undefined value. It is not an empty string, false,
zero, an empty array, or an empty map.

`stream` is a process IO resource, not a business value. It is useful only while
moving data or when explicitly converted through a built-in such as
`@type.string`, `@type.bytes`, `@type.array`, or `@type.map`.

`byte` is a scalar byte value. `bytes` is a finite byte sequence value. `stream`
is the FIFO delivery form.

`function` is a callable value. Methods, binary-backed invocations, and
optimized `@` built-ins all lower toward function values that instantiate
operation frames. Cold-start Seal does not need author-facing function type
annotations, but the runtime model should treat functions as values so frame
guards, cleanup callbacks, and callable forwarding can share one mechanism.

```seal
let version = null
let tools = ["git", "gh", "cargo"]
let archive = @type.bytes {
  | tar -czf - "./dist"
}
let release = {
  channel: "beta",
  ref: "main",
  watch: true,
  labels: ["release", "beta"],
}
```

Function values can be named and called. This is a model sketch, not a claim
that the first parser must expose every function expression form. Direct calls
are the surface form of raw execution.

```seal
method current_branch() {
  | git branch --show-current
}

let reader = current_branch
let branch = @type.string(reader())
```

The equivalent metaprogramming/debug form can be written with an explicit
argument array when the model needs to show the underlying call shape.

```seal
@call.forward(reader, [])
```

The second argument is an ordinary argument bundle array. It is not a special
current frame variable. Named arguments can be represented by map values rather
than a separate forwarding form.

Collections can nest without generic type syntax.

```seal
let matrix = [
  {
    repo: "runseal",
    checks: ["fmt", "test", "flavor"],
  },
  {
    repo: "flavor",
    checks: ["test"],
  },
]

for item in matrix {
  for check in item.checks {
    run_check(item.repo, check)
  }
}
```

## Strict equality

`==` and `!=` use strict equality. Seal does not do implicit type conversion.

```seal
1 == 1          // true
"1" == "1"      // true
1 == "1"        // false
true == "true"  // false
null == null    // true
```

Streams cannot be compared or implicitly converted to other values.

`??` only handles `null`.

```seal
let channel = $RUNSEAL_CHANNEL ?? "beta"
let version = release.version ?? null
```

These values are not null and are not replaced by `??`.

```seal
let empty_text = ""
let disabled = false
let zero = 0
let empty_array = []
let empty_map = {}
```

## Explicit conversion

Environment variables are strings. Convert explicitly when the workflow needs a
different type.

```seal
let port = @type.int($PORT ?? "8080")
let dry_run = @type.boolean($DRY_RUN ?? "false")
let attempt_limit = @type.int($ATTEMPT_LIMIT ?? "6")
```

`if` requires a boolean. It does not use truthy or falsy conversion.

```seal
let dry_run = @type.boolean($DRY_RUN ?? "false")

if dry_run {
  print_plan()
}
```

Boolean operators only accept boolean values, but expressions can be composed
directly when conversion is explicit.

```seal
if @type.boolean($DRY_RUN ?? "false") || dry_run {
  print_plan()
}
```

## Null guards

Environment reads return `string | null`. Map key access returns the value or
`null` when the key is missing.

```seal
let token = $GITHUB_TOKEN
let version = release.version
```

Use `require(value, message)` when `null` should stop the workflow. It returns
the original value when the value is not null.

```seal
let token = require($GITHUB_TOKEN, "missing GITHUB_TOKEN")
let channel = require(release.channel, "missing release channel")
```

## Membership

`in` supports array membership and map key checks.

```seal
if branch in ["main", "master"] {
  fail("refusing protected branch: {branch}")
}

if "labels" in release {
  for label in release.labels {
    @github.issue.label.add(label)
  }
}
```
