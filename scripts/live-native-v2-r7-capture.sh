#!/bin/sh
# Host-reviewed driver for the one authorized native-v2, revision-7 capture.
#
# Usage: scripts/live-native-v2-r7-capture.sh <new-capture-directory>
#
# The harness owns capture grammar and semantic assertions. This driver owns
# the run boundary: clean source identity, one serialized invocation, the
# fixed ceremony/file census, cross-file run identity, the manifest, and the
# eligibility conclusion. It deliberately has no single-test mode.
set -eu

LC_ALL=C
export LC_ALL

EXPECTED_ELEMENTSD_TIP=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0
EXPECTED_TEST_COUNT=40
EXPECTED_CEREMONY_COUNT=39
EXPECTED_SETUP_COUNT=1

# The run's own address, and the input set it is taken over.
#
# A capture names no commit, no tree and no tag of this repository.
# A name in this history is orphaned by any rewrite of it, and the
# fixture is in the tree anyway, so a coordinate gives its consumer
# nothing the carrying commit does not already give. What names a
# run is what the run was taken over: the label-derived input set
# below, canonically serialized as these lines with one trailing
# newline, digested under a separator spelling the product's name.
# A reader recomputes it over this file with
#
#   { printf 'tripod/run-of-record/address/v1\0'
#     sed -n '/^RUN_INPUT_SET=/,/^RUN_INPUT_SET_EOF$/p' FILE |
#       sed '1d;$d'; } | sha256sum
RUN_ADDRESS_SEPARATOR='tripod/run-of-record/address/v1'
RUN_INPUT_SET=$(cat <<'RUN_INPUT_SET_EOF'
id	kind	class	site	label	recipe	role	value	in_capture	note
development-network-id	network-identity	derive	the recording environment: TRIPOD_LIVE_NETWORK_ID and ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID	tripod-development-network	fixture-seed	network-identity	75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1	yes	the identity of the development network every capture records in deployment-network-id and environment-network-id; drawn as a repeated digit in the recording environment
foreign-test-asset	asset-identifier	derive	packages/target-elements-conformance/src/conservation.rs FOREIGN_TEST_ASSET; scripts/elements-native-executor.py FOREIGN_ASSET_HEX	guide11-conservation-wrong-asset	fixture-seed	foreign-asset	01eec3f3354406f66c623239f95d4c7242d673764307c51653721bc93c3c979a	no	an identifier the chain never issued, so the rows paying it cannot balance; drawn as a repeated byte at both sites, and the rows refuse at construction so it reaches no recorded transaction
wrong-blinder-sum-declaration	blinding-factor	derive	scripts/elements-native-executor.py, the wrong_blinder_sum defect and the wrong_blinding_balance mutation	guide11-conservation-wrong-blinder-sum	fixture-seed	value-blinder	3fbb528bd88b0944b3038ec7f9db6af88c079cf3288d8061ee9eadb7681ba639	no	a blinder the chain does not agree with, declared so the blinding balance fails to close; drawn as a repeated byte, and the rows refuse before submission so it reaches no recorded transaction
sponsor-secret	key	derived	scripts/elements-native-executor.py SPONSOR_SECRET_HEX	tripod/native-executor/sponsor-secret/v1	scalar		f19866b30d937c03aad43a88791e01c2031b5f0aa484770c08a1c54f568ea881	yes	the key every sponsored ceremony authorizes under; already the digest of the label stated beside it, reduced into the group
guide11-fixture-scalars	seed	derived	packages/target-elements-conformance/src/conservation.rs test_scalar, over the labels collision-probe, row2-input and row2-output in the roles asset-blinder, value-blinder, nonce and rangeproof	collision-probe | row2-input | row2-output	fixture-seed	asset-blinder | value-blinder | nonce | rangeproof	per-site	yes	the public test scalars of the conservation matrix; the recipe is stated rather than tabulated so the data is checkable without a golden file
confidential-openings	seed	derived	scripts/elements-native-executor.py confidential_derive and confidential_search_scalar, over the handles of CONFIDENTIAL_FIXTURE_CATALOGUE	ctf-v1/predecessor-dual-parity | ctf-v1/predecessor-triple-noncanceling	derivation-tag	value-blinder | nonce-secret | rangeproof-seed	per-site	yes	every opening of every confidential output, derived from the fixture handle, the output index, the role and a counter searched upward from zero, with exhaustion a refusal rather than a different source
python-golden-byte-identity	pin	retake	scripts/elements-native-executor.py CONFIDENTIAL_DIGEST_V2_BYTE_IDENTITY_DIGEST_HEX	the transcript the same file publishes beside it	fixture-digest-tag		296b23075195a1de37f5dbffb1fc02b46ba7aa29ba51fe33b36ee91fd9f39ca0	no	the digest of the published transcript under the ruled spelling of the fixture-digest separator; the file states the value taken under the retired one
python-golden-recorded-randomness	pin	retake	scripts/elements-native-executor.py CONFIDENTIAL_DIGEST_V2_RECORDED_RANDOMNESS_DIGEST_HEX	the transcript the same file publishes beside it	fixture-digest-tag		06b54728306afe104eb1a681156e966ae648a7a60b2b76e080b43bcd1c979d80	no	the same, for the recorded-randomness contract
nums-internal-key	key	published	scripts/elements-native-executor.py NUMS_INTERNAL_KEY_HEX				50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0	yes	the published nothing-up-my-sleeve point every script-path output is built over; defined by the taproot rules rather than here
anyone-can-spend-program	program	stated	scripts/elements-native-executor.py ANYONE_CAN_SPEND_HEX				51	yes	one opcode, which is what the chain's free coins sit behind; not a value anything derives
chain-policy-asset	asset-identifier	stated	packages/target-elements-conformance/src/conservation.rs CHAIN_POLICY_ASSET				0000000000000000000000000000000000000000000000000000000000000000	no	the all-zero placeholder meaning the chain's own policy asset, which the materializer substitutes with what it observed; it names a role rather than an identifier
genesis-identity	chain-identity	observed	the node's own getblockhash 0, restated by the recording environment as TRIPOD_LIVE_GENESIS_ID and checked against it				209577bda6bf4b5804bd46f8621580dd6d4e8bfa2d190e1c50e932492baca07d	yes	the disposable chain's own genesis; the run reads it back rather than choosing it, and the declaration exists to refuse a run bound to another chain
node-revision	provenance	observed	the node binary's own version line, compared by the driver's binary_revision_matches_expected_tip				b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0	yes	generated into the binary at compile time from the tree it was built from; the run cannot state it and can only report it
RUN_INPUT_SET_EOF
)

# Fixed semantic roster, sorted bytewise. The confidential-predecessor test
# is the fortieth outcome and writes the separate audit-only setup artifact.
CEREMONY_IDS='conservation-negatives
explicit-boundary-values
explicit-maximum-inputs
explicit-maximum-outputs
explicit-merge
explicit-normalization
explicit-one-destination-owner
explicit-one-to-one
explicit-repeated-owner
explicit-self-paid-fee
explicit-several-destination-owners
explicit-several-owners
explicit-several-to-several
explicit-split
explicit-sponsorless
explicit-witness-negatives
keypath-probe
multi-entry-crossing
multi-exit-crossing
multi-many-to-many
multi-one-to-one-with-fee
multi-private-merge
multi-pure-split
multi-several-owners
multi-split
multi-strict-one-to-one
owner-observation
owner-signing-negatives
pairs-arc
private-restart-control
private-restart-parity
proof-bearing-observation
report
sponsored-change-absent
sponsored-change-present
sponsored-committed-value
sponsored-missing-authorization
sponsored-private-explicit-no-change
sponsored-private-with-change'

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
    -v address="$run_address" \
    -v deployment_environment="$observed_deployment_environment" \
    -v deployment_network="$observed_deployment_network" \
    -v deployment_genesis="$observed_deployment_genesis" \
    -v deployment_target="$observed_deployment_target" \
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
      if (NR == 1 && $0 != "native-capture-schema 1") {
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
      check("native-capture-schema", "1")
      check("ceremony-id", ceremony)
      check("run-address", address)
      check("fixture-digest-algorithm", "forward-v2")
      check("handshake-protocol-schema", "7")
      check("deployment-environment", deployment_environment)
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

if [ "$#" -ne 1 ]; then
  die "usage: live-native-v2-r7-capture.sh <new-capture-directory>"
fi

capture_argument=$1

phase_start preflight

for required_tool in awk date find git grep od sort tr wc; do
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

run_address=$({ printf '%s\0' "$RUN_ADDRESS_SEPARATOR"
  printf '%s\n' "$RUN_INPUT_SET"; } | "$capture_sha256" | cut -d' ' -f1)
is_lower_hex_length "$run_address" 64 || die "the run address is not a full lower-case digest"

roster_count=0
previous_id=
for roster_id in $CEREMONY_IDS; do
  roster_count=$((roster_count + 1))
  if [ -n "$previous_id" ] && [ "$previous_id" \> "$roster_id" ]; then
    die "the declared ceremony roster is not sorted"
  fi
  previous_id=$roster_id
done
[ "$roster_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || die "the declared ceremony roster is not 39 IDs"

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
# The development network's identity is a chosen input of the run, so it
# is the digest of a stated label rather than a value an operator draws.
# The input set states it and the run takes it from there; a declaration
# that disagrees is refused rather than preferred.
table_network=$(printf '%s\n' "$RUN_INPUT_SET" |
  awk -F'\t' '$1 == "development-network-id" { print $8 }')
is_lower_hex_length "$table_network" 64 || die "the input set states no development network identity"
[ -z "$live_network" ] || [ "$live_network" = "$table_network" ] ||
  die "the declared harness network identity is not the one the input set states"
[ -z "$executor_network" ] || [ "$executor_network" = "$table_network" ] ||
  die "the declared executor network identity is not the one the input set states"
live_network=$table_network
executor_network=$table_network
export TRIPOD_LIVE_NETWORK_ID="$table_network"
export ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$table_network"
is_lower_hex_length "$live_genesis" 64 || die "TRIPOD_LIVE_GENESIS_ID must be 64 lower-case hex digits"
is_lower_hex_length "$intended_tip" 40 || die "ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP must be a full lower-case tip"

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
TRIPOD_LIVE_RUN_ADDRESS="$run_address" \
  "$capture_cargo" test -p tripod-vectors \
    --test guide13_live_native -- --ignored --test-threads=1 \
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
    found = 1
  }
  END { if (found) print passed, failed, ignored }
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
[ "$observed_test_count" -eq "$EXPECTED_TEST_COUNT" ] || record_error "the observed test census is not 40"
[ "$observed_passed" -eq "$EXPECTED_TEST_COUNT" ] || record_error "the run was not 40 of 40 passed"
[ "$observed_failed" -eq 0 ] || record_error "the run contains failed tests"
[ "$observed_ignored" -eq 0 ] || record_error "the selected run contains ignored outcomes"

expected_tip_hex=$(printf '%s' "$EXPECTED_ELEMENTSD_TIP" | od -An -tx1 | tr -d ' \n')
expected_tip_field="40 $expected_tip_hex"

reference_capture=
for ceremony_id in $CEREMONY_IDS; do
  candidate_capture=$capture_directory/$run_address.$ceremony_id.capture
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
  capture_file=$capture_directory/$run_address.$ceremony_id.capture
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
[ "$physical_capture_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the physical capture census is not 39"
[ "$observed_ceremony_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the unique ceremony census is not 39"
[ "$physical_timing_count" -eq "$EXPECTED_CEREMONY_COUNT" ] || record_error "the timing census is not 39"

setup_file=$capture_directory/$run_address.confidential-predecessor.setup
if [ -f "$setup_file" ]; then
  observed_setup_count=1
  [ -s "$setup_file" ] || record_error "the predecessor setup artifact is empty"
else
  observed_setup_count=0
  record_error "the predecessor setup artifact is missing"
fi

for entry in "$capture_directory"/* "$capture_directory"/.[!.]* "$capture_directory"/..?*; do
  [ -e "$entry" ] || continue
  entry_name=${entry##*/}
  entry_expected=no
  if [ "$entry_name" = "$run_address.confidential-predecessor.setup" ]; then
    entry_expected=yes
  else
    case "$entry_name" in
      "$run_address".*.capture.timing)
        entry_id=${entry_name#"$run_address".}
        entry_id=${entry_id%.capture.timing}
        roster_contains "$entry_id" && entry_expected=yes
        ;;
      "$run_address".*.capture)
        entry_id=${entry_name#"$run_address".}
        entry_id=${entry_id%.capture}
        roster_contains "$entry_id" && entry_expected=yes
        ;;
    esac
  fi

  [ "$entry_expected" = yes ] || record_error "unexpected or noncanonical entry $entry_name"
  if [ -f "$entry" ] && grep -q "$(printf '\r')" "$entry"; then
    record_error "$entry_name contains a CR byte"
  fi
done

[ "$observed_binary_revision" = "$expected_tip_field" ] || record_error "the binary-reported elementsd revision differs from the expected full tip"
[ "$observed_intended_tip" = "$expected_tip_field" ] || record_error "the intended executed elementsd tip differs from the expected full tip"

phase_finish

phase_start manifest

# RUN-REPORT contains the digest of MANIFEST.sha256. Including RUN-REPORT in
# that same manifest would create an unsatisfiable hash cycle. The manifest
# therefore binds the 79 harness artifacts; RUN-REPORT then binds the
# manifest. The directory still has the chartered 81-file eligible census.
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
cargo_argv="$capture_cargo test -p tripod-vectors --test guide13_live_native -- --ignored --test-threads=1"
cargo_argv_hex=$(printf '%s' "$cargo_argv" | od -An -tx1 | tr -d ' \n')

{
  printf 'run-report-schema native-v2-r7-run-report 1\n'
  printf 'capture-format-schema native-v2-r7-capture 1\n'
  printf 'run-address %s\n' "$run_address"
  printf 'suite-clean yes\n'
  printf 'rust-test-target guide13_live_native\n'
  printf 'cargo-argv %s %s\n' "${#cargo_argv}" "$cargo_argv_hex"
  printf 'expected-test-count %s\n' "$EXPECTED_TEST_COUNT"
  printf 'observed-test-count %s\n' "$observed_test_count"
  printf 'expected-ceremony-count %s\n' "$EXPECTED_CEREMONY_COUNT"
  printf 'observed-ceremony-count %s\n' "$observed_ceremony_count"
  printf 'expected-setup-count %s\n' "$EXPECTED_SETUP_COUNT"
  printf 'observed-setup-count %s\n' "$observed_setup_count"
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
  printf 'protocol-revision 7\n'
  printf 'fixture-digest-algorithm forward-v2\n'
  printf 'deployment-environment %s\n' "$observed_deployment_environment"
  printf 'deployment-network-id %s\n' "$observed_deployment_network"
  printf 'deployment-genesis-id %s\n' "$observed_deployment_genesis"
  printf 'deployment-target-contract %s\n' "$observed_deployment_target"
  printf 'ceremony-roster begin\n'
  for ceremony_id in $CEREMONY_IDS; do
    capture_file=$capture_directory/$run_address.$ceremony_id.capture
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
