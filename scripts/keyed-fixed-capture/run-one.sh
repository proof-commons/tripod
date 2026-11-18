#!/usr/bin/env bash
# Reviewed in-container entry. No native target runs as part of an ordinary gate.
set -euo pipefail
started=$SECONDS
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} != 1 ]]; then
    export PATH=/usr/local/bin:/usr/bin:/bin
fi
trap 'status=$?; printf "run-one wall=%ss exit=%s\n" "$((SECONDS-started))" "$status"; exit "$status"' EXIT
target=${1:?target required}
case "$target" in pilot|clauses|policy|scaling|histories) ;; *) exit 64;; esac
test_name="native_${target//-/_}"
capture=/capture
binary=/capture-build/keyed_fixed_capture
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} == 1 ]]; then
    [[ ! -e /capture-build ]] || exit 65
    capture=${TRIPOD_CAPTURE_TEST_ROOT:?stand-in root required}
    binary="$capture/keyed_fixed_capture"
fi
mkdir -p "$capture/tmp" "$capture/work"
export TRIPOD_FIXED_CAPTURE_DIR="$capture"
export TRIPOD_FIXED_FEE_POLICY=node-defaults
cat /proc/self/cgroup >"$capture/container-cgroup.txt"
{ for tool in python3 cat; do printf '%s=%s\n' "$tool" "$(command -v "$tool")"; done; } >"$capture/entry-tools.txt"
set +e
"$binary" --ignored --exact "$test_name" --nocapture --test-threads=1 >"$capture/test.log" 2>"$capture/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$capture/test.exit"
cat "$capture/test.log"
if [[ $status != 0 ]]; then printf 'not-run\n' >"$capture/selection.exit"; exit "$status"; fi
set +e
python3 - "$capture/test.log" <<'PY'
import re, sys
text = open(sys.argv[1], encoding='utf-8').read()
assert len(re.findall(r'^running 1 test$', text, re.M)) == 1, 'exactly one selected test required'
assert len(re.findall(r'^test result: ok\. 1 passed; 0 failed; 0 ignored;.*$', text, re.M)) == 1, 'exactly one passing result required'
assert len(re.findall(r'^test result:', text, re.M)) == 1, 'one result required'
PY
selection=$?
set -e
if [[ $selection == 0 ]]; then
set +e
python3 - "$capture" <<'TIMING'
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
requests = sorted(root.glob('exchange-*-request.json'))
responses = sorted(root.glob('exchange-*-response.json'))
assert requests and len(requests) == len(responses), 'paired exchanges required'
timing = root/'phase-timing.jsonl'
assert timing.stat().st_size <= max(8192, 2048 * len(requests)), 'timing bytes exceed exchange allowance'
exchanges = []
line_count = 0
with timing.open(encoding='utf-8') as stream:
    for line in stream:
        line_count += 1
        assert line_count <= 2 * len(requests) + 32, 'timing lines exceed exchange allowance'
        row = json.loads(line)
        if row.get('fields', {}).get('message') == 'fixed-form native exchange retained':
            exchanges.append(row['fields'])
assert len(requests) == len(exchanges), 'per-exchange timing required'
assert {field['prefix'] for field in exchanges} == {
    path.name.removesuffix('-request.json') for path in requests}, 'timing prefixes must match requests'
assert all(isinstance(field.get('wall_us'), int) for field in exchanges), 'microsecond walls required'
TIMING
selection=$?
set -e
fi
printf '%s\n' "$selection" >"$capture/selection.exit"
exit "$selection"
