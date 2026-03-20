#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${1:-$REPO_DIR/docs/.vitepress/dist}"

python3 - "$DIST_DIR" <<'PY'
from pathlib import Path
import re
import sys

dist_dir = Path(sys.argv[1])
pages = [dist_dir / "index.html", dist_dir / "zh-CN" / "index.html"]
required_fields = [
    "agent:contract:version",
    "agent:index:v1",
    "agent:mode",
    "agent:entry:install",
    "agent:entry:use",
    "agent:entry:scoreboard",
    "agent:resolution",
    "agent:locale:default",
    "agent:locale:source",
    "agent:locale:policy",
]

failures = 0


def fail(message: str) -> None:
    global failures
    failures += 1
    print(f"FAIL {message}", file=sys.stderr)


def ok(message: str) -> None:
    print(f"PASS {message}")


meta_pattern = re.compile(r'<meta\s+name="([^"]+)"\s+content="([^"]*)"\s*/?>')

for page in pages:
    if not page.exists():
        fail(f"missing_page:{page}")
        continue

    html = page.read_text(encoding="utf-8")
    page_label = str(page.relative_to(dist_dir))
    pairs = meta_pattern.findall(html)
    metas = {}

    for name, content in pairs:
        if name in metas:
            fail(f"duplicate_meta:{page}:{name}")
        metas[name] = content

    for key in required_fields:
        if key not in metas:
            fail(f"missing_meta:{page}:{key}")
        else:
            ok(f"meta_present:{page_label}:{key}")

    index_value = metas.get("agent:index:v1", "")
    indexed_keys = [item.strip() for item in index_value.split(",") if item.strip()]

    for key in indexed_keys:
        if key not in metas:
            fail(f"index_key_missing:{page}:{key}")
        else:
            ok(f"index_key_present:{page_label}:{key}")

    if metas.get("agent:mode") != "meta-first":
        fail(f"invalid_mode:{page}:{metas.get('agent:mode')}")

    resolution = metas.get("agent:resolution", "")
    if not resolution:
        fail(f"missing_resolution:{page}")

if failures:
    sys.exit(1)

ok("agent_meta_contract")
PY
