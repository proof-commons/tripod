#!/usr/bin/env python3
"""Check the workspace instrumentation dependency and its refusal boundary."""

from pathlib import Path
import sys
import tempfile
import tomllib
import unittest


def check_workspace(root):
    """Return manifest paths violating the core tracing dependency policy."""
    workspace = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]
    excluded = {
        path for pattern in workspace.get("exclude", []) for path in root.glob(pattern)
    }
    members = {
        path for pattern in workspace["members"] for path in root.glob(pattern)
    } - excluded
    if not members:
        raise ValueError("workspace has no member manifests")
    failures = []
    for member in sorted(members):
        manifest = member / "Cargo.toml"
        data = tomllib.loads(manifest.read_text())
        dependency = data.get("dependencies", {}).get("tracing")
        if (
            not isinstance(dependency, dict)
            or dependency.get("workspace") is not True
            or dependency.get("optional", False) is not False
            or dependency.get("package", "tracing") != "tracing"
        ):
            failures.append(manifest.relative_to(root).as_posix())
    return failures


class WorkspaceTracing(unittest.TestCase):
    def test_repository(self):
        self.assertEqual(check_workspace(REPOSITORY), [])

    def test_dependency_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["packages/*"]\n'
                '[workspace.dependencies]\ntracing = "0.1"\n'
            )
            member = root / "packages" / "control"
            member.mkdir(parents=True)
            manifest = member / "Cargo.toml"
            header = '[package]\nname = "control"\nversion = "0.1.0"\n'
            dependency = 'tracing = { workspace = true }\n'
            manifest.write_text(header + '[dependencies]\n' + dependency)
            self.assertEqual(check_workspace(root), [])
            # The same accepted member, with only its tracing edge removed.
            manifest.write_text(header + '[dependencies]\n')
            self.assertEqual(check_workspace(root), ['packages/control/Cargo.toml'])
            for declaration in (
                '[dev-dependencies]\n' + dependency,
                '[target.\'cfg(unix)\'.dependencies]\n' + dependency,
                '[dependencies]\ntracing = { workspace = true, optional = true }\n',
            ):
                with self.subTest(declaration=declaration):
                    manifest.write_text(header + declaration)
                    self.assertEqual(
                        check_workspace(root), ['packages/control/Cargo.toml']
                    )


if __name__ == "__main__":
    REPOSITORY = Path(sys.argv.pop(1))
    unittest.main()
