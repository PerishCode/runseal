# CLI Reference

## Command Forms

```bash
envlock [--profile <path>] [--output <shell|json>] [--strict] [-- <cmd...>]
envlock preview --profile <path> [--output <text|json>]
envlock self-update [--check] [--version <x.y.z|vX.Y.Z>] [-y|--yes]
envlock skill install [--version <x.y.z|vX.Y.Z>] [--force] [-y|--yes]
envlock plugin node init [--force] [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node validate [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node preview [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node apply [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock profiles status
envlock profiles init --type <minimal|sample> [--name <name>] [--force]
envlock alias list
envlock alias append <name> --profile <path>
envlock alias run <name> [-- <cmd...>]
envlock :<alias> [-- <cmd...>]
```

## Run Command Options

| Option | Description |
| --- | --- |
| `-p, --profile <path>` | Explicit JSON profile path. |
| `--output <shell|json>` | Output mode, default `shell`. |
| `--strict` | Fail on duplicate keys in final output. |
| `--log-level <error|warn|info|debug|trace>` | Logging level, default `warn`. |
| `--log-format <text|json>` | Logging format, default `text`. |
| `-- <cmd...>` | Run child command with injected env and return child exit code. |

When `--profile` is omitted, envlock resolves:

- `$ENVLOCK_HOME/profiles/default.json` if `ENVLOCK_HOME` is set.
- `~/.envlock/profiles/default.json` otherwise.

## `self-update` Options

| Option | Description |
| --- | --- |
| `--check` | Check availability only; no install. |
| `--version <x.y.z|vX.Y.Z>` | Install exact release version. |
| `-y, --yes` | Skip confirmation prompt. |

## `skill install` Options

| Option | Description |
| --- | --- |
| `--version <x.y.z|vX.Y.Z>` | Install skill package from exact release version. |
| `--force` | Overwrite existing installed skill directory for the same version. |
| `-y, --yes` | Skip overwrite confirmation prompt when `--force` is used. |

Skill install destination order:

1. `ENVLOCK_SKILL_INSTALL_HOME`
2. `$ENVLOCK_HOME/skills`
3. `~/.envlock/skills`

## `plugin node` Commands

- `plugin node init`: bootstrap local shell plugin script at `$ENVLOCK_HOME/plugins/node.sh`.
- `plugin node validate`: validate local node plugin inputs and emit patch JSON.
- `plugin node preview`: emit dry-run patch JSON (`env` + `symlink`).
- `plugin node apply`: apply local node symlink state and emit final patch JSON.
- `--node-bin <path>`: force plugin to use explicit node binary path.
- `--npm-bin <path>`: force npm binary path for versioned cache/prefix output.
- `--pnpm-bin <path>`: force pnpm binary path for versioned store output.
- `--yarn-bin <path>`: force yarn binary path for versioned cache output.
- `--state-dir <path>`: override plugin local state directory (default: `$ENVLOCK_HOME/plugin-node`).

## `preview` Options

| Option | Description |
| --- | --- |
| `-p, --profile <path>` | Explicit JSON profile path to inspect. |
| `--output <text|json>` | Preview format, default `text`. |

`preview` is read-only and does not execute injections. It exposes metadata only:

- `env`: key names only.
- `command`: program and argument count only.
- `symlink`: path metadata only.

## `profiles` Commands

- `profiles status`: show `$ENVLOCK_HOME/profiles` health, default profile presence, and JSON parse status.
- `profiles init --type <minimal|sample>`: create a starter profile at `$ENVLOCK_HOME/profiles/default.json`.
- `profiles init --name <name>`: write to `$ENVLOCK_HOME/profiles/<name>.json`.
- `profiles init --force`: overwrite existing target file.

## `alias` Commands

- `alias list`: show alias to profile mappings from `$ENVLOCK_HOME/aliases.json`.
- `alias append <name> --profile <path>`: append one alias mapping (fails on duplicate name).
- `alias run <name>`: run by alias with optional child command override.
- `envlock :<alias>`: shortcut for `envlock alias run <alias>`.

## Exit Behavior

- Shell/JSON output mode: exits `0` on success.
- Command mode: exits with child exit code.
- Validation/parsing failures: non-zero exit.
