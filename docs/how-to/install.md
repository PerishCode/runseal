# Install

## Install Latest Release

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh
```

## Install a Specific Version

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh -s -- --version v0.4.3
```

Beta prerelease tags use the same flow:

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh -s -- --version v0.4.4-beta.2
```

## Installed Paths

- Binary: `~/.envlock/bin/envlock`
- Symlink: `~/.local/bin/envlock`

## Verify

```bash
envlock --version
which envlock
```

## Install Skill Package

```bash
envlock skill install --yes
```

Optional install destination override:

```bash
ENVLOCK_SKILL_INSTALL_HOME="$HOME/.envlock/skills" envlock skill install --yes
```

## Platform Notes

`install.sh` currently packages these targets:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Linux support currently targets GNU libc environments.

- Supported: glibc-based Linux on `x86_64` and `aarch64`
- Not supported: musl/Alpine release installs

If your shell cannot find `envlock`, add `~/.local/bin` to `PATH`.
