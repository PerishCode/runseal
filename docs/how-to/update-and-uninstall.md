# Update and Uninstall

## Check for Updates

```bash
envlock self-update --check
```

## Upgrade

Interactive:

```bash
envlock self-update
```

Non-interactive:

```bash
envlock self-update --yes
```

After upgrade, sync matching skill package:

```bash
envlock skill install --yes
```

Pin to specific version:

```bash
envlock self-update --version v0.4.2 --yes
```

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/uninstall.sh | sh
```

Uninstall only removes:

- `~/.local/bin/envlock` symlink if it points to managed binary.
- `~/.envlock` directory tree.
