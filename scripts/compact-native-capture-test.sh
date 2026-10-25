#!/bin/sh
# Preflight refuses unsafe destinations before any invocation.
set -eu
root=${1:?repository root required}
work=$(mktemp -d "$TMPDIR/compact-capture-contract.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
git init -q "$work/suite"
cd "$work/suite"
git -c user.name=Test -c user.email=test@example.invalid -c commit.gpgsign=false commit -q --allow-empty -m test
mkdir "$work/existing"
if sh "$root/scripts/compact-native-capture.sh" "$work/existing" > "$work/log" 2>&1; then
  echo 'existing destination was accepted' >&2; exit 1
fi
grep -q 'destination already exists' "$work/log"
touch dirty
if sh "$root/scripts/compact-native-capture.sh" "$work/new" > "$work/log" 2>&1; then
  echo 'dirty source was accepted' >&2; exit 1
fi
grep -q 'source tree is dirty' "$work/log"
test ! -e "$work/new"
rm dirty
if sh "$root/scripts/compact-native-capture.sh" relative > "$work/log" 2>&1; then
  echo 'relative destination was accepted' >&2; exit 1
fi
grep -q 'destination must be absolute' "$work/log"
probe_refused() {
  if TRIPOD_COMPACT_MODE=probe TRIPOD_COMPACT_PROBE_ROOT=$1 sh "$root/scripts/compact-native-capture.sh" "$2" > "$work/log" 2>&1; then
    echo 'unsafe probe destination was accepted' >&2; exit 1
  fi
  grep -q "$3" "$work/log"
  test ! -e "$2"
}
probe_refused '' "$work/probe" 'probe root is required'
probe_refused relative "$work/probe" 'probe root must be absolute'
probe_refused /workspace/loops/tripod/evidence "$work/probe" 'probe path is under evidence'
probe_refused /workspace/loops/tripod/evidence/wave0 "$work/probe" 'probe path is under evidence'
probe_refused /workspace/loops/tripod/tmp/../evidence/wave0 "$work/probe" 'probe path is under evidence'
probe_refused "$work/scratch" "$work/probe" 'probe destination is outside scratch'
probe_refused /workspace/loops/tripod /workspace/loops/tripod/evidence/compact-probe-contract 'probe path is under evidence'
mkdir "$work/scratch"
ln -s /workspace/loops/tripod/evidence "$work/scratch/link"
probe_refused "$work/scratch" "$work/scratch/link/compact-probe-contract" 'probe path is under evidence'
cat > "$work/cargo-stub" <<'STUB'
#!/bin/sh
set -eu
capture=$TRIPOD_LIVE_REPORT_DIR/$TRIPOD_COMPACT_RUN_ADDRESS.report.capture
hex() { od -An -tx1 | tr -d ' \n'; }
{
  printf 'native-capture-schema 2\nceremony-id report\n'
  printf 'suite-commit %s\nsuite-tree %s\n' "$TRIPOD_LIVE_SUITE_COMMIT" "$TRIPOD_LIVE_SUITE_TREE"
  printf 'handshake-protocol-schema 8\nenvironment-schema 8\nterminal-state complete\n'
  printf 'operation-count %s\n' "${TEST_OPERATIONS:-92}"
  printf 'handshake-intended-executed-tip 40 %s\n' "$(printf '%s' "$ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP" | hex)"
  printf 'handshake-binary-reported-revision 40 %s\n' "$(printf '%s' "$ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP" | hex)"
} > "$capture"
digest=$(sha256sum "$capture"); digest=${digest%% *}
printf 'capture-content-sha256 %s\nnative-capture-end report\n' "$digest" >> "$capture"
{
  printf 'timing-schema compact-phases-1\nceremony-id report\nstatus passed\n'
  printf 'phase-staging-ms 1\nphase-execution-ms 2\ntotal-ms 3\n'
} > "$capture.timing"
case "${TEST_DEFECT:-none}" in
  digest) sed -i 's/operation-count 92/operation-count 91/' "$capture";;
  timing) printf 'phase-staging-ms 1\n' >> "$capture.timing";;
  extra) touch "$TRIPOD_LIVE_REPORT_DIR/extra";;
  hidden) touch "$TRIPOD_LIVE_REPORT_DIR/.extra";;
esac
printf 'test result: ok. 1 passed; 0 failed; 0 ignored;\nCOMPACT operations 92 controls 9 suspended 0\n'
STUB
chmod +x "$work/cargo-stub"
export TRIPOD_CAPTURE_CARGO=$work/cargo-stub
export TRIPOD_LIVE_EXECUTOR=$work/cargo-stub
export ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=$work/cargo-stub
export ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=$work/cargo-stub
export ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=$work
export ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1
export ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP=b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0
export TRIPOD_LIVE_NETWORK_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export TRIPOD_LIVE_GENESIS_ID=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
export ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID=$TRIPOD_LIVE_NETWORK_ID
sh "$root/scripts/compact-native-capture.sh" "$work/valid" > "$work/log" 2>&1
grep -qx 'eligible yes' "$work/valid/RUN-REPORT"
test "$(wc -l < "$work/valid/MANIFEST.sha256")" -eq 2
TRIPOD_COMPACT_MODE=probe TRIPOD_COMPACT_PROBE_ROOT=$work/scratch sh "$root/scripts/compact-native-capture.sh" "$work/scratch/valid" > "$work/log" 2>&1
grep -qx 'eligible no' "$work/scratch/valid/RUN-REPORT"
for defect in digest timing extra hidden; do
  if TEST_DEFECT=$defect sh "$root/scripts/compact-native-capture.sh" "$work/$defect" > "$work/log" 2>&1; then
    echo "accepted $defect defect" >&2; exit 1
  fi
  grep -qx 'eligible no' "$work/$defect/RUN-REPORT"
done
if TEST_OPERATIONS=91 sh "$root/scripts/compact-native-capture.sh" "$work/count" > "$work/log" 2>&1; then
  echo 'accepted operation census drift' >&2; exit 1
fi
grep -qx 'eligible no' "$work/count/RUN-REPORT"
chmod -R u+w "$work"
echo 'test result: compact capture boundary passed'
