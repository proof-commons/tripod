#!/bin/sh
# Repository CI entry point (ADR-011). A clean checkout must pass this
# script without modifying tracked files. Runner-agnostic: invoke it
# from any CI system or locally.
#
# This file is a SHIM over the Meson flow, which is where the gate now
# lives (CI-009). Every lane is a `meson test`: meson declares it, meson
# times it, meson reports its result, and `meson test --list` is the
# authoritative lane list. The entry-point name is preserved because
# ADR-011, ADR-014, and the plan tree all name it, and because a CI
# system should not have to learn a build system to run a gate.
#
# It replaced a Python driver that declared the same lanes a second time,
# in its own registry, with its own timer and its own skip vocabulary.
# That driver was retired as dead weight: a second declaration
# of a lane is a second thing that can disagree with the first, and every
# capability it justified itself with -- per-lane wall times, a skip that
# is neither pass nor failure, a machine-readable report -- meson's harness
# already had.
#
#   scripts/ci.sh                      the full gate
#   scripts/ci.sh --suite lint         one suite (CI-005 content scoping)
#   scripts/ci.sh cargo-clippy         one lane by name
#   scripts/ci.sh --list               the declared lanes
#
# Arguments are forwarded to `meson test` unchanged, which is what makes
# content-scoped runs the same command as the full one. The build
# directory is configured on first use and reused after that; set
# CI_BUILD_DIR to place it elsewhere.
#
# What this gate does NOT cover, stated rather than implied. It configures
# with mock_mode=true, so the TeX toolchain is simulated: the Meson graph,
# the census, the repair and restat edges are all exercised, but no real
# document is rendered. The complete repository gate of ADR-011 is this
# script, plus the real document lane in a clean build directory
# (`meson setup <dir> . && meson compile -C <dir> attestation && meson test
# -C <dir>`), plus scripts/check-document-reproducibility.sh. Only the
# first is cheap enough to run per batch.
set -eu

cd "$(dirname "$0")/.."
repository_root="$(pwd)"

build_directory="${CI_BUILD_DIR:-target/ci-meson}"

# mock_mode lets a machine without a TeX toolchain run every Rust and
# checker lane; it governs the LaTeX programs and nothing else. The
# publication and generation outputs are redirected into the build
# directory so a gate run cannot perturb the tracked archive and trip the
# clean-tree lane -- a gate that fails its own cleanliness check because
# it ran is worse than no check.
if [ ! -f "$build_directory/meson-info/meson-info.json" ]; then
  meson setup "$build_directory" . \
    -Dmock_mode=true \
    -Dpublication_archive_root="$repository_root/$build_directory/archive" \
    -Dartifact_generation_output_dir="$repository_root/$build_directory/generated"
fi

status=0
meson test -C "$build_directory" --print-errorlogs "$@" || status=$?

# The timing report is emitted on failure exactly as on success. The cost
# data matters most when a lane hurts, so a failing run is the one run that
# must not lose it (the wall-time ruling; CI-005). The reader runs after the
# gate and never changes its verdict.
python3 scripts/ci-timing-report.py "$build_directory" || true

exit "$status"
