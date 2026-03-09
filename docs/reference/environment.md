# Environment Variables

## Consumed by envlock

| Variable | Purpose |
| --- | --- |
| `ENVLOCK_HOME` | Base directory for default profile resolution (`profiles/default.json`). |
| `ENVLOCK_RESOURCE_HOME` | Base directory for `resource://` and `resource-content://`. |
| `ENVLOCK_SKILL_INSTALL_HOME` | Override target root for `envlock skill install` (default: `$ENVLOCK_HOME/skills`). |
| `ENVLOCK_PLUGIN_NODE_BIN` | Optional override for `envlock plugin node` binary path selection. |
| `ENVLOCK_PLUGIN_NPM_BIN` | Optional override for npm binary used by `envlock plugin node`. |
| `ENVLOCK_PLUGIN_PNPM_BIN` | Optional override for pnpm binary used by `envlock plugin node`. |
| `ENVLOCK_PLUGIN_YARN_BIN` | Optional override for yarn binary used by `envlock plugin node`. |
| `ENVLOCK_PLUGIN_NODE_STATE_DIR` | Optional override for `envlock plugin node` local state directory. |
| `HOME` | Fallback base for default profile/resource directories. |

## Default Paths

When `ENVLOCK_HOME` is unset:

- envlock home: `~/.envlock`
- default profile: `~/.envlock/profiles/default.json`

When `ENVLOCK_RESOURCE_HOME` is unset:

- resource home: `~/.envlock/resources`

If `HOME` is unavailable, envlock falls back to literal strings:

- `~/.envlock`
- `~/.envlock/resources`

These literal fallback paths are not shell-expanded by envlock.
