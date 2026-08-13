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
#   9. forbidden text   check-forbidden-text (typed repository audit)
#  10. meson contract   test-meson-mock.sh (mocked TeX; skipped without meson/ninja)
#  11. clean tree       staged, unstaged, and untracked nonignored paths
#
# Result vocabulary. A skipped check is neither failure nor success, so
# this script distinguishes two passing outcomes and never calls the
# second one green:
#
#   CI green    every lane ran and passed
#   CI partial  every lane that ran passed; one or more were skipped
#
# Lane 10 is skippable only for local convenience — it is the sole
# check of the hand-managed ADR-014 census, Meson command wiring,
# report/stamp edges, generator and publication repair, no-op restat
# behaviour, and render-failure propagation. Set CI_REQUIRE_MESON=1 to
# make a skip a hard failure; protected-branch and release runs must.
# Lane 7 is skippable by ADR-011, which defines the advisory lane as
# externally provisioned; a skip there is still reported, never
# silently passed.
#
# The checker lanes receive their subjects by argument (ADR-014). The
# census_args function in scripts/census-args.sh derives role-tagged
# argv from git ls-files; the meson build derives the same census from
# hand-managed per-directory lists, and the census-audit target welds
# the two. That argv is shell-quoted and re-parsed into positional
# parameters rather than expanded unquoted, and a preflight audit
# refuses tracked paths outside the safe argv grammar (SR3-05).
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

# shellcheck source=scripts/census-args.sh
. ./scripts/census-args.sh

# Preflight, not a lane: the checker lanes below cannot carry a tracked
# path that leaves the safe argv grammar, so the run stops before any
# lane rather than reporting a census the checkers never received.
echo "==> preflight: tracked-path argv grammar audit" >&2
audit_tracked_paths

echo "==> lane 1/11: cargo fmt" >&2
cargo fmt --all --check

echo "==> lane 2/11: cargo clippy (-D warnings)" >&2
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "==> lane 3/11: cargo test (debug)" >&2
cargo test --workspace --locked

echo "==> lane 4/11: cargo test (release)" >&2
cargo test --workspace --release --locked

echo "==> lane 5/11: check-generated" >&2
scoped_census="$(derive_census 'scoped census' census_args scoped)"
eval "set -- $scoped_census"
cargo run --locked -p tripod-artifacts --bin check-generated -- \
  --repository-root . \
  --generated-dir packages/model/generated \
  "$@" > /dev/null

echo "==> lane 6/11: check-labels" >&2
label_census="$(derive_census 'label census' census_args labels)"
eval "set -- $label_census"
cargo run --locked -p tripod-labels --bin check-labels -- \
  --repository-root . \
  "$@" > /dev/null

# Skipped lanes are accumulated rather than forgotten: the final
# result names them, so a reduced run cannot be read as a complete one.
skipped_lanes=""

echo "==> lane 7/11: cargo audit" >&2
if command -v cargo-audit > /dev/null 2>&1; then
  cargo audit
else
  echo "WARNING: cargo-audit is not installed; advisory lane SKIPPED" >&2
  skipped_lanes="$skipped_lanes advisories"
fi

echo "==> lane 8/11: plan-tree checks" >&2
sh scripts/check-plans.sh

echo "==> lane 9/11: forbidden text" >&2
# shellcheck disable=SC2046
cargo run --locked -p tripod-labels --bin check-forbidden-text -- \
  --repository-root . \
  --git git > /dev/null

# Mocked Meson contract: exercises the real Meson graph with the TeX
# toolchain simulated (execwrap --mock-child), so it needs meson+ninja but
# no TeX. Skipped loudly where meson/ninja are absent, preserving this
# script's "runs without a TeX toolchain" contract. This is also the
# only lane that runs census-audit — the complete tracked-entry mode
# audit — so a skipped lane 10 leaves repository shape unchecked and
# the run is reported as partial rather than green.
echo "==> lane 10/11: mocked Meson contract" >&2
if command -v meson > /dev/null 2>&1 && command -v ninja > /dev/null 2>&1; then
  sh scripts/test-meson-mock.sh .
else
  echo "WARNING: meson/ninja not installed; mocked Meson contract lane SKIPPED" >&2
  if [ "${CI_REQUIRE_MESON:-0}" = "1" ]; then
    echo "ERROR: CI_REQUIRE_MESON=1 and meson/ninja are unavailable" >&2
    exit 1
  fi
  skipped_lanes="$skipped_lanes meson-contract"
fi

echo "==> lane 11/11: clean working tree" >&2
tree_status="$(git status --porcelain=v1 --untracked-files=all)"
if [ -n "$tree_status" ]; then
  printf '%s\n' "$tree_status" >&2
  echo "ERROR: repository contains staged, unstaged, or untracked nonignored changes" >&2
  exit 1
fi

if [ -n "$skipped_lanes" ]; then
  echo "==> CI partial: every lane that ran passed; skipped:$skipped_lanes" >&2
  echo "    a skipped lane is neither failure nor success; this is not a green run" >&2
else
  echo "==> CI green" >&2
fi
