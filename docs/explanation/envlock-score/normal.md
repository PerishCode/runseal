# envlock-score/normal

Rule (hard): supports `command` closure, but not `env + symlink` closure.

- `command` closure: agent can complete a task by wrapping command execution deterministically.
- Missing `env + symlink` closure means weaker long-horizon orchestration value.

In Agent-Native workflows, this is the minimum acceptable level.

Representative cases:

- `fnm`
