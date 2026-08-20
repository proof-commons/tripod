#!/bin/sh
# Advisory scan of the resolved dependency graph (ADR-011), as a meson test.
#
# ADR-011 defines this lane as EXTERNALLY PROVISIONED: cargo-audit is not a
# workspace dependency and not every environment has it, so its absence is
# neither a pass nor a failure. The retired Python driver expressed that with
# a lane status of its own; meson's harness already has the concept, so this
# wrapper exits 77 -- meson's SKIP status -- and the skip appears in the test
# summary and in meson-logs/testlog.json under its own name. A skip is
# therefore reported, never silently passed, which is the property ADR-011
# asks for.
#
# The probe is at RUN time rather than configuration time on purpose.
# cargo-audit is installed into the shared ~/.cargo/bin, so a find_program()
# answer baked in at `meson setup` would need a reconfigure to notice an
# installation.
#
# # Why this does not simply run `cargo audit`
#
# `cargo audit` is `cargo` finding a `cargo-audit` on PATH, so running it that
# way requires ~/.cargo/bin to be on PATH -- and ~/.cargo/bin holds rustup's
# `cargo` shim too. A caller who adds that directory so this lane can find its
# tool silently puts rustup's toolchain in front of whichever toolchain the
# build was configured with. That is not hypothetical: it happened during this
# lane's own bring-up, and the symptom was a build failure three lanes away
# (`found crate compiled by an incompatible version of rustc`) from artifacts
# two toolchains had shared one target directory.
#
# So the advisory binary is located and executed DIRECTLY, and the search
# looks in cargo's home itself rather than asking the caller to widen PATH.
# `cargo-audit audit` is exactly what `cargo audit` dispatches to, so the lane
# runs the same program with the same argv -- it just never borrows a cargo it
# was not configured with to get there.
#
# Usage: gate-cargo-audit.sh <repository-root>
set -eu

repository_root="${1:-.}"
cd "$repository_root"

audit=""
if command -v cargo-audit > /dev/null 2>&1; then
  audit="$(command -v cargo-audit)"
else
  for candidate in "${CARGO_HOME:-$HOME/.cargo}/bin/cargo-audit" "$HOME/.cargo/bin/cargo-audit"; do
    if [ -x "$candidate" ]; then
      audit="$candidate"
      break
    fi
  done
fi

if [ -z "$audit" ]; then
  echo "cargo-audit is not installed; the advisory lane is externally" >&2
  echo "provisioned (ADR-011) and reports SKIP rather than a pass." >&2
  exit 77
fi

echo "advisory scan via $audit" >&2
exec "$audit" audit
