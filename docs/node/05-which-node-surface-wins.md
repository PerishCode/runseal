---
bodyClass: post-page
title: 05 | Which Node Surface Wins
meta: "style reference: Stripe, Rust"
---

This post is a contract, not a tour.

Its job is simple: define which Node-related surface wins when multiple surfaces disagree.

## The Rule In One Screen

When a runseal profile is active, the managed runtime must win.

More concretely:

1. explicit profile-managed runtime
2. project-local task executables
3. managed `bin/`
4. managed `corepack-bin/`
5. project metadata as signal
6. cwd/workspace discovery as signal
7. ambient machine state as fallback only

Anything lower on the list must not silently outrank anything higher.

## The Layers

### 1. Profile-managed runtime

This is the highest authority.

Once a profile is active, the runseal-managed Node surface is the runtime contract. It defines which version root is active, which paths are injected, and which helper-managed surfaces are available.

### 2. Project-local task executables

`node_modules/.bin` is allowed to win for task-local package executables, especially when a package manager is already running inside the project.

It is not allowed to back-drive runtime identity. In other words, local package executables may override command selection inside a project task context, but they must not replace the active Node runtime chosen by the profile.

### 3. Managed `bin/`

This is the primary helper-managed command surface.

If a command exists in the ordinary helper `bin/`, that path currently wins over the parallel `corepack-bin/` surface.

### 4. Managed `corepack-bin/`

This is a visible, helper-managed shim surface for `corepack`-materialized managers.

It remains part of the managed runtime, but it is currently secondary to `bin/` when both expose the same command name.

### 5. Project metadata

`packageManager` and similar signals matter, but only as signals.

They may influence manager choice, validation, warnings, or shim preparation. They do not get to outrank the active profile by themselves.

### 6. cwd / workspace discovery

Discovery decides scope, not authority.

It helps determine which project root and which metadata are relevant. It does not get to silently change the winning runtime source.

Ordinary working-directory semantics remain inherited CLI behavior. runseal may observe cwd; it does not redefine cwd.

### 7. Ambient machine state

Homebrew, Volta, fnm, nvm, global npm installs, or ambient Corepack state are never authoritative once a profile is active.

They may still exist. They do not get to win silently.

## Canonical Conflict Rules

- If profile intent and host PATH disagree, profile wins.
- If profile runtime and ambient global `pnpm` disagree, profile wins.
- If profile runtime is active and a project task resolves through `node_modules/.bin`, local task entry wins for that task only.
- If `packageManager` disagrees with an ambient manager, `packageManager` may influence helper behavior, but ambient state still does not outrank the active profile.
- If `corepack-bin/` and `bin/` disagree on a manager entrypoint, `bin/` currently wins.

## What Is Guaranteed

- Active profile intent outranks ambient machine state.
- Managed runtime surfaces are deterministic and inspectable.
- Local project task executables may win for task execution, but not for runtime identity.
- Managed `bin/` currently outranks managed `corepack-bin/` for the same command name.
- Metadata may guide selection, but it is not a silent authority layer.
- Working directory remains inherited CLI behavior, not a separate runseal authority surface.

## What Is Not Fully Defined Yet

- Exact tie-breaking for every mixed-manager monorepo shape.
- The final long-term narrative for `bin/` versus `corepack-bin/`.
- How much automatic remediation should occur when metadata and active runtime disagree.

Those are still evolving.

The contract above is the current floor.
