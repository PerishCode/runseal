# FAQ

## `resource://` 会在什么位置截断？

`env` 注入在解析 `resource://` 时，会在遇到 `:` 或 `;` 时停止。

这对 PATH 一类值是有意设计，但也意味着带 `:` 的 URL 字面量可能会比预期更早被截断。如果你需要原样 URL，请不要把它放进 `resource://` 解析里。

## runseal 会在什么时候发现资源文件缺失？

`resource://` 解析会在导出阶段把相对路径转换成绝对路径，但不会在 parse / validate 阶段保证文件存在。

如果资源文件缺失，通常会在下游工具真正读取该路径时才暴露出来。

## 如果 `HOME` 缺失会发生什么？

如果 `HOME` 不可用且 `RUNSEAL_HOME` 也未设置，runseal 会直接退出，并给出可执行的错误提示；不会再退回到字面量 `~/.runseal`。

## 默认 profile 缺失时应该先检查什么？

默认情况下，runseal 会在 `RUNSEAL_HOME` 指向的根目录下查找 `profiles/default.json`；如果 `RUNSEAL_HOME` 未设置，则使用 `~/.runseal`。

建议先检查：

- `RUNSEAL_HOME` 是否指向了你预期的根目录
- 该根目录下是否存在 `profiles/default.json`
- 你是否其实想使用项目内 profile，但没有显式传 `--profile`
