#!/bin/sh
# Clean-working-tree gate (ADR-011), as a meson test.
#
# A clean checkout must pass the gate WITHOUT modifying tracked files. This
# lane is what makes that claim checkable: a generator that repaired a
# committed publication, a checker that wrote where it should only have read,
# or a test that left a scratch file behind all show up here as a nonempty
# `git status`, and none of them show up anywhere else.
#
# --untracked-files=all is deliberate: the default collapses an untracked
# directory to one entry, which reports the directory but not what a lane put
# inside it, and the point of this lane is to name the leftover.
#
# Ordering. Under the retired driver this ran last, after every other lane
# had finished writing. Meson has no ordering primitive, so the test()
# declaration pins `is_parallel: false` and a negative priority, which is as
# close to "last" as the harness offers: the test is scheduled after the
# parallel work and runs alone. That is a scheduling preference, not a
# guarantee -- a lane that wrote into the source tree could in principle be
# observed mid-flight -- but every lane in this suite writes into the build
# directory or into a gitignored path, so there is nothing for it to race.
#
# Usage: gate-clean-tree.sh <repository-root>
set -eu

repository_root="${1:-.}"
cd "$repository_root"

status="$(git status --porcelain=v1 --untracked-files=all)"

if [ -n "$status" ]; then
  echo "$status" >&2
  echo "ERROR: repository contains staged, unstaged, or untracked" >&2
  echo "       nonignored changes" >&2
  exit 1
fi

echo "working tree is clean: no staged, unstaged, or untracked nonignored path"
