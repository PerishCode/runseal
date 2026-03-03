# CLI 参考

## 命令语法

```bash
envlock [--profile <path>] [--output <shell|json>] [--strict] [-- <cmd...>]
envlock preview --profile <path> [--output <text|json>]
envlock self-update [--check] [--version <x.y.z|vX.Y.Z>] [-y|--yes]
```

## 主命令常用选项

- `-p, --profile <path>`：显式指定 profile JSON 路径
- `--output <shell|json>`：输出模式，默认 `shell`
- `--strict`：输出结果中出现重复 key 时失败
- `--log-level <error|warn|info|debug|trace>`：日志级别，默认 `warn`
- `--log-format <text|json>`：日志格式，默认 `text`
- `-- <cmd...>`：将注入后的环境只传给子进程，并返回子进程退出码

不传 `--profile` 时，默认查找顺序：

1. `ENVLOCK_HOME/profiles/default.json`（当 `ENVLOCK_HOME` 已设置）
2. `~/.envlock/profiles/default.json`

## preview

- `preview` 只读，不执行 injections
- 输出仅包含元信息，不暴露敏感值

## self-update

- `--check`：只检查更新
- `--version <x.y.z|vX.Y.Z>`：升级到指定版本
- `-y, --yes`：跳过交互确认
