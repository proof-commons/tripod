#!/bin/sh
# Audit the hand-managed meson census lists against git ls-files
# (ADR-014).
#
# Meson passes every declared subject path as an argument; this script
# re-partitions the tracked tree with census.sh and diffs the path
# sets. Any disagreement — a tracked lint subject missing from its
# directory's meson.build list, or a declared file that is no longer
# tracked — is a hard failure naming the paths, so a stale list cannot
# silently pass a lint lane.
#
# The stamp is created only when absent, never touched on a no-op run:
# the target is build_always_stale, and an unchanged stamp keeps
# downstream lint targets clean through ninja's restat.
#
# Usage: census-audit.sh <repo-root> <stamp> <declared-path>...
set -eu

root="$1"
stamp="$2"
shift 2

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

if [ "$#" -gt 0 ]; then
  printf '%s\n' "$@" | LC_ALL=C sort > "$tmpdir/declared"
else
  : > "$tmpdir/declared"
fi
sh "$root/scripts/census.sh" "$root" | cut -f2 | LC_ALL=C sort > "$tmpdir/tracked"

if ! diff -u "$tmpdir/declared" "$tmpdir/tracked" >&2; then
  echo "error: the hand-managed meson census lists disagree with git ls-files" >&2
  echo "       (+ tracked but not declared: add it to that directory's meson.build;" >&2
  echo "        - declared but not tracked: git add the file or drop it from the list)" >&2
  exit 1
fi

[ -f "$stamp" ] || : > "$stamp"
