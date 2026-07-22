#!/bin/sh
# Integration test for the build-time stamp derivation (ADR-014). It
# builds a throwaway Git repository and asserts the four derived values
# have the intended, distinct scopes:
#
#   input change            -> document_uuid, instance_uuid, date, timestamp all move
#   non-input paper change  -> only instance_uuid and timestamp move
#   docs change             -> nothing moves
#   dirty paper subtree     -> derivation is a hard failure
#
# Usage: test-attestation-stamps.sh <repository-root>
set -eu

repository_root="${1:-.}"
repository_root="$(cd "$repository_root" && pwd)"

temporary="$(mktemp -d "${TMPDIR:-/tmp}/attestation-stamps-it-XXXXXX")"
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

cargo_target="$temporary/cargo-target"
export CARGO_TARGET_DIR="$cargo_target"

echo "==> building attestation-stamps" >&2
cargo build --release \
  --manifest-path "$repository_root/Cargo.toml" \
  --package tripod-document-stamps \
  --bin attestation-stamps >/dev/null 2>&1
binary="$cargo_target/release/attestation-stamps"
test -x "$binary"

git_program="$(command -v git)"
repo="$temporary/repo"
mkdir -p "$repo/papers/attestation/sections" "$repo/docs/attestation"

git -C "$repo" init -q
git -C "$repo" config user.email "test@example.com"
git -C "$repo" config user.name "Stamp Test"
# The throwaway repo must not inherit a global commit-signing policy:
# there is no tty for pinentry in the build environment.
git -C "$repo" config commit.gpgsign false
git -C "$repo" config tag.gpgsign false

# Fixed commit instants, each on a distinct calendar day (> 86400s
# apart) so the day-granularity `date` field also differs when an input
# commit lands. Deterministic, and never colliding within a second.
epoch=1700000000
commit() {
  message="$1"
  epoch=$((epoch + 90000))
  GIT_AUTHOR_DATE="@$epoch +0000" GIT_COMMITTER_DATE="@$epoch +0000" \
    git -C "$repo" commit -q -m "$message"
}

# The exact publication inputs (main + one section). meson.build sits in
# the same subtree but is NOT an input.
main_tex="papers/attestation/main.tex"
section_tex="papers/attestation/sections/00_title.tex"
paper_meson="papers/attestation/meson.build"
human_md="docs/attestation/human.md"

printf '%s\n' '\documentclass{article}\begin{document}A\end{document}' > "$repo/$main_tex"
printf '%s\n' '\title{Attestation}' > "$repo/$section_tex"
printf '%s\n' "# paper build definition" > "$repo/$paper_meson"
printf '%s\n' "companion" > "$repo/$human_md"
git -C "$repo" add -A
commit "initial"

derive() {
  "$binary" \
    --git "$git_program" \
    --repository-root "$repo" \
    --tree-ref HEAD \
    --tree papers/attestation \
    --input "$main_tex" \
    --input "$section_tex"
}

field() {
  printf '%s' "$1" |
    python3 -c '
import json
import sys

value = json.load(sys.stdin)
for component in sys.argv[1].removeprefix(".").split("."):
    value = value[component]
print(value)
' "$2"
}

fail() { echo "FAIL: $1" >&2; exit 1; }
assert_eq() { [ "$2" = "$3" ] || fail "$1: expected '$3' == '$2'"; }
assert_ne() { [ "$2" != "$3" ] || fail "$1: expected '$2' != '$3'"; }

capture() {
  json="$(derive)"
  doc="$(field "$json" .document_uuid)"
  inst="$(field "$json" .instance_uuid)"
  date="$(field "$json" .date)"
  ts="$(field "$json" .timestamp.epoch)"
}

echo "==> baseline" >&2
capture
doc0="$doc"; inst0="$inst"; date0="$date"; ts0="$ts"

echo "==> scope 1: change a publication input (all four move)" >&2
printf '%s\n' '\documentclass{article}\begin{document}B\end{document}' > "$repo/$main_tex"
git -C "$repo" add -A
commit "edit input"
capture
assert_ne "input->document_uuid" "$doc" "$doc0"
assert_ne "input->instance_uuid" "$inst" "$inst0"
assert_ne "input->date" "$date" "$date0"
assert_ne "input->timestamp" "$ts" "$ts0"
doc1="$doc"; inst1="$inst"; date1="$date"; ts1="$ts"

echo "==> scope 2: change a non-input paper file (only instance+timestamp move)" >&2
printf '%s\n' "# paper build definition, revised" > "$repo/$paper_meson"
git -C "$repo" add -A
commit "edit non-input paper file"
capture
assert_eq "noninput->document_uuid" "$doc" "$doc1"
assert_eq "noninput->date" "$date" "$date1"
assert_ne "noninput->instance_uuid" "$inst" "$inst1"
assert_ne "noninput->timestamp" "$ts" "$ts1"
doc2="$doc"; inst2="$inst"; date2="$date"; ts2="$ts"

echo "==> scope 3: change a docs file (nothing moves)" >&2
printf '%s\n' "companion, revised" > "$repo/$human_md"
git -C "$repo" add -A
commit "edit docs companion"
capture
assert_eq "docs->document_uuid" "$doc" "$doc2"
assert_eq "docs->instance_uuid" "$inst" "$inst2"
assert_eq "docs->date" "$date" "$date2"
assert_eq "docs->timestamp" "$ts" "$ts2"

echo "==> scope 4: a dirty paper subtree is a hard failure" >&2
printf '%s\n' '\documentclass{article}\begin{document}DIRTY\end{document}' > "$repo/$main_tex"
if derive >/dev/null 2>&1; then
  fail "dirty subtree was accepted"
fi
git -C "$repo" checkout -- "$main_tex"

echo "==> all identity-scope assertions passed" >&2
