# 使用 Profiles

`runseal` 支持“约定优先 + 显式覆盖”。

本页对应 `v0.1.0-beta.0` 公开 beta 版本线。

## 模式 A：显式路径

```bash
runseal -p ./profiles/dev.json
```

当 profile 与项目放在一起时，优先用这个模式。

## 模式 B：约定默认 profile

```bash
runseal
```

默认 profile 文件：

- `${RUNSEAL_HOME:-$HOME/.runseal}/profiles/default.json`

解析顺序：

1. 若设置了 `RUNSEAL_HOME`，从 `$RUNSEAL_HOME/profiles/default.json` 读取。
2. 否则从 `~/.runseal/profiles/default.json` 读取。

## 常用参数

- `--output shell`：输出 shell `export` 语句。
- `--output json`：输出 JSON 对象。
- `--strict`：最终输出存在重复 key 时失败。

## 资源 URI 展开

`env` 值支持基于 `RUNSEAL_RESOURCE_HOME` 的 URI 展开：

- `resource://path/to/file` -> 资源根目录下的绝对路径。
- `resource-content://path/to/file` -> 资源文件内容。

未设置 `RUNSEAL_RESOURCE_HOME` 时，默认：

- `~/.runseal/resources`
