# runseal

Run a command inside a small, explicit profile.

`runseal` currently supports three profile capabilities:

- `env`: export environment variables and ordered env operations.
- `symlink`: create symlinks for the command lifecycle, then clean them up.
- `argv`: inject fixed arguments after a matching child command token.

## Usage

```bash
runseal --profile ./runseal.toml bash -- -lc 'echo "$RUNSEAL_PROFILE_PATH"'
```

If `--profile` is omitted, profile discovery is:

1. `./runseal.toml`
2. `./runseal.yaml`
3. `./runseal.yml`
4. `./runseal.json`
5. `$RUNSEAL_PROFILE_HOME/default.toml`
6. `$RUNSEAL_PROFILE_HOME/default.yaml`
7. `$RUNSEAL_PROFILE_HOME/default.yml`
8. `$RUNSEAL_PROFILE_HOME/default.json`

`RUNSEAL_HOME` is the runseal configuration root. When unset it defaults to `~/.runseal`.

`RUNSEAL_PROFILE_HOME` is the profile directory. When unset it defaults to `$RUNSEAL_HOME/profiles`.

Each child command receives:

- `RUNSEAL_HOME`
- `RUNSEAL_PROFILE_HOME`
- `RUNSEAL_PROFILE_PATH`

## Profile

```toml
[[injections]]
type = "env"

[injections.vars]
RUNSEAL_ENV = "dev"

[[injections]]
type = "env"

[[injections.ops]]
op = "prepend"
key = "PATH"
value = "./bin"
separator = "os"
dedup = true

[[injections]]
type = "symlink"
source = "./tool"
target = "./.runseal-bin/tool"
on_exist = "replace"
cleanup = true

[[injections]]
type = "argv"
command = "ssh"
args = ["-F", ".local/ssh/config"]
```

## Validation

```bash
cargo fmt --check
cargo test
```
