# FAQ

## How does `resource://` stop parsing?

`env` injection resolves `resource://` tokens until it hits `:` or `;`.

That is intentional for PATH-like values, but it means URL-like literals that include `:` can split earlier than expected. If you need a literal URL, keep it outside `resource://` resolution.

## When does runseal detect a missing resource file?

`resource://` resolution turns relative paths into absolute paths during export, but it does not guarantee file existence at parse or validation time.

If a resource file is missing, the failure usually appears later when the downstream tool reads that path.

## What happens if `HOME` is missing?

If `HOME` is unavailable and `RUNSEAL_HOME` is unset, runseal exits with an actionable error instead of falling back to a literal `~/.runseal` path.

## What should I check first when the default profile is missing?

By default, runseal looks for `profiles/default.json` under `RUNSEAL_HOME`, or under `~/.runseal` when `RUNSEAL_HOME` is unset.

Check one of these first:

- `RUNSEAL_HOME` points to the intended root
- `profiles/default.json` exists under that root
- you are not expecting a project-local profile without passing `--profile`
