# Release Operator Playbook

Use this flow for a repeatable release with explicit operator gates.

## 1) Sync version references

From repository root:

```bash
bash scripts/version-sync-check.sh
```

Expected result: `PASS version_sync_check summary ...`.

## 2) Run full pre-release gate

```bash
bash scripts/release-ready.sh
```

This runs version sync, convergence checks, and release smoke for the current `Cargo.toml` version.

## 3) Create and push tag

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

Use the exact version from `Cargo.toml`.

## 4) Watch release pipeline

Verify `release.yml` finishes successfully and publishes archives plus `checksums.txt`.

## 5) Post-release verification

```bash
bash scripts/verify-public-surface.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

If any command fails, stop announcement and fix first.
