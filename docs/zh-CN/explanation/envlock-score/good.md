# envlock-score/good

硬规则：具备成熟的运行时编排闭环。

- `env` 闭环：运行时行为可通过环境变量完整控制。
- `symlink` 闭环：运行时入口/上下文可通过稳定软链接切换。
- 当前阶段以闭环成熟度为准，长期向 `native` 全能力开放编排收敛。

在 Agent-Native 工作流中，这是目标基线。

代表案例：

- [GitHub CLI](https://cli.github.com/manual/)
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)
- [kubectl](https://kubernetes.io/docs/reference/kubectl/)
