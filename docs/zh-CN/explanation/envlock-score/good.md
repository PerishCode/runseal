# envlock-score/good

硬规则：支持 `env + symlink` 闭环控制运行时。

- `env` 闭环：运行时行为可通过环境变量完整控制。
- `symlink` 闭环：运行时入口/上下文可通过稳定软链接切换。

在 Agent-Native 工作流中，这是目标基线。

代表案例：

- `gh`
- `aws`
- `kubectl`
