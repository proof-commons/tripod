#!/usr/bin/env python3
"""Node-free wrapper controls. Every spawned process has a wall guard."""
import json
import hashlib
import datetime
import os
from pathlib import Path
import signal
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
WRAPPERS = ROOT / "scripts/keyed-fixed-capture"
# Hang guards, not performance thresholds: record-gate load reached 1.793718 s
# for an ordinary control and censored preparation at 5.087444 s. Preparation
# starts many stand-in processes; leave it extra cleanup/retention headroom.
ORDINARY_GUARD = 20
PREPARATION_GUARD = 90
FAMILY_GUARD = 300  # Matched by scripts/meson.build.
IMAGE = "fa98959193e52e93fe9f042e0b5137a79373272ecbdeaa15177a82362857c1ea"


def executable(path, text):
    path.write_text("#!/bin/sh\nset -eu\n" + text)
    path.chmod(0o755)


def run(name, argv, env, expected, guard=ORDINARY_GUARD):
    started = time.monotonic()
    process = subprocess.Popen(argv, env=env, cwd=ROOT, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, start_new_session=True)
    try:
        output, _ = process.communicate(timeout=guard)
        status = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        status = 124
    wall = time.monotonic() - started
    print(f"capture-wrapper subject={name} wall={wall:.6f}s guard={guard}s exit={status}", flush=True)
    assert expected(status), (name, status, output.decode(errors="replace"))
    return output.decode(errors="replace")


def artifact_controls(root, env):
    warm = root / "warm"
    warm.mkdir()
    executable(warm / "test", "exit 0\n")
    source = root / "build.jsonl"
    row = dict(reason="compiler-artifact", target=dict(name="keyed_fixed_capture"),
               profile=dict(test=True), executable="/cargo-target/test")
    for name, rows, passing in (("artifact-one", [row], True),
                                ("artifact-zero", [], False),
                                ("artifact-many", [row, row], False),
                                ("artifact-outside", [dict(row, executable="/other/test")], False),
                                ("artifact-missing", [dict(row, executable="/cargo-target/missing")], False),
                                ("artifact-parent", [dict(row, executable="/cargo-target/../test")], False)):
        source.write_text("".join(json.dumps(value) + "\n" for value in rows))
        run(name, [str(WRAPPERS / "capture-prepare.sh"), "--select-artifact",
                   str(source), str(warm), str(root / "selected")], env,
            lambda code: (code == 0) == passing)


def selection_controls(root, env):
    env = dict(env, TRIPOD_CAPTURE_STANDIN="1", TRIPOD_CAPTURE_TEST_ROOT=str(root))
    passing = ("printf 'running 1 test\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0s\\n'\n"
               "printf '{}\\n' >\"$TRIPOD_CAPTURE_TEST_ROOT/exchange-00001-request.json\"\n"
               "printf '{}\\n' >\"$TRIPOD_CAPTURE_TEST_ROOT/exchange-00001-response.json\"\n"
               "printf '%s\\n' '{\"fields\":{\"message\":\"fixed-form native exchange retained\",\"prefix\":\"exchange-00001\",\"wall_us\":7}}' >\"$TRIPOD_CAPTURE_TEST_ROOT/phase-timing.jsonl\"\n")
    cases = [("one-passing", passing, lambda c: c == 0, ORDINARY_GUARD),
             ("missing-exchange-timing", "printf 'running 1 test\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0s\\n'\n", lambda c: c != 0, ORDINARY_GUARD),
             ("excess-timing-lines", passing +
              "for n in $(seq 1 34); do printf '{}\\n' >>\"$TRIPOD_CAPTURE_TEST_ROOT/phase-timing.jsonl\"; done\n",
              lambda c: c != 0, ORDINARY_GUARD),
             ("excess-timing-bytes", passing +
              "python3 -c 'import json; print(json.dumps({\"fields\": {\"pad\": \"x\" * 9000}}))' >>\"$TRIPOD_CAPTURE_TEST_ROOT/phase-timing.jsonl\"\n",
              lambda c: c != 0, ORDINARY_GUARD),
             ("nonzero-test", "exit 7\n", lambda c: c == 7, ORDINARY_GUARD),
             ("empty-selection", "printf 'running 0 tests\\ntest result: ok. 0 passed; 0 failed; 0 ignored;\\n'\n", lambda c: c != 0, ORDINARY_GUARD),
             ("test-timeout", "sleep 20\n", lambda c: c == 124, 1)]
    for name, body, expected, guard in cases:
        executable(root / "keyed_fixed_capture", body)
        for file in ('test.exit','selection.exit','phase-timing.jsonl','exchange-00001-request.json','exchange-00001-response.json'):
            (root/file).unlink(missing_ok=True)
        output = run(name, [str(WRAPPERS / "run-one.sh"), "clauses"], env, expected, guard)
        if name == 'excess-timing-lines':
            assert 'timing lines exceed exchange allowance' in output
        if name == 'excess-timing-bytes':
            assert 'timing bytes exceed exchange allowance' in output
        if name == 'one-passing': assert (root/'selection.exit').read_text().strip() == '0'
        if name == 'empty-selection':
            assert (root/'test.exit').read_text().strip() == '0'
            assert (root/'selection.exit').read_text().strip() != '0'
        assert (root/'container-cgroup.txt').exists()


def unit_controls(root, env):
    tools = root / "tools"
    tools.mkdir()
    env = dict(env, PATH=str(tools) + os.pathsep + env["PATH"], TRIPOD_CAPTURE_STANDIN="1")
    cleanup = 'case "$1" in stop|inspect|rm) printf "%s\\n" "$*" >>"$CAPTURE_UNIT_DIR/cleanup.log"; if grep -q KFF_CAPTURE_DONE "$CAPTURE_UNIT_DIR/unit.log" 2>/dev/null; then exit 99; fi; exit 0;; container) exit 1;; esac\n'
    passing_runtime = cleanup + f'if [ "$1" = image ]; then echo {IMAGE}; else printf "0\\n" >"$CAPTURE_UNIT_DIR/test.exit"; touch "$CAPTURE_UNIT_DIR/admitted"; fi\nexit 0\n'
    executable(tools / "systemctl", 'case "$*" in *ActiveState*) echo "${CAPTURE_ACTIVE:-inactive}";; *) echo 0;; esac\n')

    def case(name, runtime=passing_runtime):
        run_root = root / name
        (run_root / 'build').mkdir(parents=True)
        (run_root / 'build/prepared.json').write_text(json.dumps(dict(image=IMAGE)))
        source = run_root / 'source/scripts/keyed-fixed-capture'
        source.mkdir(parents=True)
        for name in ('run-one', 'capture-unit'):
            shutil.copy2(WRAPPERS / (name+'.sh'), source / (name+'.sh'))
            shutil.copy2(WRAPPERS / (name+'.sh'), run_root / name)
        (run_root/'prepare.log').write_text('KFF_PREPARE_DONE run='+run_root.name+' exit=0\n')
        directory = run_root / 'clauses'
        directory.mkdir()
        (directory / 'capture.env').write_text('TRIPOD_FIXED_CAPTURE_DIR=/capture\n')
        executable(tools / 'podman', runtime)
        command = [str(WRAPPERS / 'capture-unit.sh'), str(directory), 'clauses', '1800', IMAGE]
        return directory, command, dict(env, CAPTURE_UNIT_DIR=str(directory))

    directory, command, selected = case('success')
    (directory/'regtest/elementsregtest/regtest').mkdir(parents=True)
    (directory/'regtest/elementsregtest/regtest/debug.log').write_text('node debug control\n')
    (directory/'regtest/node.out').write_text('node output control\n')
    run('unit-success', command, selected, lambda c:c==0)
    run('watch-success', [command[0], '--wait', *command[1:]], selected, lambda c:c==0)
    output = run('check-success', [command[0], '--check', *command[1:]], selected, lambda c:c==0)
    assert 'capture-state=ended-and-sealed' in output
    assert (directory/'container-final-state.json').exists()
    assert (directory/'tools.txt').exists()
    assert (directory/'debug.log').read_text() == 'node debug control\n'
    assert (directory/'node.out').read_text() == 'node output control\n'
    assert 'debug.log' in (directory/'retention.sha256').read_text()
    assert 'node.out' in (directory/'retention.sha256').read_text()
    command_text = (directory/'command.shell').read_text()
    assert '--cgroups=split' in command_text and '--cgroupns=host' in command_text
    assert '--init' in command_text
    before = json.loads((directory/'before-resources.json').read_text())
    after = json.loads((directory/'after-resources.json').read_text())
    assert before['cgroup_path'] == after['cgroup_path']
    assert 'inclusion unverified' in after['scope']
    # Execute the actual snapshot body with synthetic membership that moves.
    # Only the two operating-system paths are relocated in this internal control.
    body = (WRAPPERS/'capture-unit.sh').read_text().split('<<\'PY\'\n',1)[1].split('\nPY',1)[0]
    membership = tools/'membership'
    groups = tools/'groups'
    (groups/'original').mkdir(parents=True)
    (groups/'moved').mkdir()
    (groups/'original/cpu.stat').write_text('usage_usec 1\n')
    snapshot = tools/'snapshot.py'
    snapshot.write_text(body.replace("'/proc/self/cgroup'", repr(str(membership))).replace("'/sys/fs/cgroup'", repr(str(groups))))
    membership.write_text('0::/original\n')
    before_path = tools/'before.json'
    run('resource-before', ['python3',str(snapshot),str(before_path),str(before_path),'before'], env, lambda c:c==0)
    membership.write_text('0::/moved\n')
    (groups/'original/cpu.stat').write_text('usage_usec 2\n')
    after_path = tools/'after.json'
    run('resource-after-move', ['python3',str(snapshot),str(after_path),str(before_path),'after'], env, lambda c:c==0)
    sampled = json.loads(after_path.read_text())
    assert sampled['cgroup_path'] == str(groups/'original')
    assert sampled['cgroup_membership'] == '0::/moved\n'
    assert sampled['counters']['cpu.stat'] == 'usage_usec 2\n'
    # Watch is about settled evidence, not the later state of the source copy.
    shutil.rmtree(directory.parent/'source')
    (directory.parent/'prepare.log').write_text('changed after completion\n')
    output = run('watch-changed-source', [command[0], '--check', *command[1:]], selected, lambda c:c==0)
    assert 'capture-state=ended-and-sealed' in output
    # Restore launch authority for the reuse refusal controls below.
    source = directory.parent/'source/scripts/keyed-fixed-capture'
    source.mkdir(parents=True)
    for name in ('run-one','capture-unit'):
        shutil.copy2(WRAPPERS/(name+'.sh'), source/(name+'.sh'))
    (directory.parent/'prepare.log').write_text('KFF_PREPARE_DONE run=success exit=0\n')
    d, cmd, e = case('never-started')
    output = run('check-not-started', [cmd[0], '--check', *cmd[1:]], e, lambda c:c==76)
    assert 'capture-state=not-started' in output and not (d/'retention.sha256').exists()
    for changed in ('prepare', 'run-one', 'capture-unit'):
        d, cmd, e = case('untrusted-'+changed)
        file = d.parent/'prepare.log' if changed=='prepare' else (d.parent/'run-one' if changed=='run-one' else d.parent/'source/scripts/keyed-fixed-capture/capture-unit.sh')
        file.write_text('changed\n')
        run('untrusted-'+changed, cmd, e, lambda c:c!=0)
        assert not (d/'start.utc').exists()
    d, cmd, e = case('outside-root')
    run('outside-evidence-root', cmd, dict(e,TRIPOD_CAPTURE_STANDIN='0'), lambda c:c==64)

    original = (directory/'unit.log').read_bytes()
    run('relaunch-refused', command, selected, lambda c:c==73)
    assert (directory/'unit.log').read_bytes() == original
    for prior in ('start.utc','unit.pid','test.exit','end.utc','marker'):
        d, cmd, e = case('reuse-'+prior.replace('.','-'))
        (d/('unit.log' if prior=='marker' else prior)).write_text('KFF_CAPTURE_DONE target=clauses exit=0\n')
        run('reuse-'+prior, cmd, e, lambda c:c==73)
    directory, command, selected = case('tampered')
    (directory/'capture.env').write_text('TRIPOD_FIXED_CAPTURE_DIR=/capture\nTRIPOD_CAPTURE_STANDIN=1\nTRIPOD_CAPTURE_TEST_ROOT=/capture\n')
    run('tampered-environment', command, selected, lambda c:c!=0)
    assert not (directory/'test.exit').exists()
    d, cmd, e = case('standin-evidence')
    relocated = tools/'relocated-unit'
    relocated.write_text((WRAPPERS/'capture-unit.sh').read_text().replace('/var/home/tripod/loops/evidence/fixed-form', str(d.parent.parent)))
    relocated.chmod(0o755)
    run('standin-inside-evidence-root', [str(relocated), *cmd[1:]], e, lambda c:c==64)
    directory, command, selected = case('running')
    output = run('check-starting', [command[0], '--check', *command[1:]], dict(selected,CAPTURE_ACTIVE='activating'), lambda c:c==75)
    assert 'awaiting-start=yes' in output
    # A foreground waiter must stay with a unit that has not written start.utc.
    executable(tools / 'systemctl', 'case "$*" in *ActiveState*) if [ ! -f "$CAPTURE_UNIT_DIR/start.utc" ]; then echo activating; else echo inactive; fi;; *) echo 0;; esac\n')
    (directory/'unit.log').write_text('KFF_CAPTURE_DONE target=clauses exit=0\n')
    process = subprocess.Popen([command[0], '--wait', *command[1:]], env=selected, cwd=ROOT,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    time.sleep(1.2)
    assert process.poll() is None, 'waiter exited while awaiting start'
    (directory/'start.utc').write_text(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())+'\n')
    output, _ = process.communicate(timeout=ORDINARY_GUARD)
    assert process.returncode == 0, output.decode(errors='replace')
    executable(tools / 'systemctl', 'case "$*" in *ActiveState*) echo "${CAPTURE_ACTIVE:-inactive}";; *) echo 0;; esac\n')
    (directory/'start.utc').write_text(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())+'\n')
    output = run('check-running', [command[0], '--check', *command[1:]], dict(selected,CAPTURE_ACTIVE='active'), lambda c:c==75)
    assert 'capture-state=still-running' in output
    (directory/'start.utc').write_text('2000-01-01T00:00:00Z\n')
    run('check-old-start', [command[0], '--check', *command[1:]], dict(selected,CAPTURE_ACTIVE='active'), lambda c:c not in (0,75))
    directory, command, selected = case('missing-marker')
    (directory/'unit.log').write_text('')
    (directory/'start.utc').write_text('2026-11-14T00:00:00Z\n')
    run('missing-marker', [command[0], '--check', *command[1:]], selected, lambda c:c==74)
    assert (directory/'retention.sha256').exists()
    cases = [('missing-admission',cleanup+f'if [ "$1" = image ]; then echo {IMAGE}; fi\n',74),
             ('image-drift',cleanup+'echo different-image\n',65),
             ('unit-nonzero',cleanup+f'if [ "$1" = image ]; then echo {IMAGE}; exit 0; fi\nexit 7\n',7)]
    for name, runtime, status in cases:
        directory, command, selected = case(name,runtime)
        run(name, command, selected, lambda c:c==status)
        assert f'KFF_CAPTURE_DONE target=clauses exit={status}' in (directory/'unit.log').read_text()
        run('seal-'+name, [command[0], '--check', *command[1:]], selected, lambda c:c==74)
        assert (directory/'retention.sha256').exists()
    directory, command, selected = case('cleanup-still-present', passing_runtime.replace('container) exit 1', 'container) exit 0'))
    run('cleanup-unverified', command, selected, lambda c:c==74)
    assert 'KFF_CAPTURE_DONE' not in (directory/'unit.log').read_text()
    directory, command, selected = case('terminated', cleanup + f'if [ "$1" = image ]; then echo {IMAGE}; exit 0; fi\nkill -TERM "$CAPTURE_WRAPPER_PID"\nexit 0\n')
    signal_launcher = tools/'signal-launch'
    executable(signal_launcher, 'export CAPTURE_WRAPPER_PID=$$\nexec "$@"\n')
    run('unit-term', [str(signal_launcher), *command], selected, lambda c:c==143)
    assert 'KFF_CAPTURE_DONE target=clauses exit=143' in (directory/'unit.log').read_text()
    assert 'stop --time 10 tripod-terminated-clauses' in (directory/'cleanup.log').read_text()
    directory, command, selected = case('timeout')
    executable(tools/'timeout', 'if [ "$1" = --kill-after=10 ]; then exit 124; fi\nif [ "$1" = --kill-after=5 ]; then shift; fi\nshift\nexec "$@"\n')
    run('unit-timeout', command, selected, lambda c:c==124)
    assert 'KFF_CAPTURE_DONE target=clauses exit=124' in (directory/'unit.log').read_text()
    calls=(directory/'cleanup.log').read_text()
    assert 'stop --time 10 tripod-timeout-clauses' in calls
    assert 'rm --force --ignore tripod-timeout-clauses' in calls
    executable(tools/'timeout', 'if [ "$1" = --kill-after=5 ] && [ "$2" = 60 ]; then exit 124; fi\nexec /usr/bin/timeout "$@"\n')
    run('seal-timeout', [command[0], '--check', *command[1:]], selected, lambda c:c==124)
    assert not (directory/'retention.sha256').exists()
    for target, allowance in (('scaling',240),):
        executable(tools/'timeout', f'if [ "$1" = --kill-after=5 ] && [ "$2" = {allowance} ]; then exit 124; fi\nexec /usr/bin/timeout "$@"\n')
        changed = [command[0], '--check', command[1], target, '3600' if target=='scaling' else '1800', IMAGE]
        run('seal-allowance-'+target, changed, selected, lambda c:c==124)


def preparation_controls(root, env):
    tools = root / 'tools'
    tools.mkdir()
    env = dict(env, PATH=str(tools)+os.pathsep+env['PATH'],
               TRIPOD_CAPTURE_STANDIN='1', TRIPOD_CAPTURE_TEST_ROOT=str(root),
               CAPTURE_REPO=str(ROOT), CAPTURE_IMAGE=IMAGE)
    git = tools/'git'
    git.write_text('''#!/usr/bin/env python3
import os, pathlib, shutil, sys
a=sys.argv[1:]; repo=pathlib.Path(os.environ['CAPTURE_REPO'])
if a[0]=='clone':
    destination=pathlib.Path(a[-1]); (destination/'.git').mkdir(parents=True)
    for relative in ('scripts/keyed-fixed-capture/capture-prepare.sh','scripts/keyed-fixed-capture/run-one.sh','scripts/keyed-fixed-capture/capture-unit.sh','scripts/elements-native-executor.py','packages/vectors/fixtures/adapter-request-comparisons/CURRENT-REVISION-9'):
        path=destination/relative; path.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(repo/relative,path)
    if os.environ.get('CAPTURE_PIN_DRIFT'):
        pin=destination/'packages/vectors/fixtures/adapter-request-comparisons/CURRENT-REVISION-9'
        pin.write_text(pin.read_text().replace('0fcb8c7016bdd892671da8e64a331333bbc7665706949ed95f0bd9d2f7104c89', '0'*64))
elif a[0]=='rev-parse': print(repo)
elif a[2]=='checkout': (pathlib.Path(a[1])/'.git/HEAD').write_text(a[-1]+'\\n')
elif a[2]=='rev-parse': print('1'*40 if pathlib.Path(a[1])==repo else (pathlib.Path(a[1])/'.git/HEAD').read_text().strip())
elif a[2]=='status' and os.environ.get('CAPTURE_DIRTY_SOURCE'): print(' M source')
elif a[2]=='ls-files': print('Cargo.lock'+('\\nextra' if os.environ.get('CAPTURE_EXTRA_FILE') else ''))
''')
    git.chmod(0o755)
    podman = tools/'podman'
    podman.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
a=sys.argv[1:]
if a[0]=='image': print(os.environ['CAPTURE_IMAGE']); sys.exit(0)
if a[0] in ('stop','inspect','rm'): print('{}'); sys.exit(0)
if a[0]=='container': sys.exit(0 if os.environ.get('CAPTURE_STILL_PRESENT') else 1)
if 'cargo' in a:
    if os.environ.get('CAPTURE_BUILD_FAIL'): sys.exit(int(os.environ['CAPTURE_BUILD_FAIL']))
    mounts={}
    for i,arg in enumerate(a):
        if arg=='-v':
            host,guest,*_=a[i+1].split(':'); mounts[guest]=pathlib.Path(host)
    executable=mounts['/cargo-target']/'mock'; executable.write_text('#!/bin/sh\\nexit 0\\n'); executable.chmod(0o755)
    (mounts['/src']/'Cargo.lock').write_text('# synthetic resolved lock\\n')
    print(json.dumps(dict(reason='compiler-artifact',target=dict(name='keyed_fixed_capture'),profile=dict(test=True),executable='/cargo-target/mock')))
else:
    if not any(a[i] == '-e' and a[i+1].startswith('HOME=') and
               a[i+1] not in ('HOME=', 'HOME=/') for i in range(len(a)-1)):
        sys.exit(77)
    print('rustc synthetic\\ncargo synthetic\\nPython synthetic')
    print('a'*64+'  /usr/bin/python3')
    print('b'*64+'  /elements/bin/elementsd')
    print('c'*64+'  /elements/bin/elements-cli')
    print('Elements Core daemon version synthetic')
    print('d'*40)
    print('tip: '+'d'*40+' synthetic')
    print('b'*64+'  elementsd')
''')
    podman.chmod(0o755)
    command = [str(WRAPPERS/'capture-prepare.sh'), '1'*40]
    run('prepare-complete', [*command,'control'], env, lambda c:c==0, guard=PREPARATION_GUARD)
    measured=json.loads((root/'control/build/prepared.json').read_text())
    assert measured['trust']=='mock' and measured['framework']=='d'*40
    assert (root/'control/build/Cargo.lock').exists()
    for directory in (root/'control', root/'control/build'):
        import hashlib
        for line in (directory/'retention.sha256').read_text().splitlines():
            digest, name = line.split('  ',1)
            assert hashlib.sha256((directory/name).read_bytes()).hexdigest() == digest
    root_seal = (root/'control/retention.sha256').read_text()
    build_seal = root/'control/build/retention.sha256'
    assert hashlib.sha256(build_seal.read_bytes()).hexdigest() + '  build/retention.sha256\n' in root_seal
    for target in ('clauses','policy','histories','pilot','scaling'):
        assert '  '+target+'/capture.env\n' in root_seal

    run('prepare-cleanup-unverified', [*command,'unremoved'], dict(env,CAPTURE_STILL_PRESENT='1'), lambda c:c==74, guard=PREPARATION_GUARD)
    assert not (root/'unremoved/prepare.log').exists()
    run('prepare-image-drift', [*command,'drift'], dict(env,CAPTURE_IMAGE='wrong'), lambda c:c==65, guard=PREPARATION_GUARD)
    run('prepare-adapter-pin-drift', [*command,'pin-drift'], dict(env,CAPTURE_PIN_DRIFT='1'), lambda c:c!=0, guard=PREPARATION_GUARD)
    run('prepare-build-failure', [*command,'failed'], dict(env,CAPTURE_BUILD_FAIL='9'), lambda c:c==9, guard=PREPARATION_GUARD)
    assert (root/'failed/build/build.exit').read_text().strip()=='9'
    assert 'KFF_PREPARE_DONE run=control exit=0' in (root/'control/prepare.log').read_text()
    assert 'KFF_PREPARE_DONE run=failed exit=9' in (root/'failed/prepare.log').read_text()
    run('prepare-build-timeout', [*command,'timeout'], dict(env,CAPTURE_BUILD_FAIL='124'), lambda c:c==124, guard=PREPARATION_GUARD)
    assert (root/'timeout/build/build-stop.log').exists()
    assert (root/'timeout/build/build-final-state.json').exists()
    assert 'KFF_PREPARE_DONE run=timeout exit=124' in (root/'timeout/prepare.log').read_text()
    run('prepare-extra-untracked', [*command,'extra'], dict(env,CAPTURE_EXTRA_FILE='1'), lambda c:c!=0, guard=PREPARATION_GUARD)
    assert not (root/'extra/build/prepared.json').exists()
    run('prepare-dirty-checkout', [*command,'dirty'], dict(env,CAPTURE_DIRTY_SOURCE='1'), lambda c:c!=0, guard=PREPARATION_GUARD)
    run('prepare-wrong-commit', [command[0], '2'*40, 'wrong'], env, lambda c:c!=0, guard=PREPARATION_GUARD)
    executable(tools/'timeout', 'if [ "$1" = --kill-after=5 ] && [ "$2" = 60 ] && [ "$3" = python3 ]; then exit 124; fi\nexec /usr/bin/timeout "$@"\n')
    for name, extra, status, original in (('seal-only',{},74,0),('double-failure',{'CAPTURE_BUILD_FAIL':'9'},9,9)):
        run('prepare-'+name, [*command,name], dict(env,**extra), lambda c:c==status, guard=PREPARATION_GUARD)
        assert f'KFF_PREPARE_DONE run={name} exit={original} seal=failed\n' in (root/name/'prepare.log').read_text()


def s8c_campaign_controls(root, env):
    tools = root / 'tools'
    tools.mkdir()
    selected = dict(env, PATH=str(tools)+os.pathsep+env['PATH'],
                    TRIPOD_CAPTURE_STANDIN='1')
    run_name = 's8c-control'
    directory = root / run_name
    (directory/'build').mkdir(parents=True)
    for family in ('sponsor', 'legacy-copy'):
        (directory/family).mkdir()
    start = datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0)
    (root/(run_name+'.campaign-start.utc')).write_text(start.isoformat().replace('+00:00','Z')+'\n')
    (root/(run_name+'.campaign-deadline.utc')).write_text(
        (start+datetime.timedelta(seconds=6060)).isoformat().replace('+00:00','Z')+'\n')
    (directory/'prepare.log').write_text('')
    (directory/'campaign.log').write_text(f'KFF_CAMPAIGN_DONE run={run_name} exit=0\n')
    for item in ('retention.sha256','build/retention.sha256',
                 'sponsor/retention.sha256','legacy-copy/retention.sha256'):
        (directory/item).write_text(item+'\n')
    with (directory/'campaign-retention.sha256').open('w') as output:
        for item in ('retention.sha256','build/retention.sha256',
                     'sponsor/retention.sha256','legacy-copy/retention.sha256'):
            output.write(hashlib.sha256((directory/item).read_bytes()).hexdigest()+'  '+item+'\n')
    executable(tools/'systemctl',
               'case "$*" in *ActiveState*) printf "KFF_PREPARE_DONE run=s8c-control exit=0\\n" >>"$CAPTURE_RUN_DIR/prepare.log"; echo inactive;; *) echo 0;; esac\n')
    waiter = [str(WRAPPERS/'capture-unit.sh'), '--wait-campaign', str(directory)]
    run('s8c-marker-race', waiter, dict(selected,CAPTURE_RUN_DIR=str(directory)), lambda c:c==0)
    assert (directory/'prepare.log').read_text().count('KFF_PREPARE_DONE') == 1
    executable(tools/'systemctl', 'case "$*" in *ActiveState*) echo inactive;; *) echo 0;; esac\n')
    (directory/'prepare.log').write_text('')
    run('s8c-missing-marker', waiter, selected, lambda c:c==74)
    (directory/'prepare.log').write_text(f'KFF_PREPARE_DONE run={run_name} exit=0\n')
    (directory/'sponsor/unit.log').write_text('KFF_CAPTURE_DONE target=sponsor exit=0\n')
    executable(tools/'podman', 'if [ "$1" = container ]; then exit 0; fi\nexit 0\n')
    seal = [str(WRAPPERS/'capture-unit.sh'), '--seal', str(directory/'sponsor'),
            'sponsor', '1800', IMAGE]
    run('s8c-live-writer', seal, selected, lambda c:c==74)
    executable(tools/'systemctl', 'case "$*" in *ActiveState*) echo active;; *) echo 0;; esac\n')
    (root/(run_name+'.campaign-deadline.utc')).write_text('2000-01-01T00:00:00Z\n')
    run('s8c-old-deadline', [str(WRAPPERS/'capture-unit.sh'), '--check-campaign', str(directory)],
        selected, lambda c:c not in (0,75))

    # Exercise the parent in its own process with stand-in preparation and
    # family children. The earlier cases isolate the waiter's refusal edges.
    unit_root = root/'unit-root'
    unit_root.mkdir()
    checkout = root/'checkout'
    wrapper_dir = checkout/'scripts/keyed-fixed-capture'
    wrapper_dir.mkdir(parents=True)
    executable(wrapper_dir/'capture-prepare.sh',
               'root="$TRIPOD_CAPTURE_TEST_ROOT/$2"\n'
               'mkdir -p "$root/build" "$root/sponsor" "$root/legacy-copy"\n'
               'printf "KFF_PREPARE_DONE run=%s exit=0\\n" "$2" >"$root/prepare.log"\n'
               'printf "root\\n" >"$root/retention.sha256"\n'
               'printf "build\\n" >"$root/build/retention.sha256"\n')
    executable(wrapper_dir/'capture-unit.sh',
               'if [ "$1" = --seal ]; then\n'
               '  printf "%s\\n" "$3" >"$2/retention.sha256"\n'
               'else\n'
               '  printf "admitted\\n" >"$1/admitted"\n'
               '  printf "KFF_CAPTURE_DONE target=%s exit=0\\n" "$2"\n'
               'fi\n')
    executable(tools/'systemctl',
               'case "$*" in *ActiveState*) '
               'if grep -q "^KFF_CAMPAIGN_DONE " '
               '"$TRIPOD_CAPTURE_TEST_ROOT/s8c-unit/campaign.log" 2>/dev/null; '
               'then echo inactive; else echo active; fi;; *) echo 0;; esac\n')
    unit_env = dict(selected, TRIPOD_CAPTURE_TEST_ROOT=str(unit_root))
    run('s8c-detached-parent',
        [str(WRAPPERS/'capture-unit.sh'), '--campaign', str(checkout),
         'a'*40, 's8c-unit', IMAGE], unit_env, lambda c:c==0)
    run('s8c-detached-waiter',
        [str(WRAPPERS/'capture-unit.sh'), '--wait-campaign', str(unit_root/'s8c-unit')],
        unit_env, lambda c:c==0)
    assert (unit_root/'s8c-unit/campaign-retention.sha256').is_file()
    assert (unit_root/'s8c-unit/campaign.log').read_text().count('KFF_CAMPAIGN_DONE') == 1

    legacy_env = dict(selected, TRIPOD_CAPTURE_TEST_ROOT=str(root))
    passing = ('printf "running 1 test\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0s\\n"\n'
               'printf "{}\\n" >"$TRIPOD_CAPTURE_TEST_ROOT/exchange-00001-request.json"\n'
               'printf "{}\\n" >"$TRIPOD_CAPTURE_TEST_ROOT/exchange-00001-response.json"\n'
               'printf "%s\\n" \'{"fields":{"message":"fixed-form native exchange retained","prefix":"exchange-00001","wall_us":7}}\' >"$TRIPOD_CAPTURE_TEST_ROOT/phase-timing.jsonl"\n')
    missing_timing = ''.join(passing.splitlines(keepends=True)[:-1])
    for name, body, expected in (('s8c-legacy-timing-control', passing, lambda c:c==0),
                                 ('s8c-legacy-missing-timing', missing_timing, lambda c:c!=0)):
        executable(root/'keyed_fixed_capture', body)
        for item in ('test.exit', 'selection.exit', 'exchange-00001-request.json',
                     'exchange-00001-response.json', 'phase-timing.jsonl'):
            (root/item).unlink(missing_ok=True)
        run(name, [str(WRAPPERS/'run-one.sh'), 'legacy-copy'], legacy_env, expected)


def main():
    started = time.monotonic()
    # tempfile chooses an exclusively created root, never a clock-derived name.
    with tempfile.TemporaryDirectory(prefix="keyed-capture-wrappers-") as directory:
        root = Path(directory)
        env = os.environ.copy()
        for name, test in (("artifacts", artifact_controls), ("selection", selection_controls),
                           ("units", unit_controls), ("preparation", preparation_controls),
                           ("s8c", s8c_campaign_controls)):
            child = root / name
            child.mkdir()
            test(child, env)
    print(f"capture-wrapper family wall={time.monotonic()-started:.6f}s guard={FAMILY_GUARD}s", flush=True)


if __name__ == "__main__":
    main()
