# 发布操作指南

维护者最小发布流程（可复用、可追溯）。

## 1）校验版本同步

```bash
bash scripts/version-sync-check.sh
```

## 2）执行发布前总闸门

```bash
bash scripts/release-ready.sh
```

## 3）打标签并推送

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

## Beta 路径

如果是 prerelease 验证，不要走 stable tag 发布。

1. 先把 `Cargo.toml` 版本改成 beta 版本，例如 `0.4.4-beta.2`。
2. 推送包含 beta 改动的分支。
3. 手动运行 `release-beta.yml`，输入 `v0.4.4-beta.2`。
4. 等待 GitHub Releases 上出现 prerelease 产物。
5. 直接针对已发布 beta tag 做安装验证：

```bash
bash scripts/install.sh --version v0.4.4-beta.2
envlock --version
envlock plugin node init --help
```

## 4）确认发布流水线完成

检查 `release.yml` 成功并产出压缩包与 `checksums.txt`。

如果是 beta，检查 `release-beta.yml` 成功，并确认 GitHub prerelease 中包含压缩包与 `checksums.txt`。

## 5）发布后校验

```bash
bash scripts/verify-public-surface.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

任一失败，先修复再对外公告。
