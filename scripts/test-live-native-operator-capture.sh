#!/bin/sh
# Pure-shell contract for live-native-operator-capture.sh.
#
# Usage: test-live-native-operator-capture.sh <repository-root>
#
# Synthetic captures exercise only the driver's boundary: names, censuses,
# hashes, CR rejection, terminal state, run identities, and timing sidecars.
# Full transcript grammar and semantic request/response validation belong to
# the Rust harness and importer.
set -eu

LC_ALL=C
export LC_ALL

root=${1:?usage: test-live-native-operator-capture.sh <repository-root>}
root=$(cd "$root" && pwd -P)
driver=$root/scripts/live-native-operator-capture.sh

[ -x "$driver" ] || {
  echo "FAIL: capture driver is not executable" >&2
  exit 1
}

[ -n "${TMPDIR:-}" ] || {
  echo "FAIL: TMPDIR must name disk-backed temporary storage" >&2
  exit 1
}

work=$(mktemp -d "$TMPDIR/live-native-operator-capture-contract.XXXXXX")
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
printf '// synthetic native target\n' > "$suite/packages/vectors/tests/maturity_operator.rs"

mock_program=$suite/mock-program
printf '#!/bin/sh\nexit 0\n' > "$mock_program"
chmod 755 "$mock_program"

mock_cargo=$suite/mock-cargo
cat > "$mock_cargo" <<'MOCK_CARGO'
#!/bin/sh
set -eu

if [ "$#" -ne 8 ] ||
   [ "$1" != test ] ||
   [ "$2" != -p ] ||
   [ "$3" != tripod-vectors ] ||
   [ "$4" != --test ] ||
   [ "$5" != maturity_operator ] ||
   [ "$6" != -- ] ||
   [ "$7" != --ignored ] ||
   [ "$8" != --test-threads=1 ]; then
  echo "mock cargo: unexpected argv: $*" >&2
  exit 99
fi

scenario=${MOCK_SCENARIO:-happy}
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
  capture_schema=1
  [ "$scenario" = non-prefix-revision ] && binary_tip=$non_prefix_revision
  [ "$scenario" = short-revision ] && binary_tip=$short_revision
  [ "$scenario" = non-hex-revision ] && binary_tip=$non_hex_revision
  [ "$scenario" = no-diagnostics ] && binary_tip=$expected_tip
  [ "$scenario" = intended-tip-mismatch ] && intended_tip=$wrong_tip
  [ "$scenario" = incomplete-transcript ] && [ "$ceremony_id" = report ] && terminal_state=incomplete
  [ "$scenario" = deployment-mismatch ] && [ "$ceremony_id" = report ] && deployment_environment=staging
  [ "$scenario" = capture-schema ] && [ "$ceremony_id" = report ] && capture_schema=2

  {
    printf 'native-capture-schema %s\n' "$capture_schema"
    printf 'ceremony-id %s\n' "$internal_id"
    text_field rust-test-name the_live_transfer_candidate_runs_against_a_real_target
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
      printf 'legacy-rendering 1 00\n'
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
  grep -qx 'run-report-schema native-operator-run-report 1' "$case_output/RUN-REPORT" || fail "happy path report schema differs"
  grep -qx 'capture-format-schema native-operator-capture 1' "$case_output/RUN-REPORT" || fail "happy path capture schema differs"
  grep -qx 'observed-test-count 1' "$case_output/RUN-REPORT" || fail "happy path test census differs"
  grep -qx 'observed-ceremony-count 1' "$case_output/RUN-REPORT" || fail "happy path ceremony census differs"
  grep -qx 'observed-setup-count 0' "$case_output/RUN-REPORT" || fail "happy path setup census differs"
  grep -qx 'cargo-exit-code 0' "$case_output/RUN-REPORT" || fail "happy path Cargo exit differs"
  grep -qx 'protocol-revision 8' "$case_output/RUN-REPORT" || fail "happy path protocol differs"
  grep -qx 'fixture-digest-algorithm forward-v2' "$case_output/RUN-REPORT" || fail "happy path digest algorithm differs"
  grep -qx "elementsd-expected-tip $expected_tip" "$case_output/RUN-REPORT" || fail "happy path expected tip differs"
  grep -qx 'diagnostics-present yes' "$case_output/RUN-REPORT" || fail "happy path diagnostics presence differs"
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
grep -qx 'rust-test-target maturity_operator' "$case_output/RUN-REPORT" || fail "test target differs"
expected_argv="$mock_cargo test -p tripod-vectors --test maturity_operator -- --ignored --test-threads=1"
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

run_case no-diagnostics no-diagnostics no
[ "$case_status" -eq 0 ] || fail "no-diagnostics path exited $case_status"
[ -f "$case_output/RUN-REPORT" ] || fail "no-diagnostics path did not emit RUN-REPORT"
if [ -f "$case_output/RUN-REPORT" ]; then
  grep -qx 'eligible yes' "$case_output/RUN-REPORT" || fail "no-diagnostics path is not eligible"
  grep -qx 'diagnostics-present no' "$case_output/RUN-REPORT" || fail "no-diagnostics presence differs"
fi

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

echo 'test result: native operator capture contract: all cases passed' >&2
