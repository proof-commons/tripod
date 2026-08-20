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
# cargo-audit is installed into the shared ~/.cargo/bin, which a lane may put
# on PATH after the toolchain's own enable.sh runs; a find_program() answer
# baked in at `meson setup` would then report a skip for a tool that is on
# PATH by the time the test runs, and would need a reconfigure to notice an
# installation.
#
# Usage: gate-cargo-audit.sh <repository-root>
set -eu

repository_root="${1:-.}"
cd "$repository_root"

if ! command -v cargo-audit > /dev/null 2>&1; then
  echo "cargo-audit is not installed; the advisory lane is externally" >&2
  echo "provisioned (ADR-011) and reports SKIP rather than a pass." >&2
  exit 77
fi

exec cargo audit
