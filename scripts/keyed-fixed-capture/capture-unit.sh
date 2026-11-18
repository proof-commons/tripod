#!/usr/bin/env bash
# Called by the detached unit, or its --check (single-pass) / --wait watcher.
set -euo pipefail
started=$SECONDS
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} != 1 ]]; then
    export PATH=/usr/local/bin:/usr/bin:/bin
fi
mode=run
if [[ ${1:-} == --wait || ${1:-} == --check ]]; then mode=${1#--}; shift; fi
run_dir=${1:?run directory required}
target=${2:?target required}
guard=${3:?guard required}
image=${4:?immutable image required}
case "$target" in pilot|clauses|policy|scaling|histories) ;; *) exit 64;; esac
[[ $guard =~ ^[1-9][0-9]*$ ]] || exit 64
[[ -d "$run_dir" ]] || exit 66
run_dir=$(realpath -e "$run_dir")
evidence_root=/var/home/tripod/loops/evidence/fixed-form
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} == 1 ]]; then
    [[ $run_dir != "$evidence_root" && $run_dir != "$evidence_root/"* ]] || exit 64
else
    [[ $run_dir == "$evidence_root/"* ]] || exit 64
fi
root=$(dirname "$run_dir")
run=$(basename "$root")
unit="tripod-$run-$target"
container_started=0
if [[ $mode == run ]]; then
python3 - "$root/prepare.log" "$run" <<'PREPARED'
import pathlib, sys
lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
assert lines and lines[-1] == 'KFF_PREPARE_DONE run=' + sys.argv[2] + ' exit=0', 'successful preparation marker required'
assert sum(line.startswith('KFF_PREPARE_DONE ') for line in lines) == 1
PREPARED
cmp "$root/run-one" "$root/source/scripts/keyed-fixed-capture/run-one.sh"
cmp "$0" "$root/source/scripts/keyed-fixed-capture/capture-unit.sh"
fi
if [[ $mode != run ]]; then
    trap 'status=$?; printf "capture-watch wall=%ss exit=%s\n" "$((SECONDS-started))" "$status"; exit "$status"' EXIT
    while :; do
        state=$(timeout 10 systemctl --user show "$unit" --property=ActiveState --value)
        if [[ ! -e "$run_dir/start.utc" && $state == inactive ]]; then
            printf 'capture-state=not-started\n'
            exit 76
        fi
        case "$state" in
            active|activating|deactivating)
                # A poll may precede the wrapper's first instruction. Do not
                # invent a start time or wait with a freshly renewed budget.
                if [[ ! -f "$run_dir/start.utc" ]]; then
                    printf 'capture-state=still-running awaiting-start=yes\n'
                    if [[ $mode == check ]]; then exit 75; fi
                    sleep 1
                    continue
                fi
                # Reconnection cannot renew a run's budget.
                timeout --kill-after=5 10 python3 - "$run_dir/start.utc" "$guard" <<'BOUND'
import datetime, pathlib, sys, time
start = datetime.datetime.fromisoformat(pathlib.Path(sys.argv[1]).read_text().strip().replace('Z','+00:00')).timestamp()
assert time.time() <= start + int(sys.argv[2]) + 120, 'retained run deadline exceeded'
BOUND
                if [[ $mode == check ]]; then printf 'capture-state=still-running\n'; exit 75; fi
                sleep 1;;
            inactive|failed) break;;
            *) printf 'capture-state=failed unknown-unit-state=%s\n' "$state"; exit 74;;
        esac
    done
    # Hash failed runs too, but only after the writer is demonstrably absent.
    set +e
    timeout --kill-after=5 10 podman container exists "$unit"
    absent=$?
    set -e
    [[ $absent == 1 ]] || exit 74
    # Envelope journals are larger: these new actions have their own bounds.
    # Scaling allows 120 s for 1.2 GB at 10 MB/s, plus 120 s file-walk margin.
    seal_guard=60
    case "$target" in scaling) seal_guard=240;; esac
    timeout --kill-after=5 "$seal_guard" python3 - "$run_dir" <<'SEAL'
import hashlib, os, pathlib, sys
root = pathlib.Path(sys.argv[1])
partial = root/'retention.sha256.partial'
with partial.open('w') as output:
    for path in sorted(root.rglob('*')):
        if path.is_file() and path.name not in ('retention.sha256','retention.sha256.partial') and 'regtest' not in path.parts:
            output.write(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + str(path.relative_to(root)) + '\n')
    output.flush()
    os.fsync(output.fileno())
partial.replace(root/'retention.sha256')
directory = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
try: os.fsync(directory)
finally: os.close(directory)
SEAL
    if ! timeout --kill-after=5 10 python3 - "$run_dir/unit.log" "$target" <<'MARKER'
import re, sys
text = open(sys.argv[1], encoding='utf-8').read()
markers = re.findall(r'^KFF_CAPTURE_DONE target=([^ ]+) exit=([0-9]+)$', text, re.M)
assert markers == [(sys.argv[2], '0')], 'one successful end marker required'
MARKER
    then printf 'capture-state=failed sealed=yes\n'; exit 74; fi
    [[ $(timeout 10 systemctl --user show "$unit" --property=ExecMainStatus --value) == 0 ]] || { printf 'capture-state=failed sealed=yes\n'; exit 74; }
    printf 'capture-state=ended-and-sealed\n'
    exit 0
fi
# Refuse reuse before installing a trap that could overwrite old evidence.
for prior in start.utc unit.pid test.exit end.utc; do [[ ! -e "$run_dir/$prior" ]] || exit 73; done
if [[ -f "$run_dir/unit.log" ]] && grep -q '^KFF_CAPTURE_DONE ' "$run_dir/unit.log"; then exit 73; fi

snapshot() {
    python3 - "$run_dir/$1-resources.json" "$run_dir/before-resources.json" "$1" <<'PY'
import json, pathlib, time, sys
proc = pathlib.Path('/proc/self/cgroup').read_text()
rows = [line[3:] for line in proc.splitlines() if line.startswith('0::')]
data = dict(epoch_ns=time.time_ns(), cgroup_membership=proc, scope='fixed before-sample cgroup; container inclusion unverified until native placement check; aggregate, not clause cost', loadavg=pathlib.Path('/proc/loadavg').read_text(), counters={})
if sys.argv[3] == 'before':
    fixed = str(pathlib.Path('/sys/fs/cgroup') / rows[0].lstrip('/')) if len(rows) == 1 else None
else:
    fixed = json.loads(pathlib.Path(sys.argv[2]).read_text())['cgroup_path']
data['cgroup_path'] = fixed
if fixed is not None:
    group = pathlib.Path(fixed)
    for name in ('cpu.stat','cpu.pressure','memory.pressure','io.pressure','memory.current','memory.peak'):
        path = group/name
        data['counters'][name] = path.read_text() if path.is_file() else None
pathlib.Path(sys.argv[1]).write_text(json.dumps(data, sort_keys=True, indent=2)+'\n')
PY
}
finish() {
    status=$?
    trap - EXIT
    if [[ $container_started == 1 ]]; then
        # The client may have died while conmon kept the container alive.
        # Do not publish an end marker until removal is verified.
        timeout --kill-after=5 20 podman stop --time 10 "$unit" >"$run_dir/container-stop.log" 2>&1 || true
        timeout --kill-after=5 10 podman inspect "$unit" >"$run_dir/container-final-state.json" 2>"$run_dir/container-inspect.stderr" || true
        if ! timeout --kill-after=5 20 podman rm --force --ignore "$unit" >"$run_dir/container-remove.log" 2>&1; then
            printf 'container removal unverified; no end marker\n' >>"$run_dir/unit.log"
            exit 74
        fi
        set +e
        timeout --kill-after=5 10 podman container exists "$unit"
        absent=$?
        set -e
        if [[ $absent != 1 ]]; then
            printf 'container absence unverified; no end marker\n' >>"$run_dir/unit.log"
            exit 74
        fi
        python3 - "$run_dir" <<'LOGS'
import json, pathlib, shutil, sys
root = pathlib.Path(sys.argv[1])
source = root/'regtest'
copied = {}
for name in ('node.out', 'debug.log'):
    matches = sorted(source.rglob(name)) if source.is_dir() else []
    if len(matches) > 1:
        raise RuntimeError('more than one node log named ' + name)
    if matches:
        shutil.copyfile(matches[0], root/name)
        copied[name] = str(matches[0].relative_to(root))
(root/'node-log-copies.json').write_text(json.dumps(copied, sort_keys=True) + '\n')
LOGS
    fi
    date -u +%Y-%m-%dT%H:%M:%SZ >"$run_dir/end.utc"
    if ! snapshot after; then
        printf 'resource snapshot unavailable\n' >>"$run_dir/unit.log"
        if [[ $status == 0 ]]; then status=74; fi
    fi
    python3 - "$run_dir" <<'SCOPE'
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
record = root/'container-cgroup.txt'
container = [line[3:] for line in record.read_text().splitlines() if line.startswith('0::')] if record.is_file() else []
for name in ('before', 'after'):
    path = root/(name + '-resources.json')
    if not path.is_file():
        continue
    data = json.loads(path.read_text())
    fixed = data['cgroup_path']
    verified = len(container) == 1 and fixed is not None and (container[0] == fixed.removeprefix('/sys/fs/cgroup') or container[0].startswith(fixed.removeprefix('/sys/fs/cgroup').rstrip('/') + '/'))
    if verified:
        data['scope'] = 'container inclusion verified by container-cgroup.txt; aggregate, not clause cost'
    path.write_text(json.dumps(data, sort_keys=True, indent=2) + '\n')
SCOPE
    printf 'capture-unit wall=%ss exit=%s\n' "$((SECONDS-started))" "$status" >>"$run_dir/unit.log"
    printf 'KFF_CAPTURE_DONE target=%s exit=%s\n' "$target" "$status" >>"$run_dir/unit.log"
    exit "$status"
}
trap finish EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
printf '%s\n' "$$" >"$run_dir/unit.pid"
printf '%s\n' "$unit" >"$run_dir/unit.name"
date -u +%Y-%m-%dT%H:%M:%SZ >"$run_dir/start.utc"
{
    for tool in podman systemctl timeout python3 date dirname basename grep cat mkdir; do
        printf '%s=%s\n' "$tool" "$(command -v "$tool")"
    done
} >"$run_dir/tools.txt"
# A writable capture directory supplies evidence, never environment authority.
python3 - "$run_dir/capture.env" <<'ENV'
import pathlib, sys
assert pathlib.Path(sys.argv[1]).read_bytes() == b'TRIPOD_FIXED_CAPTURE_DIR=/capture\n', 'capture environment differs from prepared line'
ENV
actual_image=$(timeout 10 podman image inspect --format '{{.Id}}' localhost/tripod-native:f44)
[[ $actual_image == "$image" ]] || exit 65
python3 - "$root/build/prepared.json" "$target" "$guard" "$image" <<'PY'
import json, sys
p = json.load(open(sys.argv[1], encoding='utf-8'))
assert p['image'] == sys.argv[4], 'prepared image mismatch'
assert int(sys.argv[3]) == (3600 if sys.argv[2] == 'scaling' else 1800), 'pilot guard mismatch'
PY
mkdir -p "$run_dir/tmp" "$run_dir/work"
snapshot before
command=(timeout --kill-after=10 "$guard" podman run --init --cgroups=split --cgroupns=host --name "$unit" --network=none --userns=keep-id \
    -v "$root/source:/src:ro,z" -v "$root/build:/capture-build:ro,z" \
    -v "$root/run-one:/run-one:ro,z" -v "$run_dir:/capture:z" \
    -v /var/home/tripod/loops/toolchains/elements/b7fc5d08:/elements:ro,z \
    -e TRIPOD_FIXED_CAPTURE_DIR=/capture -e TMPDIR=/capture/tmp -e HOME=/capture/work \
    -w /capture/work "$image" /run-one "$target")
printf '%q ' "${command[@]}" >"$run_dir/command.shell"
printf '\n' >>"$run_dir/command.shell"
container_started=1
time "${command[@]}"
[[ -f "$run_dir/test.exit" && $(cat "$run_dir/test.exit") == 0 ]] || exit 74
[[ -f "$run_dir/admitted" ]] || exit 74
