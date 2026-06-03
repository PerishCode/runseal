# Contributing

Keep changes aligned with the current 0.1.0 surface:

- `env` injection
- `symlink` injection
- profile discovery through `--profile`, cwd `runseal.*`, and `RUNSEAL_PROFILE_HOME/default.*`
- command execution through `runseal [--profile <path>] <command> -- <args>`

Run local checks before sending changes:

```bash
cargo fmt --check
cargo test
```
