#!/bin/sh
# Publication-mode freshness contract for the two shell publication
# paths (R2-N04).
#
# Usage: test-publication-mode.sh <repository-root>
#
# Compare-if-changed equality is bytes AND required mode. Comparing
# bytes alone left a byte-current destination stuck at a wrong mode
# forever, because every later run saw equal bytes and skipped the
# repair. These cases mirror the Rust matrix in
# packages/cli-common/src/tests/mod.rs for:
#
#   scripts/cargo-bin-sync.sh   (synced helper binaries, 0755)
#   scripts/sync-publication.sh (PDF and flattened-source mirrors, 0644)
#
# cargo-bin-sync.sh takes the cargo program as its first argument, so a
# mock cargo keeps this test free of the Rust toolchain.
set -eu

root="${1:?usage: test-publication-mode.sh <repository-root>}"
root="$(cd "$root" && pwd)"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT HUP INT TERM

failures=0

fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}

mode_of() {
  # POSIX find matches exact permission bits; see the scripts themselves.
  if [ -n "$(find "$1" -perm 755 -print 2>/dev/null)" ]; then
    echo 755
  elif [ -n "$(find "$1" -perm 644 -print 2>/dev/null)" ]; then
    echo 644
  elif [ -n "$(find "$1" -perm 600 -print 2>/dev/null)" ]; then
    echo 600
  else
    echo other
  fi
}

expect_mode() {
  actual="$(mode_of "$1")"
  [ "$actual" = "$2" ] || fail "$3 (mode $actual, expected $2)"
}

expect_bytes() {
  actual="$(cat "$1")"
  [ "$actual" = "$2" ] || fail "$3 (bytes '$actual', expected '$2')"
}

# --- cargo-bin-sync.sh -------------------------------------------------

sync_bin="$root/scripts/cargo-bin-sync.sh"

# A mock cargo that writes the "built" binary where the real one would
# land, so the script's own copy/compare logic is what is under test.
mock_cargo="$work/mock-cargo"
cat > "$mock_cargo" <<'MOCK'
#!/bin/sh
set -eu
# Arguments mirror the real invocation; only --target-dir and --bin are
# needed to place the built artefact.
target_dir=""
bin=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --target-dir) target_dir="$2"; shift 2 ;;
    --bin) bin="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$target_dir/release"
printf '%s' "$MOCK_CARGO_BYTES" > "$target_dir/release/$bin"
chmod 755 "$target_dir/release/$bin"
MOCK
chmod 755 "$mock_cargo"

bin_case() {
  bin_dir="$work/bin-$1"
  mkdir -p "$bin_dir"
  manifest="$bin_dir/Cargo.toml"
  : > "$manifest"
  dest="$bin_dir/helper"
}

run_bin_sync() {
  MOCK_CARGO_BYTES="$1" \
    CARGO_TARGET_DIR="$bin_dir/target" \
    sh "$sync_bin" "$mock_cargo" "$manifest" pkg helper "$dest"
}

# Row 1: absent destination.
bin_case absent
run_bin_sync built-1
expect_bytes "$dest" built-1 "absent binary destination is published"
expect_mode "$dest" 755 "a newly synced binary is executable"

# Row 2: stale bytes, correct mode.
bin_case stale
printf 'built-0' > "$dest"
chmod 755 "$dest"
run_bin_sync built-1
expect_bytes "$dest" built-1 "stale binary bytes are replaced"
expect_mode "$dest" 755 "a resynced binary stays executable"

# Row 3: current bytes, correct mode — no rewrite.
bin_case current
printf 'built-1' > "$dest"
chmod 755 "$dest"
before="$(ls -l "$dest")"
run_bin_sync built-1
[ "$(ls -l "$dest")" = "$before" ] ||
  fail "a current binary destination must not be rewritten"

# Rows 4/5/6/7: current bytes, missing executable bit — mode-only repair.
bin_case mode-only
printf 'built-1' > "$dest"
chmod 644 "$dest"
run_bin_sync built-1
expect_mode "$dest" 755 "a byte-current binary missing its x bit is repaired"
expect_bytes "$dest" built-1 "a binary mode repair must not alter bytes"

# Row 8: the run after the repair is a no-op.
after_repair="$(ls -l "$dest")"
run_bin_sync built-1
[ "$(ls -l "$dest")" = "$after_repair" ] ||
  fail "the binary run after a mode repair must be a no-op"

# --- sync-publication.sh ----------------------------------------------

sync_pub="$root/scripts/sync-publication.sh"

pub_case() {
  pub_dir="$work/pub-$1"
  mkdir -p "$pub_dir"
  source_file="$pub_dir/source"
  destination="$pub_dir/mirror"
  stamp="$pub_dir/stamp"
}

# Row 1: absent destination.
pub_case absent
printf 'v1' > "$source_file"
sh "$sync_pub" "$source_file" "$destination" "$stamp"
expect_bytes "$destination" v1 "absent mirror is published"
expect_mode "$destination" 644 "a new mirror is publicly readable"
[ -e "$stamp" ] || fail "publishing a mirror touches its stamp"

# Row 2: stale bytes, correct mode.
pub_case stale
printf 'v2' > "$source_file"
printf 'v1' > "$destination"
chmod 644 "$destination"
sh "$sync_pub" "$source_file" "$destination" "$stamp"
expect_bytes "$destination" v2 "stale mirror bytes are replaced"
expect_mode "$destination" 644 "a remirrored file stays publicly readable"

# Row 3: current bytes, correct mode — no rewrite.
pub_case current
printf 'v1' > "$source_file"
printf 'v1' > "$destination"
chmod 644 "$destination"
: > "$stamp"
before="$(ls -l "$destination")"
sh "$sync_pub" "$source_file" "$destination" "$stamp"
[ "$(ls -l "$destination")" = "$before" ] ||
  fail "a current mirror must not be rewritten"

# Rows 4/6/7: current bytes, owner-only mode — mode-only repair.
pub_case mode-only
printf 'v1' > "$source_file"
printf 'v1' > "$destination"
chmod 600 "$destination"
: > "$stamp"
sh "$sync_pub" "$source_file" "$destination" "$stamp"
expect_mode "$destination" 644 "a byte-current owner-only mirror is repaired"
expect_bytes "$destination" v1 "a mirror mode repair must not alter bytes"

# Row 8: the run after the repair is a no-op.
after_repair="$(ls -l "$destination")"
sh "$sync_pub" "$source_file" "$destination" "$stamp"
[ "$(ls -l "$destination")" = "$after_repair" ] ||
  fail "the mirror run after a mode repair must be a no-op"

if [ "$failures" -ne 0 ]; then
  echo "publication-mode contract: $failures failure(s)" >&2
  exit 1
fi

echo "publication-mode contract: all cases passed" >&2
