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

# POSIX sh has no portable mode query; `find <file> -perm <octal>`
# matches the exact permission bits and prints the path when they agree.
publication_mode_is() {
  [ -n "$(find "$1" -perm "$2" -print 2>/dev/null)" ]
}

if ! cmp -s "$source_file" "$destination" 2>/dev/null; then
  staged="$(mktemp "$(dirname "$destination")/.$(basename "$destination").XXXXXX")"
  trap 'rm -f "$staged"' EXIT HUP INT TERM
  cp "$source_file" "$staged"
  # mktemp creates the staged file owner-only, and `cp` into an
  # existing file keeps that mode, so the publication mode is set
  # explicitly before the rename (SR3-04). Every destination of this
  # script — the rendered PDF mirror and the flattened source — is
  # public data, matching cli_common::PublicationMode::Public; there is
  # no executable publication here. Set exactly, not umask-masked: a
  # publication's readability is not a property of the build
  # environment.
  chmod 644 "$staged"
  mv "$staged" "$destination"
  touch "$stamp"
else
  # Equal bytes with the wrong mode is not a current publication
  # (R2-N04): comparing bytes alone left a mirror stuck at an
  # owner-only mode forever, because every later run saw equal bytes
  # and skipped the chmod. Repair the mode on the destination itself,
  # leaving the mirrored bytes and their mtime untouched — a mode
  # repair is not new content, so it does not re-touch the stamp.
  if ! publication_mode_is "$destination" 644; then
    chmod 644 "$destination"
  fi
  if [ ! -e "$stamp" ]; then
    touch "$stamp"
  fi
fi