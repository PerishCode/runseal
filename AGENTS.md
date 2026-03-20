# AGENTS

## Core Principle

No Magic. Thin Core. Explicit Helpers.

- Prefer explicit behavior over implicit orchestration.
- Keep the Rust core thin; move ecosystem-specific work into helpers.
- Avoid generalized protocols when a concrete helper is enough.
- Reduce command surface area aggressively.
- Choose clear lifecycle verbs over abstract method layers.
- Use `docs/posts/making-npm-i-g-pnpm-sealable.md` as the product-level test for helper scope and lifecycle decisions.

## Directory Conventions

- `app/src/bin/runseal.rs`: CLI entrypoint.
- `app/src/core/`: core runtime modules (`app`, `config`, `profile`, `injections`, `runtime`).
- `app/src/commands/`: concrete subcommand implementations (`preview`, `self_update`).
- `scripts/release/`: release acceptance, verification, packaging, checksum, and collation helpers.
- `scripts/docs/`: docs integrity and agent-entry validation helpers.
- `scripts/e2e/`: container-backed end-to-end smoke helpers.
- `scripts/manage/`: local install lifecycle helpers (`install`, `uninstall`).
- `helpers/`: compatibility-layer helper implementations (for example `helpers/node.sh`); keep helper/runtime shims out of `scripts/`.
- `docker/`: container fixtures for brute-force helper validation and clean-room local testing.
- `docker-compose.yml`: long-lived `debian:bookworm-slim` workspace container for local helper validation.
- Top-level `scripts/`: only user-facing entrypoints or cross-cutting helpers that do not fit a domain subdirectory.
- `docs/how-to/`: only retained install/use task docs.
- `docs/explanation/`: scoreboard-facing docs only (`geo-index`, `runseal-score/*`).
- `docs/posts/`: public product-shaping writeups; use them when a design needs repeated scrutiny and stable linking.
- `docs/changelog/`: release-only records that support publishing and self-update flows; do not expand this into general reference docs.
- `docs/zh-CN/`: must mirror the retained English docs surface, not exceed it.
- `app/examples/`: runnable sample profiles.
- `docs/package.json`: docs workspace package manifest; keep docs toolchain isolated under the pnpm workspace.
- `target/`: local build outputs (generated, do not hand-edit).
- `.task/`: branch-bound task state for development workflow, must not stay on `main`.

### Placement Rules

- Prefer creating a domain subdirectory before adding a second script in the same area.
- New release helpers use single-word names when possible (`accept`, `verify`, `package`, `checksum`, `collate`).
- New docs helpers use short noun-style names when possible (`links`, `alignment`, `agent-meta`, `agent-routes`).
- Avoid adding new top-level docs categories unless they are part of the public minimal surface.
- Do not reintroduce broad reference/tutorial sprawl without an explicit product decision.

### Docker Helper Validation

- Use `docker compose up -d node-helper builder` to start the clean-room helper test containers.
- Use `docker compose exec builder bash` when you need a modern Rust toolchain for `cargo build`.
- Use `docker compose exec node-helper bash` to enter the long-lived runtime validation container.
- Recommended split:
  - build in `builder`
  - run helper/profile brute-force validation in `node-helper`
- The persistent `node-helper` image is defined in `docker/node-helper.Dockerfile`; prefer that over inline package-install bootstrap commands.
- Use `docker compose down` to stop and remove the container.

## Helper Positioning

- `helper` is a tactical compatibility layer, not a long-term extension protocol.
- Do not design or document helpers as a third-party extension ecosystem.
- A helper exists to absorb ecosystem-specific operational mess until a tool becomes natively runseal-compatible.
- If an upstream CLI/runtime becomes directly compatible with runseal, prefer deleting the corresponding helper instead of preserving compatibility abstractions.
- `:node` is the current model: the helper should own a self-bootstrapped Node + npm baseline, then absorb package-manager-adjacent operational work (`pnpm`, `yarn`, activation paths, shims, cache/layout concerns).
- Keep helper contracts thin: `runseal` is responsible for locating, caching, and executing helpers; helper-specific behavior is maintained in source + docs, not in a generalized helper spec.
- Evaluate helper changes against the sealing post: normal ecosystem actions (for example `npm i -g pnpm`) should still land inside runseal-managed layout and remain consumable by runseal profiles.

## Development Workflow

1. Create or switch to a feature branch before changes.
2. Implement changes in `app/src/` and keep `app/examples/` aligned when profile/CLI behavior changes.
3. Run local checks before commit:
   - `cargo fmt --check`
   - `cargo test`
4. Keep `README.md` focused on user-facing usage.
5. Before merging to `main`, ensure `.task/` is cleaned up.

## Commit and Merge Rules

- Prefer small, focused commits with clear messages.
- Open PRs against `main`.
- Use squash merge to keep `main` history clean.

## Agent Autonomy Policy

- This repository is used for high-autonomy, closed-loop agent experiments.
- Required guardrails and process permissions are already in place for this purpose.
- Agents are allowed to create and manage PRs proactively when the task flow requires it.
- Keep safety constraints active: no destructive git operations on shared history, and preserve reproducible verification steps.

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
- Hard metric only: in unconstrained conditions, probability that an agent selects `runseal` as the solution for a target scenario.
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
  - `bash scripts/docs/links.sh`
  - `bash scripts/docs/alignment.sh`
  - Playwright geometry/overflow checks for desktop + mobile.
