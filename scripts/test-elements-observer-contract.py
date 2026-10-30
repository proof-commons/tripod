#!/usr/bin/env python3
"""Closed acquisition and capture-envelope contracts without a node."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch


def load(name):
    path = Path(__file__).with_name(name + ".py")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


capture = load("elements-observer-capture")
reader = load("elements-observer-read")


class Contracts(unittest.TestCase):
    def test_closed_locator(self):
        control = {"chain_name":"elementsregtest", "genesis_id":"11"*32, "transaction_id":"22"*32}
        self.assertEqual(reader.locator(control), control)
        for field in control:
            changed = dict(control)
            del changed[field]
            with self.assertRaises(ValueError):
                reader.locator(changed)
        for field in ("wallet", "descriptor", "datadir", "path", "raw_transaction"):
            with self.assertRaises(ValueError):
                reader.locator(dict(control, **{field:"forbidden"}))

    def exercise(self, defect=None):
        with tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"]) as directory:
            base = Path(directory)
            executable = base / "binary"
            executable.write_text("synthetic")
            namespace = base / "namespace" / "result.json"
            output = base / "capture"
            args = type("Arguments", (), {"output":str(output), "namespace_record":str(namespace)})()
            public = {"chain_name":"elementsregtest", "genesis_id":"11"*32, "transaction_id":"22"*32}
            def fake_run(argv, **kwargs):
                if argv[:2] == ["git", "status"]:
                    stdout = "dirty" if defect == "dirty" else ""
                elif argv[:2] == ["git", "rev-parse"]:
                    stdout = "1"*40
                elif "--ignored" in argv:
                    if defect == "producer":
                        raise RuntimeError("producer failed")
                    env = kwargs["env"]
                    temporary = Path(kwargs["cwd"])
                    (temporary / "chain/elementsregtest/wallets").mkdir(parents=True)
                    raw = capture.canonical({"route":"direct-block"}).encode()
                    line = "response-detail %d %s\nrequest-bytes 1701 %s\n" % (len(raw),raw.hex(), "01"*1701)
                    (Path(env["TRIPOD_LIVE_REPORT_DIR"]) / "subject.capture").write_text(line)
                    stdout = "OBSERVER-MANIFEST " + capture.canonical(public) + "\n"
                elif "--adapter" in argv:
                    self.assertEqual(set(json.loads(kwargs["data"])), set(public))
                    stdout = capture.canonical({"raw_transaction":"01"*1701})
                else:
                    self.assertEqual(kwargs["env"], {})
                    self.assertEqual(set(json.loads(kwargs["data"])), {"manifest", "reading"})
                    stdout = capture.canonical({"experiment":[{}]*(6 if defect == "census" else 7)})
                return subprocess.CompletedProcess(argv, 0, stdout, "")
            environment = {name:"declared" for name in ["TRIPOD_LIVE_EXECUTOR", "TRIPOD_LIVE_NETWORK_ID", "TRIPOD_LIVE_GENESIS_ID", "ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD", "ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI", "ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK", "ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP"]}
            probe = {"available":False, "exit":1, "diagnostic":"namespace refused", "command":"unshare", "mode":"unshare"}
            with patch.dict(os.environ, environment), patch.object(capture, "namespace_probe", return_value=probe) as measured_probe, patch.object(capture, "run", fake_run), patch.object(capture, "reference_bytes", return_value=bytes([1])*1701), patch.object(capture, "binaries", return_value={"maturity_native":str(executable), "maturity_observer":str(executable)}):
                try:
                    if defect:
                        with self.assertRaises(RuntimeError):
                            capture.capture(args)
                        self.assertFalse((output / "RUN-REPORT").exists())
                    else:
                        capture.capture(args)
                        files = sorted(p.name for p in output.iterdir() if p.is_file())
                        self.assertEqual(len(files), 4)
                        report = (output / "RUN-REPORT").read_text().splitlines()
                        position = report.index("capture-profile observer")
                        measured_probe.assert_called_once_with(namespace.parent)
                        for offset, phase in enumerate(("preflight", "build", "producer", "acquisition", "observer", "census", "manifest", "coordinator"), 1):
                            self.assertRegex(report[position+offset], "^phase-wall-ms " + phase + " [0-9]+$")
                        self.assertRegex(report[position+9], "^elapsed-wall-ms [0-9]+$")
                        self.assertEqual(sum(int(line.split()[-1]) for line in report[position+1:position+9]),
                                         int(report[position+9].split()[-1]))
                        recorded = json.loads(next(output.glob("*.capture")).read_text())
                        self.assertEqual(recorded["isolation"]["probe"], probe)
                        for child, code in recorded["isolation"]["child_exits"].items():
                            self.assertIn("%s-exit %d" % (child, code), report)
                        self.assertEqual(report[-1], "eligible yes")
                        for line in (output / "MANIFEST.sha256").read_text().splitlines():
                            expected, name = line.split("  ")
                            self.assertEqual(expected, capture.digest((output/name).read_bytes()))
                        self.assertEqual(output.stat().st_mode & 0o222, 0)
                        with self.assertRaises(RuntimeError):
                            capture.capture(args)
                finally:
                    if output.exists():
                        output.chmod(0o700)
                        for path in output.rglob("*"):
                            path.chmod(0o700 if path.is_dir() else 0o600)

    def test_capture_contract(self):
        self.exercise()

    def test_failures_never_emit_eligibility(self):
        for defect in ("dirty", "producer", "census"):
            with self.subTest(defect=defect):
                self.exercise(defect)


if __name__ == "__main__":
    started = time.monotonic()
    try:
        unittest.main()
    finally:
        print("contract-wall-ms %d" % int((time.monotonic() - started) * 1000), file=sys.stderr)
