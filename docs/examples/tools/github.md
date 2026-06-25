# GitHub Tool Examples

These examples show the intended `runseal @tool github ...` usage patterns.

Implementation is pure GitHub HTTP API. The public shape does not depend on the
`gh` CLI.

## Token resolution

Priority:

1. `--token`
2. `--token-file`
3. `--token-env`
4. default `GITHUB_TOKEN`

So the normal path is to inject `GITHUB_TOKEN` through runseal env/profile
mechanics and keep the call site clean.

## Issue comment create

```bash
runseal @tool github issue comment create \
  --repo PerishCode/runseal \
  --number 49 \
  --body-file body.md
```

Default body limit:

- `comment create` defaults `--body-max=100`
- `--body-max=0` disables the limit
- overflow fails fast; it does not truncate

Example with unlimited body:

```bash
runseal @tool github issue comment create \
  --repo PerishCode/runseal \
  --number 49 \
  --body-file body.md \
  --body-max 0
```

## Issue create

```bash
runseal @tool github issue create \
  --repo PerishCode/runseal \
  --title "Document Deno wrapper policy" \
  --body-file body.md
```

## Issue body update

```bash
runseal @tool github issue body update \
  --repo PerishCode/runseal \
  --number 49 \
  --body-file body.md
```

## Cross-repo prefixing

When writing from one controlled repo into another controlled repo, use
`--prefix-enable=true` so the request origin is recorded in the body.

```bash
runseal @tool github issue comment create \
  --repo PerishCode/runseal \
  --number 49 \
  --body-file body.md \
  --body-max 0 \
  --prefix-enable=true
```

The prefix is only applied when the repo-pair rule matches. It is not a
generic text mutation flag.

## Payload hygiene

For non-trivial text, always write the body to a file first and pass
`--body-file`.

Preferred:

```bash
runseal @tool github issue comment create \
  --repo PerishCode/runseal \
  --number 49 \
  --body-file body.md
```

Only use inline `--body` for a very short single-line payload.
