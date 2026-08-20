#!/usr/bin/env python3
"""Timing report over a finished `meson test` run (CI-009, CI-005).

This is a REPORT READER, not a driver. It runs nothing, decides nothing, and
gates nothing: every lane is a meson test, meson's harness times it and
writes the result, and this file reads what the harness already recorded.
The retired scripts/ci.py did both jobs, and folding the reading into the
driving is what made the driver look load-bearing when only its report was.

Two sources, both written by the run itself:

  <builddir>/meson-logs/testlog.json   one record per test -- suite, result,
                                       wall time -- from meson's harness;
  <builddir>/cargo-test-json/<profile>/<package>.json
                                       libtest's own JSON event stream, which
                                       scripts/cargo-test-json.sh captures per
                                       package, carrying PER-TEST durations.

Meson times a test, so it can say a package lane took four minutes; only
libtest can say which test inside it took three. Keeping the second reading
is what preserves the per-test attribution CI-002 chartered -- the whole
point of splitting the suite per package was to make cost attributable, and
a migration that kept the lanes but dropped their attribution would have
re-homed the structure and lost the reason for it.

The per-test stream needs a nightly libtest, which is what supplies
`--format json --report-time`. Under a stable toolchain the wrapper runs the
ordinary format, the tests still gate, and this file says per-test timing was
not measured rather than inventing it.

The `in-test` column routinely EXCEEDS a lane's wall time: libtest runs a
package's tests on several threads, so those seconds overlap. It measures
test WORK, never elapsed time, and the two columns are not meant to
reconcile.

Usage: ci-timing-report.py <builddir> [--logbase NAME]
Exit status is 0 whenever a report could be read, including for a run whose
tests failed: this file reports on the gate, it is not part of it. A run
whose log is missing exits 2, because a silently absent report is exactly
the outcome the wall-time ruling calls a process bug.
"""

from __future__ import annotations

import json
import os
import sys

# How many individual tests the report names, per package and overall. The
# complete data stays in the per-package streams, whose paths the table
# carries, so this trims the summary and hides nothing. The two depths are
# equal on purpose: a repository-wide ranking assembled from per-package
# lists shorter than the ranking could not be exact, because one package may
# legitimately own more than its share of the slowest tests.
SLOWEST_PER_PACKAGE = 25
SLOWEST_OVERALL = 10


def format_duration(seconds: float) -> str:
    """A duration in mm:ss.s, which reads at a glance at CI timescales."""
    minutes = int(seconds // 60)
    return "%d:%04.1f" % (minutes, seconds - minutes * 60)


def read_records(path: str):
    """Meson's per-test records: (suite, name, result, seconds)."""
    records = []
    with open(path, "r", errors="surrogateescape") as handle:
        for line in handle:
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            full = entry.get("name") or ""
            # Meson spells a record's name "<suite> - <project>:<test>".
            suite, _, remainder = full.partition(" - ")
            name = remainder.split(":")[-1] if remainder else full
            records.append({
                "suite": suite or "(none)",
                "name": name,
                "result": entry.get("result") or "?",
                "seconds": float(entry.get("duration") or 0.0),
            })
    return records


def parse_libtest_stream(path: str):
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


def stream_for(build_directory: str, test_name: str):
    """The libtest stream a `cargo-test-<profile>-<package>` lane wrote."""
    if not test_name.startswith("cargo-test-"):
        return None
    remainder = test_name[len("cargo-test-"):]
    profile, _, package = remainder.partition("-")
    if profile not in ("debug", "release") or not package:
        return None
    return os.path.join(build_directory, "cargo-test-json", profile, package + ".json")


def suite_table(records):
    """Per-suite totals: the figure a cadence decision is argued from."""
    order = []
    totals = {}
    for record in records:
        suite = record["suite"]
        if suite not in totals:
            totals[suite] = {"seconds": 0.0, "tests": 0, "failed": 0, "skipped": 0}
            order.append(suite)
        bucket = totals[suite]
        bucket["seconds"] += record["seconds"]
        bucket["tests"] += 1
        if record["result"] in ("FAIL", "TIMEOUT", "ERROR"):
            bucket["failed"] += 1
        elif record["result"] == "SKIP":
            bucket["skipped"] += 1
    return order, totals


def print_suites(records) -> None:
    """The per-suite summary, which is what a cadence ruling reads."""
    order, totals = suite_table(records)
    width = max([len("suite")] + [len(suite) for suite in order])
    header = "%-*s  %6s  %6s  %7s  %11s" % (
        width, "suite", "tests", "failed", "skipped", "in-suite"
    )
    print("==> per-suite timing")
    print("    " + header)
    print("    " + "-" * len(header))
    for suite in order:
        bucket = totals[suite]
        print(
            "    %-*s  %6d  %6d  %7d  %11s"
            % (
                width, suite, bucket["tests"], bucket["failed"],
                bucket["skipped"], format_duration(bucket["seconds"]),
            )
        )
    print("    " + "-" * len(header))
    # The sum of test durations, NOT elapsed time: meson runs tests in
    # parallel, so this exceeds the wall clock of the run and is a measure of
    # test work. The elapsed figure belongs to whoever timed the command.
    print(
        "    %-*s  %6d  %6d  %7d  %11s"
        % (
            width, "total", len(records),
            sum(b["failed"] for b in totals.values()),
            sum(b["skipped"] for b in totals.values()),
            format_duration(sum(r["seconds"] for r in records)),
        )
    )


def print_lanes(records) -> None:
    """Every lane by descending wall time: where the gate's cost actually is."""
    ranked = sorted(records, key=lambda record: record["seconds"], reverse=True)
    width = max([len("lane")] + [len(record["name"]) for record in ranked])
    header = "%-*s  %-8s  %11s" % (width, "lane", "result", "duration")
    print("")
    print("==> lanes by wall time")
    print("    " + header)
    print("    " + "-" * len(header))
    for record in ranked:
        print(
            "    %-*s  %-8s  %11s"
            % (width, record["name"], record["result"],
               format_duration(record["seconds"]))
        )


def print_tests(records, build_directory) -> None:
    """Per-test attribution inside the package lanes (CI-002)."""
    ranked = []
    unmeasured = []
    for record in records:
        path = stream_for(build_directory, record["name"])
        if path is None:
            continue
        if not os.path.exists(path):
            unmeasured.append(record["name"])
            continue
        tests = parse_libtest_stream(path)
        timed = [test for test in tests if test["seconds"] is not None]
        if not timed:
            unmeasured.append(record["name"])
            continue
        timed.sort(key=lambda test: test["seconds"], reverse=True)
        for test in timed[:SLOWEST_PER_PACKAGE]:
            ranked.append({
                "lane": record["name"],
                "name": test["name"],
                "seconds": test["seconds"],
            })

    if unmeasured:
        print("")
        print(
            "==> per-test timing was not measured for %d package lane(s): no libtest"
            % len(unmeasured)
        )
        print("    JSON stream was written (a non-nightly toolchain reports the")
        print("    human format). Lanes: " + " ".join(sorted(unmeasured)))

    if not ranked:
        return
    ranked.sort(key=lambda entry: entry["seconds"], reverse=True)
    ranked = ranked[:SLOWEST_OVERALL]
    print("")
    print("==> slowest individual tests (top %d)" % len(ranked))
    width = max(len(entry["lane"]) for entry in ranked)
    for entry in ranked:
        print(
            "    %11s  %-*s  %s"
            % (format_duration(entry["seconds"]), width, entry["lane"], entry["name"])
        )


def main(argv) -> int:
    """Read one finished run's logs and print the timing report."""
    build_directory = argv[0] if argv else "target/ci-meson"
    logbase = "testlog"
    if "--logbase" in argv:
        logbase = argv[argv.index("--logbase") + 1]
    path = os.path.join(build_directory, "meson-logs", logbase + ".json")
    if not os.path.exists(path):
        sys.stderr.write(
            "ERROR: no test log at %s; a run that reported no timing is itself\n"
            "       a defect to report, not a missing convenience (CI-005)\n" % path
        )
        return 2

    records = read_records(path)
    if not records:
        sys.stderr.write("ERROR: %s holds no test records\n" % path)
        return 2

    print_suites(records)
    print_lanes(records)
    print_tests(records, build_directory)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
