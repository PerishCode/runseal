# envlock

用一个 JSON 配置文件构建可复现的 shell/命令环境。

English canonical README: [README.md](README.md).

## 30 秒安装

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh
```

安装路径：

- 二进制：`~/.envlock/bin/envlock`
- 软链接：`~/.local/bin/envlock`

## 60 秒快速开始（v0.2.1）

先创建默认 profile，再直接运行 `envlock`：

```bash
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"ENVLOCK_PROFILE":"default"}}]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

当不传 `--profile` 时，默认查找顺序：

- 先看 `ENVLOCK_HOME/profiles/default.json`（如果设置了 `ENVLOCK_HOME`）
- 否则使用 `~/.envlock/profiles/default.json`

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
- 安装指南：[docs/how-to/install.md](docs/how-to/install.md)
- 常用配方（中文）：[docs/zh-CN/how-to/common-recipes.md](docs/zh-CN/how-to/common-recipes.md)
- CLI 参考：[docs/reference/cli.md](docs/reference/cli.md)
- FAQ（中文）：[docs/zh-CN/explanation/faq.md](docs/zh-CN/explanation/faq.md)
- FAQ（英文）：[docs/explanation/faq.md](docs/explanation/faq.md)
- 设计说明：[docs/explanation/design-boundaries.md](docs/explanation/design-boundaries.md)
- 语言维护策略：[docs/explanation/language-maintenance.md](docs/explanation/language-maintenance.md)

## 故障排查快路径

提交 issue 前，先执行这 3 条命令：

```bash
envlock --version
envlock preview --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
envlock --profile "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json" --output json
```

如果命令失败，请把命令和完整输出一起贴到 issue。

## 项目信号

- Releases：https://github.com/PerishCode/envlock/releases
- 变更记录：https://github.com/PerishCode/envlock/releases
- 文档站点：https://perishcode.github.io/envlock/
- 迁移指南（v0.2）：https://perishcode.github.io/envlock/how-to/migrate-to-v0.2
