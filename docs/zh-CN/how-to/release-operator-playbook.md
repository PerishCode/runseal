# 发布操作手册

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

## 4）确认发布流水线完成

检查 `release.yml` 成功并产出压缩包与 `checksums.txt`。

## 5）发布后校验

```bash
bash scripts/verify-public-surface.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

任一失败，先修复再对外公告。
