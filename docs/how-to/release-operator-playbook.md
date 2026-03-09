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

## Beta path

For prerelease validation, do not push a stable tag.

1. Set `Cargo.toml` to a beta version such as `0.4.4-beta.2`.
2. Push the branch that contains the beta changes.
3. Run `release-beta.yml` with input `v0.4.4-beta.2`.
4. Wait for the prerelease assets to appear on GitHub Releases.
5. Run install-based validation against the published beta tag:

```bash
bash scripts/install.sh --version v0.4.4-beta.2
envlock --version
envlock plugin node init --help
```

## 4) Watch release pipeline

Verify `release.yml` finishes successfully and publishes archives plus `checksums.txt`.

For beta, verify `release-beta.yml` finishes successfully and publishes a GitHub prerelease with archives plus `checksums.txt`.

## 5) Post-release verification

```bash
bash scripts/verify-public-surface.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

If any command fails, stop announcement and fix first.
