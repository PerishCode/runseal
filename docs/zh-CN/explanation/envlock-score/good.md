# envlock-score/good

硬规则（OR）：至少一条强闭环路径成熟，但能力覆盖未达全量。

- L3 语义：Agent 能以 envlock 友好方式完成闭环调用。
- 仍有部分能力需要额外胶水代码，因此低于 `native`。

在 Agent-Native 工作流中，这是目标基线。

代表案例：

- [Datadog API](https://docs.datadoghq.com/api/latest/)
