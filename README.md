# envlock

Deterministic shell and command environments from one JSON profile.

Chinese docs entrypoint: [README.zh-CN.md](README.zh-CN.md).

## 30-second install

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh
```

Installed paths:

- Binary: `~/.envlock/bin/envlock`
- Symlink: `~/.local/bin/envlock`

## 60-second quick start (v0.2.1)

Create the default profile, run `envlock` with no flags, then verify:

```bash
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"ENVLOCK_PROFILE":"default"}}]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

Default profile resolution when `--profile` is omitted:

- `ENVLOCK_HOME/profiles/default.json` when `ENVLOCK_HOME` is set
- `~/.envlock/profiles/default.json` otherwise

## Common commands

```bash
# run with default profile
envlock

# run with explicit profile path
envlock --profile ./profiles/dev.json

# preview profile metadata (read-only)
envlock preview --profile ./profiles/dev.json

# update checks and upgrade
envlock self-update --check
envlock self-update
```

## Docs

- Site: https://perishcode.github.io/envlock/
- Chinese README: [README.zh-CN.md](README.zh-CN.md)
- Tutorial: [docs/tutorials/quick-start.md](docs/tutorials/quick-start.md)
- How-to: [docs/how-to/install.md](docs/how-to/install.md)
- Quick reference: [docs/reference/quick-reference.md](docs/reference/quick-reference.md)
- Common recipes: [docs/how-to/common-recipes.md](docs/how-to/common-recipes.md)
- CI integration: [docs/how-to/ci-integration.md](docs/how-to/ci-integration.md)
- CLI reference: [docs/reference/cli.md](docs/reference/cli.md)
- FAQ: [docs/explanation/faq.md](docs/explanation/faq.md)
- Explanation: [docs/explanation/design-boundaries.md](docs/explanation/design-boundaries.md)
- Language policy: [docs/explanation/language-maintenance.md](docs/explanation/language-maintenance.md)

## Validation

```bash
scripts/converge-check.sh
bash scripts/release-smoke.sh --version v0.2.1
```

## Troubleshooting Fast Path

Run these before filing an issue:

```bash
envlock --version
envlock preview --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
envlock --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json" --output json
```

If one command fails, include the exact command and output in your issue.

## Project Signals

- Releases: https://github.com/PerishCode/envlock/releases
- Changelog: https://github.com/PerishCode/envlock/releases
- Docs site: https://perishcode.github.io/envlock/
- Migration guide (v0.2): https://perishcode.github.io/envlock/how-to/migrate-to-v0.2
