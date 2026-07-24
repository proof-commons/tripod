#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

# Label linting lives in ci.sh lane 6 and the meson labels-check
# target; this gate covers plan-tree structure and whitespace hygiene.
# The checker receives its census by argument (ADR-014). Paths in this
# repository never contain whitespace, so the unquoted expansion is
# deliberate.
# shellcheck disable=SC2046
cargo run -p tripod-labels --bin check-plans -- \
	--repository-root . \
	$(git ls-files adr plans | grep '\.md$' | sed 's/^/--subject /') > /dev/null

git diff --check
git diff --cached --check
echo "==> documentation tree is valid" >&2
