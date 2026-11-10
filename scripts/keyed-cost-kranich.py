#!/usr/bin/env python3
"""Proposed Wave-1 engine cost campaign. Node only; dry run is not a cost result.

No engine code is changed for runtime samples. The identical measurement-only
unit module is installed in both scratch clones. The baseline has no keyed plan
entry; only the candidate enables the harness's cost-keyed feature.
"""
import argparse
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import sys
import time
import tomllib

BASE = "0.7.96-dev"
HARNESS = "packages/compiler/tests/measurement/engine_cost.rs"
INSTALLED = "packages/compiler/src/tests/engine_cost_measurement.rs"
PIN = "packages/compiler/tests/fixtures/engine_base/MANIFEST"
EDIT = "packages/compiler/src/engine_attestation/source.rs"
EDIT_LINE = b"\nconst _: () = (); // measurement-only reversible lowering edit\n"
TEST = "tests::engine_cost_measurement::measure_engine_subject"
SUBJECTS = [f"{kind}-{scope}" for kind in ("realization", "compiler") for scope in ("C", "T", "CT", "A", "CTA")]
KEYED = ["keyed-SOURCE", "keyed-MERGE", "keyed-joint"]


def output(*args):
    print(*args, flush=True)


def command(argv, **kwargs):
    return subprocess.check_output(argv, text=True, **kwargs).strip()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def metrics():
    relative = next(line.split(":", 2)[2] for line in Path("/proc/self/cgroup").read_text().splitlines() if line.startswith("0::"))
    root = Path("/sys/fs/cgroup") / relative.lstrip("/")
    data = {f"cpu.{key}": int(value) for key, value in (line.split() for line in (root / "cpu.stat").read_text().splitlines())}
    for resource in ("cpu", "memory", "io"):
        for line in (root / f"{resource}.pressure").read_text().splitlines():
            fields = line.split()
            data[f"{resource}.{fields[0]}.stall_us"] = int(next(v[6:] for v in fields if v.startswith("total=")))
    return data


def delta(start, end):
    return {key: end[key] - value for key, value in start.items()}


def govern(args):
    """Use slot 1 released by the node governor; retain its lock while waiting on the unit."""
    root = Path.home() / "loops/locks"
    queue, slots = root / "code-gate-queue", root / "code-gate-slots"
    queue.mkdir(parents=True, exist_ok=True)
    slots.mkdir(parents=True, exist_ok=True)
    ticket = queue / f"{time.time_ns()}-w20b1-cost-{os.getpid()}"
    ticket.touch(exist_ok=False)
    held = []
    start = time.monotonic()
    try:
        while not held:
            for entry in queue.iterdir():
                try:
                    os.kill(int(entry.name.rsplit("-", 1)[1]), 0)
                except ProcessLookupError:
                    entry.unlink(missing_ok=True)
                except (ValueError, PermissionError):
                    pass
            if sorted(queue.iterdir())[0] == ticket:
                attempt = []
                try:
                    for index in (1,):
                        handle = (slots / f"slot-{index}").open("a")
                        attempt.append(handle)
                        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    held = attempt
                except BlockingIOError:
                    for handle in attempt:
                        handle.close()
            if not held:
                output(f"== queued exclusive tripod battery waited={time.monotonic()-start:.1f}s")
                time.sleep(15)
        ticket.unlink()
        wait = time.monotonic() - start
        output(f"== governed slot 1 reserved wait={wait:.3f}s")
        budget = 600 if args.dry_run else 14400
        if args.resume_preflight:
            root = Path.home() / "loops/evidence/wave1/keyed-cost" / args.run
            failures = [root / "failure.json", *root.glob("preflight-history/*/failure.json")]
            if not args.dry_run or not failures[0].exists() or list(root.glob("sample-*")):
                raise RuntimeError("preflight resume requires a dry run that failed before any sample")
            budget -= math.ceil(sum(json.loads(path.read_text())["wall_seconds"] for path in failures))
            if budget <= 0:
                raise RuntimeError("cumulative dry-run budget exhausted")
        argv = ["systemd-run", "--user", "--quiet", "--wait", "--pipe", "--collect",
                f"--unit=tripod-keyed-cost-{args.run}", f"--property=RuntimeMaxSec={budget}",
                "--property=KillMode=control-group", "-E", f"HOME={Path.home()}", "-E", f"PATH={os.environ['PATH']}",
                sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], "--worker", "--slot-wait", str(wait)]
        return subprocess.call(argv)
    finally:
        ticket.unlink(missing_ok=True)
        for handle in held:
            handle.close()


class Campaign:
    def __init__(self, args):
        self.args = args
        self.loops = Path.home() / "loops"
        self.result = self.loops / "evidence/wave1/keyed-cost" / args.run
        resumed = 0
        if args.resume_preflight:
            history = self.result / "preflight-history"
            history.mkdir(exist_ok=True)
            resumed = len(list(history.iterdir())) + 1
            archive = history / str(resumed)
            archive.mkdir()
            for path in self.result.iterdir():
                if path != history:
                    path.rename(archive / path.name)
        else:
            self.result.mkdir(parents=True, exist_ok=False)
        self.work = self.loops / "tmp/keyed-cost" / args.run
        if resumed:
            self.work = self.work / f"resume-{resumed}"
        self.work.mkdir(parents=True, exist_ok=False)
        self.toolchain = Path("/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu")
        self.image = command(["podman", "image", "inspect", "--format", "{{.Id}}", "localhost/tripod-doc:f44"])
        self.cpus = sorted(os.sched_getaffinity(0))[:4]
        self.events = []
        script = Path(__file__).read_bytes()
        (self.result / "runner.py").write_bytes(script)
        self.clones = {}
        self.executables = {}
        self.targets = {}
        self.started = time.monotonic()
        self.prior_seconds = sum(json.loads(path.read_text())["wall_seconds"] for path in self.result.glob("preflight-history/*/failure.json"))
        self.manifest = {"schema": 1, "mode": "dry-run" if args.dry_run else "campaign", "baseline": BASE,
                         "candidate_ref": args.candidate, "image": self.image, "image_label": "localhost/tripod-doc:f44",
                         "toolchain_path": str(self.toolchain), "rustflags": "", "profile": "gate", "incremental": "1",
                         "cpus": self.cpus, "jobs": 4, "slot_policy": "slot 1 under slot-governor.sh; slots 2-4 held by governor",
                         "slot_wait_seconds": args.slot_wait, "source_mount": "/src", "target_mount": "/cargo-target",
                         "packages": ["tripod-compiler", "tripod-architecture", "tripod-realization"],
                         "cache_policy": "isolated per-side targets; dry-run reflink seed only; full builds start empty",
                         "baseline_keyed_scopes": "absent; never substituted by record oracle", "samples": [], "runner_sha256": digest(script),
                         "edit": {"file": EDIT, "append_utf8": EDIT_LINE.decode(), "restoration": "original bytes in finally"}}

    def run(self, argv, label, *, cwd=None):
        log = self.result / f"{label}.log"
        start, began = metrics(), time.monotonic_ns()
        with log.open("w") as stream:
            rc = subprocess.call(argv, stdout=stream, stderr=subprocess.STDOUT, cwd=cwd)
        end = metrics()
        event = {"label": label, "argv": argv, "cwd": str(cwd) if cwd else None, "rc": rc,
                 "wall_ns": time.monotonic_ns()-began, "start": start, "end": end, "delta": delta(start, end), "log": log.name}
        self.events.append(event)
        save(self.result / "commands.json", self.events)
        output(f"== {label} rc={rc} wall={event['wall_ns']/1e9:.3f}s")
        if rc:
            output(log.read_text())
            raise RuntimeError(f"{label} failed: {log}")
        return event

    def container(self, side, target, env=()):
        clone = self.clones[side]
        build = self.work / f"build-{side}"
        build.mkdir(exist_ok=True)
        target.mkdir(parents=True, exist_ok=True)
        return ["podman", "run", "--rm", "--userns=keep-id", "--cgroups=split",
                "-v", f"{clone}:/src:z", "-v", f"{target}:/cargo-target:z",
                "-v", f"{self.loops / 'cargo-home'}:/cargo:z", "-v", f"{build}:/build:z",
                "-v", f"{self.result}:/result:z", "-v", "/usr/local/rustup:/usr/local/rustup:ro",
                "-e", "RUSTUP_HOME=/usr/local/rustup", "-e", "CARGO_HOME=/cargo", "-e", "CARGO_TARGET_DIR=/cargo-target",
                "-e", "HOME=/build", "-e", "TMPDIR=/build", "-e", "RUSTFLAGS=", "-e", "CARGO_INCREMENTAL=1",
                "-e", "CARGO_BUILD_JOBS=4", "-e", f"PATH={self.toolchain}/bin:/usr/bin:/bin",
                *sum((["-e", item] for item in env), []), "-w", "/src", self.image, "taskset", "-c", ",".join(map(str, self.cpus))]

    def prepare(self):
        for side, ref in [("baseline", BASE), ("candidate", self.args.candidate)]:
            clone = self.work / side
            self.run(["git", "clone", "--quiet", "--no-checkout", str(Path.home()/"git/tripod.git"), str(clone)], f"clone-{side}")
            self.run(["git", "fetch", "--quiet", "origin", ref], f"fetch-{side}", cwd=clone)
            self.run(["git", "checkout", "--quiet", "--detach", "FETCH_HEAD"], f"checkout-{side}", cwd=clone)
            self.clones[side] = clone
            self.manifest[side] = {"commit": command(["git", "rev-parse", "HEAD"], cwd=clone),
                                   "tree": command(["git", "rev-parse", "HEAD^{tree}"], cwd=clone)}
        source = (self.clones["candidate"] / HARNESS).read_bytes()
        (self.result / "engine_cost.rs").write_bytes(source)
        self.manifest["harness_sha256"] = digest(source)
        baseline_pin = (self.clones["baseline"] / PIN).read_bytes()
        if baseline_pin != (self.clones["candidate"] / PIN).read_bytes():
            raise RuntimeError("preservation manifest changed")
        (self.result / "MANIFEST").write_bytes(baseline_pin)
        for side, clone in self.clones.items():
            (clone / INSTALLED).write_bytes(source)
            with (clone / "packages/compiler/src/tests/mod.rs").open("a") as stream:
                stream.write("\nmod engine_cost_measurement;\n")
            cargo = clone / "packages/compiler/Cargo.toml"
            text = cargo.read_text()
            if text.count("[features]") != 1:
                raise RuntimeError("unexpected feature table")
            cargo.write_text(text.replace("[features]", '[features]\ncost-keyed = ["keyed-experimental"]'))
            self.manifest[side]["runtime_features"] = ["cost-keyed"] if side == "candidate" else []
            self.targets[side] = self.work / f"target-{side}-runtime"
            if self.args.dry_run:
                self.run(["cp", "-a", "--reflink=always", str(self.loops/"cargo-target-lanes/w20b1"), str(self.targets[side])], f"seed-{side}")
            else:
                self.targets[side].mkdir()
            original = (clone / EDIT).read_bytes()
            try:
                (clone / EDIT).write_bytes(original + EDIT_LINE)
                if (clone / EDIT).read_bytes() != original + EDIT_LINE:
                    raise RuntimeError("edit verification")
            finally:
                (clone / EDIT).write_bytes(original)
            self.manifest[side]["edit_original_sha256"] = digest(original)
        self.run(self.container("candidate", self.targets["candidate"]) + ["rustc", "--version", "--verbose"], "toolchain")
        self.manifest["toolchain"] = (self.result/"toolchain.log").read_text()
        dependencies = {}
        for side in ("baseline", "candidate"):
            self.run(self.container(side, self.targets[side]) + ["cargo", "generate-lockfile"], f"lockfile-{side}")
            lock = (self.clones[side] / "Cargo.lock").read_bytes()
            (self.result / f"Cargo-{side}.lock").write_bytes(lock)
            self.manifest[side]["cargo_lock_sha256"] = digest(lock)
            dependencies[side] = {(p["name"], p["version"], p.get("source", ""), p.get("checksum", "")) for p in tomllib.loads(lock.decode())["package"] if "source" in p}
        if dependencies["baseline"] != dependencies["candidate"]:
            raise RuntimeError("external dependency lock sets differ")
        save(self.result / "manifest.json", self.manifest)

    def build_runtime(self, side):
        argv = self.container(side, self.targets[side]) + ["cargo", "test", "--locked", "-p", "tripod-compiler", "--lib", "--profile", "gate", "--no-run", "--message-format=json"]
        if side == "candidate":
            argv += ["--features", "cost-keyed"]
        self.run(argv, f"runtime-build-{side}")
        for line in (self.result/f"runtime-build-{side}.log").read_text().splitlines():
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if record.get("reason") == "compiler-artifact" and record.get("executable") and record["target"]["name"] == "compiler":
                self.executables[side] = record["executable"]
        if side not in self.executables:
            raise RuntimeError("compiler test executable not found")

    def sample(self, side, repetition, subject):
        label = f"sample-{repetition}-{side}-{subject}"
        env = [f"TRIPOD_COST_SUBJECT={subject}", f"TRIPOD_COST_RESULT=/result/{label}.tsv", "TRIPOD_COST_CGROUP=/sys/fs/cgroup"]
        self.run(self.container(side, self.targets[side], env) + [self.executables[side], "--ignored", "--exact", TEST, "--test-threads=1", "--nocapture"], label)
        result = dict(line.split("=", 1) for line in (self.result/f"{label}.tsv.result").read_text().splitlines())
        phases = []
        for line in (self.result/f"{label}.tsv").read_text().splitlines()[1:]:
            phase, wall, start, end = line.split("\t")
            start, end = json.loads(start), json.loads(end)
            phases.append({"phase": phase, "wall_ns": int(wall), "start": start, "end": end, "delta": delta(start, end)})
        if subject.startswith("keyed-") and sum(p["phase"] == "projection_nested" for p in phases) != 2:
            raise RuntimeError("keyed plan must publish twice: producer and independent validator")
        inclusive = next(p for p in phases if p["phase"] == "derivation_inclusive")
        nested = [p for p in phases if p["phase"] == "projection_nested"]
        exclusive = {"wall_ns": inclusive["wall_ns"] - sum(p["wall_ns"] for p in nested),
                     "delta": {key: value - sum(p["delta"][key] for p in nested) for key, value in inclusive["delta"].items()},
                     "definition": "inclusive derivation minus nested publication intervals; includes instrumentation overhead"}
        row = {"side": side, "repetition": repetition, "warmup": repetition == "warmup", "subject": subject,
               "result": result, "phases": phases, "derivation_excluding_nested_publication": exclusive}
        self.manifest["samples"].append(row)
        save(self.result / f"{label}.json", row)
        save(self.result / "manifest.json", self.manifest)

    def builds(self):
        if self.args.dry_run:
            self.manifest["build_matrix"] = "not run in dry-run; seeded runtime builds only; edit apply/restore verified"
            return
        for repetition in range(4):
            order = ("baseline", "candidate") if repetition % 2 == 0 else ("candidate", "baseline")
            for side in order:
                for feature in ("default", "keyed-experimental"):
                    target = self.work / f"build-target-{side}-{feature}"
                    if target.exists():
                        shutil.rmtree(target)
                    argv = self.container(side, target) + ["cargo", "build", "--locked", "-p", "tripod-architecture", "-p", "tripod-realization", "-p", "tripod-compiler", "--lib", "--profile", "gate"]
                    if feature != "default":
                        argv += ["--features", "tripod-compiler/keyed-experimental"]
                    label = f"build-{repetition}-{side}-{feature}"
                    self.run(argv, label + "-clean")
                    source = self.clones[side] / EDIT
                    original = source.read_bytes()
                    try:
                        source.write_bytes(original + EDIT_LINE)
                        patch = command(["git", "diff", "--", EDIT], cwd=self.clones[side])
                        (self.result / f"{label}.patch").write_text(patch + "\n")
                        self.run(argv, label + "-incremental")
                    finally:
                        source.write_bytes(original)
                    if digest(source.read_bytes()) != self.manifest[side]["edit_original_sha256"]:
                        raise RuntimeError("edit not restored")
        self.manifest["build_matrix"] = "one warmup (0), three measured repetitions; alternating side order; both feature states"

    def execute(self):
        try:
            self.prepare()
            for side in ("baseline", "candidate"):
                self.build_runtime(side)
            repetitions = ["warmup", "1"] if self.args.dry_run else ["warmup", "1", "2", "3"]
            for number, repetition in enumerate(repetitions):
                order = ("baseline", "candidate") if number % 2 == 0 else ("candidate", "baseline")
                for side in order:
                    subjects = ["realization-C"] if self.args.dry_run else SUBJECTS + (KEYED if side == "candidate" else [])
                    for subject in subjects:
                        self.sample(side, repetition, subject)
            self.builds()
            distributions = {}
            for sample in self.manifest["samples"]:
                if sample["warmup"]:
                    continue
                for phase in sample["phases"]:
                    key = f"{sample['side']}/{sample['subject']}/{phase['phase']}"
                    distributions.setdefault(key, []).append({"wall_ns": phase["wall_ns"], **phase["delta"]})
            distributions = {key: {metric: {"raw": [row[metric] for row in rows],
                "min": min(row[metric] for row in rows), "median": statistics.median(row[metric] for row in rows),
                "max": max(row[metric] for row in rows)} for metric in rows[0]} for key, rows in distributions.items()}
            save(self.result / "distributions.json", distributions)
            summary = {"schema": 1, "mode": self.manifest["mode"], "sample_count": len(self.manifest["samples"]),
                       "all_checked_pins_equal": True, "baseline_keyed": "absent", "full_campaign": not self.args.dry_run,
                       "wall_seconds": self.prior_seconds + time.monotonic()-self.started, "this_unit_wall_seconds": time.monotonic()-self.started, "prior_preflight_seconds": self.prior_seconds, "slot_wait_seconds": self.args.slot_wait,
                       "artifact_path": str(self.result), "performance_claim": "none"}
            save(self.result / "manifest.json", self.manifest)
            save(self.result / "summary.json", summary)
            output(json.dumps(summary, sort_keys=True))
            output("== dry-run green" if self.args.dry_run else "== campaign complete; review required")
        except BaseException as error:
            save(self.result / "failure.json", {"error": str(error), "wall_seconds": time.monotonic()-self.started})
            raise
        finally:
            output(f"== end artifacts={self.result}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", required=True)
    parser.add_argument("--candidate", default="refs/lanes/w20b1")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--resume-preflight", action="store_true", help="resume only a failed dry preflight with no samples; shares the 600-second budget")
    parser.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--slot-wait", type=float, default=0, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not re.fullmatch(r"[a-zA-Z0-9-]{1,64}", args.run):
        parser.error("run must be 1-64 alphanumeric or hyphen characters")
    if not args.worker:
        return govern(args)
    if not os.environ.get("INVOCATION_ID"):
        parser.error("worker requires its systemd unit; use the governor entry")
    Campaign(args).execute()
    return 0


if __name__ == "__main__":
    sys.exit(main())
