# 运行子命令模式

用 command mode 在注入后的环境中运行子进程。

## 基础用法

```bash
envlock -p profile.json -- bash -lc 'echo "$ENVLOCK_PROFILE"'
```

父 shell 不会被修改。

## 退出码透传

`envlock` 会直接返回子进程退出码：

```bash
envlock -p profile.json -- bash -lc 'exit 17'
echo $?  # 17
```

## 常见工具链模式

```bash
envlock -p profile.json -- npm run build
```

适用于 CI 与本地脚本中“只在当前进程作用域注入环境”的场景。
