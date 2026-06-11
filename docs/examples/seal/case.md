# Seal `case` / Argv Shapes

This file documents the canonical `.seal` shapes for `case "$1" in` argv
parsing. These are intentionally narrower than general shell scripting.

## Supported option arm: `--name=value`

```sh
--body=*)
  body=${1#--body=}
  shift
  ;;
```

This is the canonical inline-value string option arm.

## Supported option arm: `--name <value>`

Single-line guarded form:

```sh
--body)
  if [ "$#" -lt 2 ]; then fail "missing value for --body"; fi
  body=$2
  shift 2
  ;;
```

Multi-line guarded form:

```sh
--body)
  if [ "$#" -lt 2 ]; then
    fail "missing value for --body"
  fi
  body=$2
  shift 2
  ;;
```

Both are canonical string-option arms.

The guard predicate is intentionally exact:

```sh
if [ "$#" -lt 2 ]; then
```

This is not a general parser for arbitrary pre-check logic.

## Supported flag arm

```sh
--dry-run)
  dry_run=true
  shift
  ;;
```

## Supported help arm

```sh
-h|--help|help)
  __seal_help=true
  shift
  ;;
```

## Supported positional fallback arm

This is the one supported positional sink shape:

```sh
*)
  if [ -z "$message" ]; then
    message=$1
    shift
  else
    fail "unexpected argument: $1"
  fi
  ;;
```

Semantics:

- The first unmatched argument fills one positional target.
- The next unmatched argument fails with a stable operator-facing message.
- This is not the same as "take one arg and break out of parsing."

## Not supported

These shapes are intentionally outside the current canonical surface:

```sh
*)
  message=$1
  shift
  break
  ;;
```

```sh
*)
  shift
  ;;
```

```sh
--body)
  body=$2
  shift 2
  ;;
```

The last form looks natural in shell, but runseal currently requires the
canonical missing-value guard for separated-value option arms.

## Complete minimal example

```sh
print() {
  printf '%s\n' "$1"
}

fail() {
  print "$1"
  exit 1
}

__seal_argc=$#
__seal_help=false
body=
message=

while [ "$#" -gt 0 ]; do
  case "$1" in
    --body)
      if [ "$#" -lt 2 ]; then
        fail "missing value for --body"
      fi
      body=$2
      shift 2
      ;;
    --body=*)
      body=${1#--body=}
      shift
      ;;
    -h|--help|help)
      __seal_help=true
      shift
      ;;
    *)
      if [ -z "$message" ]; then
        message=$1
        shift
      else
        fail "unexpected argument: $1"
      fi
      ;;
  esac
done

if [ "$__seal_help" = true ]; then
  print help
  exit 0
fi

print "$body"
print "$message"
```
