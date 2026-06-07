# AGENTS

## Core Principle

Small CLI. Explicit profile. No hidden orchestration.

- Keep the Rust core thin and concrete.
- Support only `env`, `symlink`, fixed-prefix `argv`, explicit `:wrapper` command resolution, and read-only `@internal` introspection unless a new product decision explicitly expands the surface.
- Use mature CLI parsing through `clap`; do not hand-roll argument parsing.
- Treat `RUNSEAL_HOME` as the runseal configuration root.
- Treat `RUNSEAL_PROFILE_HOME` as the profile directory, defaulting to `<RUNSEAL_HOME>/profiles`.
- Resolve one concrete `RUNSEAL_PROFILE_PATH` during app initialization.
- Treat `runseal` and `flavor` as installed developer infrastructure, at the
  same level as `git`, `gh`, and `cargo`; this repository does not bootstrap
  them.
- Preserve command lifecycle semantics: load profile, register symlinks, export env, run command, cleanup symlinks.
- Keep command namespaces explicit: `<cmd>` is external, `:<cmd>` is profile wrapper, `@<cmd>` is runseal internal.
- Treat `.runseal/wrappers/*.seal` as first-class wrappers executed directly by
  runseal. `@transpile` is an isomorphic debug/export tool, not the normal
  wrapper execution path.

## Directory Conventions

- `app/src/bin/runseal.rs`: CLI entrypoint.
- `app/src/core/config.rs`: app configuration and profile discovery.
- `app/src/core/profile.rs`: profile format loading and normalization.
- `app/src/core/runtime.rs`: command execution lifecycle.
- `app/src/core/transpile/runner.rs`: direct Seal wrapper runtime.
- `app/src/core/injections/`: `env` and `symlink` implementations.
- `app/tests/`: integration tests and focused unit tests.
- `runseal.toml`: repo-local operator profile.
- `.runseal/wrappers/`: repo-local `:wrapper` entrypoints. Prefer `.seal`
  wrappers; platform scripts exist only while a wrapper has not migrated.
- `manage.sh` and `manage.ps1`: public install/uninstall managers.
- `.task/`: branch-bound task state, ignored by git.

## Profile Discovery

Priority order:

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

## Development Workflow

1. Work on a feature branch.
2. Use `runseal :init`, `runseal :cloudflare`, `runseal :pr`, and
   `runseal :release` for repo management.
3. Keep changes scoped to the reduced CLI surface.
4. Run:

```bash
cargo fmt --check
cargo test
```

Release and manager downloads use R2 metadata and artifacts as the source of
truth. Public install/uninstall entrypoints are `manage.sh` and `manage.ps1`;
release and smoke scripts should reference those root files.

Cloudflare manager redirects are exact-path rules for
`runseal.perish.uk/manage.sh` and `runseal.perish.uk/manage.ps1`, pointing to
`releases.runseal.perish.uk/manage.sh` and
`releases.runseal.perish.uk/manage.ps1`.

## Commit Rules

- Prefer small focused commits.
- Do not commit `.task/`.
- Do not reintroduce broader command surfaces without an explicit product decision.
