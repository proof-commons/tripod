#!/bin/sh
set -eu

epoch_file="$1"
shift

SOURCE_DATE_EPOCH="$(cat "$epoch_file")"
test -n "$SOURCE_DATE_EPOCH"

export SOURCE_DATE_EPOCH
export FORCE_SOURCE_DATE=1

exec "$@"