#!/usr/bin/env python3
"""Refuse split lanes that do not cover Cargo's vectors target inventory."""

import copy
import json
import subprocess
import sys
import time


def check(targets, lanes):
    expected = []
    for target in targets:
        kind = target["kind"]
        if kind == ["lib"]:
            if target["test"]:
                expected.append("lib")
            if target["doctest"]:
                expected.append("doc")
        elif kind == ["test"]:
            expected.append("test:" + target["name"])
        elif target["test"]:
            raise ValueError(f"unsupported tested target: {target['name']} {kind}")
    if sorted(expected) != sorted(lanes):
        raise ValueError(f"vectors target census differs: cargo={sorted(expected)} lanes={sorted(lanes)}")


def main(argv):
    started = time.monotonic()
    self_test = argv[0] == "--self-test"
    if self_test:
        argv = argv[1:]
    cargo, manifest, *lanes = argv
    metadata = json.loads(subprocess.check_output([
        cargo, "metadata", "--offline", "--no-deps", "--format-version", "1",
        "--manifest-path", manifest,
    ]))
    targets = next(p["targets"] for p in metadata["packages"] if p["name"] == "tripod-vectors")
    check(targets, lanes)
    print(f"vectors target census: {len(lanes)} lanes match Cargo", flush=True)
    if self_test:
        added = copy.deepcopy(targets)
        added.append({"kind": ["test"], "name": "unregistered_negative", "test": True})
        for name, subjects, roster in [
            ("new target", added, lanes),
            ("missing doctests", targets, [x for x in lanes if x != "doc"]),
            ("duplicate lane", targets, lanes + [lanes[0]]),
            ("missing harnessless target", targets,
             [x for x in lanes if x != "test:maturity_observer"]),
        ]:
            case_start = time.monotonic()
            try:
                check(subjects, roster)
            except ValueError:
                print(f"negative {name}: refused wall={time.monotonic() - case_start:.6f}s")
            else:
                raise AssertionError(f"accepted {name}")
    print(f"target census wall={time.monotonic() - started:.3f}s")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
