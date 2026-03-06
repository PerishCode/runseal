# Release Pipeline

## Triggers

- CI: pull requests and pushes to `main`.
- Release: tag push matching `v*`.
- Docs deploy: pushes to `main` affecting `docs/**` or docs workflow files.

## Release Workflow

1. `release.yml` validates tag/version consistency (`vX.Y.Z` vs `Cargo.toml`).
2. Build runs per target:
   - `x86_64-unknown-linux-gnu`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
3. Binary archives, `skill-vX.Y.Z.zip`, and `checksums.txt` are generated.
4. Artifacts are published to GitHub Release.

## Maintainer Steps

```bash
# after merging changes and bumping Cargo.toml version
git tag v0.4.2
git push origin v0.4.2
```

## Breaking Change Rule

- Do not keep backward-compat branches inside runtime code to preserve old behavior.
- If external behavior or API breaks, bump Y version and publish migration docs in EN + zh-CN for the same release.
