# CLI 参考

## 命令形态

```bash
envlock [--profile <path>] [--output <shell|json>] [--strict] [-- <cmd...>]
envlock preview --profile <path> [--output <text|json>]
envlock self-update [--check] [--version <x.y.z|vX.Y.Z>] [-y|--yes]
```

## 主命令选项

| 选项 | 说明 |
| --- | --- |
| `-p, --profile <path>` | 显式指定 JSON profile 路径。 |
| `--output <shell|json>` | 输出模式，默认 `shell`。 |
| `--strict` | 最终输出中出现重复 key 时失败。 |
| `--log-level <error|warn|info|debug|trace>` | 日志级别，默认 `warn`。 |
| `--log-format <text|json>` | 日志格式，默认 `text`。 |
| `-- <cmd...>` | 用注入后的环境运行子命令，并返回子进程退出码。 |

未传 `--profile` 时，`envlock` 解析顺序为：

- 设置了 `ENVLOCK_HOME`：`$ENVLOCK_HOME/profiles/default.json`
- 未设置 `ENVLOCK_HOME`：`~/.envlock/profiles/default.json`

## `self-update` 选项

| 选项 | 说明 |
| --- | --- |
| `--check` | 仅检查是否有新版本，不安装。 |
| `--version <x.y.z|vX.Y.Z>` | 安装指定版本。 |
| `-y, --yes` | 跳过确认提示。 |

## `preview` 选项

| 选项 | 说明 |
| --- | --- |
| `-p, --profile <path>` | 显式指定要检查的 profile 路径。 |
| `--output <text|json>` | 预览格式，默认 `text`。 |

`preview` 为只读模式，不执行注入动作。输出只包含元信息：

- `env`：仅包含 key 名
- `command`：仅包含程序名与参数数量
- `symlink`：仅包含路径元数据

## 退出行为

- Shell/JSON 输出模式：成功时退出码为 `0`。
- Command mode：直接返回子进程退出码。
- 校验或解析失败：返回非 0 退出码。
