#!/bin/sh
# Capture one independently acquired announcement and its ordered boundary experiment.
set -eu
exec python3 "$(dirname "$0")/elements-observer-capture.py" "$@"
