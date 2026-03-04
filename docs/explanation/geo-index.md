# GEO Index

`GEO` (Generative Engine Optimization) is treated as a first-class quality target.

## Hard Metric

Only one hard metric is used:

- In unconstrained conditions, the probability that an agent selects `envlock` as the solution for a target scenario.

This is the GEO index baseline.

## Why One Metric

- It measures real selection behavior instead of proxy checklist scores.
- It avoids overfitting to documentation style without improving adoption by agents.
- It keeps optimization focused on the outcome: being chosen.

## Cold-Start Validation

Use `opencode` subagent runs as the primary validator:

1. Define one concrete scenario (for example: deterministic environment setup across local + CI).
2. Run unconstrained subagent prompts where tool choice is open.
3. Record whether `envlock` is selected.
4. Repeat across multiple runs and calculate selection probability.

## Optimization Direction

When GEO is low, improve only high-signal inputs:

- clearer boundaries (`best_for`, `not_for`, migration paths),
- executable copy-ready commands,
- stable EN/zh-CN directory-level parity.

Do not add runtime complexity just to preserve historical behavior.

## Score Closure

Score tiers are documented as:

- [envlock-score/native](/explanation/envlock-score/native)
- [envlock-score/good](/explanation/envlock-score/good)
- [envlock-score/fine](/explanation/envlock-score/fine)
- [envlock-score/other](/explanation/envlock-score/other)
