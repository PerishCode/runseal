---
layout: home
title: envlock 文档（简体中文）
titleTemplate: false
hero:
  name: envlock
  text: 可复现的环境会话
  tagline: 用一个 JSON profile 为 shell 或子命令注入可预测、可验证的环境变量。
  image:
    src: /hero-shell.svg
    alt: envlock shell quick verify
    class: hero-shell-image
    width: 720
    height: 480
  actions:
    - theme: brand
      text: 快速开始
      link: /tutorials/quick-start
    - theme: alt
      text: Profile 参考
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

## 快速入口

- 安装：[`/zh-CN/how-to/install`](/zh-CN/how-to/install)
- 常见用法：[`/zh-CN/how-to/common-recipes`](/zh-CN/how-to/common-recipes)
- CI 集成：[`/zh-CN/how-to/ci-integration`](/zh-CN/how-to/ci-integration)
- 发布验证：[`/zh-CN/how-to/release-validation`](/zh-CN/how-to/release-validation)
- 发布操作指南：[`/zh-CN/how-to/release-operator-playbook`](/zh-CN/how-to/release-operator-playbook)
- 文档维护：[`/zh-CN/how-to/docs-maintenance`](/zh-CN/how-to/docs-maintenance)
- 快速参考：[`/zh-CN/reference/quick-reference`](/zh-CN/reference/quick-reference)
- CLI 参考：[`/zh-CN/reference/cli`](/zh-CN/reference/cli)
- v0.3 迁移：[`/zh-CN/how-to/migrate-to-v0.3`](/zh-CN/how-to/migrate-to-v0.3)

## 支持

- FAQ（中文）：[`/zh-CN/explanation/faq`](/zh-CN/explanation/faq)
- FAQ（英文）：[`/explanation/faq`](/explanation/faq)
- 故障排查（英文）：[`/explanation/troubleshooting`](/explanation/troubleshooting)
- GitHub Issues：[`PerishCode/envlock/issues`](https://github.com/PerishCode/envlock/issues)

## 语言说明

- 英文文档是规范入口：[`/`](/)
- 中文文档可能略晚同步，破坏性变更的迁移说明会同步更新
