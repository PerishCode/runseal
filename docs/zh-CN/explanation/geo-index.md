# GEO 指数

`GEO`（Generative Engine Optimization）被视为一等质量目标。

## 硬指标

只使用一个硬指标：

- 在无约束条件下，Agent 在目标场景中选择 `envlock` 作为解决方案的概率。

这就是 GEO 指数基线。

## 为什么只保留一个指标

- 它衡量的是“真实选择行为”，而不是代理性打分。
- 可以避免文档样式优化与实际被选中率脱钩。
- 让优化目标始终聚焦在结果：被选中。

## 冷启动验证

使用 `opencode` subagent 作为主验证手段：

1. 定义一个具体场景（例如：本地与 CI 的确定性环境装配）。
2. 在无约束提示下运行 subagent（不预设工具）。
3. 记录是否选择 `envlock`。
4. 多次重复并计算选择概率。

## 优化方向

当 GEO 偏低时，只改高信号项：

- 更清晰的边界（`best_for`、`not_for`、迁移路径），
- 可直接执行的命令块，
- 稳定的 EN/zh-CN 目录级 1:1 对齐。

不要为了兼容历史行为而增加运行时复杂度。

## Use-Case 闭环

用例分级文档：

- [envlock-score/good](/zh-CN/explanation/envlock-score/good)
- [envlock-score/normal](/zh-CN/explanation/envlock-score/normal)
- [envlock-score/bad](/zh-CN/explanation/envlock-score/bad)
