# 更新与卸载

## 检查更新

```bash
envlock self-update --check
```

## 升级

交互式：

```bash
envlock self-update
```

非交互：

```bash
envlock self-update --yes
```

升级后可同步安装匹配 skill 包：

```bash
envlock skill install --yes
```

固定到指定版本：

```bash
envlock self-update --version v0.4.2 --yes
```

## 卸载

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/uninstall.sh | sh
```

卸载仅会删除：

- `~/.local/bin/envlock` 软链接（仅当它指向受管二进制时）。
- `~/.envlock` 目录树。
