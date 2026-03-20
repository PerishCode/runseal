---
bodyClass: post-page
title: 05 | Which Node Surface Wins
meta: "style reference: Stripe, Rust"
---

这篇文章是一份合同，不是一篇散文。

它只回答一件事：当多个 Node 相关表面彼此冲突时，到底谁赢。

## 一屏内说清楚

一旦 runseal profile 激活，managed runtime 就必须赢。

更具体地说，当前的优先顺序是：

1. profile 管理的 runtime
2. 项目内 task 级可执行入口
3. managed `bin/`
4. managed `corepack-bin/`
5. 项目 metadata 作为 signal
6. cwd / workspace discovery 作为 signal
7. 宿主机环境只作为 fallback

低一层的东西，不能悄悄反超高一层。

## 各层是什么意思

### 1. profile 管理的 runtime

这是最高权威层。

只要 profile 已经激活，runseal 管理的 Node surface 就是运行时契约。它决定当前活跃的是哪个版本根目录、哪些路径被注入、以及哪些 helper-managed surface 可用。

### 2. 项目内 task 级可执行入口

`node_modules/.bin` 可以在项目内 task 执行时赢，尤其是在 package manager 已经进入项目上下文之后。

但它不能反过来决定 runtime identity。也就是说，本地 package executable 可以覆盖项目内 task context 里的具体命令选择，但不能替换掉 profile 已经选定的 Node runtime。

### 3. managed `bin/`

这是当前更主要的 helper-managed command surface。

如果同一个命令名同时出现在普通 `bin/` 和 `corepack-bin/` 里，那么当前阶段 `bin/` 先赢。

### 4. managed `corepack-bin/`

这是通过 `corepack` materialize 出来的、但仍然被 helper 管理的 shim 表面。

它依然属于 managed runtime，只是当前在同名命令冲突时，优先级低于普通 `bin/`。

### 5. 项目 metadata

`packageManager` 这类信息很重要，但它首先是 signal。

它可以参与 helper 的 manager 选择、校验、warning 或 shim 准备，但它不能只凭自己就越过 active profile 成为最高权威。

### 6. cwd / workspace discovery

discovery 决定的是作用域，不是权威。

它帮助我们判断当前相关的 project root 和 metadata 是什么，但不能悄悄改变最终的 runtime 赢家。

另外，普通的工作目录语义仍然属于 CLI 世界的约定俗成行为。runseal 可以观察 cwd，但不应该重新定义 cwd。

### 7. 宿主机环境

Homebrew、Volta、fnm、nvm、全局 npm install、宿主机 Corepack 状态，在 profile 激活之后都不再具有权威性。

它们可以存在，但不能悄悄赢。

## 几条典型冲突规则

- profile intent 和 host PATH 冲突时，profile 赢。
- profile runtime 和宿主机全局 `pnpm` 冲突时，profile 赢。
- profile 已经激活时，如果某个项目任务通过 `node_modules/.bin` 解析到本地入口，本地 task entry 只对这个任务生效。
- `packageManager` 和 ambient manager 冲突时，`packageManager` 可以影响 helper 行为，但 ambient state 仍然不能反超 active profile。
- `corepack-bin/` 和 `bin/` 在 manager 入口上冲突时，当前由 `bin/` 先赢。

## 当前明确保证的事情

- active profile intent 一定高于宿主机环境
- managed runtime surface 是可检查、可预测的
- 本地 task executable 可以赢 task execution，但不能赢 runtime identity
- 在同名 manager 上，当前普通 `bin/` 高于 `corepack-bin/`
- metadata 可以提供信号，但不是悄悄生效的隐藏权威层
- 工作目录属于继承来的 CLI 行为，不是 runseal 另起的一层权威面

## 还没有彻底定义的部分

- 所有 mixed-manager monorepo 的精确 tie-breaking
- `bin/` 与 `corepack-bin/` 的长期最终叙事
- metadata 与 active runtime 冲突时，应该自动修复到什么程度

这些部分还在演进。

上面这份 contract，是当前不能后退的地板。
