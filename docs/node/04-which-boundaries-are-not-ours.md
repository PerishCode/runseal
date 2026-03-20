---
bodyClass: post-page
title: 04 | Which Boundaries Are Not Ours
meta: "style reference: Stripe, Rust"
---

One of the easiest ways to make a helper system too large is to attribute every messy edge case to the helper itself.

That is a mistake.

Some problems belong to the surrounding ecosystem. Some problems are not created by runseal, but are forced into the open once runseal refuses magic. Only some problems are truly ours.

This split matters because it determines what runseal should solve, what it should merely expose, and what it should refuse to absorb into the core.

## 1. Ecosystem Baggage That Pre-exists runseal

These are not problems introduced by runseal or by the `:node` helper. They already exist in the Node ecosystem:

- `npm`, `pnpm`, `yarn`, and `corepack` carrying different historical assumptions
- `packageManager` metadata disagreeing with actual executable entrypoints
- lockfile switching between package-manager families
- cache/store/global-state differences between managers
- ordinary CLI working-directory semantics
- monorepo and nested-workspace ambiguity
- native addon and postinstall dependencies on host toolchains
- historical tension between `npm i -g ...` and `corepack` activation flows

None of these begin with runseal.

## 2. Problems runseal Forces Into The Open

These are not newly created either, but runseal makes them impossible to ignore.

Once runseal insists on explicit runtime boundaries, questions that used to remain fuzzy now need real answers:

- who wins when profile intent, `packageManager`, corepack shims, local binaries, and absorbed globals disagree?
- what does PATH precedence actually mean in a sealed runtime?
- which runtime surface owns the final executable entrypoint?
- what counts as contamination once a profile is active?
- how much state can a helper keep before it stops feeling deterministic?

One subtle example is `cwd`.

The current working directory matters enormously in package-manager behavior, but it is not a runseal-invented concept. It is inherited CLI behavior. runseal may need to acknowledge it in diagnostics and documentation, but that does not mean runseal should redefine it.

runseal does not invent these tensions.

It just refuses to leave them hidden.

## 3. Boundaries That Are Actually Ours

These are the things that *do* belong to runseal and the helper design:

- the versioned helper layout under `RUNSEAL_HELPER_NODE_HOME`
- the command surface: `install`, `list`, `which`, `remote list`, `snapshot`, `uninstall`, `help`, `example`
- whether `bin/` and `corepack-bin/` remain separate or converge
- which runtime paths are exposed in profile patches
- how `help`, `stdout`, `stderr`, `which`, and `snapshot` make provenance visible
- the promise that once a profile is active, managed paths win without compromise

What does **not** belong here is inventing a new working-directory model. runseal can consume cwd as an input signal, but it should not pretend ownership over ordinary CLI directory semantics.

These are not inherited problems.

These are product decisions.

## Why This Split Matters

If we fail to distinguish these layers, two bad things happen.

First, the helper starts trying to “solve” the entire Node ecosystem instead of maintaining a clean runtime boundary.

Second, the core starts taking on implementation debt for problems it did not create.

The right move is stricter:

- ecosystem baggage should be acknowledged
- forced-open ambiguity should be made explicit
- runseal-owned boundaries should be designed carefully and documented clearly

That is how the helper stays useful without becoming a bottomless compatibility pit.

## The Practical Consequence

This split is the real lead-in to a precedence contract.

Before writing more sugar or adding more policy, we need to know which questions are:

- ecosystem realities we must respect
- ambiguities we must make explicit
- product rules we are willing to own

Only then does it make sense to decide what should win when the surfaces disagree.
