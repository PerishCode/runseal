# runseal

Run a command inside a small, explicit profile.

`runseal` currently supports three profile capabilities plus explicit wrapper
and internal command namespaces:

- `env`: export environment variables and ordered env operations.
- `symlink`: create symlinks for the command lifecycle, then clean them up.
- `argv`: inject fixed arguments after a matching child command token.

Command routing is based on the first command token:

- `runseal <cmd>` runs an external command inside the profile.
- `runseal :<cmd>` runs a profile wrapper.
- `runseal @<cmd>` runs a read-only runseal internal command.

## Usage

```bash
runseal --profile ./runseal.toml bash -- -lc 'echo "$RUNSEAL_PROFILE_PATH"'
```

If `--profile` is omitted, profile discovery walks from the current directory
to filesystem root. At each directory, format priority is:

1. `runseal.toml`
2. `runseal.yaml`
3. `runseal.yml`
4. `runseal.json`

If no ancestor profile is found, discovery falls back to:

1. `$RUNSEAL_PROFILE_HOME/default.toml`
2. `$RUNSEAL_PROFILE_HOME/default.yaml`
3. `$RUNSEAL_PROFILE_HOME/default.yml`
4. `$RUNSEAL_PROFILE_HOME/default.json`

`RUNSEAL_HOME` is the runseal configuration root. When unset it defaults to `~/.runseal`.

`RUNSEAL_PROFILE_HOME` is the profile directory. When unset it defaults to `$RUNSEAL_HOME/profiles`.

Each child command receives:

- `RUNSEAL_HOME`
- `RUNSEAL_PROFILE_HOME`
- `RUNSEAL_PROFILE_PATH`
- `RUNSEAL_WRAPPER_PATH`

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

## Wrappers

If the command token starts with `:`, runseal resolves it as a wrapper
executable instead of a literal program name:

```bash
runseal :ssh-run host ./probe.sh -- arg
```

Wrapper lookup order is:

1. `<profile-dir>/.runseal/wrappers/<name>`
2. `$RUNSEAL_HOME/wrappers/<name>`

The profile directory is the directory containing `RUNSEAL_PROFILE_PATH`.
Successful profile and wrapper paths are normalized absolute paths.
The child working directory is not changed. A resolved wrapper receives:

- `RUNSEAL_WRAPPER_NAME`
- `RUNSEAL_WRAPPER_FILE`

On Windows, runseal also checks `.exe`, `.cmd`, and `.bat` when the wrapper
name has no extension. On Unix, the wrapper file must be executable.

## Internal Commands

If the command token starts with `@`, runseal resolves it as a runseal internal
command instead of a literal program name:

```bash
runseal @profile
runseal @wrappers
runseal @which :ssh-run
```

Internal commands are read-only and do not run profile injections.

- `@profile` prints the resolved runseal runtime paths.
- `@wrappers` lists the effective wrappers visible to the current profile.
- `@which :<name>` prints the wrapper file that `:<name>` resolves to.

Use `runseal profile` without `@` to run an external command named `profile`.

## Validation

```bash
cargo fmt --check
cargo test
```
