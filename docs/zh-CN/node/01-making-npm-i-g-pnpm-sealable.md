---
bodyClass: post-page
title: 01 | Making npm i -g pnpm Sealable
meta: "style reference: Stripe, Rust"
---

这篇文章讨论一个非常具体、但也非常核心的问题：

如何让用户执行 `npm i -g pnpm` 之后，`pnpm` 仍然稳定落在 runseal 管理的结构里，并继续可以被 profile 体系 seal 住。

## `:node` helper 的目标

`:node` helper 的目标不是简单地“切一个 Node 版本”。

它真正要做的，是把 Node 生态周围那些容易四处泄漏的边界重新收拢起来，让它们落入 runseal 可以显式管理的范围。

理想结果是，用户可以继续做熟悉的事情：

```bash
runseal helper :node install --node-version 24.12.0
runseal :node24-profile npm i -g pnpm
```

但最终生成的工具链仍然：

- 落在 runseal 管理的目录结构里
- 继续可以被 `profile.json` 消费
- 不污染系统默认全局状态

这里 helper 负责准备环境，profile 才是日常执行入口。

在当前阶段，`install` 建立的是一个自举出来的 Node + npm baseline。像 `pnpm` 这样的包管理器，则通过 `npm i -g pnpm` 这类正常运行时动作，被继续收编进同一个受控版本根目录。

## 必须被 seal 的脏边界

对于某个目标 Node 版本，helper 必须把这些脏边界都压进同一个受控版本根目录：

- `bin/` 中的可执行入口
- 包管理器产生的 shim
- `node_modules/` 与 `node_modules/.bin/`
- npm 的 cache 与 prefix 状态
- pnpm 的 store 状态
- yarn 的 cache 与 global 状态
- manager 自己运行时需要的 home-like 状态目录
- 当前版本安装过程中需要的 lock 状态

## 目标目录模型

helper 应当生成一个以 Node 版本为中心的目录结构：

```text
$RUNSEAL_HELPER_NODE_HOME/
  versions/
    vX.Y.Z/
      .lock/
      bin/
      corepack-bin/
      node_modules/
      cache/
        corepack/
      home/
      lib/
        node_modules/
```

这里有一个关键点：`npm` 属于 helper 建立的 baseline；`pnpm`、`yarn` 也不应该再拥有各自平级的独立版本树，而是应在出现时继续被收编进同一个被 seal 的 Node 环境。`corepack` 如果参与其中，它的 cache 与 shim 也同样必须留在这个版本根目录里。

## 设计上的含义

- 优先在临时目录中完成装配，再原子落位，而不是一边运行一边污染正式目录。
- 优先使用显式的受控路径，而不是依赖宿主机上的隐式全局默认值。
- Rust 核心保持薄，只负责 alias 解析、获取、缓存、执行。
- 生态特定的操作性脏活留在 helper 里。
- 一旦上游生态直接变得 runseal-compatible，就删除 helper 复杂性，而不是继续供着一层历史抽象。
