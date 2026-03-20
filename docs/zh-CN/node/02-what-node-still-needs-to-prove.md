---
bodyClass: post-page
title: 02 | What :node Still Needs to Prove
meta: "style reference: Stripe, Rust"
---

即使第一个 sealing 案例已经跑通，`:node` 也远没有结束。

这篇文章不是为了穷举所有可能的测试组合，而是为了定义：在我们当前的产品理解里，什么才算充分覆盖。

## 什么叫充分覆盖

我们不需要组合爆炸。

但我们必须确保：每一条独立的产品承诺，都在真正会发生分叉失败的地方被验证过一次。

对当前的 `:node` 模型来说，这些承诺包括：

- helper 能自举出一个可用的 `node + npm` baseline
- 二级包管理器可以通过显式 runtime entry 被吸收到受控版本树里
- 一旦被吸收，这些 manager 不只是能跑 `--version`，而是真的能承载项目工作流
- managed layout 依然保持显式，不会悄悄退回 host-global 状态

只要这些承诺都被证明了，当前模型就算得上被正确覆盖。只要其中一条还没测到，再多通过的命令也不算完整。

## 分层验证模型

避免盲区和组合爆炸的最好方式，不是继续堆命令，而是按行为层来验证。

### 1. Baseline Runtime

这一层回答的是：`:node` 能不能先把一个可用的 Node runtime 建起来，并通过 runseal profile 暴露出来？

这里至少包括：

- `install --node-version ...`
- `node -v`
- `npm -v`
- 通过受控版本树解析到正确二进制

### 2. Manager Absorption

这一层回答的是：二级包管理器能不能通过正常运行时动作，被吸收到同一个受控版本根目录里？

最典型的就是：

- `runseal :node24-profile npm i -g pnpm`
- `runseal :node24-profile npm i -g yarn`

这里关心的不只是命令成功，而是这些 manager 的二进制入口和包体内容，是否真的落在 helper 管理的版本树里。

### 3. 项目内工作流

一个只能通过 `--version` 的 manager，没有任何意义。

下一层必须验证的是真实项目工作：

- `pnpm install`
- `pnpm add`
- `pnpm run <script>`
- `yarn install`
- `yarn add`
- `yarn run <script>`

到了这一层，我们证明的已经不是“manager 存在”，而是“这个被 seal 的环境，真的能承载日常开发”。

### 4. 隔离与不变量

快乐路径跑通之后，还要继续证明模型在正常压力下不会变形。

这一层主要看：

- host PATH 污染
- managed-tree 的入口解析
- 项目产物仍然留在项目内部
- global manager 状态仍然留在 helper 管理范围内

这一层决定的是：这个 helper 不只是功能上可用，而是在结构上也诚实。

### 5. 生命周期信心

最后还需要一层纵向信心：

- 新 shell / 新进程里的重新进入
- 至少一次有意义的第二个 Node 版本切换
- version-scoped uninstall

这层故意比“完整迁移混沌测试”要窄。它要证明的是生命周期形状，而不是假装 helper 现在已经承诺了所有未来恢复路径。

## 已经被证明的部分

当前实现已经证明的东西，已经不只是最初那个 sealing 故事了。

到现在为止，测试已经覆盖了：

- 通过 `install --node-version` 自举出 `node + npm` baseline
- `list`、`which`、`snapshot`、按版本 `uninstall`
- 通过 `npm i -g pnpm` 吸收 `pnpm`
- 通过 `npm i -g yarn` 吸收 `yarn`
- 项目内的 `pnpm add` 与 `pnpm run`
- 项目内的 `yarn add` 与 `yarn run`
- profile 激活后的 host PATH 污染防护
- 两个 helper 管理的 Node 版本之间的隔离
- `corepack` 的 cache/state 被收进 managed version root
- `corepack enable` 与 `corepack prepare` 已经进入受控 runtime surface

这意味着当前模型已经不再只是理论上的分层，而是已经被证明覆盖了 bootstrap、absorption、项目内工作流，以及最重要的几条不变量。

## 当前最小场景集

在当前产品理解里，最小但有效的场景集应该长这样：

### Baseline Runtime

- 新鲜 helper bootstrap：`install --node-version`
- 通过 profile 进入后运行 `node -v`
- 通过 profile 进入后运行 `npm -v`
- 用 `which` 验证 managed path

### Manager Absorption

- 通过 `npm i -g pnpm` 吸收 `pnpm`
- 通过 `npm i -g yarn` 吸收 `yarn`

### 项目内工作流

- `pnpm install`
- `pnpm add`
- `pnpm run <script>`
- `yarn install`
- `yarn add`
- `yarn run <script>`
- 再补一个只依赖 baseline 的 npm 项目 sanity case

### 隔离与不变量

- host PATH 污染防护
- manager 被吸收后仍然从 managed tree 解析
- cache/global/store 落在 helper 预期范围内

### 生命周期信心

- 新 shell / 新进程里的重入
- 一次有意义的第二版本隔离验证
- 按版本卸载

## 现在可以暂缓的

有些事情当然重要，但它们还不构成当前产品承诺里的独立一层。

现在可以先后放的包括：

- 大规模 env poisoning 矩阵
- 任意损坏后的混沌恢复
- 完整的 `corepack` 共存矩阵
- 多版本迁移图谱
- 各种 shell 组合排列

这些以后都会重要，但在当前阶段，它们更多只会增加体量，而不会显著提高覆盖质量。

## 还需要继续证明的部分

即使到了现在，仍然还有几件真正重要的事没有完成：

- `corepack` 作为完整 runtime entry path，而不仅仅是 cache/state 路径
- `bin/` 与 `corepack-bin/` 的长期叙事进一步收稳
- 多版本长期并存时更完整的迁移故事
- 安装中断或 managed tree 部分损坏后的更强恢复保证

换句话说，当前矩阵已经证明了模型的轮廓，但还没有证明我们未来可能会对外承诺的一切。

## 什么情况下矩阵必须扩张

矩阵应该在模型变化时扩张，而不是在命令数量变化时扩张。

典型触发包括：

- `corepack` 成为正式用户路径的一部分
- 多个 helper 管理的 Node 版本开始需要共享或协作
- helper 开始承诺自动修复或自动迁移
- runtime entry 不再只有当前这种显式 profile alias 模型

一旦产品多了一条新承诺，测试矩阵就应该多出对应的一层或一组场景。

这篇文章的意义就在这里。

它不是为了证明一切。

它只是为了保证：在我们当前对 `:node` 的理解范围内，我们证明的是对的东西。
