#!/usr/bin/env bash
# Clone or update ../vitepress (vuejs/vitepress) at the ref pinned in
# parity/upstream-ref.txt. That clone is the content source for
# diff:upstream, the verbatim-corpus tests (tests/upstream_sync.rs,
# tests/feature_parity.rs), and CI — the upstream gates fail loudly when
# it is missing, so this is the one command that unblocks them.
#
# Shallow by default; `git -C ../vitepress fetch --unshallow` restores
# history when you want to read upstream commits (PARITY.md step 0).
set -euo pipefail
cd "$(dirname "$0")/.."

REF=$(grep -v '^[[:space:]]*#' parity/upstream-ref.txt | sed '/^[[:space:]]*$/d' | head -n1 | tr -d '[:space:]')
if [ -z "$REF" ]; then
  echo "sync-upstream: no ref found in parity/upstream-ref.txt" >&2
  exit 2
fi

UP=../vitepress
if [ -d "$UP/.git" ]; then
  git -C "$UP" fetch --depth 1 --force origin "refs/tags/$REF:refs/tags/$REF"
  git -C "$UP" checkout --force "$REF"
else
  git -c advice.detachedHead=false clone --depth 1 --branch "$REF" \
    https://github.com/vuejs/vitepress "$UP"
fi
echo "sync-upstream: ../vitepress at $REF"
