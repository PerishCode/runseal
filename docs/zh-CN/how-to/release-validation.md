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

## Beta 验证

如果目标版本是 `v0.4.4-beta.2` 这类 prerelease，走下面这条路径：

1. 用精确 beta tag 手动触发 `release-beta.yml`。
2. 从已发布 beta release 安装：

```bash
bash scripts/install.sh --version v0.4.4-beta.2
```

3. 验证的是安装后的行为，而不是本地构建产物：

```bash
envlock --version
envlock plugin node init --help
envlock plugin node preview --help
```

4. 在提升到 stable 之前，重新跑一轮安装后二进制上的 plugin-node 端到端验证。
