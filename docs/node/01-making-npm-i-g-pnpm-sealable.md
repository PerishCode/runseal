---
bodyClass: post-page
title: 01 | Making npm i -g pnpm Sealable
meta: "style reference: Stripe, Rust"
---

Goal-oriented lifecycle definition for helpers.

## Node Helper Goal

The `:node` helper exists to turn messy Node ecosystem behavior into a stable runseal-managed environment.

Success means a user can do normal Node-global operations such as:

```bash
runseal helper :node install --node-version 24.12.0
runseal :node24-profile npm i -g pnpm
```

and the resulting toolchain still lands inside runseal-managed layout, stays sealable by `profile.json`, and does not leak into unmanaged global state. The helper prepares the environment; the profile is the runtime entrypoint.

At the current stage, `install` establishes a self-bootstrapped Node + npm baseline. Package managers such as `pnpm` are then pulled into the sealed version root through normal runtime actions like `npm i -g pnpm`.

## What Must Be Sealed

For a target Node version, the helper must keep these dirty boundaries inside the managed version root:

- executable entrypoints in `bin/`
- package-manager-created shims
- `node_modules/` and `node_modules/.bin/`
- npm cache and prefix state
- pnpm store state
- yarn cache and global state
- manager-specific home-like state needed for stable execution
- install lock state for the version being prepared

## Required Outcome

The helper should produce a version-centered layout like:

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

`npm` is part of the helper-managed baseline for the version. `pnpm` and `yarn` are not modeled as separate version roots; when they appear, they should be absorbed into the same sealed Node environment for that version. `corepack` may also participate, but its cache and shim state should still remain inside the same version root.

## Design Implications

- Prefer temporary assembly + atomic placement over mutating partially prepared live directories.
- Prefer explicit managed paths over implicit reliance on host-global defaults.
- Keep the Rust side thin: alias resolution, fetch, cache, execute.
- Keep ecosystem-specific operational mess in the helper itself.
- If the upstream ecosystem becomes directly runseal-compatible, remove helper complexity instead of preserving abstraction.
