#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

# shellcheck disable=SC2046
cargo run -p tripod-labels --bin check-labels -- \
  --repository-root . \
  $(sh scripts/census-args.sh . labels) > /dev/null

# The plan-structure lane covers everything under adr/ and plans/.
# shellcheck disable=SC2046
python3 scripts/check_plans.py \
  $(sh scripts/census.sh . | awk -F'\t' '$2 ~ /^(adr|plans)\//{print $2}')

git diff --check
git diff --cached --check
echo "==> documentation tree is valid" >&2
