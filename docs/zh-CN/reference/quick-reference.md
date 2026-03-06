# 快速参考

面向 v0.4.2 的高频命令速查。

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
envlock self-update --version v0.4.2 --yes

# 同步安装 skill 包
envlock skill install --yes
```

## Profiles 与 Alias

```bash
# 查看本地 profile 状态
envlock profiles status

# 初始化模板 profile
envlock profiles init --type minimal

# 注册 alias 到 profile
envlock alias append work --profile ~/.envlock/profiles/work.json
envlock alias list

# 显式执行
envlock alias run work

# 快捷执行
envlock :work
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
