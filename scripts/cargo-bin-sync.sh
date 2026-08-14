#!/bin/sh
# Build one workspace binary with cargo and sync it into a declared
# meson custom-target output.
#
# Usage: cargo-bin-sync.sh <cargo> <manifest> <package> <bin> <dest>
#
# Cargo is its own dependency tracker, so the meson target that runs
# this script is build_always_stale: cargo returns quickly when the
# binary is fresh. The copy happens only when the bytes changed, so
# ninja's restat pass leaves downstream (paper) targets clean on a
# no-op rebuild. Cargo resolves the dependency versions from
# Cargo.toml on every build.
set -eu

cargo="$1"
manifest="$2"
package="$3"
bin="$4"
dest="$5"

# Cargo's effective target directory is not always <workspace>/target:
# CARGO_TARGET_DIR or a cargo config can move it. Pass the directory
# explicitly so the build and the copy always agree on one path
# (honoring an externally set CARGO_TARGET_DIR when present).
target_dir="${CARGO_TARGET_DIR:-$(dirname "$manifest")/target}"

"$cargo" build --release \
  --manifest-path "$manifest" \
  --target-dir "$target_dir" \
  --package "$package" \
  --bin "$bin" >&2

built="$target_dir/release/$bin"

# POSIX sh has no portable mode query; `find <file> -perm <octal>`
# matches the exact permission bits and prints the path when they agree.
publication_mode_is() {
  [ -n "$(find "$1" -perm "$2" -print 2>/dev/null)" ]
}

if ! cmp -s "$built" "$dest" 2>/dev/null; then
  # Unique staging name: ninja serializes this target, but a fixed
  # "$dest.tmp" would let any direct concurrent invocation corrupt the
  # copy. mktemp creates the file 0600, so restore executability.
  staged="$(mktemp "$(dirname "$dest")/.$(basename "$dest").XXXXXX")"
  cp "$built" "$staged"
  chmod 755 "$staged"
  mv "$staged" "$dest"
elif ! publication_mode_is "$dest" 755; then
  # Equal bytes with the wrong mode is not a current publication
  # (R2-N04). Without this branch a synced helper binary that lost its
  # executable bit stayed broken forever: cargo reports the build
  # fresh, cmp reports equal bytes, the copy and chmod are skipped, and
  # the consumer fails with permission denied on every rebuild. Repair
  # the mode on the destination itself — the bytes and their mtime are
  # untouched, so ninja's restat pass still leaves downstream (paper)
  # targets clean.
  chmod 755 "$dest"
fi
