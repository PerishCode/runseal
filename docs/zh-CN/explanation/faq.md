# 常见问题（FAQ）

## v0.2+ 默认 profile 在哪里？

不传 `--profile` 时，`envlock` 按以下顺序查找：

1. `$ENVLOCK_HOME/profiles/default.json`
2. `~/.envlock/profiles/default.json`

## 什么时候需要 `--profile`？

当你希望临时切换 profile，或项目内 profile 不在默认路径时使用：

```bash
envlock --profile ./profiles/dev.json
```

## `preview` 会执行注入动作吗？

不会。`preview` 是只读检查：

- 不执行 command injection
- 不写入环境
- 不创建/修改 symlink

## 如何选择 shell / JSON / command mode？

- shell 模式（默认）：`eval "$(envlock)"`，适合交互式 shell。
- JSON 模式：`envlock --output json`，适合读取结构化输出的自动化脚本。
- command mode：`envlock -- <cmd...>`，适合把注入范围限制在单个子进程。

## `ENVLOCK_HOME` 与 `ENVLOCK_RESOURCE_HOME` 区别是什么？

- `ENVLOCK_HOME`：默认 profile 根目录（`profiles/default.json`）
- `ENVLOCK_RESOURCE_HOME`：`resource://` 与 `resource-content://` 的资源根目录

## 老的 `--use` 怎么迁移？

迁移到“约定优先 + 显式覆盖”即可：

- 日常：`envlock`
- 临时覆盖：`envlock --profile <path>`

提示：`--use` 与 `ENVLOCK_PROFILE_HOME` 是 v0.1 的旧行为。

## 出现 “default profile not found” 时先做什么？

先创建默认 profile 再重试：

```bash
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
envlock preview --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
```
