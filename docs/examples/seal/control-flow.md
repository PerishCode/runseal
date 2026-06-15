# Seal Control Flow

Control blocks use braces. Seal should not inherit `then`/`fi`, `do`/`done`, or
`case`/`esac` surface syntax.

## If / else if / else

```seal
method ensure_release_branch(base) {
  let branch = @type.string {
    | git branch --show-current
  }

  if branch == "" {
    fail("not on a branch")
  } else if branch == base {
    fail("refusing to release from base branch: {branch}")
  } else if branch == "main" || branch == "master" {
    fail("refusing to release from base branch: {branch}")
  }
}
```

## Match

Use `match` for value selection. It replaces shell `case` for ordinary control
flow.

```seal
let workflow = match channel {
  "stable" => "release-stable.yml"
  "beta" => "release-beta.yml"
  _ => fail("invalid choice: {channel}", code: 2)
}
```

Match arms may also run blocks when the branch is effectful.

```seal
match target {
  "macos" => {
    | brew --version
    | ./manage.sh install --channel {channel}
  }
  "linux" => {
    | systemctl --version
    | ./manage.sh install --channel {channel}
  }
  _ => fail("unsupported target: {target}")
}
```

## For

`for` iterates over lists. The loop variable is scoped to the block.

```seal
let required = ["git", "gh", "cargo", "runseal", "flavor"]

for tool in required {
  if !@process.exists(tool) {
    fail("missing required tool: {tool}")
  }
}
```

## While

`while` is useful for bounded polling and retry loops. Operational wrappers
should keep loops finite and visible.

```seal
let attempt = 0
let run_id = ""

while attempt < 6 && run_id == "" {
  let runs = @type.array {
    | gh run list --workflow {workflow} --branch {ref} --limit 1 --json databaseId
  }

  if runs != [] {
    run_id = runs[0].databaseId
  } else {
    @time.sleep(2)
    attempt = attempt + 1
  }
}

if run_id == "" {
  fail("could not find a recent workflow run for {workflow}")
}
```
