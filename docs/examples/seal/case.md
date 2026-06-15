# Seal Argv Parsing

This file replaces the old shell-shaped `case "$1" in` examples with the target
Seal source shape for wrapper argument parsing.

The first design target is still a normal operational loop: explicit defaults,
explicit flags, and clear failures. A future declarative argv parser can be
added later if the repeated shape becomes valuable enough.

## Cursor-based parsing

```seal
method main(argv) {
  let channel = ""
  let ref = "main"
  let version = ""
  let watch = false
  let dry_run = false

  let args = argv.cursor()

  while args.has_next() {
    let arg = args.next()

    match arg {
      "--channel" => channel = args.value_or_fail("--channel")
      when arg.starts_with("--channel=") => channel = arg.strip_prefix("--channel=")

      "--ref" => ref = args.value_or_fail("--ref")
      when arg.starts_with("--ref=") => ref = arg.strip_prefix("--ref=")

      "--version" => version = args.value_or_fail("--version")
      when arg.starts_with("--version=") => version = arg.strip_prefix("--version=")

      "--watch" => watch = true
      "--dry-run" => dry_run = true

      "-h" | "--help" | "help" => {
        usage()
        exit(0)
      }

      _ => fail("unknown option: {arg}")
    }
  }

  if channel == "" {
    fail("release: --channel is required")
  }

  release(channel, ref: ref, version: version, watch: watch, dry_run: dry_run)
}
```

## Positional fallback

The old canonical shell shape allowed one positional sink and rejected the
second unmatched argument. In Seal, that should be visible as ordinary state.

```seal
method main(argv) {
  let body = ""
  let message = ""
  let dry_run = false
  let args = argv.cursor()

  while args.has_next() {
    let arg = args.next()

    match arg {
      "--body" => body = args.value_or_fail("--body")
      when arg.starts_with("--body=") => body = arg.strip_prefix("--body=")
      "--dry-run" => dry_run = true
      "-h" | "--help" | "help" => {
        usage()
        exit(0)
      }
      _ => {
        if message == "" {
          message = arg
        } else {
          fail("unexpected argument: {arg}")
        }
      }
    }
  }

  create_comment(body: body, message: message, dry_run: dry_run)
}
```

## Possible future parser shape

If this pattern repeats enough, Seal can grow a first-class argv declaration.
That should be a deliberate syntax feature, not a hidden shell parser.

```seal
method main(argv) {
  let opts = argv.parse {
    option channel required "--channel"
    option ref = "main" "--ref"
    option version = null "--version"
    flag watch "--watch"
    flag dry_run "--dry-run"
    help "-h" "--help" "help"
  }

  release(
    opts.channel,
    ref: opts.ref,
    version: opts.version,
    watch: opts.watch,
    dry_run: opts.dry_run,
  )
}
```
