#!/bin/sh
# Document-reproducibility gate (B0-006 / ADR-011): the same source
# must render a byte-identical PDF in two independent clean build
# directories. All TeX date material derives from SOURCE_DATE_EPOCH
# (captured by meson from the source commit) with FORCE_SOURCE_DATE=1;
# hyperxmp date keys and the instance id are pinned in main.tex.
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
