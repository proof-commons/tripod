#!/bin/sh
# Repository CI entry point (ADR-011). A clean checkout must pass this
# script without modifying tracked files. Runner-agnostic: invoke it
# from any CI system or locally.
#
# This file is a shim. The gate itself lives in scripts/ci.py, which
# declares every lane in a typed registry, wall-times each one, and
# emits a JSON timing report (target/ci-report.json, and the final
# lines of stdout) on failure exactly as on success. The entry-point
# name is preserved so every documented invocation and every reference
# from ADR-011, ADR-014, meson.build, and the plan tree keeps working:
# read scripts/ci.py for the lane list, the result vocabulary
# (green/partial), and the CI_REQUIRE_MESON contract.
#
# Arguments are forwarded unchanged; the default run, with no
# arguments, is the full lane sequence in its declared order.
set -eu

cd "$(dirname "$0")/.."

exec python3 scripts/ci.py "$@"
