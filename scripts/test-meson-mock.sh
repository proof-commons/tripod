#!/bin/sh
# Mocked Meson contract test (ADR-014). Exercises the real production
# Meson graph with the TeX toolchain simulated by execwrap --mock-child,
# so no XeLaTeX/Biber/latexmk is invoked. Replaces the heavyweight
# meson-generator-repair and meson-publication-repair integration tests:
# same repair / restat / failure-propagation properties, but without a
# TeX toolchain and without a second real document build.
#
# Every mocked output stays under <repo>/build/mocks (git-ignored).
# Requires meson, ninja, cargo, and git; does NOT require a TeX toolchain.
#
# Usage: test-meson-mock.sh <repository-root>
set -eu

repository_root="${1:-.}"
repository_root="$(cd "$repository_root" && pwd)"
cd "$repository_root"

mock_root="$repository_root/build/mocks"
build="$mock_root/meson"
rm -rf "$mock_root"

echo "==> configuring mocked Meson build (no TeX toolchain)" >&2
meson setup "$build" "$repository_root" \
  -Dmock_mode=true \
  -Dpublication_archive_root="$mock_root/archive" \
  -Dartifact_generation_output_dir="$mock_root/generated" \
  >/dev/null

echo "==> compiling the paper (mock TeX)" >&2
meson compile -C "$build" attestation >/dev/null

pdf="$(find "$mock_root/archive/rendered" -type f -name '*.pdf' | sort | sed -n 1p)"
flat="$(find "$mock_root/archive/flattened" -type f -name '*.tex' | sort | sed -n 1p)"
stamps="$build/papers/attestation/stamps.tex"
test -n "$pdf"
test -n "$flat"

# The real attestation-stamps ran: stamps.tex carries the four macros and
# no unresolved placeholder, and the mock PDF is the deterministic marker.
grep -q 'AttestationDate' "$stamps"
grep -q 'AttestationInstanceUUID' "$stamps"
if grep -q '@ATTESTATION_' "$stamps"; then
  echo "stamps.tex has an unresolved placeholder" >&2
  exit 1
fi
grep -q 'MOCK TRIPOD ATTESTATION PDF' "$pdf"

# Byte equality alone cannot tell "the edge did not run" from "the edge
# reran and restat restored identical bytes". Ninja's build log appends a
# line every time an output edge executes, so counting an output's lines
# before and after a no-op build detects a spurious rerun. The always-
# stale stamp derivation is expected to rerun (Git state is not a Ninja
# input), but its unchanged compare-if-changed outputs must keep the
# downstream document render and the flattener from re-executing.
ninja_log="$build/.ninja_log"
test -f "$ninja_log"
ninja_runs_for() {
  awk -F '\t' -v output="$1" '$4 == output { count += 1 } END { print count + 0 }' "$ninja_log"
}
render_output="$(
  awk -F '\t' '$4 ~ /papers\/attestation\/main[.]pdf$/ { output = $4 } END { print output }' \
    "$ninja_log"
)"
flat_output="$(
  awk -F '\t' \
    '$4 ~ /papers\/attestation\/Tripod_Attestation_.*_flat[.]tex$/ { output = $4 } END { print output }' \
    "$ninja_log"
)"
test -n "$render_output"
test -n "$flat_output"

echo "==> mirror repair (replaces publication-repair)" >&2
cp "$pdf" "$mock_root/expected.pdf"
cp "$flat" "$mock_root/expected.tex"
printf 'corrupted\n' > "$pdf"
printf 'corrupted\n' > "$flat"
meson compile -C "$build" attestation >/dev/null
cmp -s "$pdf" "$mock_root/expected.pdf" || { echo "PDF mirror not repaired" >&2; exit 1; }
cmp -s "$flat" "$mock_root/expected.tex" || { echo "flat mirror not repaired" >&2; exit 1; }

before="$(sha256sum "$pdf" "$flat")"
render_runs_before="$(ninja_runs_for "$render_output")"
flat_runs_before="$(ninja_runs_for "$flat_output")"
meson compile -C "$build" attestation >/dev/null
after="$(sha256sum "$pdf" "$flat")"
render_runs_after="$(ninja_runs_for "$render_output")"
flat_runs_after="$(ninja_runs_for "$flat_output")"
[ "$before" = "$after" ] || { echo "no-op rebuild changed mirror bytes" >&2; exit 1; }
[ "$render_runs_before" = "$render_runs_after" ] \
  || { echo "no-op build reran the mock document render" >&2; exit 1; }
[ "$flat_runs_before" = "$flat_runs_after" ] \
  || { echo "no-op build reran the LaTeX flattener" >&2; exit 1; }

echo "==> generator repair (replaces generator-repair)" >&2
meson compile -C "$build" generate-artifacts >/dev/null
artifact="$(find "$mock_root/generated" -type f -name '*.json' | sort | sed -n 1p)"
test -n "$artifact"
cp "$artifact" "$mock_root/expected.json"
printf '{"corrupted":true}\n' > "$artifact"
meson compile -C "$build" generate-artifacts >/dev/null
cmp -s "$artifact" "$mock_root/expected.json" \
  || { echo "generated artifact not repaired" >&2; exit 1; }

echo "==> failure propagation (injected mock latexmk failure blocks publish)" >&2
# mock_fail_child is a paper-subproject option, hence the qualified name.
meson configure "$build" -Dattestation:mock_fail_child=latexmk >/dev/null
if meson compile -C "$build" attestation >/dev/null 2>&1; then
  echo "publication succeeded despite an injected render failure" >&2
  exit 1
fi

echo "==> mocked Meson contract passed" >&2
