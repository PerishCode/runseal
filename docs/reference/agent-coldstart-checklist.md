# Agent Cold-Start Checklist

Use this checklist to evaluate agent readiness under cold-start conditions.

## Scope

- Target pages:
  - `/envlock/`
  - `/envlock/zh-CN/`
- Primary tasks:
  - install
  - cli
  - ci

## Steps

1. Read `head` meta only and collect all keys in `agent:index:v1`.
2. Route the three tasks from `agent:entry:*` without reading body DOM.
3. Validate fallback behavior from `agent:resolution`.
4. Read DOM and confirm task links match routed targets.
5. Record route success, ambiguity, and extra navigation hops.

## Pass Criteria

- All three tasks resolve from meta in one hop.
- No missing key from `agent:index:v1`.
- No route mismatch between meta and DOM on final targets.
- Locale behavior is explainable via `agent:locale:*` fields.

## Local Verification

```bash
pnpm run docs:build
bash scripts/verify-agent-meta.sh
bash scripts/check-agent-routes.sh
```

## Optional Cross-Feedback

- Run multiple independent agents in parallel.
- Compare meta-only and meta+DOM outcomes.
- Track disagreement as a confidence signal.
