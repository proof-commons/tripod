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
elif [ ! -e "$stamp" ]; then
  touch "$stamp"
fi