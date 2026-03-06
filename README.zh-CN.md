# envlock

用一个 JSON 配置文件构建可复现的 shell/命令环境。

English canonical README: [README.md](README.md).

## 10 秒价值句

一条命令加载一个 profile，并用一个可观察结果完成验收（`ENVLOCK_PROFILE=default`）。

## 60 秒验证路径

```bash
# 1) 安装
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh

# 2) 创建默认 profile
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"ENVLOCK_PROFILE":"default"}}]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"

# 3) 应用并验证
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

期望输出：

```text
default
```

不传 `--profile` 时，默认查找顺序：

- 若设置了 `ENVLOCK_HOME`：`ENVLOCK_HOME/profiles/default.json`
- 否则：`~/.envlock/profiles/default.json`

## 适用 / 不适用边界

适用场景：

- 用 JSON profile 复现 shell 环境
- 应用前先只读预览（`envlock preview`）
- 在 CI 中稳定复用同一环境注入逻辑

不适用场景：

- 作为密钥管理器
- 作为运行时/容器编排器
- 替代完整包管理器

## 直达链接

- 安装：[docs/zh-CN/how-to/install.md](docs/zh-CN/how-to/install.md)
- CLI 参考：[docs/zh-CN/reference/cli.md](docs/zh-CN/reference/cli.md)
- CI 集成：[docs/zh-CN/how-to/ci-integration.md](docs/zh-CN/how-to/ci-integration.md)
- First-star 触发路径：[docs/zh-CN/tutorials/first-star-trigger.md](docs/zh-CN/tutorials/first-star-trigger.md)

安装路径：

- 二进制：`~/.envlock/bin/envlock`
- 软链接：`~/.local/bin/envlock`

## 常用命令

```bash
# 使用默认 profile
envlock

# 指定 profile 路径
envlock --profile ./profiles/dev.json

# 只预览 profile 元信息（只读）
envlock preview --profile ./profiles/dev.json

# 检查更新与执行更新
envlock self-update --check
envlock self-update
```

## 文档

- 文档站点：https://perishcode.github.io/envlock/
- 英文 README：[README.md](README.md)
- 快速开始：[docs/tutorials/quick-start.md](docs/tutorials/quick-start.md)
- First-star 触发路径：[docs/zh-CN/tutorials/first-star-trigger.md](docs/zh-CN/tutorials/first-star-trigger.md)
- 安装指南：[docs/zh-CN/how-to/install.md](docs/zh-CN/how-to/install.md)
- 快速参考（中文）：[docs/zh-CN/reference/quick-reference.md](docs/zh-CN/reference/quick-reference.md)
- 常见用法（中文）：[docs/zh-CN/how-to/common-recipes.md](docs/zh-CN/how-to/common-recipes.md)
- CI 集成（中文）：[docs/zh-CN/how-to/ci-integration.md](docs/zh-CN/how-to/ci-integration.md)
- CLI 参考（中文）：[docs/zh-CN/reference/cli.md](docs/zh-CN/reference/cli.md)
- FAQ（中文）：[docs/zh-CN/explanation/faq.md](docs/zh-CN/explanation/faq.md)
- FAQ（英文）：[docs/explanation/faq.md](docs/explanation/faq.md)
- 设计说明：[docs/explanation/design-boundaries.md](docs/explanation/design-boundaries.md)
- 语言维护策略：[docs/explanation/language-maintenance.md](docs/explanation/language-maintenance.md)

## 验证

```bash
bash scripts/version-sync-check.sh
bash scripts/release-ready.sh
bash scripts/converge-check.sh
bash scripts/release-smoke.sh --version v0.4.2
```

## 故障排查（快速定位）

提交 issue 前，先执行这 3 条命令：

```bash
envlock --version
envlock preview --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
envlock --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json" --output json
```

如果命令失败，请把命令和完整输出一起贴到 issue。

## 项目链接

- Releases：https://github.com/PerishCode/envlock/releases
- 变更记录：https://github.com/PerishCode/envlock/releases
- 文档站点：https://perishcode.github.io/envlock/
- 迁移指南（v0.3）：https://perishcode.github.io/envlock/how-to/migrate-to-v0.3
