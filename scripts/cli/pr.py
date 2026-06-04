from __future__ import annotations

import argparse
import json
import subprocess
import sys

from lib.utils.cli import CliError, run_checked


def usage() -> None:
    print(
        """Usage: runseal :pr [options]

Create or update, watch, and squash-merge the GitHub PR for the current branch.

Options:
  --base <branch>       PR base branch (default: main)
  --title <title>       title when creating a new PR
  --body-file <path>    body file when creating a new PR
  --draft              create the PR as draft and require --no-merge
  --no-watch           do not watch PR checks
  --no-merge           do not squash-merge after checks
  --no-push            do not push the current branch first
  --dry-run            print planned actions without changing remote state
"""
    )


def output(argv: list[str]) -> str:
    result = run_checked(argv, stdout=subprocess.PIPE)
    return result.stdout.decode("utf-8").strip()


def current_branch() -> str:
    branch = output(["git", "branch", "--show-current"])
    if not branch:
        raise CliError("not on a branch")
    return branch


def require_operator_tools() -> None:
    run_checked(["git", "--version"], stdout=subprocess.DEVNULL)
    run_checked(["gh", "--version"], stdout=subprocess.DEVNULL)
    run_checked(["gh", "auth", "status"], stdout=subprocess.DEVNULL)


def find_pr(branch: str) -> dict[str, object] | None:
    raw = output(
        [
            "gh",
            "pr",
            "list",
            "--head",
            branch,
            "--json",
            "number,title,state,url,isDraft",
        ]
    )
    items = json.loads(raw)
    if not items:
        return None
    return items[0]


def create_pr(
    branch: str,
    base: str,
    title: str | None,
    body_file: str | None,
    *,
    draft: bool,
) -> dict[str, object]:
    argv = [
        "gh",
        "pr",
        "create",
        "--base",
        base,
        "--head",
        branch,
    ]
    if draft:
        argv.append("--draft")
    if title:
        argv.extend(["--title", title])
    else:
        argv.append("--fill")
    if body_file:
        argv.extend(["--body-file", body_file])
    elif title:
        argv.append("--fill")
    run_checked(argv)
    found = find_pr(branch)
    if found is None:
        raise CliError(f"created PR for {branch}, but could not find it afterward")
    return found


def cmd_default(args: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="runseal :pr", add_help=False)
    parser.add_argument("--base", default="main")
    parser.add_argument("--title")
    parser.add_argument("--body-file")
    parser.add_argument("--draft", action="store_true")
    parser.add_argument("--no-watch", action="store_true")
    parser.add_argument("--no-merge", action="store_true")
    parser.add_argument("--no-push", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parsed = parser.parse_args(args)

    require_operator_tools()
    branch = current_branch()
    if branch in {parsed.base, "main", "master"}:
        raise CliError(f"refusing to open a PR from base branch: {branch}")
    if parsed.draft and not parsed.no_merge:
        raise CliError("--draft requires --no-merge")

    if parsed.dry_run:
        print(f"branch: {branch}")
        print(f"base: {parsed.base}")
        print(f"push: {not parsed.no_push}")
        print("pr: create if missing, otherwise reuse existing")
        print(f"draft: {parsed.draft}")
        print(f"ready: {not parsed.draft}")
        print(f"watch: {not parsed.no_watch}")
        print(f"squash_merge: {not parsed.no_merge}")
        return 0

    if not parsed.no_push:
        run_checked(["git", "push", "-u", "origin", branch])

    pr = find_pr(branch)
    if pr is None:
        pr = create_pr(
            branch,
            parsed.base,
            parsed.title,
            parsed.body_file,
            draft=parsed.draft,
        )
        print(f"created PR #{pr['number']}: {pr['url']}", flush=True)
    else:
        print(f"found PR #{pr['number']}: {pr['url']}", flush=True)

    number = str(pr["number"])
    if pr.get("isDraft") and not parsed.draft:
        run_checked(["gh", "pr", "ready", number])
        print(f"marked PR #{number} ready")
    if not parsed.no_watch:
        run_checked(["gh", "pr", "checks", number, "--watch", "--interval", "10"])
    if not parsed.no_merge:
        run_checked(["gh", "pr", "merge", number, "--squash", "--delete-branch"])
        print(f"squash-merged PR #{number}")
    return 0


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args or args[0] in {"-h", "--help", "help"}:
        usage()
        return 0
    try:
        return cmd_default(args)
    except (CliError, RuntimeError, OSError, subprocess.CalledProcessError) as exc:
        print(f"pr: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
