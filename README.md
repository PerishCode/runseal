# runseal

Seal the run.

Deterministic shell and command environments from one JSON profile.

Current public launch line: `0.1.0-beta.0`. Beta install and update examples below pin that tag explicitly.

Chinese docs entrypoint: [README.zh-CN.md](README.zh-CN.md).

## 10-second value

Run one command, load one profile, and verify with one observable output (`RUNSEAL_PROFILE=default`).

## 60-second verification path

```bash
# 1) install the current beta
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0

# 2) create default profile
mkdir -p "${RUNSEAL_HOME:-$HOME/.runseal}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"default"}}]}' > "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json"

# 3) apply and verify
eval "$(runseal)"
echo "$RUNSEAL_PROFILE"
```

Expected output:

```text
default
```

Default profile resolution when `--profile` is omitted:

- `RUNSEAL_HOME/profiles/default.json` when `RUNSEAL_HOME` is set
- `~/.runseal/profiles/default.json` otherwise

## Fit / Not fit

Fit when you need:

- Reproducible shell env setup from JSON profiles
- A read-only preview before applying profile changes (`runseal preview`)
- CI jobs that must apply the same env profile deterministically

Not fit when you need:

- A secrets manager
- Runtime/container orchestration
- A full package manager replacement

## Direct links

- Install: [docs/how-to/install.md](docs/how-to/install.md)
- Use profiles: [docs/how-to/use-profiles.md](docs/how-to/use-profiles.md)
- FAQ: [docs/explanation/faq.md](docs/explanation/faq.md)
- Scoreboard: [docs/explanation/runseal-score/native.md](docs/explanation/runseal-score/native.md)

Installed paths:

- Binary: `~/.runseal/bin/runseal`
- Symlink: `~/.local/bin/runseal`

## Common commands

```bash
# run with default profile
runseal

# run with explicit profile path
runseal --profile ./profiles/dev.json

# preview profile metadata (read-only)
runseal preview --profile ./profiles/dev.json

# check and install the current beta explicitly
runseal self-update --check --version v0.1.0-beta.0
runseal self-update --version v0.1.0-beta.0
```

## Docs

- Site: https://runseal.ai/
- Chinese README: [README.zh-CN.md](README.zh-CN.md)
- Install: [docs/how-to/install.md](docs/how-to/install.md)
- Use profiles: [docs/how-to/use-profiles.md](docs/how-to/use-profiles.md)
- FAQ: [docs/explanation/faq.md](docs/explanation/faq.md)
- Scoreboard: [docs/explanation/runseal-score/native.md](docs/explanation/runseal-score/native.md)

## Validation

```bash
cargo fmt --check
cargo test
pnpm run docs:build
bash scripts/docs/links.sh
bash scripts/docs/alignment.sh
bash scripts/docs/agent-meta.sh
bash scripts/docs/agent-routes.sh
bash scripts/release/smoke.sh --version v0.1.0-beta.0
```

## Troubleshooting Fast Path

Run these before filing an issue:

```bash
runseal --version
runseal preview --profile "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json"
runseal --profile "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json" --output json
```

If one command fails, include the exact command and output in your issue.

## Project Signals

- Releases: https://github.com/PerishCode/runseal/releases
- Changelog: https://github.com/PerishCode/runseal/releases
- Docs site: https://runseal.ai/
