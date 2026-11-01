#!/bin/sh
# Pure-shell contract for live-native-maturity-retained-capture.sh.
#
# Usage: test-live-native-maturity-retained-capture.sh <repository-root>
#
# Synthetic captures exercise only the driver's boundary: names, censuses,
# hashes, CR rejection, terminal state, run identities, and timing sidecars.
# Full transcript grammar and semantic request/response validation belong to
# the Rust harness and importer.
set -eu

LC_ALL=C
export LC_ALL

root=${1:-.}
root=$(cd "$root" && pwd -P)
driver=$root/scripts/live-native-maturity-retained-capture.sh

[ -x "$driver" ] || {
  echo "FAIL: capture driver is not executable" >&2
  exit 1
}

[ -n "${TMPDIR:-}" ] || {
  echo "FAIL: TMPDIR must name disk-backed temporary storage" >&2
  exit 1
}

work=$(mktemp -d "$TMPDIR/live-native-maturity-retained-capture-contract.XXXXXX")
cleanup() {
  chmod -R u+w "$work" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT HUP INT TERM

suite=$work/suite
results=$work/results
mock_tmp=$work/tmp
mkdir -p "$suite/packages/vectors/tests" "$suite/framework" "$results" "$mock_tmp"

printf '[workspace]\nmembers = []\n' > "$suite/Cargo.toml"
printf '// synthetic native target\n' > "$suite/packages/vectors/tests/maturity_retained.rs"

mock_program=$suite/mock-program
printf '#!/bin/sh\nexit 0\n' > "$mock_program"
chmod 755 "$mock_program"

mock_cargo=$suite/mock-cargo
cat > "$mock_cargo" <<'MOCK_CARGO'
#!/bin/sh
set -eu

if [ "$#" -ne 10 ] ||
   [ "$1" != test ] ||
   [ "$2" != -p ] ||
   [ "$3" != tripod-vectors ] ||
   [ "$4" != --test ] ||
   [ "$5" != maturity_retained ] ||
   [ "$6" != -- ] ||
   [ "$7" != --ignored ] ||
   [ "$8" != --exact ] ||
   [ "$9" != the_retained_maturity_routes_run_against_a_real_target ] ||
   [ "${10}" != --test-threads=1 ]; then
  echo "mock cargo: unexpected argv: $*" >&2
  exit 99
fi

scenario=${MOCK_SCENARIO:-happy}
[ "${TRIPOD_MATURITY_CAPTURE_PROFILE:-}" = retained ] || exit 98
expected_tip=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0
wrong_tip=0000000000000000000000000000000000000000
binary_tip_prefix=b7fc5d080a7e
non_prefix_revision=000000000000
short_revision=b7fc5d080a7
non_hex_revision=b7fc5d080a7g

ceremony_ids='report'

hex_text() {
  printf '%s' "$1" | od -An -tx1 | tr -d ' \n'
}

text_field() {
  text_key=$1
  text_value=$2
  printf '%s %s %s\n' "$text_key" "${#text_value}" "$(hex_text "$text_value")"
}

for ceremony_id in $ceremony_ids; do
  if [ "$scenario" = missing-ceremony ] && [ "$ceremony_id" = report ]; then
    continue
  fi

  internal_id=$ceremony_id
  [ "$scenario" = unknown-ceremony ] && internal_id=unknown

  capture_name=$TRIPOD_LIVE_SUITE_SHORT_SHA.$ceremony_id.capture
  [ "$scenario" = noncanonical-name ] && [ "$ceremony_id" = report ] &&
    capture_name=$TRIPOD_LIVE_SUITE_SHORT_SHA.Report.capture
  capture_file=$TRIPOD_LIVE_REPORT_DIR/$capture_name

  binary_tip=$binary_tip_prefix
  intended_tip=$expected_tip
  terminal_state=complete
  deployment_environment=development
  capture_schema=2
  [ "$scenario" = non-prefix-revision ] && binary_tip=$non_prefix_revision
  [ "$scenario" = short-revision ] && binary_tip=$short_revision
  [ "$scenario" = non-hex-revision ] && binary_tip=$non_hex_revision
  [ "$scenario" = no-diagnostics ] && binary_tip=$expected_tip
  [ "$scenario" = intended-tip-mismatch ] && intended_tip=$wrong_tip
  [ "$scenario" = incomplete-transcript ] && [ "$ceremony_id" = report ] && terminal_state=incomplete
  [ "$scenario" = deployment-mismatch ] && [ "$ceremony_id" = report ] && deployment_environment=staging
  [ "$scenario" = capture-schema ] && [ "$ceremony_id" = report ] && capture_schema=1

  {
    printf 'native-capture-schema %s\n' "$capture_schema"
    printf 'ceremony-id %s\n' "$internal_id"
    text_field rust-test-name the_retained_maturity_routes_run_against_a_real_target
    printf 'suite-commit %s\n' "$TRIPOD_LIVE_SUITE_COMMIT"
    printf 'suite-tree %s\n' "$TRIPOD_LIVE_SUITE_TREE"
    printf 'fixture-digest-algorithm forward-v2\n'
    printf 'deployment-environment %s\n' "$deployment_environment"
    text_field deployment-network-id "$TRIPOD_LIVE_NETWORK_ID"
    text_field deployment-genesis-id "$TRIPOD_LIVE_GENESIS_ID"
    text_field deployment-target-contract elements-tapscript-v2
    printf 'handshake-protocol-schema 8\n'
    printf 'environment-schema 8\n'
    text_field environment-network-id "$TRIPOD_LIVE_NETWORK_ID"
    text_field environment-genesis-id "$TRIPOD_LIVE_GENESIS_ID"
    text_field handshake-adapter-name mock-adapter
    text_field handshake-adapter-version 1.0.0
    text_field handshake-node-name elementsd
    text_field handshake-node-version 23.2.1
    text_field handshake-binary-reported-revision "$binary_tip"
    text_field handshake-intended-executed-tip "$intended_tip"
    if [ "$scenario" = cr-byte ] && [ "$ceremony_id" = report ]; then
      printf 'legacy-rendering 1 00\r\n'
    else
      policy='{"block":"0.00000100","block_identifier":"DEFAULT_BLOCK_MIN_TX_FEE","block_provenance":"declared by the pinned source, no override","mempool":"0.00000100","relay":"0.00000100","relay_provenance":"reported by the node","start_arguments":["elementsd","-daemon=0"],"unit":"BTC/kvB"}'
      [ "$scenario" = wrong-policy ] && policy='{}'
      control='{"layer":"accepted","readback":"matched","route":"direct-block","step":"control"}'
      semantic='{"layer":"script_path_rejection","readback":"not-applicable","route":"refused","step":"semantic"}'
      successor='{"layer":"accepted","readback":"matched","route":"direct-block","step":"successor"}'
      expectation1=agreement
      expectation2=agreement
      stopped='[]'
      stopped1=false
      accepted1=true
      case "$scenario" in
        semantic-acceptance|both-contradictions) semantic=$successor; expectation1=contradiction ;;
      esac
      case "$scenario" in
        successor-refusal|both-contradictions) successor='{"layer":"script_path_rejection","readback":"not-applicable","route":"refused","step":"successor"}'; expectation2=contradiction ;;
        other-boundary) successor='{"layer":"consensus_rejection","readback":"not-applicable","route":"refused","step":"successor"}'; expectation2=contradiction ;;
      esac
      if [ "$scenario" = stopped-control ]; then
        control=$semantic
        stopped='["wrong-semantic-metadata"]'
        stopped1=true
        accepted1=false
      fi
      families=$(printf '[{"claim":{"section":"16.6","name":"successor-from-wrong-semantic-metadata"},"control":%s,"control_accepted":%s,"expectation":"%s","family":"wrong-semantic-metadata","stopped":%s,"subject":%s},{"claim":{"section":"16.6","name":"two-state-successors"},"control":%s,"control_accepted":true,"expectation":"%s","family":"two-state-successors","stopped":false,"subject":%s}]' "$control" "$accepted1" "$expectation1" "$stopped1" "$semantic" "$control" "$expectation2" "$successor")
      row1='{"claim":{"section":"16.6","name":"successor-from-wrong-semantic-metadata"},"declared_boundary":"script_path_rejection","expected_consumer":"native-evidence-fold","family":"wrong-semantic-metadata","operation":"announce_maturity"}'
      row2='{"claim":{"section":"16.6","name":"two-state-successors"},"declared_boundary":"script_path_rejection","expected_consumer":"native-evidence-fold","family":"two-state-successors","operation":"announce_maturity"}'
      [ "$scenario" = malformed-families ] && families='['
      [ "$scenario" = wrong-families ] && families='[]'
      [ "$scenario" = wrong-stopped ] && stopped='["foreign"]'
      [ "$scenario" = wrong-row ] && row2=$row1
      [ "$scenario" = malformed-row ] && row2='{'
      [ "$scenario" = override-policy ] && policy=$(printf '%s' "$policy" | sed 's/elementsd/elementsd -minrelaytxfee=0/')
      payload=$(
        printf 'maturity-retained-evidence-schema 1\n'
        text_field retained-policy "$policy"
        text_field retained-families "$families"
        text_field retained-stopped "$stopped"
        printf 'campaign-manifest-schema 1\ncampaign-manifest-count 2\n'
        text_field campaign-row "$row1"
        text_field campaign-row "$row2"
      )
      [ "$scenario" = missing-families ] && payload=$(printf '%s\n' "$payload" | sed '/^retained-families /d')
      [ "$scenario" = missing-policy ] && payload=$(printf '%s\n' "$payload" | sed '/^retained-policy /d')
      [ "$scenario" = wrong-campaign-schema ] && payload=$(printf '%s\n' "$payload" | sed 's/^campaign-manifest-schema 1$/campaign-manifest-schema 2/')
      [ "$scenario" = wrong-campaign-count ] && payload=$(printf '%s\n' "$payload" | sed 's/^campaign-manifest-count 2$/campaign-manifest-count 3/')
      [ "$scenario" = missing-stopped ] && payload=$(printf '%s\n' "$payload" | sed '/^retained-stopped /d')
      [ "$scenario" = missing-row ] && payload=$(printf '%s\n' "$payload" | sed '$d')
      if [ "$scenario" = duplicate-families ]; then
        payload=$(printf '%s\n' "$payload"; text_field retained-families "$families")
      fi
      if [ "$scenario" = duplicate-policy ]; then
        payload=$(printf '%s\n' "$payload"; text_field retained-policy "$policy")
      fi
      if [ "$scenario" = third-row ]; then
        payload=$(printf '%s\n' "$payload"; text_field campaign-row "$row2")
      fi
      text_field legacy-rendering "$payload"
    fi
    printf 'terminal-state %s\n' "$terminal_state"
    printf 'capture-content-sha256 %s\n' 1111111111111111111111111111111111111111111111111111111111111111
    printf 'native-capture-end %s\n' "$internal_id"
  } > "$capture_file"

  case "$scenario" in
    duplicate-ceremony) sed -i '/^ceremony-id /a ceremony-id report' "$capture_file" ;;
    suite-commit) sed -i "s/^suite-commit .*/suite-commit $wrong_tip/" "$capture_file" ;;
    suite-tree) sed -i "s/^suite-tree .*/suite-tree $wrong_tip/" "$capture_file" ;;
    protocol) sed -i 's/^handshake-protocol-schema 8$/handshake-protocol-schema 7/' "$capture_file" ;;
    environment-schema) sed -i 's/^environment-schema 8$/environment-schema 7/' "$capture_file" ;;
    deployment-network|deployment-genesis|environment-network|environment-genesis)
      sed -i "s/^$scenario-id .*/$scenario-id 1 00/" "$capture_file" ;;
    deployment-target) sed -i 's/^deployment-target-contract .*/deployment-target-contract 1 00/' "$capture_file" ;;
    rust-test) sed -i 's/^rust-test-name .*/rust-test-name 4 74657374/' "$capture_file" ;;
    missing-adapter) sed -i '/^handshake-adapter-name /d' "$capture_file" ;;
    end-marker) sed -i 's/^native-capture-end report$/native-capture-end unknown/' "$capture_file" ;;
    trailing-content) printf 'trailing content\n' >> "$capture_file" ;;
  esac

  if [ "$scenario" = missing-timing ] && [ "$ceremony_id" = report ]; then
    continue
  fi

  timing_status=passed
  case "$scenario" in
    timing-incomplete) timing_status=incomplete ;;
    timing-failed) timing_status=failed-after-transcript ;;
    timing-unknown) timing_status=unknown ;;
  esac
  [ "$scenario" = cargo-failure ] && [ "$ceremony_id" = report ] &&
    timing_status=failed-after-transcript
  {
    printf 'timing-schema 1\n'
    printf 'ceremony-id %s\n' "$ceremony_id"
    printf 'status %s\n' "$timing_status"
  } > "$TRIPOD_LIVE_REPORT_DIR/$TRIPOD_LIVE_SUITE_SHORT_SHA.$ceremony_id.capture.timing"
  timing_file=$TRIPOD_LIVE_REPORT_DIR/$TRIPOD_LIVE_SUITE_SHORT_SHA.$ceremony_id.capture.timing
  case "$scenario" in
    timing-schema) sed -i 's/^timing-schema 1$/timing-schema 2/' "$timing_file" ;;
    timing-ceremony) sed -i 's/^ceremony-id report$/ceremony-id unknown/' "$timing_file" ;;
    timing-duplicate) printf 'status passed\n' >> "$timing_file" ;;
    timing-cr) printf '\r\n' >> "$timing_file" ;;
  esac
done

if [ "$scenario" = unexpected-setup ]; then
  printf 'audit-only confidential predecessor setup\n' \
    > "$TRIPOD_LIVE_REPORT_DIR/$TRIPOD_LIVE_SUITE_SHORT_SHA.confidential-predecessor.setup"
fi

[ "$scenario" = unexpected-file ] && printf 'unexpected\n' > "$TRIPOD_LIVE_REPORT_DIR/unexpected"

if [ "$scenario" != no-diagnostics ]; then
  mkdir -p "$TRIPOD_LIVE_REPORT_DIR/diagnostics"
  printf 'audit-only executor diagnostics\n' > "$TRIPOD_LIVE_REPORT_DIR/diagnostics/executor-diagnostics.txt"
fi
if [ "$scenario" = second-unexpected-directory ]; then
  mkdir -p "$TRIPOD_LIVE_REPORT_DIR/other-diagnostics/report"
  printf 'unexpected directory\n' > "$TRIPOD_LIVE_REPORT_DIR/other-diagnostics/report/stderr"
fi

case "$scenario" in
  test-count)
    echo 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    ;;
  ignored-outcome)
    echo 'test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    ;;
  duplicate-summary)
    printf '%s\n' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    ;;
  cargo-failure)
    echo 'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    exit 7
    ;;
  *)
    echo 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    ;;
esac
MOCK_CARGO
chmod 755 "$mock_cargo"

real_sha256sum=$(command -v sha256sum)
mock_sha256sum=$suite/mock-sha256sum
cat > "$mock_sha256sum" <<'MOCK_SHA'
#!/bin/sh
set -eu
if [ "${MOCK_MANIFEST_VERIFY_FAIL:-no}" = yes ] && [ "${1:-}" = -c ]; then
  echo 'mock sha256sum: forced manifest verification failure' >&2
  exit 9
fi
exec "$REAL_SHA256SUM" "$@"
MOCK_SHA
chmod 755 "$mock_sha256sum"

git -C "$suite" init -q
git -C "$suite" config user.name 'Capture Contract'
git -C "$suite" config user.email capture-contract.invalid
git -C "$suite" config commit.gpgsign false
git -C "$suite" add .
git -C "$suite" commit -qm 'test: seed synthetic suite'

network_id=1111111111111111111111111111111111111111111111111111111111111111
genesis_id=2222222222222222222222222222222222222222222222222222222222222222
expected_tip=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0

export TMPDIR=$mock_tmp
export TRIPOD_CAPTURE_CARGO=$mock_cargo
export TRIPOD_CAPTURE_SHA256SUM=$real_sha256sum
export TRIPOD_LIVE_EXECUTOR=$mock_program
export TRIPOD_LIVE_NETWORK_ID=$network_id
export TRIPOD_LIVE_GENESIS_ID=$genesis_id
export ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=$mock_program
export ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=$mock_program
export ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=$suite/framework
export ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID=$network_id
export ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP=$expected_tip
export ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1
export ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults
export ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1
export ELEMENTS_NATIVE_EXECUTOR_PYTHON=$mock_program
export ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=$suite/mock-library
export TRIPOD_LIVE_SUITE_COMMIT=stale-commit
export TRIPOD_LIVE_SUITE_TREE=stale-tree
export TRIPOD_LIVE_SUITE_SHORT_SHA=stale-short-id
export TRIPOD_LIVE_REPORT_DIR=$results/stale-report-directory
export REAL_SHA256SUM=$real_sha256sum

failures=0

fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}

run_case() {
  case_name=$1
  case_scenario=$2
  case_manifest_failure=$3
  case_output=$results/$case_name
  case_stdout=$results/$case_name.stdout
  case_stderr=$results/$case_name.stderr
  case_sha256=$real_sha256sum
  [ "$case_manifest_failure" = yes ] && case_sha256=$mock_sha256sum
  if (
    cd "$suite"
    MOCK_SCENARIO=$case_scenario \
    MOCK_MANIFEST_VERIFY_FAIL=$case_manifest_failure \
    TRIPOD_CAPTURE_SHA256SUM=$case_sha256 \
      "$driver" "$case_output"
  ) > "$case_stdout" 2> "$case_stderr"; then
    case_status=0
  else
    case_status=$?
  fi
}

assert_ineligible_case() {
  ineligible_name=$1
  ineligible_scenario=$2
  refusal_pattern=$3
  run_case "$ineligible_name" "$ineligible_scenario" no
  [ "$case_status" -ne 0 ] || fail "$ineligible_name unexpectedly exited zero"
  [ -f "$case_output/RUN-REPORT" ] || fail "$ineligible_name did not emit RUN-REPORT"
  if [ -f "$case_output/RUN-REPORT" ]; then
    grep -qx 'eligible no' "$case_output/RUN-REPORT" || fail "$ineligible_name did not report eligible no"
  fi
  grep -q "$refusal_pattern" "$case_stderr" || fail "$ineligible_name did not report its refusal"
}

run_case happy happy no
if [ "$case_status" -ne 0 ]; then
  sed 's/^/happy driver: /' "$case_stderr" >&2
  fail "happy path exited $case_status"
fi
[ -f "$case_output/RUN-REPORT" ] || fail "happy path did not emit RUN-REPORT"
if [ -f "$case_output/RUN-REPORT" ]; then
  grep -qx 'eligible yes' "$case_output/RUN-REPORT" || fail "happy path is not eligible"
  grep -qx 'run-report-schema native-maturity-run-report 1' "$case_output/RUN-REPORT" || fail "happy path report schema differs"
  grep -qx 'capture-format-schema native-maturity-capture 1' "$case_output/RUN-REPORT" || fail "happy path capture schema differs"
  grep -qx 'observed-test-count 1' "$case_output/RUN-REPORT" || fail "happy path test census differs"
  grep -qx 'observed-ceremony-count 1' "$case_output/RUN-REPORT" || fail "happy path ceremony census differs"
  grep -qx 'observed-setup-count 0' "$case_output/RUN-REPORT" || fail "happy path setup census differs"
  grep -qx 'cargo-exit-code 0' "$case_output/RUN-REPORT" || fail "happy path Cargo exit differs"
  grep -qx 'protocol-revision 8' "$case_output/RUN-REPORT" || fail "happy path protocol differs"
  grep -qx 'fixture-digest-algorithm forward-v2' "$case_output/RUN-REPORT" || fail "happy path digest algorithm differs"
  grep -qx "elementsd-expected-tip $expected_tip" "$case_output/RUN-REPORT" || fail "happy path expected tip differs"
  grep -qx 'diagnostics-present yes' "$case_output/RUN-REPORT" || fail "happy path diagnostics presence differs"
  awk '
    /^capture-profile retained$/ { profile++; start = NR; next }
    /^phase-wall-ms / { phases++ }
    /^elapsed-wall-ms / { elapsed++ }
    start && NR == start + 1 && /^phase-wall-ms preflight [0-9]+$/ { matched++ }
    start && NR == start + 2 && /^phase-wall-ms cargo [0-9]+$/ { matched++ }
    start && NR == start + 3 && /^phase-wall-ms census [0-9]+$/ { matched++ }
    start && NR == start + 4 && /^phase-wall-ms manifest [0-9]+$/ { matched++ }
    start && NR == start + 5 && /^elapsed-wall-ms [0-9]+$/ { matched++ }
    END { exit !(profile == 1 && phases == 4 && elapsed == 1 && matched == 5) }
  ' "$case_output/RUN-REPORT" || fail "happy path report duration order or grammar differs"
fi

happy_files=$(find "$case_output" -mindepth 1 -maxdepth 1 -type f | wc -l | tr -d ' ')
[ "$happy_files" -eq 4 ] || fail "happy path has $happy_files files, expected 4"
happy_directories=$(find "$case_output" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
[ "$happy_directories" -eq 1 ] || fail "happy path has $happy_directories top-level directories, expected diagnostics only"
manifest_lines=$(wc -l < "$case_output/MANIFEST.sha256" | tr -d ' ')
[ "$manifest_lines" -eq 2 ] || fail "manifest has $manifest_lines entries, expected 2 harness artifacts"
if ! (cd "$case_output" && "$real_sha256sum" -c MANIFEST.sha256 >/dev/null); then
  fail "happy path manifest does not independently verify"
fi
if grep -q 'RUN-REPORT' "$case_output/MANIFEST.sha256"; then
  fail "manifest creates a cycle by naming RUN-REPORT"
fi
if grep -q 'diagnostics' "$case_output/MANIFEST.sha256"; then
  fail "manifest includes audit-only diagnostics"
fi
suite_commit=$(git -C "$suite" rev-parse --verify HEAD)
suite_tree=$(git -C "$suite" rev-parse --verify 'HEAD^{tree}')
suite_short=$(git -C "$suite" rev-parse --short HEAD)
capture_name=$suite_short.report.capture
[ -f "$case_output/$capture_name" ] || fail "canonical capture is missing"
[ -f "$case_output/$capture_name.timing" ] || fail "canonical timing sidecar is missing"
expected_manifest_names=$(printf '%s\n' "$capture_name" "$capture_name.timing")
observed_manifest_names=$(awk '{print $2}' "$case_output/MANIFEST.sha256")
[ "$observed_manifest_names" = "$expected_manifest_names" ] || fail "manifest names or sort order differ"
capture_hash=$("$real_sha256sum" "$case_output/$capture_name")
capture_hash=${capture_hash%% *}
manifest_hash=$("$real_sha256sum" "$case_output/MANIFEST.sha256")
manifest_hash=${manifest_hash%% *}
grep -qx "ceremony report $capture_hash" "$case_output/RUN-REPORT" || fail "ceremony digest differs"
[ "$(grep -c '^ceremony ' "$case_output/RUN-REPORT")" -eq 1 ] || fail "ceremony digest roster is not singular"
grep -qx "manifest-sha256 $manifest_hash" "$case_output/RUN-REPORT" || fail "manifest digest differs"
grep -qx "suite-commit $suite_commit" "$case_output/RUN-REPORT" || fail "suite commit was not derived"
grep -qx "suite-tree $suite_tree" "$case_output/RUN-REPORT" || fail "suite tree was not derived"
grep -qx 'rust-test-target maturity_retained' "$case_output/RUN-REPORT" || fail "test target differs"
test_name=the_retained_maturity_routes_run_against_a_real_target
test_name_hex=$(printf '%s' "$test_name" | od -An -tx1 | tr -d ' \n')
grep -qx "rust-test-name ${#test_name} $test_name_hex" "$case_output/RUN-REPORT" || fail "report test name differs"
awk '
  /^capture-profile retained$/ { start=NR }
  start && NR == start+6 && /^retained-policy [0-9]+ [0-9a-f]+$/ { matched++ }
  start && NR == start+7 && /^retained-families [0-9]+ [0-9a-f]+$/ { matched++ }
  start && NR == start+8 && /^retained-stopped [0-9]+ [0-9a-f]+$/ { matched++ }
  start && NR == start+9 && /^campaign-manifest-schema 1$/ { matched++ }
  start && NR == start+10 && /^campaign-manifest-count 2$/ { matched++ }
  start && (NR == start+11 || NR == start+12) && /^campaign-row [0-9]+ [0-9a-f]+$/ { matched++ }
  start && NR == start+13 && /^ceremony-roster begin$/ { matched++ }
  start && NR == start+14 && /^ceremony report [0-9a-f]+$/ { matched++ }
  start && NR == start+15 && /^ceremony-roster end$/ { matched++ }
  start && NR == start+16 && /^manifest-sha256 [0-9a-f]+$/ { matched++ }
  start && NR == start+17 && /^eligible yes$/ { matched++ }
  END {exit !(matched == 12 && NR == start+17)}
' "$case_output/RUN-REPORT" || fail "retained tail grammar differs"
expected_argv="$mock_cargo test -p tripod-vectors --test maturity_retained -- --ignored --exact the_retained_maturity_routes_run_against_a_real_target --test-threads=1"
expected_argv_hex=$(printf '%s' "$expected_argv" | od -An -tx1 | tr -d ' \n')
grep -qx "cargo-argv ${#expected_argv} $expected_argv_hex" "$case_output/RUN-REPORT" || fail "recorded Cargo argv differs"
for field in 'expected-test-count 1' 'expected-ceremony-count 1' 'expected-setup-count 0' \
             'test-passed-count 1' 'test-failed-count 0' 'test-ignored-count 0' 'suite-clean yes'; do
  grep -qx "$field" "$case_output/RUN-REPORT" || fail "happy path field differs: $field"
done
[ ! -w "$case_output/$capture_name" ] || fail "happy path capture remains writable"
[ ! -w "$case_output/$capture_name.timing" ] || fail "happy path timing remains writable"
[ ! -w "$case_output/MANIFEST.sha256" ] || fail "happy path manifest remains writable"
[ ! -w "$case_output/RUN-REPORT" ] || fail "happy path files remain writable"
[ ! -w "$case_output" ] || fail "happy path directory remains writable"
for phase in preflight cargo census manifest report readonly; do
  grep -q "phase $phase wall-ms" "$case_stderr" || fail "happy path omitted $phase timing"
done

assert_ineligible_case wrong-policy wrong-policy 'missing retained policy premise'
assert_ineligible_case missing-policy missing-policy 'invalid retained policy encoding'
assert_ineligible_case duplicate-policy duplicate-policy 'invalid retained policy encoding'
assert_ineligible_case wrong-campaign-schema wrong-campaign-schema 'invalid campaign schema'
assert_ineligible_case wrong-campaign-count wrong-campaign-count 'invalid campaign census'
assert_ineligible_case third-row third-row 'invalid campaign row count'
assert_ineligible_case malformed-row malformed-row 'invalid campaign row'
assert_ineligible_case override-policy override-policy 'default policy has an override'
assert_ineligible_case wrong-families wrong-families 'invalid retained families'
assert_ineligible_case malformed-families malformed-families 'invalid retained families'
assert_ineligible_case missing-families missing-families 'invalid retained families encoding'
assert_ineligible_case duplicate-families duplicate-families 'invalid retained families encoding'
assert_ineligible_case missing-stopped missing-stopped 'invalid retained stopped encoding'
assert_ineligible_case wrong-stopped wrong-stopped 'invalid retained stopped'
assert_ineligible_case missing-row missing-row 'invalid campaign row count'
assert_ineligible_case wrong-row wrong-row 'invalid campaign row'
for diagnostic in semantic-acceptance successor-refusal both-contradictions other-boundary stopped-control; do
  run_case "$diagnostic" "$diagnostic" no
  [ "$case_status" -eq 0 ] || fail "$diagnostic prevented a structurally complete capture"
  grep -qx 'eligible yes' "$case_output/RUN-REPORT" || fail "$diagnostic lost structural eligibility"
done
mkdir "$results/with-log"
printf 'audit log\n' > "$results/with-log/driver.log"
run_case with-log happy no
[ "$case_status" -eq 0 ] || fail "audit log prevented capture"
grep -q 'driver.log' "$case_output/MANIFEST.sha256" && fail "audit log entered manifest"
grep -qx 'capture-profile retained' "$case_output/RUN-REPORT" || fail "profile absent"
mkdir "$results/link-log"
ln -s "$results/happy.stdout" "$results/link-log/driver.log"
run_case link-log happy no
[ "$case_status" -ne 0 ] || fail "symlink log accepted"

assert_ineligible_case no-diagnostics no-diagnostics 'missing diagnostics directory'

assert_ineligible_case missing-ceremony missing-ceremony 'missing capture for report'
assert_ineligible_case duplicate-ceremony duplicate-ceremony 'ceremony-id is missing or duplicated'
assert_ineligible_case missing-timing missing-timing 'missing timing sidecar for report'
assert_ineligible_case unexpected-setup unexpected-setup 'setup census is not 0'
assert_ineligible_case unexpected-file unexpected-file 'unexpected or noncanonical entry unexpected'
assert_ineligible_case cr-byte cr-byte 'contains a CR byte'
assert_ineligible_case noncanonical-name noncanonical-name 'unexpected or noncanonical entry'
assert_ineligible_case incomplete-transcript incomplete-transcript 'terminal-state differs'
assert_ineligible_case test-count test-count 'observed test census is not 1'
assert_ineligible_case non-prefix-revision non-prefix-revision 'binary-reported elementsd revision'
assert_ineligible_case short-revision short-revision 'binary-reported elementsd revision'
assert_ineligible_case non-hex-revision non-hex-revision 'binary-reported elementsd revision'
assert_ineligible_case intended-tip-mismatch intended-tip-mismatch 'intended executed elementsd tip differs'
assert_ineligible_case deployment-mismatch deployment-mismatch 'deployment-environment differs'
assert_ineligible_case capture-schema capture-schema 'native-capture-schema differs'
assert_ineligible_case second-unexpected-directory second-unexpected-directory 'unexpected or noncanonical entry other-diagnostics'

while IFS='|' read -r scenario refusal; do
  assert_ineligible_case "$scenario" "$scenario" "$refusal"
done <<'REFUSALS'
unknown-ceremony|unknown ceremony ID
suite-commit|suite-commit differs
suite-tree|suite-tree differs
protocol|handshake-protocol-schema differs
environment-schema|environment-schema differs
deployment-network|deployment-network-id differs
deployment-genesis|deployment-genesis-id differs
environment-network|environment-network-id differs
environment-genesis|environment-genesis-id differs
deployment-target|deployment-target-contract differs
rust-test|rust-test-name differs
missing-adapter|handshake-adapter-name is missing or duplicated
end-marker|native-capture-end differs
trailing-content|canonical end marker differs
timing-incomplete|timing status is incomplete
timing-failed|timing status is failed-after-transcript
timing-unknown|timing status is unknown
timing-schema|timing-schema differs
timing-ceremony|ceremony-id differs
timing-duplicate|status is missing or duplicated
timing-cr|contains a CR byte
ignored-outcome|selected run contains ignored outcomes
duplicate-summary|Cargo test census is missing
REFUSALS

run_case cargo-failure cargo-failure no
[ "$case_status" -ne 0 ] || fail "Cargo failure unexpectedly exited zero"
[ -f "$case_output/RUN-REPORT" ] || fail "Cargo failure did not emit RUN-REPORT"
if [ -f "$case_output/RUN-REPORT" ]; then
  grep -qx 'eligible no' "$case_output/RUN-REPORT" || fail "Cargo failure did not report eligible no"
  grep -qx 'cargo-exit-code 7' "$case_output/RUN-REPORT" || fail "Cargo failure exit was not retained"
fi

run_case manifest-verification happy yes
[ "$case_status" -ne 0 ] || fail "manifest verification failure unexpectedly exited zero"
[ -f "$case_output/RUN-REPORT" ] || fail "manifest verification failure did not emit RUN-REPORT"
if [ -f "$case_output/RUN-REPORT" ]; then
  grep -qx 'eligible no' "$case_output/RUN-REPORT" || fail "manifest verification failure did not report eligible no"
fi
grep -q 'MANIFEST.sha256 did not verify' "$case_stderr" || fail "manifest verification failure was not reported"

dirty_output=$results/dirty-tree
printf '# dirty\n' >> "$suite/Cargo.toml"
if (cd "$suite" && "$driver" "$dirty_output") > "$results/dirty.stdout" 2> "$results/dirty.stderr"; then
  fail "dirty tree unexpectedly exited zero"
fi
[ ! -e "$dirty_output" ] || fail "dirty tree created a capture directory"
grep -q 'suite tree is dirty' "$results/dirty.stderr" || fail "dirty tree refusal was not reported"
git -C "$suite" restore Cargo.toml

nonempty_output=$results/nonempty
mkdir "$nonempty_output"
printf 'sentinel\n' > "$nonempty_output/sentinel"
if (cd "$suite" && "$driver" "$nonempty_output") > "$results/nonempty.stdout" 2> "$results/nonempty.stderr"; then
  fail "non-empty directory unexpectedly exited zero"
fi
grep -q 'capture directory is non-empty' "$results/nonempty.stderr" || fail "non-empty directory refusal was not reported"
[ ! -e "$nonempty_output/RUN-REPORT" ] || fail "preflight refusal overwrote the non-empty directory"

if [ "$failures" -ne 0 ]; then
  echo "live-native capture contract: $failures failure(s)" >&2
  exit 1
fi

echo 'test result: native maturity capture contract: all cases passed' >&2
