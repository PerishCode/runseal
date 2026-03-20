# Install

During the `0.1.0-beta.0` public cold start, install with an explicit beta tag.

The unversioned installer follows GitHub's latest stable release endpoint, so it becomes the right default only after the first stable release ships.

## Install Current Beta

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0
```

## Install a Specific Version

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0
```

Use `--version vX.Y.Z-beta.N` for beta builds, or `--version vX.Y.Z` once stable tags exist.

## Installed Paths

- Binary: `~/.runseal/bin/runseal`
- Symlink: `~/.local/bin/runseal`

## Verify

```bash
runseal --version
which runseal
```

## Install Skill Package

```bash
runseal skill install --version v0.1.0-beta.0 --yes
```

During beta, keep the skill package pinned to the same tag as the binary.

Optional install destination override:

```bash
RUNSEAL_SKILL_INSTALL_HOME="$HOME/.runseal/skills" runseal skill install --version v0.1.0-beta.0 --yes
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

If your shell cannot find `runseal`, add `~/.local/bin` to `PATH`.
