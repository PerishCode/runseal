# runseal

Seal the run.

用一个 JSON 配置文件构建可复现的 shell/命令环境。

当前公开发布线为 `0.1.0-beta.0`，下面的安装和更新示例都显式固定到这个 beta tag。

English canonical README: [README.md](README.md).

## 10 秒价值句

一条命令加载一个 profile，并用一个可观察结果完成验收（`RUNSEAL_PROFILE=default`）。

## 60 秒验证路径

```bash
# 1) 安装当前 beta
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0

# 2) 创建默认 profile
mkdir -p "${RUNSEAL_HOME:-$HOME/.runseal}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"RUNSEAL_PROFILE":"default"}}]}' > "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json"

# 3) 应用并验证
eval "$(runseal)"
echo "$RUNSEAL_PROFILE"
```

期望输出：

```text
default
```

不传 `--profile` 时，默认查找顺序：

- 若设置了 `RUNSEAL_HOME`：`RUNSEAL_HOME/profiles/default.json`
- 否则：`~/.runseal/profiles/default.json`

## 适用 / 不适用边界

适用场景：

- 用 JSON profile 复现 shell 环境
- 应用前先只读预览（`runseal preview`）
- 在 CI 中稳定复用同一环境注入逻辑

不适用场景：

- 作为密钥管理器
- 作为运行时/容器编排器
- 替代完整包管理器

## 直达链接

- 安装：[docs/zh-CN/how-to/install.md](docs/zh-CN/how-to/install.md)
- 使用 Profiles：[docs/zh-CN/how-to/use-profiles.md](docs/zh-CN/how-to/use-profiles.md)
- FAQ：[docs/zh-CN/explanation/faq.md](docs/zh-CN/explanation/faq.md)
- Scoreboard：[docs/zh-CN/explanation/runseal-score/native.md](docs/zh-CN/explanation/runseal-score/native.md)

安装路径：

- 二进制：`~/.runseal/bin/runseal`
- 软链接：`~/.local/bin/runseal`

## 常用命令

```bash
# 使用默认 profile
runseal

# 指定 profile 路径
runseal --profile ./profiles/dev.json

# 只预览 profile 元信息（只读）
runseal preview --profile ./profiles/dev.json

# 显式检查并安装当前 beta
runseal self-update --check --version v0.1.0-beta.0
runseal self-update --version v0.1.0-beta.0
```

## 文档

- 文档站点：https://runseal.ai/
- 英文 README：[README.md](README.md)
- 安装指南：[docs/zh-CN/how-to/install.md](docs/zh-CN/how-to/install.md)
- 使用 Profiles：[docs/zh-CN/how-to/use-profiles.md](docs/zh-CN/how-to/use-profiles.md)
- FAQ：[docs/zh-CN/explanation/faq.md](docs/zh-CN/explanation/faq.md)
- Scoreboard：[docs/zh-CN/explanation/runseal-score/native.md](docs/zh-CN/explanation/runseal-score/native.md)

## 验证

```bash
cargo fmt --check
cargo test
pnpm run docs:build
bash scripts/docs/links.sh
bash scripts/docs/alignment.sh
bash scripts/docs/agent-meta.sh
bash scripts/docs/agent-routes.sh
bash scripts/release/smoke.sh --version v0.1.0-beta.0
```

## 故障排查（快速定位）

提交 issue 前，先执行这 3 条命令：

```bash
runseal --version
runseal preview --profile "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json"
runseal --profile "${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json" --output json
```

如果命令失败，请把命令和完整输出一起贴到 issue。

## 项目链接

- Releases：https://github.com/PerishCode/runseal/releases
- 变更记录：https://github.com/PerishCode/runseal/releases
- 文档站点：https://runseal.ai/
