# 发布流水线

## 触发条件

- CI：`main` 分支 push 与 Pull Request。
- Release：匹配 `v*` 的 tag push。
- 文档部署：`main` 上影响 `docs/**` 或 docs workflow 文件的提交。

## 发布工作流

1. `release.yml` 校验 tag 与版本一致性（`vX.Y.Z` 与 `Cargo.toml`）。
2. 按目标平台构建：
   - `x86_64-unknown-linux-gnu`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
3. 生成二进制压缩包、`skill-vX.Y.Z.zip` 与 `checksums.txt`。
4. 将产物发布到 GitHub Release。

## 维护者步骤

```bash
# 合并改动并更新 Cargo.toml 版本后
git tag v0.4.2
git push origin v0.4.2
```

## 破坏性变更规则

- 不在运行时代码里维护向后兼容分支来保留旧行为。
- 若外部行为或 API 发生破坏性变更，必须升级 Y 版本，并在同一发布周期同步 EN + zh-CN 迁移文档。
