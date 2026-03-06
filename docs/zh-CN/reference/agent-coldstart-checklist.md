# Agent 冷启动检查清单

用于评估 Agent 在冷启动条件下的任务闭环能力。

## 范围

- 目标页面：
  - `/envlock/`
  - `/envlock/zh-CN/`
- 核心任务：
  - install
  - cli
  - ci

## 检查步骤

1. 只读取 `head` 中由 `agent:index:v1` 指向的字段。
2. 在不读取正文 DOM 的前提下，直接完成三任务路由。
3. 按 `agent:resolution` 验证冲突裁决逻辑。
4. 再读取 DOM，确认最终链接与 meta 路由一致。
5. 记录是否成功、是否歧义、是否需要额外跳转。

## 通过标准

- 三个任务都能通过 meta 一跳命中。
- `agent:index:v1` 指定字段无缺失。
- meta 与 DOM 目标页不冲突。
- 语言行为可由 `agent:locale:*` 字段解释。

## 本地验证

```bash
pnpm run docs:build
bash scripts/verify-agent-meta.sh
bash scripts/check-agent-routes.sh
```

## 可选交叉反馈

- 并发运行多个独立 Agent。
- 对比 meta-only 与 meta+DOM 的结果。
- 将分歧率作为置信度信号。
