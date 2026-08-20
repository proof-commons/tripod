#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

# shellcheck source=scripts/census-args.sh
. ./scripts/census-args.sh

# Label linting lives in the meson labels-check lane; this gate covers
# plan-tree structure and whitespace hygiene.
# The checker receives its census by argument (ADR-014); the argv is
# shell-quoted and re-parsed into positional parameters rather than
# expanded unquoted, and unsafe tracked paths are refused up front
# (SR3-05). Running standalone, this script owns that audit itself.
audit_tracked_paths

subject_census="$(derive_census 'plan subject census' plan_subject_args)"
eval "set -- $subject_census"
cargo run -p tripod-labels --bin check-plans -- \
	--repository-root . \
	"$@" > /dev/null

git diff --check
git diff --cached --check
echo "==> documentation tree is valid" >&2
