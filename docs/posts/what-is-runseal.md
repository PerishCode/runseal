---
bodyClass: post-page
title: What is runseal?
meta: "style reference: Stripe, Rust"
---

runseal is a thin environment runtime for command execution.

It takes explicit context, materializes that context through `env + symlink`, and makes the result usable by both humans and agents.

## Core Definition

At its core, runseal is built around a very small set of ideas:

- a profile describes environment intent
- `env` makes context explicit
- `symlink` makes entrypoints explicit
- helpers absorb ecosystem-specific operational mess

That means runseal is not trying to be a giant platform. It is trying to be a small, inspectable layer that makes execution context visible, reproducible, and composable.

## What runseal is

runseal is:

- a profile-driven command environment tool
- a way to seal context around a command or workflow
- a thin core with explicit helper boundaries
- a system designed to minimize diagnostic distance for agents

In practice, that means runseal should help answer questions like:

- what environment is this command running in?
- which entrypoint will actually execute?
- where do cache, store, prefix, and helper-managed state live?
- which part of the system is responsible when something goes wrong?

## What runseal is not

runseal is not:

- a generalized plugin ecosystem
- a thick runtime manager for every language and toolchain
- a hidden shell-magic layer that silently rewrites your machine
- a container replacement for every isolation problem

Helpers may do dirty work for specific ecosystems, but the core should stay thin. If an ecosystem becomes directly runseal-compatible, helper complexity should be removed instead of institutionalized.

## Profiles

Profiles are where intent lives.

A profile says what context should exist, not which invisible shell state you are expected to trust. In the long run, the value of runseal is not “it switched something for me,” but “it described and applied a context I can inspect.”

## Helpers

Helpers exist to keep the core small.

They absorb fast-changing, ecosystem-specific, operationally messy behavior without forcing that complexity into the center of runseal.

The `:node` helper is a good example. Its job is not merely to switch Node versions. Its job is to establish a self-bootstrapped Node + npm baseline, then gather the dirty boundaries around Node tooling - `pnpm`, `yarn`, caches, globals, wrappers, stores, and versioned entrypoints - into a layout that runseal profiles can still seal.

## Why `env + symlink`

`env + symlink` is the floor.

`env` exposes context. `symlink` exposes entrypoints. Between them, the two most important hidden facts in environment management become visible again:

- what changed in the environment
- where execution actually lands

That is why runseal keeps returning to the same baseline. If those two things stop being explicit, runseal stops being itself.

## Success Conditions

runseal succeeds when a strong agent can read `help`, `stdout`, and `stderr`, identify the correct debugging direction, and move forward without guessing.

It also succeeds when a user can name a context, apply it, inspect it, and remove it without losing track of what happened in between.

That is what runseal is trying to be: not a magical environment manager, but a visible execution context layer.
