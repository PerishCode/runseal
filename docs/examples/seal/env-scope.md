# Seal Environment And Scope

Environment access is a primitive process-boundary concept. Seal variables and
process environment are related, but not the same namespace.

## Reading environment

```seal
let token = $GITHUB_TOKEN
let channel = $RUNSEAL_CHANNEL ?? "beta"
let profile = $RUNSEAL_PROFILE_PATH
```

Environment reads return `string | null`. Use `??` when the wrapper owns a
default, and `require(...)` when the wrapper needs a hard precondition.

```seal
let ref = $RUNSEAL_REF ?? "main"
let token = require($GITHUB_TOKEN, "missing GITHUB_TOKEN")
```

## Temporary environment

Temporary environment is scoped with `with env { ... } { ... }`. The first block
is pure environment binding: no commands, no control flow, no side effects.

```seal
with env {
  RUST_LOG = "debug"
  RUNSEAL_CHANNEL = channel
} {
  | cargo test --locked --workspace
}
```

The injected values apply only to process nodes and tool calls inside the block.

```seal
method publish(channel) {
  with env {
    RUNSEAL_CHANNEL = channel
  } {
    @cloudflare.api.request("GET", "/zones", query: ["per_page=50"])
    | gh workflow run "release-beta.yml" --ref "main"
  }
}
```

Config paths and values use the same environment projection model.

```seal
let configs = @fs.list($PERISH_TOP_KUBE_DIR, glob: "*.yaml", files: true)
let kubeconfig = @string.join(configs, separator: "path")

with env {
  KUBECONFIG = kubeconfig
} {
  | kubectl config current-context
  | kubectl apply -f "deploy.yaml"
}
```

## Variables and block scope

`let` declares a variable in the current block. Assignment updates the nearest
visible variable.

```seal
method prepare() {
  let root = @type.string {
    | git rev-parse --show-toplevel
  }

  if root == "" {
    fail("not inside a git repository")
  }

  {
    let hooks_dir = "{root}/.git/hooks"
    @fs.mkdir(hooks_dir, mode: 700)
  }

  // hooks_dir is not visible here.
}
```

Cold-start Seal does not use type annotations. Runtime values still have
concrete types, and mismatched operations fail fast.

```seal
let dry_run = false
let attempts = 6
let body_file = "body.md"
```
