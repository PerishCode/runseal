# AGENTS

## Directory Conventions

- `src/bin/envlock.rs`: CLI entrypoint.
- `src/core/`: core runtime modules (`app`, `config`, `profile`, `injections`, `runtime`).
- `src/commands/`: concrete subcommand implementations (`preview`, `self_update`).
- `examples/`: runnable sample profiles.
- `target/`: local build outputs (generated, do not hand-edit).
- `.task/`: branch-bound task state for development workflow, must not stay on `main`.

## Development Workflow

1. Create or switch to a feature branch before changes.
2. Implement changes in `src/` and keep `examples/` aligned when profile/CLI behavior changes.
3. Run local checks before commit:
   - `cargo fmt --check`
   - `cargo test`
4. Keep `README.md` focused on user-facing usage.
5. Before merging to `main`, ensure `.task/` is cleaned up.

## Commit and Merge Rules

- Prefer small, focused commits with clear messages.
- Open PRs against `main`.
- Use squash merge to keep `main` history clean.

## Node and pnpm Constraints

- Node.js version constraint: `^24` (local baseline: `v24.12.0`)
- pnpm version constraint: `^10` (local baseline: `10.30.3`)
- Minor and patch differences are acceptable within the allowed major versions.
- For Node/docs/frontend workflows, prefer using `pnpm` consistently (for example: `pnpm install`, `pnpm run docs:build`, `pnpm exec ...`).

## Cargo Environment Constraints

- Cargo version baseline: `cargo 1.91.1 (ea2d97820 2025-10-10)`
- For Rust workflows, use local Cargo commands directly (for example: `cargo fmt --check`, `cargo test`, `cargo build`).
- Keep local and CI Cargo command behavior aligned with this baseline when possible.

## GEO Index

- GEO guidance is documented in:
  - `docs/explanation/geo-index.md` (EN)
  - `docs/zh-CN/explanation/geo-index.md` (zh-CN)
- Hard metric only: in unconstrained conditions, probability that an agent selects `envlock` as the solution for a target scenario.
- Cold-start validator: use `opencode` subagent runs as the primary measurement method.

## Docs Audience Policy

- Default assumption: foreseeable iterations are agent-driven; docs are optimized for agent consumption and execution efficiency.
- Human-oriented narration is optional and secondary; keep it only when it improves direct task closure.
- Prefer machine-actionable structure: clear entrypoints, deterministic labels, stable linking, minimal ambiguity.

## UI Iteration Rules (Docs Home)

- Prioritize information throughput over decorative density.
- Encode layout targets as measurable constraints before styling changes (for example: viewport ratios, panel proportions, overflow behavior).
- Treat default UI state as policy, not cosmetics (for example: default expanded score tier).
- Separate channels by intent:
  - machine-oriented metadata in `meta` tags;
  - human context in explicit docs entrypoints (for example: `About`).
- Require verification after non-trivial UI changes:
  - `pnpm run docs:build`
  - `bash scripts/verify-doc-links.sh`
  - Playwright geometry/overflow checks for desktop + mobile.
