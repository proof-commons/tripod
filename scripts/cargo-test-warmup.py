#!/usr/bin/env python3
"""Build each lane's package selection, without executing its tests.

Workspace feature unification is different from selecting one package.
Using the same selection in warm-up and execution prevents rebuilding the
dependency graph inside a timed lane. Rustdoc still compiles its examples
as part of their assertions (including compile-fail examples).
"""

import subprocess
import sys
import time


def main(argv):
    cargo, manifest, profile, *packages = argv
    for package in packages:
        started = time.monotonic()
        result = subprocess.run([
            cargo, "test", "--no-run", "--manifest-path", manifest,
            "--profile", profile, "-p", package,
        ], check=False)
        print(f"warmup {profile} {package}: rc={result.returncode} "
              f"wall={time.monotonic() - started:.3f}s", flush=True)
        if result.returncode:
            return result.returncode
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
