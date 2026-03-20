---
bodyClass: post-page
title: 04 | Which Boundaries Are Not Ours
meta: "style reference: Stripe, Rust"
---

helper 很容易越做越大，一个常见原因就是：把所有脏边界都算到 helper 自己头上。

这是错误的。

有些问题属于上游生态的历史包袱；有些问题不是 runseal 发明的，只是在 runseal 拒绝魔法之后被强行显式化了；只有其中一部分，才真的是 runseal 自己要承担的设计边界。

把这三层拆开很重要，因为它决定了 runseal 应该解决什么、应该暴露什么、以及应该拒绝把什么继续吞进核心里。

## 1. 早于 runseal 存在的生态包袱

这些问题不是 runseal 或 `:node` helper 引入的，它们原本就存在于 Node 生态里：

- `npm`、`pnpm`、`yarn`、`corepack` 各自背着不同的历史假设
- `packageManager` 字段与真实命令入口可能不一致
- 不同 package manager 之间的 lockfile 切换
- cache/store/global-state 语义本来就不统一
- 普通 CLI 的工作目录语义
- monorepo 与 nested workspace 本来就存在解析歧义
- native addon 与 postinstall 对宿主系统工具链的依赖
- `npm i -g ...` 与 `corepack` 激活流之间本来就有张力

这些都不是从 runseal 开始的。

## 2. runseal 逼出来的显式问题

这些问题也不是 runseal 凭空制造的，但一旦 runseal 坚持运行时边界显式，它们就不能继续含糊过去。

比如：

- profile intent、`packageManager`、corepack shim、本地二进制、吸收进来的全局 manager，一旦冲突谁说了算？
- 在一个被 seal 的 runtime 里，PATH precedence 到底是什么意思？
- 最终命令入口的所有权到底归谁？
- profile 激活之后，什么才算 contamination？
- helper 到底能保留多少状态，才不会失去确定性？

`cwd` 就是一个很典型的例子。

工作目录会深刻影响 package-manager 的行为，但它不是 runseal 发明出来的概念，而是 CLI 世界约定俗成的前提。runseal 当然需要在文档和诊断里承认它，但这不等于 runseal 要重新定义它。

runseal 没有发明这些张力。

它只是拒绝再让这些张力继续藏着。

## 3. 真正属于 runseal 的边界

下面这些，才是 runseal 和 helper 自己必须承担的事情：

- `RUNSEAL_HELPER_NODE_HOME` 下的版本目录布局
- `install / list / which / remote list / snapshot / uninstall / help / example` 这些命令面
- `bin/` 与 `corepack-bin/` 是继续分开还是逐步收敛
- 哪些 runtime 路径会被显式暴露到 profile patch 里
- `help / stdout / stderr / which / snapshot` 如何把 provenance 说清楚
- 一旦通过 profile 进入，就必须绝对优先走 managed path 这条承诺

而**不**属于 runseal 的，是发明一套新的工作目录模型。runseal 可以把 cwd 当作 signal，但不应该假装自己拥有普通 CLI 的目录语义。

这些不是继承来的问题。

这些是 runseal 的产品决策。

## 为什么这层拆分很重要

如果不先拆清楚，就会发生两件坏事。

第一，helper 会开始试图“解决整个 Node 生态”，而不是维持一个清楚的 runtime boundary。

第二，核心会开始替那些它并没有制造的问题背负实现债务。

更合理的做法应该是：

- 生态包袱承认它存在
- 被 runseal 逼出来的模糊地带，主动把它显式化
- 真正属于 runseal 的边界，认真设计并认真文档化

只有这样，helper 才不会滑向一个没有尽头的兼容性黑洞。

## 实际上的下一步

这篇文章真正要导向的是 precedence contract。

在继续增加语法糖或策略之前，我们得先知道：

- 哪些是必须尊重的生态现实
- 哪些是必须显式回答的模糊地带
- 哪些是我们愿意自己负责到底的产品规则

只有这三层拆清楚了，后面讨论“冲突时谁优先”才有意义。
