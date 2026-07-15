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
#   6. advisories       cargo audit (skipped loudly when not installed)
#   7. plans            check-plans.sh (planning-tree hygiene)
#   8. clean tree       git diff --exit-code
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

echo "==> lane 1/8: cargo fmt" >&2
cargo fmt --all --check

echo "==> lane 2/8: cargo clippy (-D warnings)" >&2
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "==> lane 3/8: cargo test (debug)" >&2
cargo test --workspace --locked

echo "==> lane 4/8: cargo test (release)" >&2
cargo test --workspace --release --locked

echo "==> lane 5/8: check-generated" >&2
cargo run --locked -p tripod-artifacts --bin check-generated > /dev/null

echo "==> lane 6/8: cargo audit" >&2
if command -v cargo-audit > /dev/null 2>&1; then
  cargo audit
else
  echo "WARNING: cargo-audit is not installed; advisory lane SKIPPED" >&2
fi

echo "==> lane 7/8: plan-tree checks" >&2
sh scripts/check-plans.sh

echo "==> lane 8/8: clean working tree" >&2
git diff --exit-code

echo "==> CI green" >&2
