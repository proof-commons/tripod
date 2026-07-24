#!/bin/sh
set -eu

source_file="$1"
destination="$2"
stamp="$3"

# The stamp is an always-stale publication-mirror marker, never a freshness
# oracle, so plain `touch` is correct here — deliberately outside the strict
# empty-stamp refusal that cli_common::touch_stamp applies to checker --stamp
# outputs (reviewed as F4-003; not a policy bypass).
mkdir -p "$(dirname "$destination")"

if ! cmp -s "$source_file" "$destination" 2>/dev/null; then
  staged="$(mktemp "$(dirname "$destination")/.$(basename "$destination").XXXXXX")"
  trap 'rm -f "$staged"' EXIT HUP INT TERM
  cp "$source_file" "$staged"
  mv "$staged" "$destination"
  touch "$stamp"
elif [ ! -e "$stamp" ]; then
  touch "$stamp"
fi