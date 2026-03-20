# 安装

在 `0.1.0-beta.0` 这条公开冷启动发布线上，请显式指定 beta tag 安装。

不带版本的安装脚本会走 GitHub 的 latest stable release 接口，因此要等首个稳定版发布后才适合作为默认入口。

## 安装当前 beta

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0
```

## 安装指定版本

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh -s -- --version v0.1.0-beta.0
```

beta 版本使用 `--version vX.Y.Z-beta.N`；稳定版发布后再使用 `--version vX.Y.Z`。

## 安装后路径

- 二进制：`~/.runseal/bin/runseal`
- 软链接：`~/.local/bin/runseal`

## 验证

```bash
runseal --version
which runseal
```

## 安装 Skill 包

```bash
runseal skill install --version v0.1.0-beta.0 --yes
```

beta 阶段建议让 skill 包和二进制保持同一个 tag。

可选：覆盖安装目录

```bash
RUNSEAL_SKILL_INSTALL_HOME="$HOME/.runseal/skills" runseal skill install --version v0.1.0-beta.0 --yes
```

## 平台说明

`install.sh` 当前打包以下目标：

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Linux 当前仅面向 GNU libc 环境：

- 支持：基于 glibc 的 `x86_64` / `aarch64` Linux
- 不支持：musl/Alpine 安装路径

如果 shell 找不到 `runseal`，请将 `~/.local/bin` 加入 `PATH`。
