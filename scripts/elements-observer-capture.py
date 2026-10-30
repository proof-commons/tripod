#!/usr/bin/env python3
"""One capture coordinator with separate producer and public observer children."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def run(argv, *, data=None, env=None, cwd=None):
    result = subprocess.run(argv, input=data, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env, cwd=cwd, check=False)
    if result.returncode:
        sys.stderr.write(result.stdout + result.stderr)
        raise RuntimeError("child failed (%d): %s" % (result.returncode, argv[0]))
    return result


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def submitted_bytes(capture_text):
    subjects = [bytes.fromhex(line.split()[2]) for line in capture_text.splitlines()
                if line.startswith("request-bytes ") and line.split()[1] != "0"]
    if len(subjects) != 1 or len(subjects[0]) != 1701:
        raise RuntimeError("one exact sponsorless subject required")
    return subjects[0]


def reference_bytes(root):
    return submitted_bytes((root / "packages/vectors/fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture").read_text())


def namespace_probe(directory):
    directory.mkdir(parents=True, exist_ok=False)
    command = shutil.which("bwrap")
    if command:
        result = subprocess.run([command, "--ro-bind", "/", "/", "--unshare-user", "--", "/bin/true"],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=False)
        record = {"available": result.returncode == 0, "exit": result.returncode,
                  "diagnostic": result.stderr, "command": command}
    else:
        record = {"available": False, "exit": None, "diagnostic": "bwrap unavailable", "command": None}
    if not record["available"]:
        fallback = shutil.which("unshare")
        if fallback:
            result = subprocess.run([fallback, "--user", "--map-root-user", "--mount", "/bin/true"],
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=False)
            record = {"available": result.returncode == 0, "exit": result.returncode,
                      "diagnostic": record["diagnostic"].rstrip("\n") + "\n" + result.stderr, "command": fallback,
                      "mode": "unshare"}
    (directory / "result.json").write_text(canonical(record) + "\n")
    return record


def binaries(root):
    result = run(["cargo", "test", "-p", "tripod-vectors", "--test", "maturity_native",
                  "--test", "maturity_observer", "--no-run", "--message-format=json"], cwd=root)
    found = {}
    for line in result.stdout.splitlines():
        item = json.loads(line)
        if item.get("reason") == "compiler-artifact" and item.get("executable"):
            found[item["target"]["name"]] = item["executable"]
    if set(found) != {"maturity_native", "maturity_observer"}:
        raise RuntimeError("two exact executable targets required")
    return found


def namespace_prefix(record, excluded, temporary):
    if not record["available"]:
        return []
    if record.get("mode") == "unshare":
        import shlex
        empty = temporary / "excluded"
        empty.mkdir(exist_ok=True)
        commands = ["mount --bind " + shlex.quote(str(empty)) + " " + shlex.quote(str(path)) for path in excluded]
        commands.append('exec "$@"')
        return [record["command"], "--user", "--map-root-user", "--mount", "/bin/sh", "-ec", "; ".join(commands), "observe"]
    command = [record["command"], "--ro-bind", "/", "/", "--bind", str(temporary), str(temporary), "--unshare-user"]
    for path in excluded:
        command += ["--tmpfs", str(path)]
    return command + ["--"]


def capture(args):
    started = time.monotonic()
    phase_started = started
    walls = []
    def phase(name, child_started=None):
        nonlocal phase_started
        now = time.monotonic()
        walls.append((name, int((now - (phase_started if child_started is None else child_started)) * 1000)))
        print("phase %s wall-ms %d" % walls[-1], flush=True)
        phase_started = now
    root = Path(__file__).resolve().parent.parent
    output = Path(args.output).resolve()
    if output.exists():
        raise RuntimeError("capture address already exists")
    output.mkdir(parents=True)
    temporary = Path(tempfile.mkdtemp(prefix="maturity-observer-", dir=os.environ["TMPDIR"]))
    chain = temporary / "chain"
    producer_dir = temporary / "producer"
    producer_dir.mkdir()
    chain.mkdir()
    exits = {}
    try:
        if run(["git", "status", "--porcelain", "--untracked-files=normal"], cwd=root).stdout:
            raise RuntimeError("clean source required")
        revision = run(["git", "rev-parse", "HEAD"], cwd=root).stdout.strip()
        tree = run(["git", "rev-parse", "HEAD^{tree}"], cwd=root).stdout.strip()
        short = run(["git", "rev-parse", "--short", "HEAD"], cwd=root).stdout.strip()
        required = ["TRIPOD_LIVE_EXECUTOR", "TRIPOD_LIVE_NETWORK_ID", "TRIPOD_LIVE_GENESIS_ID",
                    "ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD", "ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI",
                    "ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK", "ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP"]
        if any(not os.environ.get(name) for name in required):
            raise RuntimeError("missing declared execution environment")
        isolation = namespace_probe(Path(args.namespace_record).parent)
        phase("preflight")
        built = binaries(root)
        phase("build")
        launcher = temporary / "producer-executor.sh"
        import shlex
        launcher.write_text("#!/bin/sh\nexec " + shlex.quote(os.environ["TRIPOD_LIVE_EXECUTOR"]) +
                            " --datadir " + shlex.quote(str(chain)) + " --wallet-name observer-producer \"$@\"\n")
        launcher.chmod(0o700)
        environment = dict(os.environ)
        environment.update(TRIPOD_LIVE_EXECUTOR=str(launcher), TRIPOD_MATURITY_CAPTURE_PROFILE="observer",
                           TRIPOD_LIVE_REPORT_DIR=str(producer_dir), TRIPOD_LIVE_SUITE_COMMIT=revision,
                           TRIPOD_LIVE_SUITE_TREE=tree, TRIPOD_LIVE_SUITE_SHORT_SHA=short,
                           ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY="node-defaults",
                           ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION="1")
        child_started = time.monotonic()
        producer = run([built["maturity_native"], "--ignored", "--exact",
                        "the_maturity_announcement_runs_against_a_real_target", "--nocapture", "--test-threads=1"],
                       env=environment, cwd=temporary)
        exits["producer"] = producer.returncode
        phase("producer", child_started)
        (output / "producer.log").write_text(producer.stdout + producer.stderr)
        manifests = [line.split("OBSERVER-MANIFEST ", 1)[1] for line in producer.stdout.splitlines() if "OBSERVER-MANIFEST " in line]
        if len(manifests) != 1:
            raise RuntimeError("one public manifest required")
        manifest = json.loads(manifests[0])
        wallets = chain / "elementsregtest" / "wallets"
        if not wallets.is_dir():
            raise RuntimeError("producer wallet directory absent before removal")
        shutil.rmtree(wallets)
        if wallets.exists():
            raise RuntimeError("producer wallet survived")
        observer = temporary / "observer"
        shutil.copy2(built["maturity_observer"], observer)
        helper = temporary / "read.py"
        adapter = temporary / "adapter.py"
        shutil.copy2(root / "scripts/elements-observer-read.py", helper)
        shutil.copy2(root / "scripts/elements-native-executor.py", adapter)
        excluded = [root, producer_dir, output]
        prefix = namespace_prefix(isolation, excluded, temporary)
        public = {name: manifest[name] for name in ("chain_name", "genesis_id", "transaction_id")}
        helper_env = {"PATH": os.environ["PATH"], "TMPDIR": str(temporary), "PYTHONDONTWRITEBYTECODE": "1"}
        observer_env = {}
        isolation_record = {"probe": isolation, "namespace": "available" if isolation["available"] else "unavailable",
                            "layer": "construction+namespace" if isolation["available"] else "construction",
                            "excluded_paths": "unopenable" if isolation["available"] else "unnamed",
                            "observer_arguments": [], "observer_environment": observer_env,
                            "producer_wallet_survived": wallets.exists(), "child_exits":exits}
        if isolation["available"]:
            checks = run(prefix + ["/bin/sh", "-c", "test ! -e \"$1/Cargo.toml\" && test ! -e \"$2/" + short + ".report.capture\"", "check", str(root), str(producer_dir)], cwd=temporary)
            isolation_record["exclusion_check_exit"] = checks.returncode
        child_started = time.monotonic()
        acquired = run(prefix + [sys.executable, str(helper), "--adapter", str(adapter), "--elementsd",
                                 os.environ["ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD"], "--elements-cli",
                                 os.environ["ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI"], "--chain-directory", str(chain)],
                       data=canonical(public), env=helper_env, cwd=temporary)
        exits["acquisition"] = acquired.returncode
        phase("acquisition", child_started)
        reading = json.loads(acquired.stdout)
        child_started = time.monotonic()
        answer = run(prefix + [str(observer)], data=canonical({"manifest":manifest,"reading":reading}),
                     env=observer_env, cwd=temporary)
        exits["observer"] = answer.returncode
        observation = json.loads(answer.stdout)
        phase("observer", child_started)
        captures = list(producer_dir.glob("*.capture"))
        if len(captures) != 1 or len(observation["experiment"]) != 7:
            raise RuntimeError("capture or experiment census disagrees")
        producer_capture = captures[0].read_text()
        if submitted_bytes(producer_capture) != bytes.fromhex(reading["raw_transaction"]) or submitted_bytes(producer_capture) != reference_bytes(root):
            raise RuntimeError("exact variable subject disagrees")
        details = [json.loads(bytes.fromhex(line.split()[2])) for line in producer_capture.splitlines()
                   if line.startswith("response-detail ") and line.split()[1] != "0"]
        routes = [detail.get("route") for detail in details if isinstance(detail, dict)]
        if "direct-block" not in routes:
            raise RuntimeError("direct-block route not observed")
        record = {"schema":1,"suite_commit":revision,"suite_tree":tree,"producer_capture":producer_capture,
                  "observation":observation,"isolation":isolation_record,"route":"relay refusal then direct block"}
        name = short + ".report.capture"
        (output / name).write_text(canonical(record) + "\n")
        (output / (name + ".timing")).write_text("timing-schema 1\nceremony-id report\nstatus passed\n")
        phase("census")
        manifest_text = "".join(digest((output / n).read_bytes()) + "  " + n + "\n" for n in [name,name+".timing"])
        (output / "MANIFEST.sha256").write_text(manifest_text)
        phase("manifest")
        elapsed = int((time.monotonic()-started)*1000)
        coordinator = elapsed - sum(duration for _, duration in walls)
        if coordinator < 0:
            raise RuntimeError("phase walls exceed elapsed wall")
        walls.append(("coordinator", coordinator))
        report = ["run-report-schema maturity-observer-run-report 1", "suite-commit " + revision,
                  "suite-tree " + tree, "suite-clean yes", "protocol-revision 8", "capture-profile observer"]
        report += ["phase-wall-ms %s %d" % wall for wall in walls]
        report += ["elapsed-wall-ms %d" % elapsed]
        report += ["%s-exit %d" % (child, exits[child]) for child in ("producer", "acquisition", "observer")]
        report += ["experiment-count 7",
                   "ceremony report " + digest((output/name).read_bytes()),
                   "manifest-sha256 " + digest(manifest_text.encode()), "eligible yes"]
        (output / "RUN-REPORT").write_text("\n".join(report)+"\n")
        diagnostics = output / "diagnostics"
        diagnostics.mkdir()
        shutil.move(output / "producer.log", diagnostics / "producer.log")
        (diagnostics / "acquisition.log").write_text(acquired.stderr)
        (diagnostics / "isolation.json").write_text(canonical(isolation_record)+"\n")
        for path in output.rglob("*"):
            path.chmod(0o555 if path.is_dir() else 0o444)
        output.chmod(0o555)
    finally:
        shutil.rmtree(temporary)
        print("capture-wall-ms %d" % int((time.monotonic()-started)*1000), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output")
    parser.add_argument("--namespace-record", required=True)
    args = parser.parse_args()
    try:
        capture(args)
    except Exception as error:
        print("== observer RED: " + str(error), flush=True)
        return 1
    print("== observer green", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
