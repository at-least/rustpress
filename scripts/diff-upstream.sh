#!/usr/bin/env bash
# Content axis of the parity check: demo/content must stay a verbatim
# copy of the VitePress docs (see PARITY.md). A non-empty diff after a
# VitePress update means: re-copy the changed pages, then re-run the
# build so the theme axis (parity check) sees the new content.
set -euo pipefail
cd "$(dirname "$0")/.."

UP="${1:-../vitepress/docs/en}"
if [ ! -d "$UP" ]; then
  echo "diff:upstream: upstream docs not found at $UP" >&2
  echo "  clone vuejs/vitepress next to this repo (or pass the path):" >&2
  echo "    git clone https://github.com/vuejs/vitepress ../vitepress" >&2
  exit 2
fi

if diff -rq demo/content "$UP"; then
  echo "diff:upstream: demo/content matches $UP"
fi
