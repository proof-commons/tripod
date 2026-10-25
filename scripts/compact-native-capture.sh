#!/bin/sh
# One serialized compact ceremony with an immutable evidence boundary.
set -eu
LC_ALL=C
export LC_ALL
started=$(date +%s%3N)
EXPECTED_ELEMENTSD_TIP=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0
EXPECTED_OPERATIONS=92
RUN_ADDRESS_SEPARATOR='tripod/compact-run/address/v1'
RUN_INPUT_SET='candidate rule:phase7:wave0-rulings 128
pins FIXTURE_MERKLE_ROOT FIXTURE_REFERENCE_OUTPUT_KEY PINNED_PROGRAM
roster compact-native-requirements-v1 48
controls A2 A3 A4 N2 N3 N4 C2 C3 C4
staged 16
ceremony report'
die() { echo "compact capture refusal: $*" >&2; exit 1; }
field() { awk -v key="$2" '$1 == key { n++; sub(/^[^ ]+ /, ""); value=$0 } END { if(n!=1) exit 1; print value }' "$1"; }
hex() { od -An -tx1 | tr -d ' \n'; }
lowerhex() { [ "${#1}" -eq "$2" ] || return 1; case "$1" in *[!0-9a-f]*) return 1;; esac; }
root=$(git rev-parse --show-toplevel)
cd "$root"
[ "$#" -eq 1 ] || die 'one new destination is required'
destination=$1
case "$destination" in /*) ;; *) die 'destination must be absolute';; esac
[ ! -e "$destination" ] || die 'destination already exists'
[ -z "$(git status --porcelain --untracked-files=normal)" ] || die 'source tree is dirty'
mode=${TRIPOD_COMPACT_MODE:-record}
case "$mode" in probe|record) ;; *) die 'unknown mode';; esac
attempt=${TRIPOD_COMPACT_ATTEMPT:-1}
case "$attempt" in ''|*[!0-9]*) die 'attempt must be numeric';; esac
if [ "$mode" = probe ]; then
  [ -n "${TRIPOD_COMPACT_PROBE_ROOT:-}" ] || die 'probe root is required'
  case "$TRIPOD_COMPACT_PROBE_ROOT" in /*) ;; *) die 'probe root must be absolute';; esac
  probe_root=$(realpath -m -- "$TRIPOD_COMPACT_PROBE_ROOT")
  probe_destination=$(realpath -m -- "$destination")
  for probe_path in "$probe_root" "$probe_destination"; do
    case "$probe_path" in /workspace/loops/tripod/evidence|/workspace/loops/tripod/evidence/*) die 'probe path is under evidence';; esac
  done
  case "$probe_destination" in "$probe_root"/*) ;; *) die 'probe destination is outside scratch';; esac
fi
suite_commit=$(git rev-parse HEAD)
suite_tree=$(git rev-parse 'HEAD^{tree}')
lowerhex "$suite_commit" 40 || die 'source identity'
lowerhex "$suite_tree" 40 || die 'source tree identity'
for variable in TRIPOD_LIVE_EXECUTOR ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI; do
  eval "value=\${$variable:-}"
  case "$value" in /*) ;; *) die "$variable must be absolute";; esac
  [ -x "$value" ] || die "$variable is not executable"
done
[ -d "${ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK:-}" ] || die 'functional framework missing'
[ -n "${ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET:-}" ] || die 'wallet capability required'
[ "${ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP:-}" = "$EXPECTED_ELEMENTSD_TIP" ] || die 'intended target differs'
lowerhex "${TRIPOD_LIVE_NETWORK_ID:-}" 64 || die 'deployment identity'
lowerhex "${TRIPOD_LIVE_GENESIS_ID:-}" 64 || die 'genesis identity'
[ "${ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID:-}" = "$TRIPOD_LIVE_NETWORK_ID" ] || die 'deployment identities differ'
[ -d "${TMPDIR:-}" ] && [ -w "$TMPDIR" ] || die 'writable temporary root required'
cargo=${TRIPOD_CAPTURE_CARGO:-cargo}
cargo=$(command -v "$cargo") || die 'cargo missing'
address=$( { printf '%s\000%s\n' "$RUN_ADDRESS_SEPARATOR" "$RUN_INPUT_SET"; printf 'mode %s\nattempt %s\n' "$mode" "$attempt"; } | sha256sum)
address=${address%% *}
mkdir "$destination"
temporary=$(mktemp -d "$TMPDIR/compact-capture.XXXXXX")
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
preflight_finished=$(date +%s%3N)
set +e
TRIPOD_LIVE_REPORT_DIR="$destination" TRIPOD_LIVE_SUITE_COMMIT="$suite_commit" TRIPOD_LIVE_SUITE_TREE="$suite_tree" TRIPOD_COMPACT_RUN_ADDRESS="$address" \
  "$cargo" test -p tripod-vectors --test compact_native -- --ignored --test-threads=1 --nocapture > "$temporary/cargo.log" 2>&1
cargo_rc=$?
set -e
cat "$temporary/cargo.log"
execution_finished=$(date +%s%3N)
capture=$destination/$address.report.capture
timing=$capture.timing
valid=yes
check() { if ! "$@"; then valid=no; echo "compact check failed: $*" >&2; fi; }
check test "$cargo_rc" -eq 0
check grep -q '^test result: ok. 1 passed; 0 failed; 0 ignored;' "$temporary/cargo.log"
check test -f "$capture"
check test -f "$timing"
if [ -f "$capture" ] && [ -f "$timing" ]; then
  check grep -qx 'native-capture-schema 2' "$capture"
  check grep -qx 'ceremony-id report' "$capture"
  check grep -qx "suite-commit $suite_commit" "$capture"
  check grep -qx "suite-tree $suite_tree" "$capture"
  check grep -qx 'handshake-protocol-schema 8' "$capture"
  check grep -qx 'environment-schema 8' "$capture"
  check grep -qx 'terminal-state complete' "$capture"
  check test "$(tail -1 "$capture")" = 'native-capture-end report'
  digest=$(field "$capture" capture-content-sha256) || digest=missing
  prefix_digest=$(awk '/^capture-content-sha256 / {exit} {print}' "$capture" | sha256sum)
  check test "$digest" = "${prefix_digest%% *}"
  check grep -qx "handshake-intended-executed-tip 40 $(printf '%s' "$EXPECTED_ELEMENTSD_TIP" | hex)" "$capture"
  revision=$(field "$capture" handshake-binary-reported-revision) || revision=missing
  length=${revision%% *}
  encoded=${revision#* }
  case "$length" in ''|*[!0-9]*) valid=no;; *)
    check test "$length" -ge 12
    check test "$length" -le 40
    prefix=$(printf '%s' "$EXPECTED_ELEMENTSD_TIP" | cut -c "1-$length")
    check test "$encoded" = "$(printf '%s' "$prefix" | hex)"
    ;;
  esac
  check grep -qx 'timing-schema compact-phases-1' "$timing"
  check grep -qx 'status passed' "$timing"
  check awk 'BEGIN {n=0} /^(phase-staging-ms|phase-execution-ms|total-ms) [0-9]+$/ {n++} END {exit n!=3}' "$timing"
fi
diagnostics_present=no
for entry in "$destination"/* "$destination"/.[!.]* "$destination"/..?*; do
  [ -e "$entry" ] || [ -L "$entry" ] || continue
  case "$entry" in "$capture"|"$timing") check test ! -L "$entry";; "$destination/diagnostics") check test -d "$entry"; check test ! -L "$entry"; diagnostics_present=yes;; *) valid=no;; esac
done
manifest=$destination/MANIFEST.sha256
: > "$manifest"
for file in "$capture" "$timing"; do
  if [ -f "$file" ]; then
    digest=$(sha256sum "$file"); digest=${digest%% *}
    printf '%s  %s\n' "$digest" "${file##*/}" >> "$manifest"
  fi
done
check test "$(wc -l < "$manifest")" -eq 2
check sh -c 'cd "$1" && sha256sum -c MANIFEST.sha256' sh "$destination"
manifest_digest=$(sha256sum "$manifest"); manifest_digest=${manifest_digest%% *}
operations=missing
[ ! -f "$capture" ] || operations=$(field "$capture" operation-count)
eligible=no
if [ "$mode" = record ] && [ "$valid" = yes ] && [ "$operations" = "$EXPECTED_OPERATIONS" ] && grep -qx "COMPACT operations $EXPECTED_OPERATIONS controls 9 suspended 0" "$temporary/cargo.log"; then eligible=yes; fi
{
  printf 'run-report-schema compact-native-run-report 1\nrun-address %s\n' "$address"
  printf 'suite-commit %s\nsuite-tree %s\nsuite-clean yes\n' "$suite_commit" "$suite_tree"
  printf 'mode %s\nattempt %s\n' "$mode" "$attempt"
  printf 'expected-test-count 1\nexpected-ceremony-count 1\nexpected-setup-count 0\n'
  printf 'expected-capture-count 1\nexpected-timing-count 1\nexpected-archive-file-count 4\ndiagnostics-present %s\n' "$diagnostics_present"
  printf 'expected-operation-count %s\nobserved-operation-count %s\n' "$EXPECTED_OPERATIONS" "$operations"
  printf 'elementsd-expected-tip %s\nprotocol-revision 8\n' "$EXPECTED_ELEMENTSD_TIP"
  printf 'cargo-exit-code %s\nmanifest-sha256 %s\neligible %s\n' "$cargo_rc" "$manifest_digest" "$eligible"
} > "$destination/RUN-REPORT"
finished=$(date +%s%3N)
printf 'phase preflight wall-ms %s\nphase invocation wall-ms %s\nphase sealing wall-ms %s\nscript total wall-ms %s\n' "$((preflight_finished-started))" "$((execution_finished-preflight_finished))" "$((finished-execution_finished))" "$((finished-started))"
chmod -R a-w "$destination"
[ "$valid" = yes ] || exit 1
[ "$mode" = probe ] || [ "$eligible" = yes ]
