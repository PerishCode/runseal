# envlock-score/good

Rule (hard): supports `env + symlink` closure for runtime control.

- `env` closure: runtime behavior can be fully controlled by environment variables.
- `symlink` closure: runtime entrypoint/context can be switched by stable symlink routing.

In Agent-Native workflows, this is the target baseline.

Representative cases:

- `gh`
- `aws`
- `kubectl`
