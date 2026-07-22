#!/bin/sh
# Document-reproducibility gate (B0-006 / ADR-011): the same source
# must render a byte-identical PDF in two independent clean build
# directories. All TeX date material derives from a build-time
# SOURCE_DATE_EPOCH artifact with FORCE_SOURCE_DATE=1; hyperxmp date
# keys and the instance id are pinned in main.tex.
#
# Requires meson, ninja, and the TeX toolchain (xelatex, biber,
# latexmk) on PATH. Exits nonzero when the two PDFs differ.
set -eu

cd "$(dirname "$0")/.."

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

tree_status="$(git status --porcelain=v1 --untracked-files=all)"
if [ -n "$tree_status" ]; then
  echo "==> skipping reused-build epoch probe in dirty worktree" >&2
  printf '%s
' "$tree_status" >&2
  exit 0
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
