---
bodyClass: post-page
title: 02 | What :node Still Needs to Prove
meta: "style reference: Stripe, Rust"
---

Even after the first sealing case works, `:node` is not finished.

The goal of this post is not to enumerate every imaginable test. The goal is to define what counts as sufficient coverage within the current product understanding.

## What Sufficient Coverage Means

We do not need combinatorial exhaustion.

We do need every distinct product promise to be exercised at the point where failure would actually differ.

For the current `:node` model, those promises are:

- the helper can self-bootstrap a usable `node + npm` baseline
- secondary managers can be absorbed through explicit runtime entry
- once absorbed, those managers work for real project workflows
- the managed layout stays explicit and does not silently fall back to host-global state

If all of those are true, then the current model is meaningfully covered. If one of them is untested, then the coverage is incomplete even if dozens of commands appear to pass.

## A Layered Validation Model

The cleanest way to avoid both blind spots and combinatorial explosion is to validate by behavior layer.

### 1. Baseline Runtime

This layer answers a very simple question: can `:node` bootstrap a usable Node runtime and expose it through a runseal profile?

This includes:

- `install --node-version ...`
- `node -v`
- `npm -v`
- binary resolution through the managed tree

### 2. Manager Absorption

This layer answers the next question: can secondary managers be pulled into the managed version root through normal runtime actions?

The canonical examples are:

- `runseal :node24-profile npm i -g pnpm`
- `runseal :node24-profile npm i -g yarn`

The point is not just that the command exits successfully. The point is that the resulting manager entrypoints and package contents land inside the managed tree.

### 3. Project-Local Workflows

A globally installed manager that only survives `--version` is not enough.

The next layer is real project work:

- `pnpm install`
- `pnpm add`
- `pnpm run <script>`
- `yarn install`
- `yarn add`
- `yarn run <script>`

At this point we are no longer proving that a manager exists. We are proving that the sealed environment can actually carry day-to-day development work.

### 4. Isolation And Invariants

Once the happy path works, the model still needs to show that it remains true under normal pressure.

That means checking things like:

- host PATH contamination
- managed-tree binary resolution
- project-local artifacts staying project-local
- global manager state staying helper-managed

This is where we confirm that the helper is not merely functional, but structurally honest.

### 5. Lifecycle Confidence

Finally, the model needs one layer of longitudinal confidence:

- re-entering the same profile in a fresh shell or process
- switching to a second Node version once
- uninstalling a managed version and observing expected removal

This is intentionally narrower than full migration chaos testing. It is enough to prove the shape of the lifecycle without pretending the helper already guarantees every future recovery path.

## What Has Already Been Proven

The current implementation already proves more than the original sealing story.

Right now, the test suite already covers:

- self-bootstrapping a `node + npm` baseline through `install --node-version`
- `list`, `which`, `snapshot`, and version-scoped `uninstall`
- runtime absorption of `pnpm` through `npm i -g pnpm`
- runtime absorption of `yarn` through `npm i -g yarn`
- project-local `pnpm add` and `pnpm run`
- project-local `yarn add` and `yarn run`
- host-path contamination protection once a profile is active
- version isolation between two helper-managed Node versions
- `corepack` cache/state entering the managed version root
- `corepack enable` and `corepack prepare` flowing into managed runtime surfaces

That means the current model is no longer only theoretical. It already covers bootstrap, absorption, project-local workflows, and several of the most important invariants.

## The Minimum Scenario Set

Within the current product understanding, the minimum useful scenario set looks like this:

### Baseline Runtime

- fresh helper bootstrap through `install --node-version`
- profile-entered `node -v`
- profile-entered `npm -v`
- `which` confirming managed paths

### Manager Absorption

- absorb `pnpm` through `npm i -g pnpm`
- absorb `yarn` through `npm i -g yarn`

### Project-Local Workflows

- `pnpm install`
- `pnpm add`
- `pnpm run <script>`
- `yarn install`
- `yarn add`
- `yarn run <script>`
- one npm-local sanity project using only the baseline

### Isolation And Invariants

- host-path contamination guard
- managed binary resolution checks after absorption
- helper-managed cache/global/store path checks

### Lifecycle Confidence

- re-entry in a fresh shell/process
- one meaningful second-version isolation check
- version-scoped uninstall

## What Can Wait

Some things matter, but they do not yet represent separate product promises.

For now, these can be deprioritized:

- broad env poisoning matrices
- deep chaos recovery after arbitrary corruption
- full `corepack` coexistence matrices
- exhaustive multi-version migration graphs
- shell-by-shell integration permutations

Those will matter when the product starts explicitly promising them. Right now they would mostly add volume without increasing coverage quality.

## What Still Needs To Be Proven

Even with the current coverage, some meaningful work remains:

- `corepack` as a fully trusted runtime entry path, not only a cache/state path
- a more settled long-term narrative for `bin/` versus `corepack-bin/`
- a fuller migration story when multiple versions are used over longer-lived workspaces
- stronger recovery guarantees after interrupted installs or partially damaged managed trees

In other words, the current matrix proves the shape of the model. It does not yet prove every future promise we may want to make.

## When The Matrix Must Expand

The matrix should expand when the model changes, not just when a new command appears.

Good triggers are:

- `corepack` becomes part of the intended user path
- multiple helper-managed Node versions are expected to cooperate or share state
- helper starts promising automatic repair or migration
- runtime entry grows beyond the current explicit profile alias model

When a new promise appears, a new test layer or scenario should appear with it.

That is the point of this document.

It is not trying to prove everything forever.

It is trying to make sure that, within our current understanding of `:node`, we are proving the right things.
