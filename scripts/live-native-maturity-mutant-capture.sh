#!/bin/sh
# Capture driver for the native maturity mutant ceremony, protocol revision 8.
#
# Usage: scripts/live-native-maturity-mutant-capture.sh <profile> <new-capture-directory>
#
# The harness owns capture grammar and semantic assertions. This driver owns
# the run boundary: clean source identity, one serialized invocation, the
# fixed ceremony/file census, cross-file run identity, the manifest, and the
# eligibility conclusion. The single report ceremony has no setup artifact.
#
# Invoke from a clean suite worktree. Cargo argv:
#   cargo test -p tripod-vectors --test maturity_predecessor_mutants -- --ignored --test-threads=1
#   cargo test -p tripod-vectors --test maturity_successor_mutants -- --ignored --test-threads=1
# Required environment:
#   TRIPOD_LIVE_EXECUTOR: absolute executable adapter path.
#   TRIPOD_LIVE_NETWORK_ID, TRIPOD_LIVE_GENESIS_ID: 64 lower-case hex digits;
#     genesis stays in the node's printed order.
#   ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD, ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI:
#     absolute executable paths; ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK: absolute directory.
#   ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID: identical to TRIPOD_LIVE_NETWORK_ID.
#   ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP: full lower-case expected node revision.
#   ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET: non-empty for native funding.
#   TMPDIR: existing writable disk-backed temporary directory.
# Optional environment:
#   TRIPOD_CAPTURE_CARGO (default cargo), TRIPOD_CAPTURE_SHA256SUM (default sha256sum).
#   ELEMENTS_NATIVE_EXECUTOR_PYTHON (adapter default python3),
#   ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY and other adapter settings pass through.
# The driver derives TRIPOD_LIVE_SUITE_COMMIT, TRIPOD_LIVE_SUITE_TREE and
# TRIPOD_LIVE_SUITE_SHORT_SHA from HEAD, and sets TRIPOD_LIVE_REPORT_DIR to the
# new capture directory. It writes MANIFEST.sha256 and RUN-REPORT, then removes
# write permissions from the directory and its contents.
# The loop job uses the pinned Elements development toolchain with SQLite-only
# wallets and its src/test/functional framework, through the revision-8 adapter
# (elements-native-executor.sh, adapter version 2.1.0). The published development
# identities are supplied by the invoking loop, as in the operator run of record.
# The capture envelope is shared; the maturity payload retains the caller branch,
# declared-boundary standing and outstanding acceptance routes.
set -eu

LC_ALL=C
export LC_ALL

EXPECTED_ELEMENTSD_TIP=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0
EXPECTED_TEST_COUNT=1
EXPECTED_CEREMONY_COUNT=1
EXPECTED_SETUP_COUNT=0

# The carrier writes one capture and its three-field timing sidecar.
CEREMONY_IDS='report'

script_started_ms=$(date +%s%3N)
phase_name=startup
phase_started_ms=$script_started_ms

now_ms() {
  date +%s%3N
}

phase_start() {
  phase_name=$1
  phase_started_ms=$(now_ms)
  printf 'phase %s start\n' "$phase_name" >&2
}

phase_finish() {
  phase_finished_ms=$(now_ms)
  phase_elapsed_ms=$((phase_finished_ms - phase_started_ms))
  printf 'phase %s wall-ms %s\n' "$phase_name" "$phase_elapsed_ms" >&2
  case "$phase_name" in
    preflight) preflight_wall_ms=$phase_elapsed_ms ;;
    cargo) cargo_wall_ms=$phase_elapsed_ms ;;
    census) census_wall_ms=$phase_elapsed_ms ;;
    manifest) manifest_wall_ms=$phase_elapsed_ms ;;
  esac
  phase_name=
}

die() {
  message=$1
  if [ -n "$phase_name" ]; then
    phase_finish
  fi
  script_finished_ms=$(now_ms)
  printf 'ERROR: %s\n' "$message" >&2
  printf 'script total wall-ms %s\n' "$((script_finished_ms - script_started_ms))" >&2
  exit 1
}

is_lower_hex_length() {
  value=$1
  expected_length=$2
  [ "${#value}" -eq "$expected_length" ] || return 1
  case "$value" in
    *[!0-9a-f]*) return 1 ;;
    *) return 0 ;;
  esac
}

# The operator-declared intended tip carries all 40 hex characters
# and is compared in full below. Elements Core's binary version string carries
# only a short source revision (12 hex characters in the observed binary), so
# the binary cannot emit bytes that its version carrier does not contain. Bind
# every character it does emit: at least 12 lower-case hex characters, no more
# than the full tip, and an exact prefix of that expected full tip.
binary_revision_matches_expected_tip() {
  binary_field=$1
  case "$binary_field" in
    *' '*) ;;
    *) return 1 ;;
  esac
  binary_length=${binary_field%% *}
  binary_hex=${binary_field#* }
  case "$binary_hex" in
    ''|*' '*) return 1 ;;
  esac
  case "$binary_length" in
    ''|0|0*|*[!0-9]*) return 1 ;;
  esac
  [ "$binary_length" -ge 12 ] || return 1
  [ "$binary_length" -le 40 ] || return 1
  [ "${#binary_hex}" -eq "$((binary_length * 2))" ] || return 1
  case "$binary_hex" in
    *[!0-9a-f]*) return 1 ;;
  esac

  expected_binary_prefix=$(printf '%s\n' "$EXPECTED_ELEMENTSD_TIP" |
    awk -v prefix_len="$binary_length" '{ print substr($0, 1, prefix_len) }')
  expected_binary_hex=$(printf '%s' "$expected_binary_prefix" | od -An -tx1 | tr -d ' \n')
  [ "$binary_hex" = "$expected_binary_hex" ]
}

require_absolute_executable() {
  variable_name=$1
  variable_value=$2
  case "$variable_value" in
    /*) ;;
    *) die "$variable_name must be an absolute path" ;;
  esac
  [ -f "$variable_value" ] || die "$variable_name does not name a file"
  [ -x "$variable_value" ] || die "$variable_name is not executable"
}

roster_contains() {
  wanted_id=$1
  for roster_id in $CEREMONY_IDS; do
    [ "$roster_id" = "$wanted_id" ] && return 0
  done
  return 1
}

single_field() {
  field_file=$1
  field_name=$2
  awk -v wanted="$field_name" '
    $1 == wanted { count += 1; line = $0 }
    END {
      if (count != 1) exit 1
      sub(/^[^ ]+ /, "", line)
      print line
    }
  ' "$field_file"
}

read_field_or_missing() {
  if field_result=$(single_field "$1" "$2"); then
    printf '%s\n' "$field_result"
  else
    printf 'missing\n'
  fi
}

validation_errors=0

record_error() {
  validation_errors=$((validation_errors + 1))
  printf 'capture refusal: %s\n' "$1" >&2
}

capture_field_findings() {
  checked_file=$1
  checked_id=$2
  awk \
    -v ceremony="$checked_id" \
    -v commit="$suite_commit" \
    -v tree="$suite_tree" \
    -v deployment_network="$expected_deployment_network" \
    -v deployment_genesis="$expected_deployment_genesis" \
    -v deployment_target="$expected_deployment_target" \
    -v rust_test="$expected_rust_test" \
    -v adapter_name="$observed_adapter_name" \
    -v adapter_version="$observed_adapter_version" \
    -v node_name="$observed_node_name" \
    -v node_version="$observed_node_version" \
    -v binary_revision="$observed_binary_revision" \
    -v intended="$observed_intended_tip" '
    function refuse(message) {
      print message
      bad = 1
    }
    function check(key, expected) {
      if (count[key] != 1) {
        refuse(key " is missing or duplicated")
      } else if (value[key] != expected) {
        refuse(key " differs")
      }
    }
    {
      if (NR == 1 && $0 != "native-capture-schema 2") {
        refuse("schema is not first")
      }
      key = $1
      line = $0
      sub(/^[^ ]+ /, "", line)
      count[key] += 1
      value[key] = line
      last = $0
    }
    END {
      check("native-capture-schema", "2")
      check("ceremony-id", ceremony)
      check("suite-commit", commit)
      check("suite-tree", tree)
      check("fixture-digest-algorithm", "forward-v2")
      check("rust-test-name", rust_test)
      check("handshake-protocol-schema", "8")
      check("environment-schema", "8")
      check("environment-network-id", deployment_network)
      check("environment-genesis-id", deployment_genesis)
      check("deployment-environment", "development")
      check("deployment-network-id", deployment_network)
      check("deployment-genesis-id", deployment_genesis)
      check("deployment-target-contract", deployment_target)
      check("handshake-adapter-name", adapter_name)
      check("handshake-adapter-version", adapter_version)
      check("handshake-node-name", node_name)
      check("handshake-node-version", node_version)
      check("handshake-binary-reported-revision", binary_revision)
      check("handshake-intended-executed-tip", intended)
      check("terminal-state", "complete")
      check("native-capture-end", ceremony)
      if (last != "native-capture-end " ceremony) {
        refuse("canonical end marker differs")
      }
      exit bad
    }
  ' "$checked_file"
}

timing_field_findings() {
  checked_file=$1
  checked_id=$2
  awk -v ceremony="$checked_id" '
    function refuse(message) {
      print message
      bad = 1
    }
    function check(key, expected) {
      if (count[key] != 1) {
        refuse(key " is missing or duplicated")
      } else if (value[key] != expected) {
        refuse(key " differs")
      }
    }
    {
      if (NR == 1 && $0 != "timing-schema 1") {
        refuse("schema is not first")
      }
      key = $1
      line = $0
      sub(/^[^ ]+ /, "", line)
      count[key] += 1
      value[key] = line
    }
    END {
      if (NR != 3) refuse("timing sidecar does not have three fields")
      check("timing-schema", "1")
      check("ceremony-id", ceremony)
      if (count["status"] != 1) {
        refuse("status is missing or duplicated")
      } else if (value["status"] == "failed-after-transcript" || value["status"] == "incomplete") {
        refuse("timing status is " value["status"])
      } else if (value["status"] != "passed") {
        refuse("timing status is unknown")
      }
      exit bad
    }
  ' "$checked_file"
}

if [ "$#" -ne 2 ]; then
  die "usage: live-native-maturity-mutant-capture.sh <profile> <new-capture-directory>"
fi

profile=$1
capture_argument=$2
case "$profile" in
  predecessor-constructor)
    rust_test_target=maturity_predecessor_mutants
    maturity_test=the_maturity_predecessor_mutants_are_refused_before_their_control_is_accepted
    ;;
  successor-constructor)
    rust_test_target=maturity_successor_mutants
    maturity_test=the_maturity_successor_mutants_are_refused_before_their_control_is_accepted
    ;;
  *)
    die "usage: live-native-maturity-mutant-capture.sh <profile> <new-capture-directory>"
    ;;
esac

phase_start preflight

for required_tool in awk chmod date dirname find git grep mkdir mktemp od rm sed sort tr wc; do
  command -v "$required_tool" >/dev/null 2>&1 || die "$required_tool is required"
done

capture_cargo=${TRIPOD_CAPTURE_CARGO:-cargo}
capture_sha256=${TRIPOD_CAPTURE_SHA256SUM:-sha256sum}
capture_cargo=$(command -v "$capture_cargo") || die "TRIPOD_CAPTURE_CARGO is not executable"
capture_sha256=$(command -v "$capture_sha256") || die "TRIPOD_CAPTURE_SHA256SUM is not executable"

suite_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "the suite is not a Git worktree"
suite_root=$(cd "$suite_root" && pwd -P)
cd "$suite_root"

suite_status=$(git status --porcelain --untracked-files=normal)
[ -z "$suite_status" ] || die "the suite tree is dirty"

suite_commit=$(git rev-parse --verify HEAD)
suite_tree=$(git rev-parse --verify 'HEAD^{tree}')
suite_short_sha=$(git rev-parse --short HEAD)
is_lower_hex_length "$suite_commit" 40 || die "the suite commit is not full lower-case SHA-1"
is_lower_hex_length "$suite_tree" 40 || die "the suite tree is not full lower-case SHA-1"
case "$suite_commit" in
  "$suite_short_sha"*) ;;
  *) die "the suite short SHA is not a prefix of the full commit" ;;
esac

roster_count=0
previous_id=
for roster_id in $CEREMONY_IDS; do
  roster_count=$((roster_count + 1))
  if [ -n "$previous_id" ] && [ "$previous_id" \> "$roster_id" ]; then
    die "the declared ceremony roster is not sorted"
  fi
  previous_id=$roster_id
done
[ "$roster_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || die "the declared ceremony roster is not 1 ID"

live_executor=${TRIPOD_LIVE_EXECUTOR:-}
[ -n "$live_executor" ] || die "TRIPOD_LIVE_EXECUTOR is required"
require_absolute_executable TRIPOD_LIVE_EXECUTOR "$live_executor"

elementsd=${ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD:-}
elements_cli=${ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI:-}
framework=${ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK:-}
executor_network=${ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID:-}
intended_tip=${ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP:-}
live_network=${TRIPOD_LIVE_NETWORK_ID:-}
live_genesis=${TRIPOD_LIVE_GENESIS_ID:-}

[ -n "$elementsd" ] || die "ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD is required"
[ -n "$elements_cli" ] || die "ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI is required"
require_absolute_executable ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD "$elementsd"
require_absolute_executable ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI "$elements_cli"
case "$framework" in
  /*) ;;
  *) die "ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK must be an absolute path" ;;
esac
[ -d "$framework" ] || die "ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK does not name a directory"
is_lower_hex_length "$live_network" 64 || die "TRIPOD_LIVE_NETWORK_ID must be 64 lower-case hex digits"
is_lower_hex_length "$live_genesis" 64 || die "TRIPOD_LIVE_GENESIS_ID must be 64 lower-case hex digits"
[ "$executor_network" = "$live_network" ] || die "executor and harness network IDs differ"
is_lower_hex_length "$intended_tip" 40 || die "ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP must be a full lower-case tip"

[ -n "${ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET:-}" ] || die "ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET is required for funding"

[ -n "${TMPDIR:-}" ] || die "TMPDIR must name disk-backed temporary storage"
[ -d "$TMPDIR" ] || die "TMPDIR does not name a directory"
[ -w "$TMPDIR" ] || die "TMPDIR is not writable"

if [ -e "$capture_argument" ]; then
  if [ -d "$capture_argument" ] &&
     [ -n "$(find "$capture_argument" -mindepth 1 -print -quit)" ]; then
    die "the capture directory is non-empty"
  fi
  die "the capture directory already exists; a new directory is required"
fi

capture_parent=$(dirname "$capture_argument")
[ -d "$capture_parent" ] || die "the capture directory parent does not exist"
mkdir "$capture_argument" || die "the capture directory could not be created"
capture_directory=$(cd "$capture_argument" && pwd -P)

temporary_directory=$(mktemp -d "$TMPDIR/tripod-live-capture.XXXXXX")
cleanup() {
  rm -rf "$temporary_directory"
}
trap cleanup EXIT HUP INT TERM

cargo_log=$temporary_directory/cargo.log
seen_ids=$temporary_directory/seen-ids
manifest_names=$temporary_directory/manifest-names
manifest_names_sorted=$temporary_directory/manifest-names.sorted
manifest_verification=$temporary_directory/manifest-verification
: > "$seen_ids"

phase_finish

phase_start cargo
set +e
TRIPOD_LIVE_REPORT_DIR="$capture_directory" \
TRIPOD_LIVE_SUITE_COMMIT="$suite_commit" \
TRIPOD_LIVE_SUITE_TREE="$suite_tree" \
TRIPOD_LIVE_SUITE_SHORT_SHA="$suite_short_sha" \
  "$capture_cargo" test -p tripod-vectors \
    --test "$rust_test_target" -- --ignored --test-threads=1 \
    > "$cargo_log" 2>&1
cargo_exit_code=$?
set -e
sed 's/^/cargo: /' "$cargo_log" >&2
phase_finish

phase_start census

test_summary=$(awk '
  /^test result: / {
    passed = $4
    failed = $6
    ignored = $8
    found += 1
  }
  END { if (found == 1) print passed, failed, ignored }
' "$cargo_log")
set -- $test_summary
if [ "$#" -eq 3 ]; then
  observed_passed=$1
  observed_failed=$2
  observed_ignored=$3
  case "$observed_passed$observed_failed$observed_ignored" in
    *[!0-9]*)
      observed_passed=0
      observed_failed=0
      observed_ignored=0
      record_error "the Cargo test census is not numeric"
      ;;
  esac
else
  observed_passed=0
  observed_failed=0
  observed_ignored=0
  record_error "the Cargo test census is missing"
fi
observed_test_count=$((observed_passed + observed_failed + observed_ignored))

[ "$cargo_exit_code" -eq 0 ] || record_error "Cargo exited $cargo_exit_code"
[ "$observed_test_count" -eq "$EXPECTED_TEST_COUNT" ] || record_error "the observed test census is not 1"
[ "$observed_passed" -eq "$EXPECTED_TEST_COUNT" ] || record_error "the run was not 1 of 1 passed"
[ "$observed_failed" -eq 0 ] || record_error "the run contains failed tests"
[ "$observed_ignored" -eq 0 ] || record_error "the selected run contains ignored outcomes"

expected_tip_hex=$(printf '%s' "$EXPECTED_ELEMENTSD_TIP" | od -An -tx1 | tr -d ' \n')
expected_tip_field="40 $expected_tip_hex"

# A single ceremony must bind its run identity to the invoking environment.
expected_deployment_network="64 $(printf '%s' "$live_network" | od -An -tx1 | tr -d ' \n')"
expected_deployment_genesis="64 $(printf '%s' "$live_genesis" | od -An -tx1 | tr -d ' \n')"
expected_deployment_target="21 $(printf '%s' elements-tapscript-v2 | od -An -tx1 | tr -d ' \n')"
expected_rust_test="${#maturity_test} $(printf '%s' "$maturity_test" | od -An -tx1 | tr -d ' \n')"

reference_capture=
for ceremony_id in $CEREMONY_IDS; do
  candidate_capture=$capture_directory/$suite_short_sha.$ceremony_id.capture
  if [ -f "$candidate_capture" ]; then
    reference_capture=$candidate_capture
    break
  fi
done

if [ -n "$reference_capture" ]; then
  observed_deployment_environment=$(read_field_or_missing "$reference_capture" deployment-environment)
  observed_deployment_network=$(read_field_or_missing "$reference_capture" deployment-network-id)
  observed_deployment_genesis=$(read_field_or_missing "$reference_capture" deployment-genesis-id)
  observed_deployment_target=$(read_field_or_missing "$reference_capture" deployment-target-contract)
  observed_adapter_name=$(read_field_or_missing "$reference_capture" handshake-adapter-name)
  observed_adapter_version=$(read_field_or_missing "$reference_capture" handshake-adapter-version)
  observed_node_name=$(read_field_or_missing "$reference_capture" handshake-node-name)
  observed_node_version=$(read_field_or_missing "$reference_capture" handshake-node-version)
  observed_binary_revision=$(read_field_or_missing "$reference_capture" handshake-binary-reported-revision)
  observed_intended_tip=$(read_field_or_missing "$reference_capture" handshake-intended-executed-tip)
else
  observed_deployment_environment=missing
  observed_deployment_network=missing
  observed_deployment_genesis=missing
  observed_deployment_target=missing
  observed_adapter_name=missing
  observed_adapter_version=missing
  observed_node_name=missing
  observed_node_version=missing
  observed_binary_revision=missing
  observed_intended_tip=missing
fi

physical_capture_count=0
physical_timing_count=0
for ceremony_id in $CEREMONY_IDS; do
  capture_file=$capture_directory/$suite_short_sha.$ceremony_id.capture
  timing_file=$capture_file.timing

  if [ ! -f "$capture_file" ]; then
    record_error "missing capture for $ceremony_id"
  else
    physical_capture_count=$((physical_capture_count + 1))

    if internal_id=$(single_field "$capture_file" ceremony-id); then
      if roster_contains "$internal_id"; then
        if awk -v wanted="$internal_id" '$1 == wanted { found = 1 } END { exit !found }' "$seen_ids"; then
          record_error "duplicate ceremony ID $internal_id"
        else
          printf '%s\n' "$internal_id" >> "$seen_ids"
        fi
      else
        record_error "${capture_file##*/}: unknown ceremony ID $internal_id"
      fi
      [ "$internal_id" = "$ceremony_id" ] || record_error "${capture_file##*/}: ceremony ID differs from its name"
    else
      record_error "${capture_file##*/}: ceremony-id is missing or duplicated"
    fi

    if capture_findings=$(capture_field_findings "$capture_file" "$ceremony_id"); then
      :
    else
      record_error "${capture_file##*/}: $capture_findings"
    fi
  fi

  if [ ! -f "$timing_file" ]; then
    record_error "missing timing sidecar for $ceremony_id"
  else
    physical_timing_count=$((physical_timing_count + 1))
    if timing_findings=$(timing_field_findings "$timing_file" "$ceremony_id"); then
      :
    else
      record_error "${timing_file##*/}: $timing_findings"
    fi
  fi
done

observed_ceremony_count=$(wc -l < "$seen_ids" | tr -d ' ')
[ "$physical_capture_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the physical capture census is not 1"
[ "$observed_ceremony_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the unique ceremony census is not 1"
[ "$physical_timing_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the timing census is not 1"

observed_setup_count=0

diagnostics_present=no
for entry in "$capture_directory"/* "$capture_directory"/.[!.]* "$capture_directory"/..?*; do
  [ -e "$entry" ] || continue
  entry_name=${entry##*/}
  entry_expected=no
  if [ "$entry_name" = diagnostics ] && [ -d "$entry" ] && [ ! -L "$entry" ]; then
    # The adapter owns this audit-only root. Its contents are unparsed,
    # outside the four-file evidence census and the manifest.
    diagnostics_present=yes
    entry_expected=yes
  else
    case "$entry_name" in
      "$suite_short_sha".*.capture.timing)
        entry_id=${entry_name#"$suite_short_sha".}
        entry_id=${entry_id%.capture.timing}
        roster_contains "$entry_id" && entry_expected=yes
        ;;
      "$suite_short_sha".*.capture)
        entry_id=${entry_name#"$suite_short_sha".}
        entry_id=${entry_id%.capture}
        roster_contains "$entry_id" && entry_expected=yes
        ;;
    esac
  fi

  case "$entry_name" in
    *.setup) observed_setup_count=$((observed_setup_count + 1)) ;;
  esac
  [ "$entry_expected" = yes ] || record_error "unexpected or noncanonical entry $entry_name"
  if [ -f "$entry" ] && grep -q "$(printf '\r')" "$entry"; then
    record_error "$entry_name contains a CR byte"
  fi
done

[ "$observed_setup_count" -eq "$EXPECTED_SETUP_COUNT" ] || record_error "the setup census is not 0"

# The intended tip is the full operator declaration. The binary-reported
# revision follows the short-prefix rule documented at its validator above.
binary_revision_matches_expected_tip "$observed_binary_revision" ||
  record_error "the binary-reported elementsd revision is not a lower-hex prefix of at least 12 characters"
[ "$observed_intended_tip" = "$expected_tip_field" ] || record_error "the intended executed elementsd tip differs from the expected full tip"

phase_finish

phase_start manifest

# RUN-REPORT contains the digest of MANIFEST.sha256. Including RUN-REPORT in
# that same manifest would create an unsatisfiable hash cycle. The manifest
# therefore binds the capture and timing sidecar; RUN-REPORT then binds
# the manifest. An eligible directory has four top-level evidence files.
: > "$manifest_names"
for entry in "$capture_directory"/* "$capture_directory"/.[!.]* "$capture_directory"/..?*; do
  [ -f "$entry" ] || continue
  printf '%s\n' "${entry##*/}" >> "$manifest_names"
done
sort "$manifest_names" > "$manifest_names_sorted"

: > "$capture_directory/MANIFEST.sha256"
while IFS= read -r manifest_name; do
  manifest_result=$("$capture_sha256" "$capture_directory/$manifest_name")
  manifest_digest=${manifest_result%% *}
  printf '%s  %s\n' "$manifest_digest" "$manifest_name" >> "$capture_directory/MANIFEST.sha256"
done < "$manifest_names_sorted"

manifest_result=$("$capture_sha256" "$capture_directory/MANIFEST.sha256")
manifest_digest=${manifest_result%% *}
if (cd "$capture_directory" && "$capture_sha256" -c MANIFEST.sha256) > "$manifest_verification" 2>&1; then
  manifest_verified=yes
else
  manifest_verified=no
  sed 's/^/manifest: /' "$manifest_verification" >&2
  record_error "MANIFEST.sha256 did not verify"
fi
phase_finish

if [ "$validation_errors" -eq 0 ] && [ "$manifest_verified" = yes ]; then
  eligible=yes
else
  eligible=no
fi

phase_start report
cargo_argv="$capture_cargo test -p tripod-vectors --test $rust_test_target -- --ignored --test-threads=1"
cargo_argv_hex=$(printf '%s' "$cargo_argv" | od -An -tx1 | tr -d ' \n')

{
  printf 'run-report-schema native-maturity-mutant-run-report 1\n'
  printf 'capture-format-schema native-maturity-capture 1\n'
  printf 'suite-commit %s\n' "$suite_commit"
  printf 'suite-tree %s\n' "$suite_tree"
  printf 'suite-clean yes\n'
  printf 'rust-test-target %s\n' "$rust_test_target"
  printf 'capture-profile %s\n' "$profile"
  printf 'cargo-argv %s %s\n' "${#cargo_argv}" "$cargo_argv_hex"
  printf 'expected-test-count %s\n' "$EXPECTED_TEST_COUNT"
  printf 'observed-test-count %s\n' "$observed_test_count"
  printf 'expected-ceremony-count %s\n' "$EXPECTED_CEREMONY_COUNT"
  printf 'observed-ceremony-count %s\n' "$observed_ceremony_count"
  printf 'expected-setup-count %s\n' "$EXPECTED_SETUP_COUNT"
  printf 'observed-setup-count %s\n' "$observed_setup_count"
  printf 'diagnostics-present %s\n' "$diagnostics_present"
  printf 'test-passed-count %s\n' "$observed_passed"
  printf 'test-failed-count %s\n' "$observed_failed"
  printf 'test-ignored-count %s\n' "$observed_ignored"
  printf 'cargo-exit-code %s\n' "$cargo_exit_code"
  printf 'elementsd-expected-tip %s\n' "$EXPECTED_ELEMENTSD_TIP"
  printf 'elementsd-binary-reported-revision %s\n' "$observed_binary_revision"
  printf 'elementsd-intended-executed-tip %s\n' "$observed_intended_tip"
  printf 'executor-adapter-name %s\n' "$observed_adapter_name"
  printf 'executor-adapter-version %s\n' "$observed_adapter_version"
  printf 'node-name %s\n' "$observed_node_name"
  printf 'node-version %s\n' "$observed_node_version"
  printf 'protocol-revision 8\n'
  printf 'fixture-digest-algorithm forward-v2\n'
  printf 'deployment-environment %s\n' "$observed_deployment_environment"
  printf 'deployment-network-id %s\n' "$observed_deployment_network"
  printf 'deployment-genesis-id %s\n' "$observed_deployment_genesis"
  printf 'deployment-target-contract %s\n' "$observed_deployment_target"
  printf 'phase-wall-ms preflight %s\n' "$preflight_wall_ms"
  printf 'phase-wall-ms cargo %s\n' "$cargo_wall_ms"
  printf 'phase-wall-ms census %s\n' "$census_wall_ms"
  printf 'phase-wall-ms manifest %s\n' "$manifest_wall_ms"
  printf 'ceremony-roster begin\n'
  for ceremony_id in $CEREMONY_IDS; do
    capture_file=$capture_directory/$suite_short_sha.$ceremony_id.capture
    if [ -f "$capture_file" ]; then
      capture_result=$("$capture_sha256" "$capture_file")
      capture_digest=${capture_result%% *}
    else
      capture_digest=missing
    fi
    printf 'ceremony %s %s\n' "$ceremony_id" "$capture_digest"
  done
  printf 'ceremony-roster end\n'
  printf 'manifest-sha256 %s\n' "$manifest_digest"
  printf 'eligible %s\n' "$eligible"
} > "$capture_directory/RUN-REPORT"
phase_finish

phase_start readonly
chmod -R a-w "$capture_directory"
phase_finish

script_finished_ms=$(now_ms)
printf 'capture directory %s\n' "$capture_directory" >&2
printf 'capture eligible %s\n' "$eligible" >&2
printf 'script total wall-ms %s\n' "$((script_finished_ms - script_started_ms))" >&2

[ "$eligible" = yes ]
