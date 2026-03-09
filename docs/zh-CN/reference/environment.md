# 环境变量

## envlock 读取的变量

| 变量 | 作用 |
| --- | --- |
| `ENVLOCK_HOME` | 默认 profile 解析的基目录（`profiles/default.json`）。 |
| `ENVLOCK_RESOURCE_HOME` | `resource://` 与 `resource-content://` 的资源基目录。 |
| `ENVLOCK_SKILL_INSTALL_HOME` | 覆盖 `envlock skill install` 的安装根目录（默认：`$ENVLOCK_HOME/skills`）。 |
| `ENVLOCK_PLUGIN_NODE_BIN` | 可选：覆盖 `envlock plugin node` 的 node 二进制选择路径。 |
| `ENVLOCK_PLUGIN_NPM_BIN` | 可选：覆盖 `envlock plugin node` 的 npm 二进制路径。 |
| `ENVLOCK_PLUGIN_PNPM_BIN` | 可选：覆盖 `envlock plugin node` 的 pnpm 二进制路径。 |
| `ENVLOCK_PLUGIN_YARN_BIN` | 可选：覆盖 `envlock plugin node` 的 yarn 二进制路径。 |
| `ENVLOCK_PLUGIN_NODE_STATE_DIR` | 可选：覆盖 `envlock plugin node` 的本地状态目录。 |
| `HOME` | 默认 profile/资源目录的兜底基目录。 |

## 默认路径

当 `ENVLOCK_HOME` 未设置时：

- envlock home：`~/.envlock`
- 默认 profile：`~/.envlock/profiles/default.json`

当 `ENVLOCK_RESOURCE_HOME` 未设置时：

- 资源目录：`~/.envlock/resources`

当 `HOME` 不可用时，envlock 使用字面路径：

- `~/.envlock`
- `~/.envlock/resources`

这些字面路径不会被 envlock 做 shell 展开。
