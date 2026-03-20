---
bodyClass: post-page
title: 03 | Where Corepack Fits
meta: "style reference: Stripe, Rust"
---

`corepack` 不应该被当作 baseline。

当前 `:node` helper 的 baseline 更简单：

- `node`
- `npm`

这已经足够建立一个可用、可 seal 的 runtime，也足够通过正常运行时动作去吸收二级包管理器。

`corepack` 应该位于这层 baseline 之上。

## `corepack` 的角色

`corepack` 不是 runtime 本身。

它更像一个 manager resolver 与 shim system。

所以真正的问题不是“要不要让 `corepack` 取代 baseline”，而是：如何把 `corepack` 也收编进同一个受控版本根目录，而不让它逃逸出 runseal 的模型。

## runseal 必须控制什么

从 runseal 的视角看，`corepack` 主要影响两件事：

- cache / 下载状态
- 最终命令入口

所以最关键的两个表面就是：

- `COREPACK_HOME`
- `RUNSEAL_COREPACK_SHIMS`

只要这两者被收进受控目录，`corepack` 就不再是隐藏全局状态的来源，而会变成另一个可被 seal 的运行时行为。

## 目标目录模型

对某个目标 Node 版本来说，`corepack` 应该被压进同一个版本根目录：

```text
$RUNSEAL_HELPER_NODE_HOME/
  versions/
    vX.Y.Z/
      .lock/
      bin/
      corepack-bin/
      cache/
        corepack/
      lib/
        node_modules/
```

这样模型才是一致的：

- Node runtime 在版本根目录里
- npm baseline 在版本根目录里
- 被吸收的 managers 在版本根目录里
- `corepack` 的 cache 在版本根目录里
- `corepack` 的 shim 在独立的 `corepack-bin/` 目录里

不要再有一个额外的全局 manager 世界，也不要有宿主机层面的隐形 shim。

## 第一条实现规则

最终入口仍然属于 helper 管理的运行时路径。

这是最关键的约束。

`corepack` 可以参与解析、下载、准备 manager，但不能让最终命令入口重新变成不透明的东西。如果 `pnpm` 或 `yarn` 通过 `corepack` 变得可用，那么对外可见的入口仍然必须落在 helper 管理的运行时路径里。在当前实现里，`corepack-bin/` 会作为一层显式暴露的平行路径存在，但普通的 helper `bin/` 仍然是更主要的 runtime surface。

换句话说：

- `corepack` 可以 resolve
- `corepack` 可以 cache
- `corepack` 可以 prepare
- 但 runseal 仍然拥有 runtime boundary

也就是说，当前同时有两件事成立：

- `corepack-bin/` 是可见、可检查的一层
- `bin/` 仍然是普通命令解析时更简单也更主要的入口层

## 预期的命令故事

在叙事上，我们希望两条路径最后收敛成同一个结果。

路径 A：

```bash
runseal :node24-profile npm i -g pnpm
```

路径 B：

```bash
runseal :node24-profile corepack enable pnpm
runseal :node24-profile corepack prepare pnpm@10.30.3 --activate
```

如果两条路径都成立，那它们最终应该收敛到同一个产品承诺：

- manager 从 managed tree 解析
- manager 状态留在 managed tree
- profile 仍然是 runtime entrypoint

## 这对验证意味着什么

`corepack` 现在还不应该立刻变成一个完整矩阵维度。

更合理的做法，是先把它当作一条并行吸收路径，用一小组承诺去验证：

- `COREPACK_HOME` 是否被正确重定向到受控版本根目录
- prepare 出来的 manager 是否没有逃出 managed tree
- 最终可见 shim 是否来自 helper 管理的运行时路径，并且在 `bin/` 与 `corepack-bin/` 同时存在时，当前是否仍由 `bin/` 先赢
- `corepack` 激活之后，profile 重入是否仍然稳定

等到 `corepack` 成为正式用户路径，而不再只是探索路径时，再扩张验证矩阵。

## 已经落地的部分

当前实现已经越过了“纯设计讨论”的阶段，开始变成了具体运行时行为。

到现在为止，`:node` 已经显式暴露了：

- `COREPACK_HOME`
- `RUNSEAL_COREPACK_SHIMS`
- 版本目录内的 `cache/corepack/`
- 版本目录内的 `corepack-bin/`

并且当前测试已经证明：

- `corepack prepare pnpm@... --activate` 可以通过 runseal profile 执行
- prepare 出来的 `pnpm` 可以通过受控 runtime surface 被解析到
- `corepack enable pnpm` 与 `corepack enable yarn` 都会把状态落进受控版本树
- `corepack` 不需要另起一套 host-global 世界也能工作

所以现在剩下的问题已经不再是“corepack 能不能进入这个模型”，而是更窄、更具体的几个问题：

- 最终命令入口的所有权要怎么对外讲清楚
- `bin/` 与 `corepack-bin/` 的内部区分，到底要保留多少用户可见性
- 到什么阶段，`corepack` 才算从平行受控路径升级为正式用户路径

## 真正的边界

这也是为什么 `corepack` 值得重视，但并不可怕。

它不是另一套哲学，只是另一个需要被 runseal 压进显式运行时边界里的历史系统。

如果 runseal 能干净地做到这一点，那么 `corepack` 也会和其它东西一样，回到同一条宪法下：

agent-native、显式、可 seal。
