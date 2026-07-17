#!/bin/sh
# Repository CI entry point (ADR-011). A clean checkout must pass this
# script without modifying tracked files. Runner-agnostic: invoke it
# from any CI system or locally.
#
# Lanes:
#   1. rustfmt          cargo fmt --all --check
#   2. clippy           -D warnings, all targets, --locked
#   3. tests (debug)    cargo test --workspace --locked
#   4. tests (release)  cargo test --workspace --release --locked
#   5. generated        check-generated (non-writing stale-artifact gate)
#   6. labels           check-labels (non-writing label gate)
#   7. advisories       cargo audit (skipped loudly when not installed)
#   8. plans            check-plans.sh (documentation structure)
#   9. clean tree       git diff --exit-code
#
# The checker lanes receive their subjects by argument (ADR-014). The
# census_args function below derives role-tagged argv from git
# ls-files; the meson build derives the same census from hand-managed
# per-directory lists, and the census-audit target welds the two.
# Paths in this repository never contain whitespace, so the unquoted
# expansions are deliberate.
#
# The Meson/LaTeX document lanes are separate because they need a TeX
# toolchain:
#   meson setup <builddir> && meson compile -C <builddir> attestation \
#     && meson test -C <builddir> --print-errorlogs
#
# The complete repository gate (ADR-011) is:
#   - this script under the declared MSRV toolchain;
#   - this script under current stable;
#   - the Meson document lane above in a clean build directory;
#   - scripts/check-document-reproducibility.sh.
# All four are required before a release claim; only the Rust lanes are
# duplicated per toolchain.
set -eu

cd "$(dirname "$0")/.."

# census_args <labels|scoped>: role-tagged checker argv. The exclusion
# cases mirror the meson census: categorical non-subjects first, then
# the same-typed exclusions declared in per-directory meson.build
# lists (macros_attestation.tex, execwrap integration tests).
census_args() {
  git ls-files | LC_ALL=C sort | while IFS= read -r path; do
    case "$path" in
      .* | */.* | archive/* | scripts/*) continue ;;
      papers/attestation/macros_attestation.tex) continue ;;
      packages/execwrap/tests/*) continue ;;
      papers/attestation/main.tex) printf ' --attestation-main %s' "$path" ;;
      papers/attestation/sections/*.tex) printf ' --attestation-section %s' "$path" ;;
      docs/attestation/realization.md) printf ' --realization %s' "$path" ;;
      packages/model/src/*.rs) printf ' --model-source %s' "$path" ;;
      *)
        [ "$1" = labels ] || continue
        case "$path" in
          plans/labels/specification.md) printf ' --specification-register %s' "$path" ;;
          plans/labels/realization.md) printf ' --realization-register %s' "$path" ;;
          packages/model/generated/model_labels.json) printf ' --model-labels-json %s' "$path" ;;
          plans/*.md) printf ' --plan %s' "$path" ;;
          adr/[0-9][0-9][0-9]*.md) printf ' --adr %s' "$path" ;;
          packages/*/src/*.rs) printf ' --crate-source %s' "$path" ;;
          *.md) printf ' --doc %s' "$path" ;;
        esac
        ;;
    esac
  done
}

echo "==> lane 1/9: cargo fmt" >&2
cargo fmt --all --check

echo "==> lane 2/9: cargo clippy (-D warnings)" >&2
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "==> lane 3/9: cargo test (debug)" >&2
cargo test --workspace --locked

echo "==> lane 4/9: cargo test (release)" >&2
cargo test --workspace --release --locked

echo "==> lane 5/9: check-generated" >&2
# shellcheck disable=SC2046
cargo run --locked -p tripod-artifacts --bin check-generated -- \
  --repository-root . \
  --generated-dir packages/model/generated \
  $(census_args scoped) > /dev/null

echo "==> lane 6/9: check-labels" >&2
# shellcheck disable=SC2046
cargo run --locked -p tripod-labels --bin check-labels -- \
  --repository-root . \
  $(census_args labels) > /dev/null

echo "==> lane 7/9: cargo audit" >&2
if command -v cargo-audit > /dev/null 2>&1; then
  cargo audit
else
  echo "WARNING: cargo-audit is not installed; advisory lane SKIPPED" >&2
fi

echo "==> lane 8/9: plan-tree checks" >&2
sh scripts/check-plans.sh

echo "==> lane 9/9: clean working tree" >&2
git diff --exit-code

echo "==> CI green" >&2
