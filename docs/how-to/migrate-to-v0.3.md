# Migrate to v0.3

This page covers the breaking changes in v0.4.2.

## 1) CLI behavior in command mode is stricter

In v0.4.2, `--strict` applies to all output paths, including command mode.

- Duplicate exported keys now fail in strict mode before child command execution.
- Invalid exported env keys also fail before child command execution.

Migration action:

1. Ensure each injection pipeline produces one final value per key.
2. Rename non-shell env keys to valid names (`[A-Za-z_][A-Za-z0-9_]*`).

## 2) Rust module imports moved to explicit core/commands boundaries

The crate now uses explicit boundaries:

- runtime internals: `envlock::core::*`
- subcommands: `envlock::commands::*`

If you imported old flat paths, migrate imports directly.

Examples:

```rust
use envlock::core::app::App;
use envlock::core::config::RuntimeConfig;
use envlock::commands::preview::run as run_preview;
```

## 3) Validate migration

```bash
envlock --version
envlock preview --profile ./profiles/dev.json
envlock --profile ./profiles/dev.json --output json
envlock --strict --profile ./profiles/dev.json -- bash -lc 'env | grep ENVLOCK || true'
```

If these commands behave as expected, migration is complete.
