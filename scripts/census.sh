#!/bin/sh
# Role-tagged lint census from git ls-files (ADR-014).
#
# Emits one line per lint subject: <role><TAB><repository-relative path>.
# Tracked files only: an untracked scratch file is not a lint subject,
# which aligns the lint gate with the ADR-011 clean-tree law.
#
# The build census itself is hand-managed in per-directory meson.build
# lists (ADR-014); this script is the tracked-tree side of the weld:
# census-audit.sh diffs the declared meson lists against it on every
# build, and census-args.sh renders it into checker argv for the shell
# CI lanes. The Rust checkers independently re-discover their subjects
# on disk and hard-fail on any disagreement, so no single census
# source is trusted alone.
#
# Roles:
#   attestation-main           the Layer-0 root TeX file
#   attestation-section        Layer-0 section TeX files
#   realization           the Realization Markdown document
#   adr                   numbered ADR Markdown records
#   plan                  planning Markdown (registers excluded)
#   doc                   other authored Markdown (DOC owner)
#   model-source          model crate Rust sources
#   crate-source          other first-party crate Rust sources
#   specification-register       generated Layer-0 register publication
#   realization-register  generated realization register publication
#   model-labels-json     generated model-label publication
#   generated             other committed generated artifacts
#
# Usage: census.sh [repo-root]
set -eu

root="${1:-.}"
cd "$root"

git ls-files -z | tr '\0' '\n' | LC_ALL=C sort | while IFS= read -r path; do
  case "$path" in
    *' '* | *'	'*)
      echo "census: path with whitespace is unsupported: $path" >&2
      exit 1
      ;;
  esac
  case "$path" in
    archive/*) continue ;;
    .* | */.*) continue ;;
    papers/attestation/main.tex) role=attestation-main ;;
    papers/attestation/sections/*.tex) role=attestation-section ;;
    docs/attestation/realization.md) role=realization ;;
    plans/labels/specification.md) role=specification-register ;;
    plans/labels/realization.md) role=realization-register ;;
    plans/*.md) role=plan ;;
    adr/[0-9][0-9][0-9]*.md) role=adr ;;
    packages/model/src/*.rs) role=model-source ;;
    packages/model/generated/model_labels.json) role=model-labels-json ;;
    packages/model/generated/*) role=generated ;;
    packages/*/src/*.rs) role=crate-source ;;
    *.md) role=doc ;;
    *) continue ;;
  esac
  printf '%s\t%s\n' "$role" "$path"
done
