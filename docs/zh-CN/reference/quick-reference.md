# 快速参考

面向 v0.3.0 的高频命令速查。

## 日常运行

```bash
# 使用默认 profile（~/.envlock/profiles/default.json）
envlock

# 显式指定 profile 路径
envlock --profile ./profiles/dev.json

# 以 JSON 输出解析结果
envlock --profile ./profiles/dev.json --output json
```

## Preview（只读）

```bash
envlock preview --profile ./profiles/dev.json
envlock preview --profile ./profiles/dev.json --output json
```

## 子命令模式

```bash
# 只对子进程注入环境
envlock --profile ./profiles/dev.json -- pnpm run build

# 返回子进程退出码
envlock --profile ./profiles/dev.json -- bash -lc 'exit 17'
echo $?
```

## 自更新

```bash
envlock self-update --check
envlock self-update
envlock self-update --yes
envlock self-update --version v0.3.0 --yes
```

## 安装与卸载

```bash
# 安装
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh

# 卸载
curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/uninstall.sh | sh
```

## 延伸阅读

- CI 用法：[/zh-CN/how-to/ci-integration](/zh-CN/how-to/ci-integration)
- 子命令模式详解：[/zh-CN/how-to/command-mode](/zh-CN/how-to/command-mode)
- 完整 CLI 选项：[/zh-CN/reference/cli](/zh-CN/reference/cli)
