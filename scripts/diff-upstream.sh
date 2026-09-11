#!/usr/bin/env bash
# Content axis of the parity check: demo/content must stay a verbatim
# copy of the VitePress docs (see PARITY.md; cargo test --test
# upstream_sync enforces this when the clone is present). A non-empty
# diff after a VitePress update means: re-copy the changed pages, then
# re-run the build so the theme axis (parity check) sees the new content.
#
# Also prints an advisory list of upstream pages not pinned in
# parity/pages.txt — new upstream features usually land on new pages,
# and an unpinned page is invisible to the theme-axis landmark gate.
set -euo pipefail
cd "$(dirname "$0")/.."

UP="${1:-../vitepress/docs/en}"
if [ ! -d "$UP" ]; then
  echo "diff:upstream: upstream docs not found at $UP" >&2
  echo "  clone vuejs/vitepress at the pinned ref next to this repo:" >&2
  echo "    bash scripts/sync-upstream.sh" >&2
  exit 2
fi

set +e
diff -rq demo/content "$UP"
diff_rc=$?
set -e

pages=$(cd "$UP" && find . -name '*.md' | sed 's:^\./::' | while read -r f; do
  case "$f" in
    */index.md) echo "/${f%/index.md}/" ;;
    index.md)   echo "/" ;;
    *)          echo "/${f%.md}" ;;
  esac
done | sort)
pinned=$(grep -v '^[[:space:]]*#' parity/pages.txt | sed '/^[[:space:]]*$/d' | sort)
unpinned=$(comm -13 <(printf '%s\n' "$pinned") <(printf '%s\n' "$pages"))
if [ -n "$unpinned" ]; then
  echo "diff:upstream: upstream pages not pinned in parity/pages.txt (consider pinning representative ones):"
  printf '  %s\n' "$unpinned"
fi

exit "$diff_rc"
