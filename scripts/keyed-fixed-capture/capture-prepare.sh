#!/usr/bin/env bash
# Orchestrator preparation only. Ordinary tests use explicit node-free stand-ins.
set -euo pipefail
started=$SECONDS
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} != 1 ]]; then
    export PATH=/usr/local/bin:/usr/bin:/bin
fi
container_started=0
owned_destination=0
finish() {
    status=$?
    trap - EXIT
    if [[ $container_started == 1 ]]; then
        timeout --kill-after=5 20 podman stop --time 10 "$container" >"$destination/build/$phase-stop.log" 2>&1 || true
        timeout --kill-after=5 10 podman inspect "$container" >"$destination/build/$phase-final-state.json" 2>"$destination/build/$phase-inspect.stderr" || true
        if ! timeout --kill-after=5 20 podman rm --force --ignore "$container" >"$destination/build/$phase-remove.log" 2>&1; then exit 74; fi
        set +e
        timeout --kill-after=5 10 podman container exists "$container"
        absent=$?
        set -e
        [[ $absent == 1 ]] || exit 74
    fi
    if [[ $owned_destination == 1 ]]; then
        seal_status=ok
        # Seal settled preparation data, including dot-paths. The append-only
        # end-marker log and the source checkout are explicitly outside this seal.
        if ! timeout --kill-after=5 60 python3 - "$destination" <<'SEAL'
import hashlib, os, pathlib, sys
root = pathlib.Path(sys.argv[1])
def seal(directory, paths):
    partial = directory/'retention.sha256.partial'
    with partial.open('w') as output:
        for path in sorted(paths):
            if path.is_file() and path not in (directory/'retention.sha256', partial):
                output.write(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + str(path.relative_to(directory)) + '\n')
        output.flush(); os.fsync(output.fileno())
    partial.replace(directory/'retention.sha256')
    fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try: os.fsync(fd)
    finally: os.close(fd)
build = root/'build'
if build.is_dir(): seal(build, build.rglob('*'))
seal(root, [p for p in root.iterdir() if p.is_file() and p.name != 'prepare.log'] + list(root.glob('*/capture.env')) + ([build/'retention.sha256'] if build.is_dir() else []))
SEAL
        then seal_status=failed; fi
        detail=
        if [[ $seal_status == failed ]]; then detail=' seal=failed'; fi
        printf 'KFF_PREPARE_DONE run=%s exit=%s%s\n' "$run" "$status" "$detail" >>"$destination/prepare.log"
        printf 'KFF_PREPARE_DONE run=%s exit=%s%s\n' "$run" "$status" "$detail"
        if [[ $seal_status == failed && $status == 0 ]]; then status=74; fi
    fi
    printf 'capture-prepare wall=%ss exit=%s\n' "$((SECONDS-started))" "$status"
    exit "$status"
}
trap finish EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
select_artifact() {
    python3 - "$1" "$2" "$3" <<'PY'
import json, pathlib, shutil, sys
matches = []
for line in pathlib.Path(sys.argv[1]).read_text().splitlines():
    row = json.loads(line)
    if row.get('reason') == 'compiler-artifact' and row.get('target', {}).get('name') == 'keyed_fixed_capture' and row.get('profile', {}).get('test') is True and row.get('executable') is not None:
        matches.append(row['executable'])
assert len(matches) == 1, 'exactly one keyed_fixed_capture executable required'
path = pathlib.PurePosixPath(matches[0])
relative = path.relative_to('/cargo-target')
assert '..' not in relative.parts, 'artifact escapes the warm target'
source = pathlib.Path(sys.argv[2]) / relative
assert source.is_file(), 'artifact absent from warm target'
assert source.resolve().is_relative_to(pathlib.Path(sys.argv[2]).resolve()), 'artifact symlink escapes the warm target'
shutil.copy2(source, sys.argv[3])
PY
}
if [[ ${1:-} == --select-artifact ]]; then
    [[ $# == 4 ]] || exit 64
    select_artifact "$2" "$3" "$4"
    exit 0
fi
[[ $# == 2 || $# == 3 ]] || exit 64
commit=$1
run=$2
families_csv=pilot,clauses,policy,scaling,histories
if [[ $# == 3 ]]; then
    [[ $3 == sponsor,legacy-copy ]] || exit 64
    families_csv=$3
fi
[[ $commit =~ ^[0-9a-f]{40}$ && $run =~ ^[a-z0-9][a-z0-9_-]{0,63}$ ]] || exit 64
root=/var/home/tripod/loops/evidence/fixed-form
warm=/var/home/tripod/loops/cargo-target-fixed-w20c1
reviewed_image=fa98959193e52e93fe9f042e0b5137a79373272ecbdeaa15177a82362857c1ea
trust=reviewed_non_mock
if [[ ${TRIPOD_CAPTURE_STANDIN:-0} == 1 ]]; then
    root=${TRIPOD_CAPTURE_TEST_ROOT:?stand-in root required}
    warm="$root/warm"
    trust=mock
fi
destination="$root/$run"
mkdir -p "$root"
mkdir "$destination"
owned_destination=1
{ for tool in podman git timeout python3 hostname cp cmp mkdir; do printf '%s=%s\n' "$tool" "$(command -v "$tool")"; done; } >"$destination/tools.txt"
date -u +%Y-%m-%dT%H:%M:%SZ >"$destination/start.utc"
printf '%s\n' "$$" >"$destination/prepare.pid"
mkdir "$destination/build" "$destination/build/tmp"
repo=$(git rev-parse --show-toplevel)
[[ $(git -C "$repo" rev-parse HEAD) == "$commit" ]]
[[ -z $(git -C "$repo" status --porcelain --untracked-files=all) ]]
git clone --quiet --no-hardlinks --no-checkout "$repo" "$destination/source"
git -C "$destination/source" checkout --quiet --detach "$commit"
[[ $(git -C "$destination/source" rev-parse HEAD) == "$commit" ]]
git -C "$destination/source" diff --exit-code HEAD
git -C "$destination/source" diff --cached --exit-code
cmp "$0" "$destination/source/scripts/keyed-fixed-capture/capture-prepare.sh"
image=$(timeout 10 podman image inspect --format '{{.Id}}' localhost/tripod-native:f44)
[[ $image == "$reviewed_image" ]] || exit 65
printf '%s\n' "$image" >"$destination/build/image.id"
cp "$destination/source/scripts/keyed-fixed-capture/run-one.sh" "$destination/run-one"
cp "$destination/source/scripts/keyed-fixed-capture/capture-unit.sh" "$destination/capture-unit"
mkdir -p "$warm"
set +e
# Warm offline build hang guard; not a native measurement or subject allowance.
phase=build
container="tripod-$run-prepare-build"
build_command=(timeout --kill-after=10 1800 podman run --init --name "$container" --network=none --userns=keep-id \
    -v "$destination/source:/src:z" -v "$destination/build:/capture:z" \
    -v "$warm:/cargo-target:z" -v /var/home/tripod/loops/cargo-home:/cargo:z \
    -v /usr/local/rustup:/usr/local/rustup:ro -e RUSTUP_HOME=/usr/local/rustup \
    -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/cargo-target -e HOME=/capture \
    -e TMPDIR=/capture/tmp \
    -e PATH=/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
    -w /src "$image" cargo test --offline --profile release -p tripod-vectors \
    --test keyed_fixed_capture --no-run --message-format=json)
printf '%q ' "${build_command[@]}" >"$destination/build/build-command.shell"
printf '\n' >>"$destination/build/build-command.shell"
container_started=1
time "${build_command[@]}" \
    >"$destination/build/build.jsonl" 2>"$destination/build/build.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$destination/build/build.exit"
[[ $status == 0 ]] || exit "$status"
timeout --kill-after=5 10 podman inspect "$container" >"$destination/build/build-final-state.json"
timeout --kill-after=5 20 podman rm --force --ignore "$container" >"$destination/build/build-remove.log"
container_started=0
select_artifact "$destination/build/build.jsonl" "$warm" "$destination/build/keyed_fixed_capture"
cp "$destination/source/Cargo.lock" "$destination/build/Cargo.lock"
git -C "$destination/source" diff --exit-code HEAD
# Cargo.lock is ignored: the complete untracked set must be that file only.
[[ -z $(git -C "$destination/source" status --porcelain --untracked-files=all) ]]
[[ $(git -C "$destination/source" ls-files --others) == Cargo.lock ]]
phase=provenance
container="tripod-$run-prepare-provenance"
container_started=1
timeout --kill-after=5 60 podman run --init --name "$container" --network=none --userns=keep-id \
    -v /usr/local/rustup:/usr/local/rustup:ro \
    -v /var/home/tripod/loops/toolchains/elements/b7fc5d08:/elements:ro,z \
    -e HOME=/tmp \
    "$image" /bin/sh -eu -c '
      /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc -Vv
      /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo -V
      /usr/bin/python3 --version
      sha256sum /usr/bin/python3 /elements/bin/elementsd /elements/bin/elements-cli /elements/lib/libsecp256k1.so
      /elements/bin/elementsd --version
      git -C /elements/src rev-parse HEAD
      cat /elements/PROVENANCE.txt
    ' >"$destination/build/toolchain.txt" 2>"$destination/build/toolchain.stderr"
hostname >"$destination/build/node.txt"
python3 - "$destination" "$commit" "$image" "$trust" "$families_csv" <<'PY'
import hashlib, json, lzma, pathlib, re, sys
root = pathlib.Path(sys.argv[1]); build = root/'build'; source = root/'source'
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
data = dict(repository_commit=sys.argv[2], image=sys.argv[3], trust=sys.argv[4],
    capture_binary=digest(build/'keyed_fixed_capture'), dependency_lock=digest(build/'Cargo.lock'),
    executor=digest(source/'scripts/elements-native-executor.py'),
    toolchain=(build/'toolchain.txt').read_text(), node=(build/'node.txt').read_text().strip())
expected = (source/'packages/vectors/fixtures/adapter-request-comparisons/CURRENT-REVISION-9').read_text().splitlines()[0].split()[19]
assert data['executor'] == expected, 'adapter differs from pinned revision-nine identity'
lines = data['toolchain'].splitlines()
hashes = dict((parts[1], parts[0]) for line in lines if len(parts := line.split()) == 2 and re.fullmatch('[0-9a-f]{64}', parts[0]))
data['interpreter_digest'] = hashes['/usr/bin/python3']
data['daemon_digest'] = hashes['/elements/bin/elementsd']
data['zk_library_digest'] = hashes.get('/elements/lib/libsecp256k1.so')
data['interpreter'] = '/usr/bin/python3 sha256=' + data['interpreter_digest'] + ' ' + next(line for line in lines if line.startswith('Python '))
data['daemon_version'] = next(line for line in lines if line.startswith('Elements Core daemon version '))
data['daemon'] = '/elements/bin/elementsd sha256=' + data['daemon_digest'] + ' ' + data['daemon_version']
framework = [line for line in lines if re.fullmatch('[0-9a-f]{40}', line)]
assert len(framework) == 1, 'one framework checkout identity'
data['framework'] = framework[0]
provenance_tip = re.search(r'^tip: ([0-9a-f]{40}) ', data['toolchain'], re.M).group(1)
assert data['framework'] == provenance_tip, 'framework drift from retained installation'
assert re.search(r'^' + data['daemon_digest'] + r'  elementsd$', data['toolchain'], re.M), 'daemon drift from retained installation'
if sys.argv[4] != 'mock':
    retained = json.loads(lzma.decompress((source/'packages/vectors/fixtures/keyed-final-native/birth.json.lzma').read_bytes()))['sessions'][0]['provenance']
    data['baseline'] = {key: retained[key] for key in ('image', 'framework', 'daemon')}
    assert data['image'] == retained['image'], 'first-edition image baseline drift'
    assert data['framework'] == retained['framework'], 'first-edition framework baseline drift'
    assert data['daemon'] == retained['daemon'], 'first-edition daemon baseline drift'
    previous_zk = re.search(r'zk library:.*sha256 ([0-9a-f]{64})', retained['toolchain'])
    assert previous_zk and data['zk_library_digest'] == previous_zk.group(1), 'first-edition zk library baseline drift'
(build/'prepared.json').write_text(json.dumps(data, sort_keys=True, indent=2)+'\n')
for target in sys.argv[5].split(','):
    directory = root/target; directory.mkdir(); (directory/'capture.env').write_text('TRIPOD_FIXED_CAPTURE_DIR=/capture\n')
PY
printf 'KFF_PREPARED directory=%s image=%s\n' "$destination" "$image"
