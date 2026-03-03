# Release Validation

Use this checklist to keep release quality gates simple, clean, and verifiable.

## Pre-release

1. Run convergence checks from repository root:

```bash
bash scripts/converge-check.sh
```

2. Run release install-run-uninstall smoke for the target tag:

```bash
bash scripts/release-smoke.sh --version vX.Y.Z
```

3. Confirm both commands end with `PASS` lines.

## Post-release

1. Verify public surface from repository root:

```bash
bash scripts/verify-public-surface.sh
```

2. Re-run release smoke against the published tag:

```bash
bash scripts/release-smoke.sh --version vX.Y.Z
```

3. If either command fails, stop rollout and fix before announcing release availability.
