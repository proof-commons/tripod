#!/bin/sh
# Workspace API documentation, run as a single Meson test.
#
# Meson times the whole lane and keeps cargo's output in its test log.
# Inherit CARGO_TARGET_DIR just as the package test wrapper does, so the
# documentation build uses the gate's shared cargo cache.
#
# Usage: cargo-doc-gate.sh <cargo> <manifest> <profile>
#   profile  debug
set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: cargo-doc-gate.sh <cargo> <manifest> <profile>" >&2
  exit 2
fi

cargo="$1"
manifest="$2"
profile="$3"

case "$profile" in
  debug) ;;
  *) echo "ERROR: unknown profile '$profile'; expected debug" >&2; exit 2 ;;
esac

# Cargo's default documentation profile is debug. Set the warning policy
# explicitly so an inherited flag cannot silently weaken this gate.
# Exec preserves cargo's failure status and leaves diagnostics in Meson's log.
RUSTDOCFLAGS='-D warnings' CARGO_TERM_PROGRESS_WHEN=never \
  exec "$cargo" doc --workspace --no-deps --manifest-path "$manifest"
