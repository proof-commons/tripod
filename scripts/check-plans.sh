#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

# Label linting lives in ci.sh lane 6 and the meson labels-check
# target; this gate covers plan-tree structure and whitespace hygiene.
# Paths in this repository never contain whitespace, so the unquoted
# expansion is deliberate.
# shellcheck disable=SC2046
python3 scripts/check_plans.py $(git ls-files adr plans | grep '\.md$')

git diff --check
git diff --cached --check
echo "==> documentation tree is valid" >&2
