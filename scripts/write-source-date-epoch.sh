#!/bin/sh
set -eu

git_program="$1"
repository_root="$2"
output="$3"
explicit_epoch="${4:-}"

if [ -n "$explicit_epoch" ]; then
  epoch="$explicit_epoch"
else
  epoch="$("$git_program" -C "$repository_root" log -1 --format=%ct)"
fi

case "$epoch" in
  ''|*[!0-9]*)
    echo "invalid SOURCE_DATE_EPOCH: $epoch" >&2
    exit 1
    ;;
esac

staged="$(mktemp "$(dirname "$output")/.source-date-epoch.XXXXXX")"
trap 'rm -f "$staged"' EXIT HUP INT TERM

printf '%s\n' "$epoch" > "$staged"

if ! cmp -s "$staged" "$output" 2>/dev/null; then
  mv "$staged" "$output"
else
  rm -f "$staged"
fi