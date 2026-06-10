# AGENTS

## 1. AGENTS.md Meta Constraints

This top-level `AGENTS.md` is the repository navigation and policy layer.

- Keep this file focused on shared constraints, navigation, and recurring
  operating guidance.
- Push local implementation detail downward into child `AGENTS.md` files when a
  directory starts carrying its own stable rules.
- Do not duplicate large bodies of module-specific instruction here once a child
  `AGENTS.md` exists.
- Treat this file as the default contract for the whole repository unless a
  deeper `AGENTS.md` overrides a narrower scope.
- Keep `.task/` out of git by default. Use it only for long-running work and
  update it as live task state, not as archival prose.

Core product stance:

- `runseal` exists to reduce environment-dependency complexity in real
  cross-platform operations work: too many environment variables, too many
  machine-specific assumptions, and too much operational glue falling back to
  Python or JavaScript even when the underlying workflow is clear and finite.
- Explicit profile. No hidden orchestration.
- Prefer a stable shared subset of `bash` and PowerShell for `.seal`, plus
  explicit atomic `@tool` capabilities for needs that do not fit that shared
  shell surface cleanly.
- Preserve cross-shell semantic equivalence as the hard constraint. Do not
  require local syntax symmetry or IR-level elegance when a `.seal` behavior is
  still clear, finite, and can be translated reliably into a more awkward
  PowerShell form.
- Keep the Rust core thin and concrete.
- Support only `env`, `symlink`, fixed-prefix `argv`, explicit `:wrapper`
  resolution, direct `.seal` execution, and read-only `@internal`
  introspection unless a new product decision explicitly expands the surface.
- Use `clap` for CLI parsing. Do not hand-roll argument parsing.
- Preserve command lifecycle semantics: load profile, register symlinks, export
  env, run command, clean up symlinks.
- Keep command namespaces explicit: `<cmd>` is external, `:<cmd>` is a profile
  wrapper, `@<cmd>` is runseal internal.

Runtime path rules:

- Treat `RUNSEAL_HOME` as the runseal configuration root.
- Treat `RUNSEAL_PROFILE_HOME` as the profile directory, defaulting to
  `<RUNSEAL_HOME>/profiles`.
- Resolve one concrete `RUNSEAL_PROFILE_PATH` during app initialization.

Tooling rules:

- Treat `runseal` and `flavor` as installed developer infrastructure, at the
  same level as `git`, `gh`, and `cargo`; this repository does not bootstrap
  them.

## 2. Directory Conventions

Direct child directories with their own `AGENTS.md`:

- None yet.

Direct child directories that are likely future candidates for a child
`AGENTS.md` once their local rules become stable:

- `app/`: Rust application code, tests, and core runtime behavior.
- `.runseal/`: repo-local wrappers and operator-facing workflow glue.
- `.github/`: CI, release automation, and workflow support scripts.
- `docs/`: durable operator or contributor documentation, if this area starts
  carrying rules distinct from code.

When a direct child directory gains its own stable constraints, add an
`AGENTS.md` there and link it from this section.

## 3. Core File Index

There are no child `AGENTS.md` targets yet, so this index currently points to
the repository-owned canonical files directly.

- `app/src/bin/runseal.rs`: CLI entrypoint.
- `app/src/core/config.rs`: app configuration and profile discovery.
- `app/src/core/profile.rs`: profile format loading and normalization.
- `app/src/core/runtime.rs`: command execution lifecycle.
- `app/src/core/transpile/runner.rs`: direct Seal wrapper runtime.
- `app/src/core/injections/`: `env` and `symlink` implementations.
- `app/src/core/tool/`: built-in atomic `@tool` surface.
- `app/tests/`: integration tests and focused behavioral coverage.
- `.runseal/wrappers/`: repo-local `:wrapper` entrypoints. Prefer `.seal`
  wrappers; platform scripts exist only while a wrapper has not migrated.
- `runseal.toml`: repo-local operator profile.
- `manage.sh` and `manage.ps1`: public install and uninstall managers.

Once child `AGENTS.md` files exist, this section should prefer links to those
local guides over repeating their detail here.

## 4. Daily Iteration Workflow And Commands

Normal workflow:

1. Work on a feature branch.
2. Keep changes scoped to the current product boundary.
3. Validate locally before PR.
4. Use repo wrappers for recurring operator flows when they already encode the
   intended path.

Common validation commands:

```bash
cargo fmt --check
cargo test --locked --workspace
flavor check
```

Common repo workflow commands:

```bash
runseal :init
runseal :cloudflare
runseal :pr
runseal :release beta
```

Manager install/update path:

```bash
./manage.sh install --channel beta
```

Release and distribution rules:

- Release and manager downloads use R2 metadata and artifacts as the source of
  truth.
- Public install and uninstall entrypoints are `manage.sh` and `manage.ps1`.
- Release and smoke flows should reference those root files.
- Cloudflare manager redirects are exact-path rules for
  `runseal.perish.uk/manage.sh` and `runseal.perish.uk/manage.ps1`, pointing to
  `releases.runseal.perish.uk/manage.sh` and
  `releases.runseal.perish.uk/manage.ps1`.

Profile discovery order:

1. `--profile <path>`
2. From `<cwd>` upward to filesystem root, at each directory:
   - `runseal.toml`
   - `runseal.yaml`
   - `runseal.yml`
   - `runseal.json`
3. `<RUNSEAL_PROFILE_HOME>/default.toml`
4. `<RUNSEAL_PROFILE_HOME>/default.yaml`
5. `<RUNSEAL_PROFILE_HOME>/default.yml`
6. `<RUNSEAL_PROFILE_HOME>/default.json`

Format priority is TOML, YAML, then JSON within each searched directory.
Successful profile and wrapper paths are normalized absolute paths.

## 5. FAQ

### What defines the CLI surface?

This repository is building explicit runtime glue, not a hidden orchestrator.
New behavior should be added only when it fits one of two shapes cleanly:

- shared `bash` and PowerShell semantics that are worth making first-class in
  `.seal`
- an explicit atomic `@tool`

This boundary comes from the actual problem `runseal` is trying to solve:
clear operational workflows should not need to depend on heavyweight language
runtimes or repository-local script stacks just to survive environment drift,
cross-platform differences, and routine operator setup friction.

The goal is not "be as small as possible". The goal is to absorb the
right kind of complexity:

- clear, finite, cross-platform operational flow control should fit in
  `runseal`
- shell-specific cleverness, open-ended scripting power, and accidental runtime
  dependency sprawl should not

That is why the product boundary is a shared shell subset plus explicit atomic
tools, rather than a general scripting platform or a partial shell clone.

The hard promise is behavioral equivalence across `bash` and PowerShell, not
surface-level symmetry in generated code. Some worthwhile `.seal` semantics may
compile into elegant `bash` and relatively ugly PowerShell. That tradeoff is
acceptable when:

- the `.seal` behavior is clear and finite
- the translation is reliable and testable
- the result still serves explicit cross-platform operations flow control

### When should behavior become Seal syntax?

Only when bash and PowerShell share an elegant, stable semantic shape that is
worth making first-class.

### When should behavior become `@tool`?

When native CLI coverage is insufficient for an atomic, reusable operation and
the result still fits the explicit atomic-tool model.

### When should logic stay outside runseal?

When the behavior cannot be described cleanly as shared shell-shape syntax or a
clear atomic tool, keep it in Python, Ruby, JavaScript, or another external
script.

### Should `.seal` wrappers build multi-line config or payload text inline?

Usually no.

For operations work, persistent or semi-persistent structured text should
normally live as explicit repo material under `.runseal/` or `.local/`, not as
inline heredoc-style wrapper content. That includes things like:

- config templates
- YAML or JSON fragments
- kube-related files
- long request bodies
- other operator-facing text payloads

The wrapper should usually do the smaller, clearer job:

- validate preconditions
- choose the right file or template
- assemble paths and arguments
- set environment for the invoked command
- execute the operational flow

This is an intentional product boundary. `runseal` is meant to reduce
environment and runtime dependency complexity in operations workflows, not to
turn `.seal` into a general inline text-construction language. If a multi-line
artifact is important enough to exist, prefer making it a visible repo or local
artifact first.

### Should `.seal` wrappers be treated as first-class runtime entrypoints?

Yes. Treat `.runseal/wrappers/*.seal` as first-class wrappers executed directly
by runseal. `@transpile` is a debug/export tool, not the normal wrapper
execution path.

### What should never be committed?

- `.task/`
- accidental broad surface expansions that were not backed by an explicit
  product decision

### What is the commit style?

Prefer small focused commits.
