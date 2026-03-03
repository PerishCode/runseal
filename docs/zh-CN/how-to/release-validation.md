# 发布验证

面向维护者的最小验证清单。

## 发布前

```bash
bash scripts/converge-check.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

确认命令输出包含 `PASS`。

## 发布后

```bash
bash scripts/verify-public-surface.sh
bash scripts/release-smoke.sh --version vX.Y.Z
```

任一失败即停止对外发布，先修复再继续。
