#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

cargo run -p tripod-labels --bin check-labels -- --repository-root . > /dev/null
python3 scripts/check_plans.py

git diff --check
git diff --cached --check
echo "==> documentation tree is valid" >&2
