---
layout: home
title: envlock 文档（简体中文）
titleTemplate: false
hero:
  name: envlock
  text: 可复现的环境会话
  tagline: 用一个 JSON profile 为 shell 或子命令注入可预测、可验证的环境变量。
  actions:
    - theme: brand
      text: 快速开始（英文）
      link: /tutorials/quick-start
    - theme: alt
      text: Profile 参考（英文）
      link: /reference/profile
    - theme: alt
      text: CLI 参考
      link: /zh-CN/reference/cli
features:
  - title: 可组合注入
    details: 在同一个 profile 中组合 env、command、symlink 注入，减少手工步骤。
  - title: 默认安全
    details: 以用户目录为默认作用域，输出模式明确可控。
  - title: 易于脚本化
    details: 按需选择 shell 导出、JSON 输出或 command 模式，便于接入流水线。
  - title: 发布可验证
    details: 提供 self-update 与基于标签的发布流程，便于稳定分发。
---

## GitHub 状态

<div class="github-status-badges">

[![CI](https://github.com/PerishCode/envlock/actions/workflows/ci.yml/badge.svg)](https://github.com/PerishCode/envlock/actions/workflows/ci.yml)
[![Docs](https://github.com/PerishCode/envlock/actions/workflows/docs.yml/badge.svg)](https://github.com/PerishCode/envlock/actions/workflows/docs.yml)
[![Converge](https://github.com/PerishCode/envlock/actions/workflows/converge.yml/badge.svg)](https://github.com/PerishCode/envlock/actions/workflows/converge.yml)
[![Latest Release](https://img.shields.io/github/v/release/PerishCode/envlock?sort=semver)](https://github.com/PerishCode/envlock/releases)

</div>

## 冷启动验证

```bash
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

## 信息架构

文档按四个层次组织：

- Tutorial：一条从零到可运行的完整路径（当前为英文页面）。
- How-to：围绕具体任务的操作指南。
- Reference：命令与配置的权威说明。
- Explanation：设计边界、取舍与常见疑问。

## 支持

- FAQ：[`/zh-CN/explanation/faq`](/zh-CN/explanation/faq)
- 故障排查（英文）：[`/explanation/troubleshooting`](/explanation/troubleshooting)
- GitHub Issues：[`PerishCode/envlock/issues`](https://github.com/PerishCode/envlock/issues)

## 60 秒启动

```bash
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"ENVLOCK_PROFILE":"default"}}]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

- 需要迁移说明：见 [迁移到 v0.3](/zh-CN/how-to/migrate-to-v0.3)。
- 需要复制即用命令：见 [常见用法](/zh-CN/how-to/common-recipes)。
- 需要高频命令速查：见 [快速参考](/zh-CN/reference/quick-reference)。
- 需要 CI 接入：见 [CI 集成](/zh-CN/how-to/ci-integration)。
- 需要发布门禁：见 [发布验证](/zh-CN/how-to/release-validation)。
- 需要发布操作步骤：见 [发布操作指南](/zh-CN/how-to/release-operator-playbook)。
- 需要文档检查流程：见 [文档维护](/zh-CN/how-to/docs-maintenance)。
- 英文规范入口：见 [English Home](/)。
- 英文迁移页：见 [Migrate to v0.3](/how-to/migrate-to-v0.3)。
