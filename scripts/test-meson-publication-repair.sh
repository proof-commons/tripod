#!/bin/sh
set -eu

repository_root="${1:-.}"
repository_root="$(cd "$repository_root" && pwd)"

temporary="$(mktemp -d "${TMPDIR:-/tmp}/attestation-publication-repair-XXXXXX")"
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

build_dir="$temporary/build"
archive_dir="$temporary/archive"
cargo_target="$temporary/cargo-target"

export CARGO_TARGET_DIR="$cargo_target"

echo "==> configuring isolated Meson build" >&2
meson setup \
  "$build_dir" \
  "$repository_root" \
  -Dpublication_archive_root="$archive_dir" \
  >/dev/null

echo "==> first explicit publication" >&2
meson compile \
  -C "$build_dir" \
  attestation \
  >/dev/null

pdf="$(find "$archive_dir/rendered" -type f -name '*.pdf' | sort | sed -n '1p')"
flat="$(find "$archive_dir/flattened" -type f -name '*.tex' | sort | sed -n '1p')"
test -n "$pdf"
test -n "$flat"

expected_pdf="$temporary/expected.pdf"
expected_flat="$temporary/expected.tex"
cp "$pdf" "$expected_pdf"
cp "$flat" "$expected_flat"

echo "==> corrupting only the temporary archive mirrors" >&2
printf '%s\n' corrupted-pdf > "$pdf"
printf '%s\n' corrupted-flat > "$flat"

echo "==> second explicit publication" >&2
meson compile \
  -C "$build_dir" \
  attestation \
  >/dev/null

cmp -s "$pdf" "$expected_pdf" || {
  echo "PDF archive mirror was not repaired" >&2
  exit 1
}
cmp -s "$flat" "$expected_flat" || {
  echo "flattened archive mirror was not repaired" >&2
  exit 1
}

before="$(sha256sum "$pdf" "$flat")"

echo "==> third explicit publication no-op" >&2
meson compile \
  -C "$build_dir" \
  attestation \
  >/dev/null

after="$(sha256sum "$pdf" "$flat")"
if [ "$before" != "$after" ]; then
  echo "publication no-op changed archive bytes" >&2
  exit 1
fi

echo "==> publication aliases repair temporary archive mirrors" >&2