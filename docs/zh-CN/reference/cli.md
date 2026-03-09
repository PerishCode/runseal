# CLI 参考

## 命令形态

```bash
envlock [--profile <path>] [--output <shell|json>] [--strict] [-- <cmd...>]
envlock preview --profile <path> [--output <text|json>]
envlock self-update [--check] [--version <x.y.z|vX.Y.Z>] [-y|--yes]
envlock skill install [--version <x.y.z|vX.Y.Z>] [--force] [-y|--yes]
envlock plugin node init [--force] [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node validate [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node preview [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock plugin node apply [--node-bin <path>] [--npm-bin <path>] [--pnpm-bin <path>] [--yarn-bin <path>] [--state-dir <path>]
envlock profiles status
envlock profiles init --type <minimal|sample> [--name <name>] [--force]
envlock alias list
envlock alias append <name> --profile <path>
envlock alias run <name> [-- <cmd...>]
envlock :<alias> [-- <cmd...>]
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

## `skill install` 选项

| 选项 | 说明 |
| --- | --- |
| `--version <x.y.z|vX.Y.Z>` | 安装指定 release 版本对应的 skill 包。 |
| `--force` | 覆盖同版本已存在的 skill 目录。 |
| `-y, --yes` | 当使用 `--force` 时跳过覆盖确认。 |

skill 安装路径优先级：

1. `ENVLOCK_SKILL_INSTALL_HOME`
2. `$ENVLOCK_HOME/skills`
3. `~/.envlock/skills`

## `plugin node` 命令

- `plugin node init`：在 `$ENVLOCK_HOME/plugins/node.sh` 初始化本地 shell 插件脚本。
- `plugin node validate`：校验本地 node 插件输入并输出 patch JSON。
- `plugin node preview`：输出只读 patch JSON（`env` + `symlink`）。
- `plugin node apply`：应用本地 node symlink 状态并输出最终 patch JSON。
- `--node-bin <path>`：显式指定 node 二进制路径。
- `--npm-bin <path>`：显式指定 npm 二进制路径（用于版本隔离 cache/prefix）。
- `--pnpm-bin <path>`：显式指定 pnpm 二进制路径（用于版本隔离 store）。
- `--yarn-bin <path>`：显式指定 yarn 二进制路径（用于版本隔离 cache）。
- `--state-dir <path>`：覆盖插件状态目录（默认：`$ENVLOCK_HOME/plugin-node`）。

## `preview` 选项

| 选项 | 说明 |
| --- | --- |
| `-p, --profile <path>` | 显式指定要检查的 profile 路径。 |
| `--output <text|json>` | 预览格式，默认 `text`。 |

`preview` 为只读模式，不执行注入动作。输出只包含元信息：

- `env`：仅包含 key 名
- `command`：仅包含程序名与参数数量
- `symlink`：仅包含路径元数据

## `profiles` 命令

- `profiles status`：检查 `$ENVLOCK_HOME/profiles` 状态、默认 profile 是否存在、JSON 是否可解析。
- `profiles init --type <minimal|sample>`：在 `$ENVLOCK_HOME/profiles/default.json` 初始化模板。
- `profiles init --name <name>`：写入 `$ENVLOCK_HOME/profiles/<name>.json`。
- `profiles init --force`：覆盖已有文件。

## `alias` 命令

- `alias list`：列出 `$ENVLOCK_HOME/aliases.json` 中的 alias 映射。
- `alias append <name> --profile <path>`：追加 alias（同名时失败）。
- `alias run <name>`：按 alias 执行，可选传入子命令覆盖。
- `envlock :<alias>`：`envlock alias run <alias>` 的快捷写法。

## 退出行为

- Shell/JSON 输出模式：成功时退出码为 `0`。
- Command mode：直接返回子进程退出码。
- 校验或解析失败：返回非 0 退出码。
