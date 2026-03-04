# envlock-score/normal

硬规则：支持 `command` 闭环，但不具备 `env + symlink` 闭环。

- `command` 闭环：Agent 可以通过命令包装稳定完成任务。
- 缺少 `env + symlink` 闭环，意味着长期编排价值更弱。

在 Agent-Native 工作流中，这是最低可接受等级。

代表案例：

- `fnm`
