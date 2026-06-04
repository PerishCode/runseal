# runseal

Run a command inside a small, explicit profile.

`runseal` currently supports three profile capabilities plus explicit wrapper
and internal command namespaces:

- `env`: export environment variables and ordered env operations.
- `symlink`: create symlinks for the command lifecycle, then clean them up.
- `argv`: inject fixed arguments after a matching child command token.
- `resource://...`: resolve profile-local resource paths inside env values.

Command routing is based on the first command token:

- `runseal <cmd>` runs an external command inside the profile.
- `runseal :<cmd>` runs a profile wrapper.
- `runseal @<cmd>` runs a read-only runseal internal command.

## Usage

```bash
runseal --profile ./runseal.toml bash -- -lc 'echo "$RUNSEAL_PROFILE_PATH"'
```

## Install

Unix:

```bash
curl -fsSL https://runseal.perish.uk/manage.sh | sh
```

Windows:

```powershell
irm https://runseal.perish.uk/manage.ps1 | pwsh
```

Install a beta or one explicit version:

```bash
curl -fsSL https://runseal.perish.uk/manage.sh | sh -s -- install --channel beta
curl -fsSL https://runseal.perish.uk/manage.sh | sh -s -- install --version v0.1.0-beta.10
```

Uninstall:

```bash
curl -fsSL https://runseal.perish.uk/manage.sh | sh -s -- uninstall
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
[resources]
root = ".local"

[[injections]]
type = "env"

[injections.vars]
RUNSEAL_ENV = "dev"
LOCAL_ROOT = "resource://"
SSH_CONFIG = "resource://ssh/config"

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

`resource://path/to/file` is a profile-only path literal. A profile that uses
resource URIs must declare:

```toml
[resources]
root = ".local"
```

The resource root may be relative to the profile directory, absolute, or `~`
expanded. In env injection values, runseal rewrites resource URIs to absolute
paths under that configured root. For example, with `root = ".local"`,
`resource://ssh/config` resolves to `<profile-dir>/.local/ssh/config`.
`resource://` and `resource://.` resolve to the resource root itself.

Child commands receive only the resolved absolute path. They do not receive
or need to understand `resource://`.

Resource URIs are resolved only when the env value is exactly the URI. runseal
does not perform partial string interpolation inside env values.

Resource paths must be relative URI-style paths. Empty paths, empty path
segments, `.`, `..`, backslash separators, and `:` inside path segments are
rejected. Resource paths are resolved without checking whether the file exists.

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
runseal @resources
runseal @resolve resource:// resource://ssh/config
runseal @wrappers
runseal @which :ssh-run
```

Internal commands are read-only and do not run profile injections.

- `@profile` prints the resolved runseal runtime paths. If resources are
  configured, it also prints `RUNSEAL_RESOURCE_ROOT`.
- `@resources` prints the resolved resource root.
- `@resolve resource://...` prints resolved absolute resource paths, one per
  argument.
- `@wrappers` lists the effective wrappers visible to the current profile.
- `@which :<name>` prints the wrapper file that `:<name>` resolves to.

Use `runseal profile` without `@` to run an external command named `profile`.

YAML and JSON profiles use the same structure:

```yaml
resources:
  root: .local
injections:
  - type: env
    vars:
      LOCAL_ROOT: resource://
      SSH_CONFIG: resource://ssh/config
```

```json
{
  "resources": {
    "root": ".local"
  },
  "injections": [
    {
      "type": "env",
      "vars": {
        "LOCAL_ROOT": "resource://",
        "SSH_CONFIG": "resource://ssh/config"
      }
    }
  ]
}
```

## Validation

```bash
cargo fmt --check
cargo test
```

Repo-local operator commands use runseal itself:

```bash
runseal :cloudflare manage-inspect
runseal :pr --dry-run
runseal :release --channel beta --dry-run
```
