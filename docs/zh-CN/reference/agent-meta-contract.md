# Agent Meta 契约

这是首页冷启动的机器优先契约。Agent 应先读这些 meta 字段，再决定是否读取 DOM。

## 必填字段

- `agent:contract:version`：用于兼容性判断的版本号。
- `agent:index:v1`：最小契约字段列表（逗号分隔）。
- `agent:mode`：执行模式（`meta-first`）。
- `agent:entry:install`：安装入口。
- `agent:entry:cli`：CLI 参考入口。
- `agent:entry:ci`：CI 集成入口。
- `agent:resolution`：meta 与 DOM 冲突时的裁决策略。
- `agent:locale:default`：默认语言。
- `agent:locale:source`：当前语言来源（`html_lang`）。
- `agent:locale:policy`：语言与入口的应用策略。

## 解析规则

1. 先读取 `agent:index:v1`，按索引取字段。
2. 任务路由优先使用 `agent:entry:*`，不做自由推断。
3. 若 meta 与 DOM 不一致，按 `agent:resolution` 执行。
4. 在 `meta-first` 模式下，DOM 只用于二次校验。

## 当前策略

- 规范入口是 `/envlock/` 下的英文路径。
- 语言来源是 `<html lang>`。
- 页面文案可按语言变化，但入口契约保持稳定。

## 验证命令

```bash
pnpm run docs:build
bash scripts/verify-agent-meta.sh
bash scripts/check-agent-routes.sh
```
