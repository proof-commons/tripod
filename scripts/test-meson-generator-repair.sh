#!/bin/sh
set -eu

repository_root="${1:-.}"
repository_root="$(cd "$repository_root" && pwd)"

temporary="$(mktemp -d "${TMPDIR:-/tmp}/attestation-generator-repair-XXXXXX")"
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

build_dir="$temporary/build"
output_dir="$temporary/generated"
cargo_target="$temporary/cargo-target"

export CARGO_TARGET_DIR="$cargo_target"

echo "==> configuring isolated Meson build" >&2
meson setup \
  "$build_dir" \
  "$repository_root" \
  -Dartifact_generation_output_dir="$output_dir" \
  >/dev/null

echo "==> first explicit generation" >&2
meson compile \
  -C "$build_dir" \
  generate-artifacts \
  >/dev/null

artifact="$output_dir/architecture.json"
expected="$temporary/architecture.expected.json"

test -f "$artifact"
cp "$artifact" "$expected"

echo "==> corrupting only the temporary publication" >&2
printf '%s\n' '{"corrupted":true}' > "$artifact"

if cmp -s "$artifact" "$expected"; then
  echo "test setup failed: artifact was not changed" >&2
  exit 1
fi

echo "==> second explicit generation" >&2
meson compile \
  -C "$build_dir" \
  generate-artifacts \
  >/dev/null

if ! cmp -s "$artifact" "$expected"; then
  echo "generate-artifacts did not repair the temporary artifact" >&2
  exit 1
fi

echo "==> generator alias repaired the temporary artifact" >&2