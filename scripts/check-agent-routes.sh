#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${1:-$REPO_DIR/docs/.vitepress/dist}"

python3 - "$DIST_DIR" <<'PY'
from pathlib import Path
import re
import sys

dist_dir = Path(sys.argv[1])
pages = [dist_dir / "index.html", dist_dir / "zh-CN" / "index.html"]
route_keys = ["agent:entry:install", "agent:entry:cli", "agent:entry:ci"]
base_prefix = "/envlock/"

failures = 0


def fail(message: str) -> None:
    global failures
    failures += 1
    print(f"FAIL {message}", file=sys.stderr)


def ok(message: str) -> None:
    print(f"PASS {message}")


def resolve_dist_file(route: str):
    if not route.startswith(base_prefix):
        return None

    relative = route[len(base_prefix):]
    if not relative:
        candidate = dist_dir / "index.html"
        return candidate if candidate.exists() else None

    clean = relative.rstrip("/")
    html_candidate = dist_dir / f"{clean}.html"
    if html_candidate.exists():
        return html_candidate

    dir_candidate = dist_dir / clean / "index.html"
    if dir_candidate.exists():
        return dir_candidate

    return None


meta_pattern = re.compile(r'<meta\s+name="([^"]+)"\s+content="([^"]*)"\s*/?>')

for page in pages:
    if not page.exists():
        fail(f"missing_page:{page}")
        continue

    html = page.read_text(encoding="utf-8")
    page_label = str(page.relative_to(dist_dir))
    metas = dict(meta_pattern.findall(html))

    for key in route_keys:
        route = metas.get(key, "")
        if not route:
            fail(f"missing_route_meta:{page}:{key}")
            continue

        target = resolve_dist_file(route)
        if target is None:
            fail(f"unresolved_route:{page}:{key}:{route}")
            continue

        ok(f"route_ok:{page_label}:{key}:{route} -> {target.relative_to(dist_dir)}")

if failures:
    sys.exit(1)

ok("agent_route_integrity")
PY
