---
layout: home
title: envlock Docs
titleTemplate: false
hero:
  name: envlock
  text: Deterministic Environment Sessions
  tagline: Build reproducible shell and command environments from one JSON profile.
  actions:
    - theme: brand
      text: Quick Start
      link: /tutorials/quick-start
    - theme: alt
      text: Profile Reference
      link: /reference/profile
    - theme: alt
      text: CLI Reference
      link: /reference/cli
features:
  - title: Composable Injections
    details: Mix env variables, command bootstrap, and symlink setup in a single profile.
  - title: Safe Defaults
    details: User-scoped install paths and predictable output modes keep changes explicit.
  - title: Scriptable
    details: Use shell export mode, JSON mode, or command mode depending on your pipeline.
  - title: Release Ready
    details: Built-in self-update plus a tag-driven release workflow for binary distribution.
---

## Information Model

This documentation is split into four parts:

- Tutorial: one complete path for first success.
- How-to: task-focused guides for common operations.
- Reference: authoritative syntax and option tables.
- Explanation: design intent, boundaries, and tradeoffs.

## Support

- FAQ: [Common Questions](/explanation/faq)
- Troubleshooting: [Troubleshooting Guide](/explanation/troubleshooting)
- Issues: [GitHub Issues](https://github.com/PerishCode/envlock/issues)

## Start in 60 seconds

```bash
mkdir -p "${ENVLOCK_HOME:-$HOME/.envlock}/profiles"
printf '%s\n' '{"injections":[{"type":"env","vars":{"ENVLOCK_PROFILE":"default"}}]}' > "${ENVLOCK_HOME:-$HOME/.envlock}/profiles/default.json"
eval "$(envlock)"
echo "$ENVLOCK_PROFILE"
```

- Need migration details? See [Migrate to v0.2](/how-to/migrate-to-v0.2).
- Need copy-paste tasks? See [Common Recipes](/how-to/common-recipes).
- Need fast command lookup? See [Quick Reference](/reference/quick-reference).
- Need CI setup? See [CI Integration](/how-to/ci-integration).
- Need release gates? See [Release Validation](/how-to/release-validation).
- Prefer Chinese docs? Start at [简体中文入口](/zh-CN/).
- Chinese migration guide: [迁移到 v0.2](/zh-CN/how-to/migrate-to-v0.2).
