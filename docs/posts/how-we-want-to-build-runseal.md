---
bodyClass: post-page
title: How We Want to Build runseal
meta: "by GPT-5.4 feat. opencode | style reference: Stripe, Rust"
---

The way we want to build runseal follows from one constitutional rule: runseal must be agent-native.

That phrase is easy to say and easy to cheapen, so it is worth being explicit about what we mean.

## The Constitution

`agent-native` does not mean “optimized for prompts” or “marketed toward AI tools.” It means the system should expose enough truth, in enough places, with low enough retrieval cost, that a strong agent can use it and debug it without guessing.

## The Diagnostic Standard

That is why the real design standard is not just `help`. It is `help + stdout + stderr` together.

Those three surfaces should always point an agent toward the correct debugging direction, while making the path to diagnosis as short as possible. If a top-tier agent reads the help output, sees the normal output, sees the error output, and still cannot identify where the problem lives inside the runseal layer, then the design has already failed.

## The Floor

This is also why `env + symlink` is the floor below which runseal should never fall.

`env` makes context explicit. `symlink` makes entrypoints explicit. Between them, the two most important hidden facts in environment management become visible again: what the world looks like, and where commands actually land.

Once those two facts are visible, everything else becomes easier to reason about: cache placement, global state, version roots, helper-managed directories, PATH composition, and failure boundaries. If those two facts disappear behind orchestration magic, the entire system becomes harder for both humans and agents to trust.

That is why we do not treat `env + symlink` as one implementation choice among many. It is the irreducible floor.

## The Core Must Stay Small

From there, the architecture follows naturally.

The core should stay small. Ideally, forever under ten thousand lines.

Why? Because the core is where abstraction pressure accumulates. If too much ecosystem-specific behavior leaks inward, the core starts to grow around historical edge cases, accidental protocols, and compatibility burdens. The result is a tool that may still work, but no longer feels inspectable.

We would rather force ourselves to keep the core narrow.

The core should resolve, compose, and execute. It should not become a giant manager of every runtime ecosystem. It should not become a magical policy engine. It should not turn into a hidden state machine with a CLI attached.

## Push the Mess Outward

That is where helpers come in.

Helpers exist to absorb dirty work without contaminating the center.

Take the Node case. The hard part is not selecting a Node version in isolation. The hard part is establishing a self-bootstrapped Node + npm baseline, then dealing with everything around it: `pnpm`, `yarn`, global installs, caches, stores, wrapper scripts, home-like directories, stable shims, and the need to make all of that land inside a layout that runseal can still seal and describe.

That mess is real. It should not be denied. But it should also not be allowed to dictate the shape of the core.

So our rule is simple: all dirty, ecosystem-specific, and fast-changing work belongs to helpers.

This is not because helpers are elegant. In many cases they are not. It is because they create a boundary where complexity can live without pretending to be universal.

The core remains thin. The helper carries the ecosystem burden. And if an ecosystem eventually becomes directly runseal-compatible, we should delete helper complexity rather than preserve it as tradition.

## Documentation Is Part of the Surface

The same constitutional rule shapes our documentation.

We actively expose core information in the places where agents can retrieve it at the lowest cost. That is why key entrypoints and contract signals are pushed into metadata. That is why docs structure is kept explicit. That is why posts, scoreboard pages, and task-oriented docs are separated by purpose instead of blended into one narrative blob.

This is not documentation ornament. It is part of the product surface.

If information retrieval is expensive, agents are forced back into guessing. And once the system depends on guessing, it is no longer agent-native.

So when we say we want runseal to become more like `gh`, `aws`, or `kubectl`, we do not just mean “nice command names.” We mean something deeper: a command surface that is legible, a context model that is explicit, and a diagnostic path that stays short.

## The Shape We Want

That is the shape we want.

Small core. Explicit context. Stable entrypoints. Dirty work pushed outward. Command surfaces that help agents diagnose quickly. Documentation that reduces retrieval cost instead of hiding structure.

Everything else is secondary.

---

> <span class="post-note-title">A few rough notes.</span>
>
> The constitution is actually simple: `agent-native`.
>
> Why is `agent-native` the first constitutional rule? I think time will prove it.
>
> Keep `env + symlink` as the floor.
>
> I want the core to stay small forever. Ideally under ten thousand lines.
>
> Every dirty, ecosystem-specific, fast-changing piece of work should be pushed to helpers.
>
> I hope CLI tools and public services eventually feel as agent-native as `gh`, `aws`, and `kubectl`.
>
> Just like Rust, the point is not to hide the problem. The point is to point you toward the problem quickly and clearly.
>
> <span class="post-signoff">from PerishCode</span>
