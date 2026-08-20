#!/bin/sh
set -eu
#
# Capture the complete output of every labels checker over the whole
# tree, so a refactor of the participation scanner can be shown to be
# behavior-preserving by diffing two captures.
#
# Usage: scripts/capture-label-goldens.sh <output-dir>
#        scripts/capture-label-goldens.sh --compare <dir-a> <dir-b>
#
# Each checker gets three files in <output-dir>:
#   <name>.out    stdout, verbatim
#   <name>.err    stderr, verbatim
#   <name>.status the exit status
#
# `--compare` diffs two captures after normalizing the one field that
# is legitimately run-dependent: the RFC-3339 timestamp every tracing
# record carries on stderr. Nothing else is normalized, so a diff that
# reports no output is byte equality modulo wall-clock time.
#
# The argv mirrors the gate's checker lanes (labels-check, plans-check,
# forbidden-text-check) and the Meson generator target
# (generate-label-registers), so the capture exercises exactly the
# invocations the gate exercises. The register generator writes to
# files under <output-dir> rather than in-tree, so a capture never
# perturbs the committed publications.
#
# Diagnostics carry repository-relative paths, so the capture is
# position-independent as long as it runs from the repository root.

if [ "${1-}" = "--compare" ]; then
  if [ "$#" -ne 3 ]; then
    echo "usage: $0 --compare <dir-a> <dir-b>" >&2
    exit 2
  fi
  normalize_dir="$(mktemp -d)"
  trap 'rm -rf "$normalize_dir"' EXIT
  for side in a b; do
    case "$side" in
      a) source_dir="$2" ;;
      b) source_dir="$3" ;;
    esac
    # The register generator logs the absolute path it wrote to, which
    # differs between two captures by construction; resolve each
    # capture directory so that path can be normalized away.
    source_dir="$(cd "$source_dir" && pwd)"
    mkdir -p "$normalize_dir/$side"
    for file in "$source_dir"/*; do
      sed -e 's/"timestamp":"[^"]*"/"timestamp":"NORMALIZED"/g' \
        -e "s|$source_dir|CAPTURE-DIR|g" \
        "$file" >"$normalize_dir/$side/$(basename "$file")"
    done
  done
  if diff -ru "$normalize_dir/a" "$normalize_dir/b"; then
    echo "GOLDEN: identical" >&2
  else
    echo "GOLDEN: DIFFERENT" >&2
    exit 1
  fi
  exit 0
fi

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <output-dir>" >&2
  exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$1"
case "$out" in
  /*) ;;
  *) out="$PWD/$out" ;;
esac
mkdir -p "$out"
cd "$root"

# shellcheck source=scripts/census-args.sh
. ./scripts/census-args.sh

audit_tracked_paths

# One checker run, captured whole. `set +e` around the run keeps a
# non-zero checker status from aborting the capture: a failing checker
# is itself an observation worth diffing.
capture() {
  name="$1"
  shift
  set +e
  "$@" >"$out/$name.out" 2>"$out/$name.err"
  status=$?
  set -e
  printf '%s\n' "$status" >"$out/$name.status"
  printf 'captured %s (exit %s)\n' "$name" "$status" >&2
}

# The binaries are built once up front so cargo's own build chatter
# never lands in a captured stderr.
cargo build -p tripod-labels \
  --bin check-labels --bin check-plans --bin check-forbidden-text \
  --bin generate-label-registers >&2

bin="$root/target/debug"

label_census="$(derive_census 'label census' census_args labels)"
eval "set -- $label_census"
capture check-labels "$bin/check-labels" --repository-root . "$@"

plan_subjects="$(derive_census 'plan subject census' plan_subject_args)"
eval "set -- $plan_subjects"
capture check-plans "$bin/check-plans" --repository-root . "$@"

capture check-forbidden-text "$bin/check-forbidden-text" \
  --repository-root . --git git

register_census="$(derive_census 'register census' census_args scoped)"
eval "set -- $register_census"
# The scoped census carries --model-source, which the register
# generator does not accept; drop those pairs and keep the rest.
generator_argv=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --model-source) shift 2 ;;
    *) generator_argv="$generator_argv $(quote_word "$1")"; shift ;;
  esac
done
eval "set -- $generator_argv"
capture generate-label-registers "$bin/generate-label-registers" \
  --repository-root . "$@" \
  --specification-register-output "$out/specification-register.md" \
  --realization-register-output "$out/realization-register.md"
