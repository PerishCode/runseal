# Use Profiles

`runseal` supports convention-first resolution with explicit override.

This guide tracks the `v0.1.0-beta.0` public beta line.

## Mode A: Explicit Path

```bash
runseal -p ./profiles/dev.json
```

Use this when your profile lives next to a project.

## Mode B: Convention Default Profile

```bash
runseal
```

Default profile file:

- `${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json`

Lookup behavior:

1. If `RUNSEAL_HOME` is set, resolve from `$RUNSEAL_HOME/profiles/default.json`.
2. Otherwise resolve from `~/.runseal/profiles/default.json`.

## Useful Flags

- `--output shell`: print shell exports.
- `--output json`: print JSON object.
- `--strict`: fail on duplicate keys in final output.

## Resource URI Expansion

`env` values support URI expansion with `RUNSEAL_RESOURCE_HOME`:

- `resource://path/to/file` -> absolute path under resource home.
- `resource-content://path/to/file` -> file contents under resource home.

Default `RUNSEAL_RESOURCE_HOME` when unset:

- `~/.runseal/resources`
