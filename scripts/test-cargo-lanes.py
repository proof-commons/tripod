#!/usr/bin/env python3
"""Exercise the lane wrapper with a fake Cargo; no compiler or tests execute."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    started = time.monotonic()
    root = Path(__file__).resolve().parent
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        cargo = directory / "cargo"
        cargo.write_text("""#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:] == ['--version']:
    print('cargo 1.0-nightly')
else:
    with open(os.environ['ARGV_LOG'], 'a') as log:
        log.write(json.dumps(sys.argv[1:]) + '\\n')
    print('{"type":"suite","event":"ok","passed":1,"exec_time":0.01}')
    if os.environ.get('COMPILE'):
        print('   Compiling unexpected v1', file=sys.stderr)
    sys.exit(int(os.environ.get('FAIL', '0')))
""")
        cargo.chmod(0o755)
        log = directory / "argv.json"
        env = dict(os.environ, ARGV_LOG=str(log))
        for selection, expected in [
            ("lib", ["--lib"]), ("doc", ["--doc"]),
            ("test:maturity_observer", ["--test", "maturity_observer"]),
        ]:
            stream = directory / (selection.replace(":", "-") + ".json")
            command = ["sh", str(root / "cargo-test-json.sh"), str(cargo),
                       "Cargo.toml", "tripod-vectors", "gate", str(stream), selection]
            result = subprocess.run(command, env=env, capture_output=True, check=False)
            assert result.returncode == 0, result.stderr
            args = json.loads(log.read_text().splitlines()[-1])
            assert args[7:7 + len(expected)] == expected, args
            assert "--ignored" not in args and "--include-ignored" not in args
            assert json.loads(stream.read_text())["passed"] == 1
            for label, extra, code in [("test failure", {"FAIL": "7"}, 7),
                                        ("unexpected compilation", {"COMPILE": "1"}, 1)]:
                case_start = time.monotonic()
                result = subprocess.run(command, env=dict(env, **extra),
                                        capture_output=True, check=False)
                assert result.returncode == code, result
                assert b'"passed":1' in result.stderr
                print(f"{selection} {label}: propagated wall={time.monotonic() - case_start:.3f}s")
        subprocess.run([sys.executable, str(root / "cargo-test-warmup.py"),
                        str(cargo), "Cargo.toml", "gate", "first", "second"],
                       env=env, check=True)
        calls = [json.loads(line) for line in log.read_text().splitlines()][-2:]
        assert calls == [["test", "--no-run", "--manifest-path", "Cargo.toml",
                          "--profile", "gate", "-p", name] for name in ["first", "second"]]
    print(f"cargo lane contracts passed wall={time.monotonic() - started:.3f}s")


if __name__ == "__main__":
    main()
