#!/bin/sh
# A package or one explicitly selected target, run as a Meson test (CI-002).
#
# The gate runs no workspace-level `cargo test`: every package is its own
# Meson-driven lane, so a duration and a failure both carry the name of the
# package they belong to. This wrapper is that lane's body.
#
# Per-test timing comes from libtest's own JSON event stream, requested with
# `-Z unstable-options --format json --report-time`. Those flags change how
# the harness REPORTS, never which tests are compiled, selected, or run, and
# never what a passing test proves; the ruling that admitted them is recorded
# in plans/backlog.md (CI-002). The flags need a nightly libtest, so this
# wrapper probes the cargo it was handed and degrades to the ordinary human
# format when the toolchain is not nightly: the tests still run and still
# gate, and the stream file is simply absent, which is what the CI driver
# reports as timing it did not measure. It never invents timing, and it never
# sets RUSTC_BOOTSTRAP to force unstable flags onto a stable toolchain.
#
# The doctest harness accepts the same flags. A split package's lanes jointly
# cover unit, integration, and documentation tests; the target census checks
# that their union preserves the package invocation's complete test surface.
#
# Usage: cargo-test-json.sh <cargo> <manifest> <package> <profile> <stream>
#   profile  gate | release
#   stream   absolute path for the libtest JSON event stream (build tree)
set -eu

if [ "$#" -lt 5 ] || [ "$#" -gt 6 ]; then
  echo "usage: cargo-test-json.sh <cargo> <manifest> <package> <profile> <stream> [lib|doc|test:NAME]" >&2
  exit 2
fi

cargo="$1"
manifest="$2"
package="$3"
profile="$4"
stream="$5"
selection="${6:-all}"

case "$profile" in
  gate|release) ;;
  *) echo "ERROR: unknown profile '$profile'; expected gate or release" >&2; exit 2 ;;
esac

mkdir -p "$(dirname "$stream")"
rm -f "$stream"

set -- test --manifest-path "$manifest" -p "$package"
if [ "$profile" = release ]; then
  set -- "$@" --release
else
  set -- "$@" --profile gate
fi
case "$selection" in
  all) ;;
  lib) set -- "$@" --lib ;;
  doc) set -- "$@" --doc ;;
  test:*) set -- "$@" --test "${selection#test:}" ;;
  *) echo "ERROR: unknown test selection: $selection" >&2; exit 2 ;;
esac

if "$cargo" --version 2>/dev/null | grep -q -- '-nightly'; then
  set -- "$@" -- -Z unstable-options --format json --report-time
  mode=json
else
  mode=plain
fi

if [ "$mode" = plain ]; then
  echo "WARNING: $cargo is not a nightly cargo; per-test timing is NOT measured" >&2
  echo "         for package $package ($profile); the tests themselves are unaffected" >&2
fi

# JSON mode. The event stream is stdout and is captured whole; cargo's own
# progress and the compiler's diagnostics stay on stderr, where Meson's test
# log keeps them. A failing run prints the stream too: the failure detail a
# human needs lives inside it, and a lane must not fail silently.
status=0
output="$stream"
if [ "$mode" = plain ]; then output="$stream.plain"; fi
CARGO_TERM_PROGRESS_WHEN=never "$cargo" "$@" > "$output" 2> "$stream.stderr" || status=$?
cat "$stream.stderr" >&2
if [ "$mode" = plain ]; then cat "$output"; fi
# A lane must consume the artifacts its warm-up built. Keep diagnostics
# beside the stream and fail loudly if package feature selection drifts.
if grep -q '^[[:space:]]*Compiling ' "$stream.stderr"; then
  echo "ERROR: compilation inside warmed test lane: $package ($profile) $selection" >&2
  status=1
fi
if [ "$status" -ne 0 ]; then
  echo "==> libtest JSON stream for $package ($profile):" >&2
  cat "$output" >&2
  exit "$status"
fi

if [ "$mode" = json ]; then
  echo "$package ($profile): libtest JSON stream written to $stream" >&2
fi
