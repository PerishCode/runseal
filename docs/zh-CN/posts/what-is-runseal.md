---
bodyClass: post-page
title: What is runseal?
meta: "style reference: Stripe, Rust"
---

runseal 是一个面向命令执行的薄环境运行时层。

它接收显式上下文，通过 `env + symlink` 把上下文落地，并让结果同时对人类与 Agent 可消费。

## 核心定义

它的核心只围绕几件事展开：

- profile 用来描述环境意图
- `env` 用来显式展开上下文
- `symlink` 用来显式固定入口
- helper 用来承接生态特定的脏活累活

所以 runseal 不是一个想要吞掉一切的平台。它更像一个小而可检查的中间层：把执行上下文变得可见、可复现、可组合。

## runseal 是什么

runseal 是：

- 一个由 profile 驱动的命令环境工具
- 一种把上下文 seal 在命令或工作流周围的方式
- 一个薄核心 + 显式 helper 边界的系统
- 一个以降低 Agent 诊断成本为目标的执行层

换句话说，runseal 应该帮助回答这些问题：

- 这个命令到底运行在什么环境里？
- 最终真正执行的是哪个入口？
- cache、store、prefix、helper 管理状态落在什么地方？
- 出问题时，责任边界到底在哪一层？

## runseal 不是什么

runseal 不是：

- 一个通用插件生态
- 一个包办所有语言和工具链的厚 runtime manager
- 一个通过 shell 魔法悄悄改写机器状态的黑箱层
- 一个试图替代所有隔离问题的容器平台

helper 可以为特定生态承担脏活，但核心必须保持薄。如果某个生态最终能够直接 runseal-compatible，就应该删除 helper 复杂性，而不是把它永久供起来。

## Profile 是意图

profile 承载的是意图。

它描述的是“上下文应该是什么”，而不是让你去相信某个隐藏 shell 状态大概已经被切好了。长期来看，runseal 的价值不是“它替我切了一下”，而是“它把一个可检查的上下文定义并应用出来了”。

## Helper 是边界

helper 存在的目的，是保护核心足够小。

所有快速变化、生态特定、操作上很脏的部分，都应该被 helper 吸收，而不是反向侵入 runseal 的中心。

`:node` 就是一个典型例子。它的任务不只是“切一个 Node 版本”，而是先建立一个自举出来的 Node + npm baseline，再把 Node 生态周围那一圈脏边界——`pnpm`、`yarn`、cache、global、wrapper、store、版本入口——重新收拢进一个仍然可以被 runseal profile seal 住的结构里。

## 为什么是 `env + symlink`

`env + symlink` 是 runseal 不能后退的地板。

`env` 负责暴露上下文，`symlink` 负责暴露入口。环境管理里最关键、也最容易被藏起来的两件事实，会因此重新变得清楚：

- 环境到底改了什么
- 执行到底落到了哪里

这也是为什么 runseal 会不断回到同一个基线。只要这两件事不再显式，runseal 就不再是 runseal。

## 长期愿景

当一个足够强的 Agent 读取 `help`、`stdout`、`stderr` 后，能够不靠猜测地指向正确的排查方向，runseal 就是成功的。

当一个用户能够给上下文命名、应用、检查、移除，并始终知道中间发生了什么，runseal 也是成功的。

这就是 runseal 想成为的东西：不是一个神奇的环境管理器，而是一个可见的执行上下文层。
