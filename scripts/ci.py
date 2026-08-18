#!/usr/bin/env python3
"""Repository CI driver (ADR-011), with a typed lane tracker and wall times.

A clean checkout must pass this program without modifying tracked files.
Runner-agnostic: invoke it from any CI system or locally, through
`scripts/ci.sh`, which is a thin shim over this file.

Cadence: the FULL run is reserved for tags and other important moments
(releases, identity re-pins, review baselines). Routine batch merges run
the light gate — every lane except the two test suites, whose cost the
per-commit worker lanes already paid:

    scripts/ci.py --skip test-debug --skip test-release

A narrowed run reports "partial" by design; the merge record cites it.

Lanes
-----
  1. rustfmt          cargo fmt --all --check
  2. clippy           -D warnings, all targets, --locked
  3. tests (debug)    meson test --suite cargo-debug (per-package lanes)
  4. tests (release)  meson test --suite cargo-release (per-package lanes)
  5. generated        check-generated (non-writing stale-artifact gate)
  6. labels           check-labels (non-writing label gate)
  7. advisories       cargo audit (skipped loudly when not installed)
  8. plans            check-plans.sh (documentation structure)
  9. forbidden text   check-forbidden-text (typed repository audit)
 10. meson contract   test-meson-mock.sh (mocked TeX; skipped without meson/ninja)
 11. clean tree       staged, unstaged, and untracked nonignored paths

Result vocabulary. A skipped check is neither failure nor success, so this
program distinguishes two passing outcomes and never calls the second one
green:

  CI green    every lane ran and passed
  CI partial  every lane that ran passed; one or more were skipped

Lane 10 is skippable only for local convenience -- it is the sole check of
the hand-managed ADR-014 census, Meson command wiring, report/stamp edges,
generator and publication repair, no-op restat behaviour, and render-failure
propagation. Set CI_REQUIRE_MESON=1 to make a skip a hard failure;
protected-branch and release runs must. Lane 7 is skippable by ADR-011, which
defines the advisory lane as externally provisioned; a skip there is still
reported, never silently passed.

The checker lanes receive their subjects by argument (ADR-014). The
`census_args` function in `scripts/census-args.sh` derives role-tagged argv
from `git ls-files`; the meson build derives the same census from
hand-managed per-directory lists, and the census-audit target welds the two.
That shell function stays the single definition of the census: this driver
runs it through `sh` and re-parses its shell-quoted output with `shlex`,
which is what the shell's `eval "set -- ..."` did. A preflight audit refuses
tracked paths outside the safe argv grammar (SR3-05).

The Meson/LaTeX document lanes are separate because they need a TeX
toolchain:
  meson setup <builddir> && meson compile -C <builddir> attestation \\
    && meson test -C <builddir> --print-errorlogs

The complete repository gate (ADR-011) is:
  - this driver under the declared MSRV toolchain;
  - this driver under current stable;
  - the Meson document lane above in a clean build directory;
  - scripts/check-document-reproducibility.sh.
All four are required before a release claim; only the Rust lanes are
duplicated per toolchain.

Cost accounting
---------------
Every lane is declared, statused, and wall-timed, and the run emits a JSON
timing report -- to `target/ci-report.json`, which is ignored by Git, and on
stdout as the final output -- on failure exactly as on success. The cost data
matters most when a lane hurts, so a failing run is the one run that must not
lose it. Durations come from a monotonic clock; the wall-clock timestamps
alongside them are for correlating a report with a session log, never for
arithmetic.

Test execution lives at the Meson layer (CI-002). This driver runs no
workspace-level `cargo test`: lanes 3 and 4 configure a build directory under
`target/ci-meson`, compile that profile's warm-up build once, and then run one
Meson test per workspace package. Two consequences are deliberate. A duration
and a failure now both name the package they belong to, so the report
attributes cost instead of aggregating it. And the Rust test suites depend on
Meson: without meson and ninja those lanes SKIP, exactly like lane 10, and a
run that skipped them is partial and tested no Rust code -- which the skip
line says in as many words. CI_REQUIRE_MESON=1 turns that skip into a
failure, and protected-branch and release runs must set it.

Per-test timing comes from libtest's own JSON event stream, which
`scripts/cargo-test-json.sh` requests with `-Z unstable-options --format json
--report-time` and captures per package. Those flags change how the harness
reports, never which tests run or what passing proves. They need a nightly
libtest; under a stable toolchain the wrapper runs the ordinary format, the
tests still gate, and the report says per-test timing was not measured rather
than inventing it.

Lane output is streamed to the console exactly where the shell driver put it
and captured to `target/ci-logs/<lane>.log` at the same time, so a failure
stays diagnosable after the fact and the report can name the log. The three
checker lanes whose stdout the shell driver discarded still show nothing on
the console; their output goes to the log alone, which discards less.
"""

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
import threading
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Callable, Optional, Sequence

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPORT_PATH = os.path.join("target", "ci-report.json")
LOG_DIRECTORY = os.path.join("target", "ci-logs")

# The build directory the test lanes configure and reuse. It sits under
# target/, which Git ignores, and it is configured with mock_mode so that a
# machine without a TeX toolchain can still run the Rust test lanes: mock_mode
# governs only the LaTeX programs, and no cargo suite touches them. The
# publication and generation outputs are redirected into the same directory so
# a test run cannot perturb the tracked archive and trip the clean-tree lane.
MESON_BUILD_DIR = os.path.join("target", "ci-meson")

# Per-package lane parallelism, measured rather than assumed (CI-002). All
# figures are the thirteen debug lanes on one 14-core host, wall clock:
#
#   warm-up build then serial lanes (--num-processes 1)     0:52 + 8:32
#   warm-up build then parallel lanes (meson's default)     0:52 + 5:00
#   no warm-up, parallel lanes, cold target directory              6:53
#
# Two things follow, and both are why this file pins rather than defaults.
# Parallel is faster, so the lanes really do overlap: cargo's lock over the
# shared target directory is held only briefly once the binaries are built,
# and the serialization the lock could have forced does not materialize. The
# gain is bounded from below by the slowest single package --
# target-elements-conformance alone is about 250s serial, and 300s while
# thirteen lanes share the cores -- so it is 1.7x, not 13x.
#
# The third row is the configuration without the warm-up target, where every
# lane's cargo has to build: it is slower in total than building once and
# running many, and worse, each lane's duration then measures how long that
# lane waited for the build lock rather than how long its tests took. The
# warm-up is what makes per-package attribution mean anything.
#
# What the warm-up does NOT cover is documentation tests: `cargo build
# --tests` does not compile them, so rustdoc compiles each package's doctests
# inside its own lane, holding cargo's lock while it does. In the debug
# profile that cost is small. In release it dominates: a full gate run
# measured 7:04 of warm-up and 10:31 of lanes, where individual packages
# reported ten-minute lane durations against a tenth of a second of actual
# test execution, and the logs carry cargo's own "Blocking waiting for file
# lock" lines. The release per-package durations are therefore honest wall
# times but poor attributions -- they are mostly queueing -- and that is a
# known residual of this design, not a measurement to read as test cost.
#
# Set CI_TEST_PROCESSES to override; 1 reproduces the serial measurement.
DEFAULT_TEST_PROCESSES = 0  # 0 means meson's own default (one per core)

# How many individual tests the report names, per package and repository-wide.
# The complete per-test data stays in the per-package libtest streams, whose
# paths the report carries, so this cap trims the summary and hides nothing.
# The two depths are deliberately equal: a repository-wide ranking of the
# slowest N tests assembled from per-package lists of fewer than N could not
# be exact, because one package may legitimately own more than its share of
# the slowest tests.
SLOWEST_TEST_LIMIT = 25
SLOWEST_TESTS_PER_PACKAGE = SLOWEST_TEST_LIMIT

PENDING = "pending"
RUNNING = "running"
PASSED = "passed"
FAILED = "failed"
SKIPPED = "skipped"


class LaneFailure(Exception):
    """A lane failed. `status` is the exit status the run must carry."""

    def __init__(self, status, message=None):
        super().__init__(message or "lane failed with status %d" % status)
        self.status = status
        self.message = message


class LaneSkipped(Exception):
    """A lane declined to run. `reason` names it in the report."""

    def __init__(self, reason):
        super().__init__(reason)
        self.reason = reason


@dataclass
class Lane:
    """One declared unit of the gate.

    `label` is the short name the console header carries, kept identical to
    the shell driver's. `description` is the longer sentence `--list` prints.
    Exactly one of `command`, `build_command`, and `action` is populated:
    a fixed argv, an argv derived at run time (the census lanes), or a
    procedure that is not a single subprocess (the clean-tree lane).
    """

    name: str
    label: str
    description: str
    command: Optional[Sequence[str]] = None
    build_command: Optional[Callable[["Run"], Sequence[str]]] = None
    action: Optional[Callable[["Run", "LaneRecord", str], None]] = None
    precondition: Optional[Callable[["Run"], Optional[str]]] = None
    discard_stdout: bool = False


@dataclass
class LaneRecord:
    """The tracked state of one lane across a run."""

    lane: Lane
    status: str = PENDING
    reason: Optional[str] = None
    exit_status: Optional[int] = None
    command: Optional[Sequence[str]] = None
    log_path: Optional[str] = None
    started_at: Optional[str] = None
    finished_at: Optional[str] = None
    duration_seconds: float = 0.0
    cumulative_seconds: float = 0.0
    steps: list = field(default_factory=list)
    packages: list = field(default_factory=list)
    timing_note: Optional[str] = None

    def to_json(self):
        """The report record for this lane."""
        return {
            "name": self.lane.name,
            "label": self.lane.label,
            "description": self.lane.description,
            "status": self.status,
            "reason": self.reason,
            "exit_status": self.exit_status,
            "command": list(self.command) if self.command else None,
            "log": self.log_path,
            "started_at": self.started_at,
            "finished_at": self.finished_at,
            "duration_seconds": round(self.duration_seconds, 3),
            "cumulative_seconds": round(self.cumulative_seconds, 3),
            "steps": self.steps,
            "packages": self.packages,
            "timing_note": self.timing_note,
        }


@dataclass
class Run:
    """Run-wide state: the environment toggles and the lane records."""

    records: list = field(default_factory=list)
    require_meson: bool = False


def now_iso():
    """Wall-clock instant, ISO-8601 with an offset, for the log record."""
    return datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")


def announce(message):
    """Write one driver line where the shell driver wrote it."""
    print(message, file=sys.stderr, flush=True)


def have(program):
    """True when `program` resolves on PATH, as `command -v` decided."""
    return shutil.which(program) is not None


# --------------------------------------------------------------------------
# Census derivation. `scripts/census-args.sh` stays the single definition;
# this runs it and re-parses its shell-quoted output.
# --------------------------------------------------------------------------


def run_census_function(shell_call):
    """Run one census-args.sh function, returning (status, stdout)."""
    completed = subprocess.run(
        ["sh", "-c", ". ./scripts/census-args.sh\n" + shell_call],
        cwd=REPOSITORY_ROOT,
        stdout=subprocess.PIPE,
    )
    return completed.returncode, completed.stdout.decode("utf-8", "surrogateescape")


def audit_tracked_paths():
    """Preflight: refuse tracked paths outside the checker-argv grammar."""
    announce("==> preflight: tracked-path argv grammar audit")
    status, _ = run_census_function("audit_tracked_paths")
    if status != 0:
        raise LaneFailure(status)


def derive_census(description, shell_call):
    """Derive a census and refuse an empty result, as derive_census did."""
    status, derived = run_census_function(shell_call)
    if status != 0:
        raise LaneFailure(status)
    if not derived:
        announce(
            "ERROR: the %s is empty; refusing a vacuous checker run" % description
        )
        raise LaneFailure(1)
    return shlex.split(derived)


# --------------------------------------------------------------------------
# Subprocess execution: stream to the console where the shell driver did,
# and capture to a per-lane log at the same time.
# --------------------------------------------------------------------------


def pump(source, sinks):
    """Copy `source` to every sink in `sinks` until it closes."""
    for chunk in iter(lambda: source.read(4096), b""):
        for sink in sinks:
            sink.write(chunk)
            sink.flush()
    source.close()


def run_command(command, log_path, discard_stdout=False, append=False):
    """Run `command`, tee its streams to `log_path`, return its exit status.

    A process killed by a signal reports 128 + signal, which is the status
    the shell driver's `set -e` propagated for the same event. `append` keeps
    an earlier step's output when one lane runs several commands into one log.
    """
    os.makedirs(os.path.dirname(log_path), exist_ok=True)
    with open(log_path, "ab" if append else "wb") as log:
        process = subprocess.Popen(
            command,
            cwd=REPOSITORY_ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        out_sinks = [log] if discard_stdout else [log, sys.stdout.buffer]
        err_sinks = [log, sys.stderr.buffer]
        threads = [
            threading.Thread(target=pump, args=(process.stdout, out_sinks)),
            threading.Thread(target=pump, args=(process.stderr, err_sinks)),
        ]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        status = process.wait()
    return status if status >= 0 else 128 - status


# --------------------------------------------------------------------------
# Lane bodies that are not a single fixed command.
# --------------------------------------------------------------------------


def scoped_census_command(run):
    """Lane 5 argv: check-generated plus the scoped census."""
    del run
    census = derive_census("scoped census", "derive_census 'scoped census' census_args scoped")
    return [
        "cargo", "run", "--locked", "-p", "tripod-artifacts",
        "--bin", "check-generated", "--",
        "--repository-root", ".",
        "--generated-dir", "packages/model/generated",
    ] + census


def label_census_command(run):
    """Lane 6 argv: check-labels plus the label census."""
    del run
    census = derive_census("label census", "derive_census 'label census' census_args labels")
    return [
        "cargo", "run", "--locked", "-p", "tripod-labels",
        "--bin", "check-labels", "--",
        "--repository-root", ".",
    ] + census


def advisories_precondition(run):
    """Lane 7 skips loudly when cargo-audit is not provisioned."""
    del run
    if have("cargo-audit"):
        return None
    announce("WARNING: cargo-audit is not installed; advisory lane SKIPPED")
    return "cargo-audit is not installed"


def meson_precondition(run):
    """Lane 10 skips without meson/ninja, or fails under CI_REQUIRE_MESON."""
    if have("meson") and have("ninja"):
        return None
    announce("WARNING: meson/ninja not installed; mocked Meson contract lane SKIPPED")
    if run.require_meson:
        announce("ERROR: CI_REQUIRE_MESON=1 and meson/ninja are unavailable")
        raise LaneFailure(1)
    return "meson/ninja are not installed"


def cargo_test_precondition(run):
    """Lanes 3 and 4 skip without meson/ninja -- and then nothing tested Rust."""
    if have("meson") and have("ninja"):
        return None
    announce("WARNING: meson/ninja not installed; the per-package test lanes are SKIPPED")
    announce(
        "         the Rust test suites run at the Meson layer (CI-002), so this run "
        "executed no Rust test"
    )
    if run.require_meson:
        announce("ERROR: CI_REQUIRE_MESON=1 and meson/ninja are unavailable")
        raise LaneFailure(1)
    return "meson/ninja are not installed"


def meson_setup_command(build_directory):
    """Configure argv for the test build directory. See MESON_BUILD_DIR."""
    return [
        "meson", "setup", build_directory, ".",
        "-Dmock_mode=true",
        "-Dpublication_archive_root=" + os.path.join(
            REPOSITORY_ROOT, build_directory, "archive"
        ),
        "-Dartifact_generation_output_dir=" + os.path.join(
            REPOSITORY_ROOT, build_directory, "generated"
        ),
    ]


def test_processes():
    """The `--num-processes` value, from CI_TEST_PROCESSES or the default."""
    configured = os.environ.get("CI_TEST_PROCESSES", "")
    if configured.strip():
        try:
            return max(1, int(configured))
        except ValueError:
            announce("WARNING: CI_TEST_PROCESSES is not a number; using the default")
    return DEFAULT_TEST_PROCESSES


def parse_libtest_stream(path):
    """Per-test outcomes and durations from one libtest JSON event stream.

    Every test binary of the package writes its own suite block into the same
    stream, so unit, integration, and documentation tests all land here. A
    line that is not a JSON object -- a panic message a test printed, say --
    is not timing data and is passed over rather than guessed at.
    """
    tests = []
    with open(path, "r", errors="surrogateescape") as handle:
        for line in handle:
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                event = json.loads(line)
            except ValueError:
                continue
            if event.get("type") != "test":
                continue
            outcome = event.get("event")
            if outcome == "started" or not event.get("name"):
                continue
            seconds = event.get("exec_time")
            tests.append({
                "name": event["name"],
                "event": outcome,
                "seconds": round(float(seconds), 6) if seconds is not None else None,
            })
    return tests


def slowest(tests, limit):
    """The `limit` slowest timed tests, unmeasured ones excluded honestly."""
    timed = [test for test in tests if test.get("seconds") is not None]
    timed.sort(key=lambda test: test["seconds"], reverse=True)
    return timed[:limit]


def collect_package_attribution(profile, meson_log_path):
    """Per-package records for one test lane: Meson durations plus per-test data.

    Meson's own machine-readable test log gives each package lane's status and
    wall time; the package's libtest stream gives the tests inside it. A
    package whose stream is missing -- a stable toolchain, or a lane that never
    got to run -- is reported with its Meson duration and an explicit note, and
    never with fabricated per-test detail.
    """
    prefix = "cargo-test-" + profile + "-"
    packages = []
    if not os.path.exists(meson_log_path):
        return packages
    with open(meson_log_path, "r", errors="surrogateescape") as handle:
        for line in handle:
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            name = (entry.get("name") or "").split(":")[-1]
            if not name.startswith(prefix):
                continue
            package = name[len(prefix):]
            stream_relative = os.path.join(
                MESON_BUILD_DIR, "cargo-test-json", profile, package + ".json"
            )
            stream_absolute = os.path.join(REPOSITORY_ROOT, stream_relative)
            record = {
                "package": package,
                "meson_test": name,
                "result": entry.get("result"),
                "returncode": entry.get("returncode"),
                "duration_seconds": round(float(entry.get("duration") or 0.0), 3),
                "stream": None,
                "per_test_timing": "unavailable: no libtest JSON stream was written",
                "test_count": None,
                "measured_seconds": None,
                "failed_tests": None,
                "slowest_tests": None,
            }
            if os.path.exists(stream_absolute):
                tests = parse_libtest_stream(stream_absolute)
                measured = [test for test in tests if test["seconds"] is not None]
                record["stream"] = stream_relative
                record["per_test_timing"] = "measured"
                record["test_count"] = len(tests)
                record["measured_seconds"] = round(
                    sum(test["seconds"] for test in measured), 3
                )
                record["failed_tests"] = [
                    test["name"] for test in tests if test["event"] not in ("ok", "ignored")
                ]
                record["slowest_tests"] = slowest(tests, SLOWEST_TESTS_PER_PACKAGE)
            packages.append(record)
    packages.sort(key=lambda item: item["duration_seconds"], reverse=True)
    return packages


def cargo_test_action(profile):
    """Lane body for one profile: warm up once, then run the package lanes."""

    def action(run, record, log_path):
        del run
        build_absolute = os.path.join(REPOSITORY_ROOT, MESON_BUILD_DIR)
        configured = os.path.exists(
            os.path.join(build_absolute, "meson-info", "meson-info.json")
        )
        commands = []
        if not configured:
            commands.append(meson_setup_command(MESON_BUILD_DIR))
        # The warm-up compiles the profile's test binaries once. Without it
        # thirteen cargo invocations would each try to build and would queue on
        # cargo's lock over the shared target directory, so each lane's
        # duration would measure the queue rather than the package.
        commands.append(
            ["meson", "compile", "-C", MESON_BUILD_DIR, "cargo-test-warmup-" + profile]
        )
        # --no-rebuild keeps `meson test` from building the default targets
        # (the paper, the lint stamps): this lane owns the Rust suites only,
        # and the warm-up above is the build it needs. --logbase gives the lane
        # its own machine-readable log, so the second profile cannot overwrite
        # the first one's per-package timing.
        test_command = [
            "meson", "test", "-C", MESON_BUILD_DIR, "--no-rebuild",
            "--suite", "cargo-" + profile,
            "--print-errorlogs",
            "--logbase", "cargo-" + profile,
        ]
        processes = test_processes()
        if processes:
            test_command += ["--num-processes", str(processes)]
        commands.append(test_command)
        record.command = test_command

        failure = None
        for position, command in enumerate(commands):
            started = time.monotonic()
            status = run_command(command, log_path, append=position > 0)
            record.steps.append({
                "command": command,
                "exit_status": status,
                "duration_seconds": round(time.monotonic() - started, 3),
            })
            if status != 0:
                record.exit_status = status
                failure = LaneFailure(status)
                break

        # Attribution is read whether the lane passed or failed: a failing run
        # is the one run whose cost and whose failing package must not be lost.
        record.packages = collect_package_attribution(
            profile,
            os.path.join(build_absolute, "meson-logs", "cargo-" + profile + ".json"),
        )
        if record.packages and all(
            package["per_test_timing"] != "measured" for package in record.packages
        ):
            record.timing_note = (
                "per-test timing was not measured: no libtest JSON stream was "
                "written (a non-nightly toolchain reports the human format)"
            )
        if failure is not None:
            raise failure

    return action


def clean_tree_action(run, record, log_path):
    """Lane 11: refuse staged, unstaged, or untracked nonignored changes."""
    del run
    command = ["git", "status", "--porcelain=v1", "--untracked-files=all"]
    record.command = command
    completed = subprocess.run(command, cwd=REPOSITORY_ROOT, stdout=subprocess.PIPE)
    with open(log_path, "wb") as log:
        log.write(completed.stdout)
    if completed.returncode != 0:
        raise LaneFailure(completed.returncode)
    tree_status = completed.stdout.decode("utf-8", "surrogateescape")
    if tree_status.strip("\n"):
        sys.stderr.write(tree_status if tree_status.endswith("\n") else tree_status + "\n")
        sys.stderr.flush()
        announce(
            "ERROR: repository contains staged, unstaged, or untracked "
            "nonignored changes"
        )
        raise LaneFailure(1)


# --------------------------------------------------------------------------
# The registry.
# --------------------------------------------------------------------------


LANES = [
    Lane(
        name="fmt",
        label="cargo fmt",
        description="rustfmt over the whole workspace, checking rather than writing",
        command=["cargo", "fmt", "--all", "--check"],
    ),
    Lane(
        name="clippy",
        label="cargo clippy (-D warnings)",
        description="clippy over all targets against the locked graph, warnings denied",
        command=[
            "cargo", "clippy", "--workspace", "--all-targets", "--locked",
            "--", "-D", "warnings",
        ],
    ),
    Lane(
        name="test-debug",
        label="per-package tests (debug)",
        description="one Meson test per workspace package, debug profile, with per-test timing",
        action=cargo_test_action("debug"),
        precondition=cargo_test_precondition,
    ),
    Lane(
        name="test-release",
        label="per-package tests (release)",
        description="one Meson test per workspace package, release profile, with per-test timing",
        action=cargo_test_action("release"),
        precondition=cargo_test_precondition,
    ),
    Lane(
        name="check-generated",
        label="check-generated",
        description="non-writing stale-artifact gate over the scoped ADR-014 census",
        build_command=scoped_census_command,
        discard_stdout=True,
    ),
    Lane(
        name="check-labels",
        label="check-labels",
        description="non-writing label gate over the label census",
        build_command=label_census_command,
        discard_stdout=True,
    ),
    Lane(
        name="advisories",
        label="cargo audit",
        description="advisory scan of the locked dependency graph (externally provisioned)",
        command=["cargo", "audit"],
        precondition=advisories_precondition,
    ),
    Lane(
        name="plans",
        label="plan-tree checks",
        description="documentation structure of the plan tree",
        command=["sh", "scripts/check-plans.sh"],
    ),
    Lane(
        name="forbidden-text",
        label="forbidden text",
        description="typed repository audit for forbidden text over tracked files",
        command=[
            "cargo", "run", "--locked", "-p", "tripod-labels",
            "--bin", "check-forbidden-text", "--",
            "--repository-root", ".", "--git", "git",
        ],
        discard_stdout=True,
    ),
    Lane(
        name="meson-contract",
        label="mocked Meson contract",
        description="the Meson graph with the TeX toolchain mocked, including census-audit",
        command=["sh", "scripts/test-meson-mock.sh", "."],
        precondition=meson_precondition,
    ),
    Lane(
        name="clean-tree",
        label="clean working tree",
        description="no staged, unstaged, or untracked nonignored path was left behind",
        action=clean_tree_action,
    ),
]


def lane_by_name(name):
    """The declared lane called `name`, or None."""
    for lane in LANES:
        if lane.name == name:
            return lane
    return None


# --------------------------------------------------------------------------
# Reporting.
# --------------------------------------------------------------------------


def format_duration(seconds):
    """A duration in mm:ss.s, which reads at a glance at CI timescales."""
    minutes = int(seconds // 60)
    return "%d:%04.1f" % (minutes, seconds - minutes * 60)


def print_summary(records, total_seconds, outcome):
    """The human timing table, on stderr beside the ==> driver lines."""
    name_width = max([len("lane")] + [len(r.lane.name) for r in records])
    header = "%-*s  %-8s  %9s  %9s" % (name_width, "lane", "status", "duration", "cumulative")
    announce("==> timing report")
    announce("    " + header)
    announce("    " + "-" * len(header))
    for record in records:
        announce(
            "    %-*s  %-8s  %9s  %9s%s"
            % (
                name_width,
                record.lane.name,
                record.status,
                format_duration(record.duration_seconds),
                format_duration(record.cumulative_seconds),
                "  (%s)" % record.reason if record.reason else "",
            )
        )
    announce("    " + "-" * len(header))
    announce(
        "    %-*s  %-8s  %9s"
        % (name_width, "total", outcome, format_duration(total_seconds))
    )
    print_package_summary(records)


def print_package_summary(records):
    """Per-package and per-test attribution beneath the lane table.

    The `duration` column is the package lane's wall time, which includes
    whatever cargo still had to build. The `in-test` column is the sum of the
    per-test execution times libtest reported, and it routinely EXCEEDS the
    wall time: the harness runs a package's tests on several threads, so those
    seconds overlap each other. It is a measure of test work, never of elapsed
    time, and the two columns are not meant to reconcile.
    """
    for record in records:
        if not record.packages:
            continue
        width = max([len("package")] + [len(p["package"]) for p in record.packages])
        announce("")
        announce("==> per-package timing: %s" % record.lane.name)
        header = "%-*s  %-6s  %9s  %6s  %9s" % (
            width, "package", "result", "duration", "tests", "in-test"
        )
        announce("    " + header)
        announce("    " + "-" * len(header))
        for package in record.packages:
            announce(
                "    %-*s  %-6s  %9s  %6s  %9s"
                % (
                    width,
                    package["package"],
                    package["result"] or "?",
                    format_duration(package["duration_seconds"]),
                    "-" if package["test_count"] is None else package["test_count"],
                    "-" if package["measured_seconds"] is None
                    else format_duration(package["measured_seconds"]),
                )
            )
        if record.timing_note:
            announce("    note: %s" % record.timing_note)

    ranked = rank_tests(records, 10)
    if not ranked:
        return
    announce("")
    announce("==> slowest individual tests (top %d)" % len(ranked))
    width = max(len("%s/%s" % (t["lane"], t["package"])) for t in ranked)
    for entry in ranked:
        announce(
            "    %9s  %-*s  %s"
            % (
                format_duration(entry["seconds"]),
                width,
                "%s/%s" % (entry["lane"], entry["package"]),
                entry["name"],
            )
        )


def rank_tests(records, limit):
    """The slowest individual tests across every lane that measured any."""
    ranked = []
    for record in records:
        for package in record.packages:
            for test in package.get("slowest_tests") or []:
                ranked.append({
                    "lane": record.lane.name,
                    "package": package["package"],
                    "name": test["name"],
                    "seconds": test["seconds"],
                    "event": test["event"],
                })
    ranked.sort(key=lambda entry: entry["seconds"], reverse=True)
    return ranked[:limit]


def build_report(records, started_at, finished_at, total_seconds, outcome, exit_status):
    """The JSON timing report, emitted on failure exactly as on success."""
    return {
        "schema": "tripod.ci-report.v1",
        "outcome": outcome,
        "exit_status": exit_status,
        "started_at": started_at,
        "finished_at": finished_at,
        "total_seconds": round(total_seconds, 3),
        "skipped": [r.lane.name for r in records if r.status == SKIPPED],
        "failed": [r.lane.name for r in records if r.status == FAILED],
        "slowest_tests": rank_tests(records, SLOWEST_TEST_LIMIT),
        "lanes": [r.to_json() for r in records],
    }


def emit_report(report):
    """Write the report beside the build tree and print it as final output."""
    path = os.path.join(REPOSITORY_ROOT, REPORT_PATH)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    text = json.dumps(report, indent=2, sort_keys=False)
    with open(path, "w") as handle:
        handle.write(text + "\n")
    print(text, flush=True)


# --------------------------------------------------------------------------
# The run.
# --------------------------------------------------------------------------


def select_lanes(only, skip):
    """The lane list this invocation runs, in declared order."""
    unknown = [name for name in list(only) + list(skip) if lane_by_name(name) is None]
    if unknown:
        announce("ERROR: unknown lane(s): %s" % " ".join(sorted(set(unknown))))
        announce("       declared lanes: %s" % " ".join(lane.name for lane in LANES))
        raise LaneFailure(2)
    selected = []
    for lane in LANES:
        if only and lane.name not in only:
            continue
        if lane.name in skip:
            continue
        selected.append(lane)
    return selected


def execute_lane(run, record, position, total):
    """Run one lane, filling in its record. Raises LaneFailure on failure."""
    lane = record.lane
    announce("==> lane %d/%d: %s" % (position, total, lane.label))
    record.status = RUNNING
    record.started_at = now_iso()
    started = time.monotonic()
    try:
        if lane.precondition is not None:
            reason = lane.precondition(run)
            if reason is not None:
                raise LaneSkipped(reason)
        record.log_path = os.path.join(LOG_DIRECTORY, lane.name + ".log")
        absolute_log = os.path.join(REPOSITORY_ROOT, record.log_path)
        os.makedirs(os.path.dirname(absolute_log), exist_ok=True)
        if lane.action is not None:
            lane.action(run, record, absolute_log)
        else:
            command = list(lane.command) if lane.command else list(lane.build_command(run))
            record.command = command
            status = run_command(command, absolute_log, discard_stdout=lane.discard_stdout)
            record.exit_status = status
            if status != 0:
                raise LaneFailure(status)
        record.status = PASSED
        record.exit_status = 0 if record.exit_status is None else record.exit_status
    except LaneSkipped as skipped:
        record.status = SKIPPED
        record.reason = skipped.reason
        record.log_path = None
        raise
    except LaneFailure as failure:
        record.status = FAILED
        record.exit_status = failure.status
        if record.reason is None and failure.message:
            record.reason = failure.message
        raise
    finally:
        record.duration_seconds = time.monotonic() - started
        record.finished_at = now_iso()


def run_gate(selected, deselected, run):
    """Drive the selected lanes fail-fast, then report. Returns exit status."""
    records = []
    by_lane = {}
    for lane in LANES:
        record = LaneRecord(lane=lane)
        if lane in deselected:
            record.status = SKIPPED
            record.reason = "deselected on the command line"
        records.append(record)
        by_lane[lane.name] = record
    run.records = records

    started_at = now_iso()
    run_started = time.monotonic()
    exit_status = 0
    failing = None
    try:
        audit_tracked_paths()
        for position, lane in enumerate(selected, start=1):
            record = by_lane[lane.name]
            try:
                execute_lane(run, record, position, len(selected))
            except LaneSkipped:
                continue
    except LaneFailure as failure:
        exit_status = failure.status
        failing = failure

    total_seconds = time.monotonic() - run_started
    finished_at = now_iso()

    cumulative = 0.0
    for record in records:
        cumulative += record.duration_seconds
        record.cumulative_seconds = cumulative

    skipped = [r for r in records if r.status == SKIPPED]
    if exit_status != 0:
        outcome = "failed"
    elif skipped:
        outcome = "partial"
    else:
        outcome = "green"

    if exit_status == 0:
        if skipped:
            announce(
                "==> CI partial: every lane that ran passed; skipped:"
                + "".join(" " + r.lane.name for r in skipped)
            )
            announce(
                "    a skipped lane is neither failure nor success; this is not a green run"
            )
        else:
            announce("==> CI green")
    else:
        failed = [r.lane.name for r in records if r.status == FAILED]
        announce(
            "==> CI failed: %s (exit %d)"
            % (" ".join(failed) if failed else "preflight", exit_status)
        )
    del failing

    print_summary(records, total_seconds, outcome)
    emit_report(
        build_report(records, started_at, finished_at, total_seconds, outcome, exit_status)
    )
    return exit_status


# --------------------------------------------------------------------------
# Registry self-test and listing.
# --------------------------------------------------------------------------


def list_lanes():
    """Print the declared lanes and their descriptions; run nothing."""
    width = max(len(lane.name) for lane in LANES)
    for position, lane in enumerate(LANES, start=1):
        print("%2d. %-*s  %s" % (position, width, lane.name, lane.description))
    return 0


def self_test():
    """Check registry coherence without running any lane."""
    problems = []
    names = [lane.name for lane in LANES]
    if len(set(names)) != len(names):
        problems.append("lane names are not unique")
    for lane in LANES:
        bodies = [lane.command is not None, lane.build_command is not None, lane.action is not None]
        if sum(1 for body in bodies if body) != 1:
            problems.append("%s: needs exactly one of command/build_command/action" % lane.name)
        if not lane.label or not lane.description:
            problems.append("%s: empty label or description" % lane.name)
        if lane.name != lane.name.strip() or " " in lane.name:
            problems.append("%s: lane name is not a single bare word" % lane.name)
        if lane.action is not None and lane.discard_stdout:
            problems.append("%s: discard_stdout is meaningless for an action lane" % lane.name)
    for name in ("fmt", "clippy", "advisories", "meson-contract", "clean-tree"):
        if lane_by_name(name) is None:
            problems.append("declared lane %s is missing" % name)
    # The test lanes are Meson-driven (CI-002): no lane may carry a
    # workspace-level cargo test, and both profiles must still be declared.
    for name in ("test-debug", "test-release"):
        lane = lane_by_name(name)
        if lane is None:
            problems.append("declared lane %s is missing" % name)
        elif lane.action is None or lane.precondition is None:
            problems.append("%s: must be a Meson-driven action lane with a precondition" % name)
    for lane in LANES:
        argv = list(lane.command or [])
        if argv[:2] == ["cargo", "test"]:
            problems.append("%s: the gate runs no workspace-level cargo test" % lane.name)
    if slowest([{"name": "a", "seconds": 1.0}, {"name": "b", "seconds": None}], 5) != [
        {"name": "a", "seconds": 1.0}
    ]:
        problems.append("an unmeasured test was ranked as though it were timed")
    try:
        selected = select_lanes(["fmt"], [])
        if [lane.name for lane in selected] != ["fmt"]:
            problems.append("--only did not narrow the selection")
        selected = select_lanes([], ["fmt"])
        if "fmt" in [lane.name for lane in selected]:
            problems.append("--skip did not remove the lane")
        if len(select_lanes([], [])) != len(LANES):
            problems.append("the default selection is not the full sequence")
    except LaneFailure:
        problems.append("selection raised on a valid lane name")
    if format_duration(65.0) != "1:05.0":
        problems.append("duration formatting is wrong")

    for problem in problems:
        announce("ERROR: %s" % problem)
    if problems:
        return 1
    print("self-test: %d lanes declared, registry coherent" % len(LANES))
    return 0


def main(argv):
    """Parse arguments and drive the gate."""
    parser = argparse.ArgumentParser(
        prog="scripts/ci.py",
        description="Repository CI driver (ADR-011) with per-lane wall times.",
    )
    parser.add_argument("--list", action="store_true", help="print the declared lanes and exit")
    parser.add_argument("--self-test", action="store_true", help="check registry coherence and exit")
    parser.add_argument(
        "--only", action="append", default=[], metavar="LANE",
        help="run only this lane; repeatable",
    )
    parser.add_argument(
        "--skip", action="append", default=[], metavar="LANE",
        help="do not run this lane; repeatable",
    )
    arguments = parser.parse_args(argv)

    if arguments.list:
        return list_lanes()
    if arguments.self_test:
        return self_test()

    run = Run(require_meson=os.environ.get("CI_REQUIRE_MESON", "0") == "1")
    try:
        selected = select_lanes(arguments.only, arguments.skip)
    except LaneFailure as failure:
        return failure.status
    deselected = [lane for lane in LANES if lane not in selected]
    return run_gate(selected, deselected, run)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
