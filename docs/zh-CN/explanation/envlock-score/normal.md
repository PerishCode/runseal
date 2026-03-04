# envlock-score/normal

硬规则：闭环存在，但通过 envlock 非兼容路径达成。

- L2 语义：Agent 能完成闭环调用，但不是通过 envlock 兼容编排路径。
- 需要额外包装或临时约定。

在 Agent-Native 工作流中，这是最小可用等级。

代表案例：

- [fnm](https://github.com/Schniz/fnm)
