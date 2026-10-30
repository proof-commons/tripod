#!/bin/sh
# Exercise the public-input and capture contracts without starting a node.
set -eu
started=$(date +%s)
python3 "${1:?repository root}/scripts/test-elements-observer-contract.py"
printf 'observer-contract wall-s %s\n' "$(( $(date +%s) - started ))"
