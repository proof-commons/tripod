#!/bin/sh
# Document-reproducibility gate (B0-006 / ADR-011). Two checks:
#
#   1. clean-build determinism — the same source renders a
#      byte-identical PDF in two independent clean build directories;
#   2. reused-build source epoch — a reused build directory refreshes
#      the source epoch, so its PDF matches a fresh build at the same
#      revision.
#
# All TeX date material derives from a build-time SOURCE_DATE_EPOCH
# artifact with FORCE_SOURCE_DATE=1; hyperxmp date keys and the
# instance id are pinned in main.tex.
#
# Check 2 clones the worktree at HEAD, so it can only speak for a clean
# worktree. Exit 0 therefore means *both* advertised checks ran and
# passed; a dirty worktree is refused rather than silently reduced to
# check 1 (SR3-06). Automation must not read a reduced run as a pass.
#
# Usage: check-document-reproducibility.sh [--allow-partial]
#
#   --allow-partial   run check 1 only in a dirty worktree and report
#                     the distinct partial status instead of failing.
#                     In a clean worktree it changes nothing.
#
# Exit codes:
#
#   0  complete: every advertised check ran and passed
#   1  failure: an advertised check failed, or the worktree is dirty
#      without --allow-partial
#   2  usage error
#   3  partial: --allow-partial in a dirty worktree; check 1 passed and
#      check 2 did not run. Never a pass.
#
# Requires meson, ninja, and the TeX toolchain (xelatex, biber,
# latexmk) on PATH.
set -eu

allow_partial=0
for argument in "$@"; do
  case "$argument" in
    --allow-partial) allow_partial=1 ;;
    *)
      echo "usage: $(basename "$0") [--allow-partial]" >&2
      exit 2
      ;;
  esac
done

cd "$(dirname "$0")/.."

# Decided before the expensive builds: a run that cannot complete must
# say so immediately, not after two document renders.
tree_status="$(git status --porcelain=v1 --untracked-files=all)"
if [ -n "$tree_status" ] && [ "$allow_partial" -eq 0 ]; then
  printf '%s\n' "$tree_status" >&2
  echo "ERROR: the reused-build epoch probe needs a clean worktree, so this" >&2
  echo "       gate cannot complete here; commit or stash, or rerun with" >&2
  echo "       --allow-partial to accept the partial status (exit 3)" >&2
  exit 1
fi

first=$(mktemp -d "${TMPDIR:-/tmp}/repro-a-XXXXXX")
second=$(mktemp -d "${TMPDIR:-/tmp}/repro-b-XXXXXX")
trap 'rm -rf "$first" "$second"' EXIT

echo "==> build 1/2" >&2
meson setup "$first" > /dev/null
meson compile -C "$first" attestation > /dev/null

echo "==> build 2/2" >&2
meson setup "$second" > /dev/null
meson compile -C "$second" attestation > /dev/null

pdf_a="$first/papers/attestation/main.pdf"
pdf_b="$second/papers/attestation/main.pdf"

sha256sum "$pdf_a" "$pdf_b" >&2

if ! cmp -s "$pdf_a" "$pdf_b"; then
  echo "ERROR: rendered PDFs differ between clean builds" >&2
  exit 1
fi

echo "==> document build is reproducible" >&2

if [ -n "$tree_status" ]; then
  printf '%s\n' "$tree_status" >&2
  echo "==> PARTIAL: clean-build determinism passed; the reused-build epoch" >&2
  echo "    probe did not run in this dirty worktree. This is not a pass." >&2
  exit 3
fi

reuse_root=$(mktemp -d "${TMPDIR:-/tmp}/repro-reuse-XXXXXX")
trap 'rm -rf "$first" "$second" "$reuse_root"' EXIT

echo "==> reused-build epoch probe" >&2
git clone --local . "$reuse_root/repo" > /dev/null

cd "$reuse_root/repo"
git config user.email reproducibility@example.invalid
git config user.name "Reproducibility Probe"

meson setup reuse-build > /dev/null

printf '%s\n' '% reused-build reproducibility probe' >> papers/attestation/sections/10_limitations.tex
git add papers/attestation/sections/10_limitations.tex
GIT_AUTHOR_DATE='2001-09-09T01:47:40Z' \
GIT_COMMITTER_DATE='2001-09-09T01:47:40Z' \
  git commit -m 'reproducibility probe source change' > /dev/null

meson compile -C reuse-build attestation > /dev/null

meson setup fresh-build > /dev/null
meson compile -C fresh-build attestation > /dev/null

reuse_pdf="reuse-build/papers/attestation/main.pdf"
fresh_pdf="fresh-build/papers/attestation/main.pdf"

sha256sum "$reuse_pdf" "$fresh_pdf" >&2

if ! cmp -s "$reuse_pdf" "$fresh_pdf"; then
  echo "ERROR: reused and fresh build PDFs differ at the same revision" >&2
  exit 1
fi

echo "==> reused build directory refreshes source epoch" >&2
echo "==> document reproducibility complete: both checks ran and passed" >&2
