#!/bin/sh
set -eu

source_file="$1"
destination="$2"
stamp="$3"

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