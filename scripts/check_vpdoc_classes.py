#!/usr/bin/env python3
"""Consistency checker for the vpdoc_classes component in
templates/components.html.

The component renders one long Tailwind utility string from labeled
`{% set %}` string pieces joined with `~`. Tera renders whatever is
typed — a piece missing its leading space silently glues two utilities
together and the build still succeeds — so this script is the
enforcement. Run it after editing the component (exit 1 on any
violation):

    python3 scripts/check_vpdoc_classes.py
"""
import re
import sys

PATH = "templates/components.html"

def fail(msg):
    print(f"check_vpdoc_classes: FAIL: {msg}", file=sys.stderr)
    sys.exit(1)

src = open(PATH).read()
m = re.search(
    r"\{% component vpdoc_classes\(\) %\}(.*?)\{%(-?) endcomponent vpdoc_classes %\}",
    src, re.S,
)
if not m:
    fail("vpdoc_classes component not found")

body = m.group(1)
sets = re.findall(r"\{%- set (c\d+) = (.*?) -%\}", body, re.S)
if not sets:
    fail("no {%- set cN = ... -%} blocks found in the component body")

names, all_pieces = [], []
for name, lit in sets:
    names.append(name)
    pieces = re.findall(r'"((?:[^"\\]|\\.)*)"', lit)
    if not pieces:
        fail(f"set {name}: no string literals parsed")
    all_pieces.append(pieces)
    for i, p in enumerate(pieces):
        if i == 0:
            if p.startswith(" "):
                fail(f"set {name}: first piece must not start with a space")
            if p.endswith(" "):
                fail(f"set {name}: first piece must not end with a space")
        else:
            if not p.startswith(" "):
                fail(f"set {name} piece {i}: continuation piece is missing its "
                     f"leading space — the previous utility and this one are now "
                     f"glued together: {p[:60]!r}")
            if p.startswith("  "):
                fail(f"set {name} piece {i}: continuation piece starts with "
                     f"more than one space")
            if p.endswith(" "):
                fail(f"set {name} piece {i}: piece must not end with a space")
        if "  " in p:
            fail(f"set {name} piece {i}: contains a double space")

# the join expression must reference every set, in order
join = re.search(r"\{\{- \((.*?)\) \| safe -\}\}", body, re.S)
if not join:
    fail("final {{- (...) | safe -}} expression not found")
joined_names = re.findall(r"\bc\d+\b", join.group(1))
if joined_names != names:
    fail(f"join expression references {joined_names} but sets are {names}")

# reconstructed string: balanced brackets, no stray whitespace
full = " ".join(
    "".join(pieces).replace('\\"', '"').replace("\\\\", "\\")
    for pieces in all_pieces
)
if full != full.strip() or "  " in full:
    fail("reconstructed utility string has leading/trailing or double spaces")
depth = 0
for ch in full:
    if ch in "([":
        depth += 1
    elif ch in ")]":
        depth -= 1
        if depth < 0:
            fail("reconstructed utility string has unbalanced brackets")
if depth != 0:
    fail("reconstructed utility string has unbalanced brackets")

print(f"check_vpdoc_classes: OK ({len(sets)} sets, {len(full)} chars)")
