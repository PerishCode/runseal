# Seal Perish Workflow Sketches

This file stress-tests the current Seal terminology against real wrapper shapes
from this repository. It is intentionally written as source-feel exploration,
not as an implementation contract.

Read this after [Terminology and lowering model](./semantics.md).

## Release Workflow

```seal
method release(channel, ref = "main", version = "", watch = false, dry_run = false) {
  let workflow = match channel {
    "stable" => "release-stable.yml"
    "beta" => "release-beta.yml"
    _ => fail("release: invalid channel: {channel}", code: 2)
  }

  let command = [
    "workflow", "run", workflow,
    "--ref", ref,
    "-f", "ref={ref}",
    "-f", "version_override={version}",
  ]

  if dry_run {
    print(@text.join(["gh", *command], separator: " "))
    return
  }

  | gh --version
  | gh auth status

  let ref_sha = @type.string {
    | git rev-parse {ref}
  }

  let trigger_text = @type.string {
    | gh *command
  }
  if trigger_text != "" {
    print(trigger_text)
  }

  print("triggered {workflow} for ref {ref}")

  if watch {
    let run_id = @regex.capture(trigger_text, "/actions/runs/([0-9]+)", 1) ?? ""

    if run_id == "" {
      let attempt = 0

      while attempt < 6 && run_id == "" {
        let runs = @type.array(
          @call.process("gh", [
            "run", "list",
            "--workflow", workflow,
            "--branch", ref,
            "--commit", ref_sha,
            "--event", "workflow_dispatch",
            "--limit", "1",
            "--json", "databaseId",
          ])
        )

        if runs != [] {
          run_id = require(runs[0].databaseId, "release: run is missing databaseId")
        } else {
          @time.sleep(2)
          attempt = attempt + 1
        }
      }
    }

    if run_id == "" {
      fail("release: could not find a recent run for {workflow} on {ref}")
    }

    | gh run watch {run_id} --interval 10
  }
}
```

Immediate pressure:

- `| gh *command` keeps dynamic argv ergonomic while still marking the external
  process node.
- `@type.string { | gh *command }` gives command capture an explicit block
  boundary instead of relying on command expressions inside ordinary function
  arguments.
- Multi-line process calls can fall back to `@call.process(program, args)` when
  line-oriented argv syntax gets too dense.

## PR Workflow

```seal
method current_pr(branch, base, title = "", body_file = "", draft = false) {
  let raw = @type.array {
    | gh pr list --head {branch} --json number,title,state,url,isDraft
  }

  if raw != [] {
    return raw[0]
  }

  let args = ["pr", "create", "--base", base, "--head", branch]

  if draft {
    args = @array.push(args, "--draft")
  }

  if title != "" {
    args = @array.push(args, "--title", title)
  }

  if body_file != "" {
    args = @array.push(args, "--body-file", body_file)
  } else {
    args = @array.push(args, "--fill")
  }

  @call.process("gh", args)

  let after = @type.array {
    | gh pr list --head {branch} --json number,title,state,url,isDraft
  }
  if after == [] {
    fail("pr: created PR for {branch}, but could not find it afterward")
  }

  return after[0]
}

method pr(base = "main", draft = false, no_watch = false, no_merge = false, no_push = false, dry_run = false) {
  | git --version
  | gh --version
  | gh auth status

  let branch = @type.string {
    | git branch --show-current
  }

  if branch == "" {
    fail("pr: not on a branch")
  }

  if branch in [base, "main", "master"] {
    fail("pr: refusing to open a PR from base branch: {branch}")
  }

  if draft && !no_merge {
    fail("pr: --draft requires --no-merge")
  }

  if dry_run {
    print_plan(branch, base, draft, no_watch, no_merge, no_push)
    return
  }

  if !no_push {
    | git push -u origin {branch}
  }

  let pr = current_pr(branch, base, { draft: draft })
  let number = require(pr.number, "pr: missing PR number")
  let url = require(pr.url, "pr: missing PR url")
  let is_draft = require(pr.isDraft, "pr: missing PR draft state")

  print("PR #{number}: {url}")

  if is_draft && !draft {
    | gh pr ready {number}
  }

  if !no_watch {
    let probe = @call.completion(
      @github.pr.checks.probe(number),
      (stdin, stdout, stderr, frame) => {},
    )

    probe
      .ok((completion) => {
        | gh pr checks {number} --watch --interval 10
      })
      .failed((exit, completion) => {
        print("no checks reported on PR #{number}; skipping watch")
      })
  }

  if !no_merge {
    | gh pr merge {number} --squash --delete-branch
  }
}
```

Immediate pressure:

- Named args are intentionally omitted. Structured options should use a map
  argument, as in `current_pr(branch, base, { draft: draft })`.
- Building argv arrays is verbose. We likely need ergonomic array helpers or a
  small argv builder, but that can remain an `@` helper.
- `@call.completion(@github.pr.checks.probe(number), ...)` feels correct but
  verbose when no IO routing is needed. A helper for completion-only probing may
  be worth considering later.

## Guard Version Policy

```seal
method guard_version_policy(mode = "full") {
  let public_url = $RUNSEAL_RELEASES_PUBLIC_URL ?? "https://releases.runseal.perish.uk"
  let metadata_url = $RUNSEAL_STABLE_METADATA_URL ?? "{public_url}/stable/latest/metadata.json"

  let tmp_dir = $RUNNER_TEMP ?? ($RUNSEAL_REPO_TMP_DIR ?? ".local/tmp")
  let metadata_file = "{tmp_dir}/runseal-guard-stable-metadata.json"

  @fs.mkdir(tmp_dir, mode: 700)

  let cargo_metadata = @type.map {
    | cargo metadata --no-deps --format-version 1
  }
  let current_version = require(cargo_metadata.packages[0].version, "guard: missing current version")
  let current_hash = @type.string(@hash.tree("app/tests"))

  let completion = @call.completion(
    @call.process("curl", [
      "-sS",
      "-o", metadata_file,
      "-w", "%{http_code}",
      "{metadata_url}?version={current_version}",
    ]),
    (stdin, stdout, stderr, frame) => {},
  )

  let status = @type.string(completion.stdout)
}
```

This sketch intentionally fails: `completion` does not contain stdout by design.
That exposes a useful rule:

- If the workflow needs both stdout and completion, it must capture stdout in
  the handler:

```seal
let status = ""

let completion = @call.completion(
  @call.process("curl", [
    "-sS",
    "-o", metadata_file,
    "-w", "%{http_code}",
    "{metadata_url}?version={current_version}",
  ]),
  (stdin, stdout, stderr, frame) => {
    status = @type.string(stdout)
  },
)
```

This is semantically clean, but visually heavy. A common "completion plus stdout
value" helper may be worth considering later.

## Cloudflare Redirect Rules

```seal
method load_manage_redirect_rules() {
  let zone_name = @cloudflare.config.get("zone_name")
  let request_host = @cloudflare.config.get("manage_host")
  let redirect_host = @cloudflare.config.get("manage_origin_host")
  let prefix = @cloudflare.config.get("manage_redirect_prefix")

  let target_sh = if prefix == "" {
    "https://{redirect_host}/manage.sh"
  } else {
    "https://{redirect_host}/{prefix}/manage.sh"
  }

  let target_ps1 = if prefix == "" {
    "https://{redirect_host}/manage.ps1"
  } else {
    "https://{redirect_host}/{prefix}/manage.ps1"
  }

  let rule_sh = @cloudflare.redirect_rule.exact({
    ref: "runseal_manage_sh_redirect",
    description: "Redirect runseal manage.sh to releases bucket asset",
    host: request_host,
    path: "/manage.sh",
    target_url: target_sh,
  })

  let rule_ps1 = @cloudflare.redirect_rule.exact({
    ref: "runseal_manage_ps1_redirect",
    description: "Redirect runseal manage.ps1 to releases bucket asset",
    host: request_host,
    path: "/manage.ps1",
    target_url: target_ps1,
  })

  return {
    zone_name: zone_name,
    request_host: request_host,
    redirect_host: redirect_host,
    rule_sh: rule_sh,
    rule_ps1: rule_ps1,
  }
}
```

Immediate pressure:

- Map-heavy `@` calls feel much better than argv-heavy tool calls for structured
  operations. This supports keeping complex cross-platform operations in atomic
  tools instead of Seal syntax.
- Inline `if` expressions need to be explicitly allowed or rejected. The control
  flow examples currently show `match` for values, not `if` expressions.

## Sharp Edges Found

1. **Command boundaries are now explicit.**
   `@type.map { | gh pr view ... }` preserves command ergonomics without
   embedding a bare command in ordinary function-call parentheses. The parser
   sees `| <whitespace>` as an external process node and the block as the capture
   boundary.

2. **Array spread belongs in process argv position.**
   `@call.forward(foo, args)` consumes an argument bundle directly. Real wrapper
   code still benefits from `| gh *args` when building dynamic process argv.

3. **Named method args are removed.**
   Use ordinary map values for structured options. This removes a low-value
   second argument system.

4. **Completion plus stdout is verbose, but helper-shaped.**
   `@call.completion(...)` correctly keeps IO out of completion, but common
   cases can add an `@` helper that returns `{ completion, stdout }` without
   weakening the model. This is not a core syntax flaw.

5. **Handlers need a scheduling contract.**
   They are routing/effect setup scopes, not ordinary synchronous callbacks. The
   runtime must prevent deadlocks when callbacks read stdout/stderr while the
   target call is running.

6. **`@type.*(call)` and `:= call` must define when completion is checked.**
   The likely rule is: value conversion waits for EOF and completion;
   readonly stream binding defers EOF/completion until consumption or scope
   finalization.

7. **Map event protocols need constructors.**
   Raw map writes to `#frame` are good for the full model, but helpers that
   construct event maps would prevent typo-heavy source while still avoiding
   `@frame.*` control APIs. This also belongs in the `@` helper layer.
