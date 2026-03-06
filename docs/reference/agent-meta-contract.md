# Agent Meta Contract

Machine-first contract for homepage cold starts. Agents should read these meta fields before parsing DOM.

## Required Fields

- `agent:contract:version`: schema version for compatibility checks.
- `agent:index:v1`: comma-separated list of fields that define the minimum contract.
- `agent:mode`: execution mode (`meta-first`).
- `agent:entry:install`: canonical install entrypoint.
- `agent:entry:cli`: canonical CLI reference entrypoint.
- `agent:entry:ci`: canonical CI integration entrypoint.
- `agent:resolution`: conflict policy between meta and DOM.
- `agent:locale:default`: default locale key.
- `agent:locale:source`: current locale source (`html_lang`).
- `agent:locale:policy`: how locale is applied to entries.

## Parsing Rules

1. Read `agent:index:v1` first and treat it as the source of truth for required keys.
2. Resolve routes from `agent:entry:*` directly; do not infer alternatives unless policy requires it.
3. If meta and DOM disagree, follow `agent:resolution`.
4. Use DOM only as a secondary validation layer in `meta-first` mode.

## Current Policy

- Canonical entries are English routes under `/envlock/`.
- Locale selection is derived from `<html lang>`.
- Locale-specific rendering can differ, but route contract remains stable.

## Verification Commands

```bash
pnpm run docs:build
bash scripts/verify-agent-meta.sh
bash scripts/check-agent-routes.sh
```
