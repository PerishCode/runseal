# 迁移到 v0.3

本页说明 v0.4.2 的破坏性变更。

## 1) command mode 的 strict 规则更一致

在 v0.4.2 中，`--strict` 会对所有输出路径生效，包括 command mode。

- 严格模式下，重复 key 会在子命令执行前直接失败。
- 非法环境变量 key 也会在子命令执行前失败。

迁移动作：

1. 确保注入链路中每个 key 最终只保留一个值。
2. 把非法 key 重命名为合法格式（`[A-Za-z_][A-Za-z0-9_]*`）。

## 2) Rust 模块导入改为显式边界

crate 现在使用显式分层：

- 运行时核心：`envlock::core::*`
- 子命令：`envlock::commands::*`

如果你之前使用旧的扁平导入路径，请直接替换到新路径。

示例：

```rust
use envlock::core::app::App;
use envlock::core::config::RuntimeConfig;
use envlock::commands::preview::run as run_preview;
```

## 3) 验证迁移

```bash
envlock --version
envlock preview --profile ./profiles/dev.json
envlock --profile ./profiles/dev.json --output json
envlock --strict --profile ./profiles/dev.json -- bash -lc 'env | grep ENVLOCK || true'
```

以上命令行为符合预期，即可视为迁移完成。
