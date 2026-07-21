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

echo "==> mirror repair (replaces publication-repair)" >&2
cp "$pdf" "$mock_root/expected.pdf"
cp "$flat" "$mock_root/expected.tex"
printf 'corrupted\n' > "$pdf"
printf 'corrupted\n' > "$flat"
meson compile -C "$build" attestation >/dev/null
cmp -s "$pdf" "$mock_root/expected.pdf" || { echo "PDF mirror not repaired" >&2; exit 1; }
cmp -s "$flat" "$mock_root/expected.tex" || { echo "flat mirror not repaired" >&2; exit 1; }

before="$(sha256sum "$pdf" "$flat")"
meson compile -C "$build" attestation >/dev/null
after="$(sha256sum "$pdf" "$flat")"
[ "$before" = "$after" ] || { echo "no-op rebuild changed mirror bytes" >&2; exit 1; }

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
