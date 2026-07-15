#!/usr/bin/env python3
"""Generate and check planning-layer upstream label registers."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LABEL = re.compile(r"(?:sec|subsec|app|req|def|inv|lem|obl|trap|rem|ins|rule|pin|res|lst|fig|tbl|leaf):[a-z0-9]+(?:-[a-z0-9]+)*(?::[a-z0-9]+(?:-[a-z0-9]+)*)?")
TEX_LABEL = re.compile(r"\\label\{([^}]+)\}")
CODE_SPAN = re.compile(r"(?<!`)`([^`]+)`(?!`)")
HEADING = re.compile(r"^#{1,6}\s+(.+?)\s*$")


def fail(message: str) -> None:
    raise ValueError(message)


def prose_lines(path: Path):
    marker = None
    marker_length = 0
    for number, line in enumerate(path.read_text().splitlines(), 1):
        fence = re.match(r"^\s*(`{3,}|~{3,})", line)
        if fence:
            token = fence.group(1)
            if marker is None:
                marker, marker_length = token[0], len(token)
            elif token[0] == marker and len(token) >= marker_length:
                marker = None
            continue
        if marker is None:
            yield number, line
    if marker is not None:
        fail(f"{path}: unclosed fenced code block")


def realization_labels(path: Path) -> dict[str, str]:
    mints: dict[str, str] = {}
    citations: list[tuple[str, int]] = []
    home = path.stem
    for number, line in prose_lines(path):
        heading = HEADING.match(line)
        if heading:
            home = re.sub(r"\s*·\s*`[^`]+`\s*$", "", heading.group(1))
        if line.count("`") % 2:
            fail(f"{path}:{number}: unmatched inline backtick")
        for match in CODE_SPAN.finditer(line):
            value = match.group(1)
            if value.startswith("[") and value.endswith("]"):
                continue
            if not LABEL.fullmatch(value):
                continue
            before, after = line[:match.start()].rstrip(), line[match.end():].lstrip()
            if before.endswith("(") and after.startswith(")"):
                citations.append((value, number))
                continue
            mints.setdefault(value, home)
    for label, number in citations:
        if label not in mints:
            fail(f"{path}:{number}: unresolved internal citation {label}")
    return mints


def attestation_labels() -> dict[str, str]:
    sources = [ROOT / "papers/attestation/main.tex"] + sorted((ROOT / "papers/attestation/sections").glob("*.tex"))
    labels: dict[str, str] = {}
    for source in sources:
        relative = source.relative_to(ROOT / "papers/attestation")
        for label in TEX_LABEL.findall(source.read_text()):
            if label in labels:
                fail(f"{source}: duplicate Layer-0 label {label}")
            labels[label] = str(relative)
    return labels


def render(title: str, source: str, prefix: str, labels: dict[str, str], home_name: str) -> bytes:
    rows = [
        f"# {title}",
        "",
        f"> Generated from {source}.",
        "> Do not edit by hand.",
        "> The upstream document owns these labels.",
        "",
        "## Register",
        "",
        f"| Plan citation | Upstream label | {home_name} |",
        "|---|---|---|",
    ]
    rows.extend(f"| `[{prefix}-{label}]` | `{label}` | {home} |" for label, home in sorted(labels.items()))
    return ("\n".join(rows) + "\n").encode()


def adr_labels() -> dict[str, set[str]]:
    owners: dict[str, set[str]] = {}
    for path in sorted((ROOT / "adr").glob("[0-9][0-9][0-9]-*.md")):
        owner = f"ADR{path.name[:3]}"
        labels = set()
        for _, line in prose_lines(path):
            for match in CODE_SPAN.finditer(line):
                value = match.group(1)
                if LABEL.fullmatch(value):
                    before, after = line[:match.start()].rstrip(), line[match.end():].lstrip()
                    if not (before.endswith("(") and after.startswith(")")):
                        labels.add(value)
        owners[owner] = labels
    return owners


def imported_citations(realization: set[str], attestation: set[str], adrs: dict[str, set[str]]) -> None:
    for parent in (ROOT / "plans", ROOT / "adr"):
        for path in parent.rglob("*.md"):
            for number, line in prose_lines(path):
                for match in CODE_SPAN.finditer(line):
                    value = match.group(1)
                    if not (value.startswith("[") and value.endswith("]")):
                        continue
                    token = value[1:-1]
                    owner, separator, label = token.partition("-")
                    if not separator:
                        fail(f"{path}:{number}: malformed imported label {value}")
                    known = {"A": attestation, "RZ": realization, **adrs}
                    if owner not in known:
                        fail(f"{path}:{number}: unknown imported-label owner {owner}")
                    if label not in known[owner]:
                        fail(f"{path}:{number}: unknown imported label {value}")


def expected() -> dict[Path, bytes]:
    attestation = attestation_labels()
    realization = realization_labels(ROOT / "docs/attestation/realization.md")
    imported_citations(set(realization), set(attestation), adr_labels())
    labels = ROOT / "plans/labels"
    return {
        labels / "specification.md": render("Layer-0 Upstream Label Register", "`papers/attestation/main.tex` and `papers/attestation/sections/*.tex`", "A", attestation, "Source"),
        labels / "realization.md": render("Realization Upstream Label Register", "`docs/attestation/realization.md`", "RZ", realization, "Home"),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("update", "check"))
    command = parser.parse_args().command
    try:
        registers = expected()
    except ValueError as error:
        print(f"FAIL {error}", file=sys.stderr)
        return 1
    stale = []
    for path, content in registers.items():
        if command == "update":
            path.write_bytes(content)
        elif not path.is_file() or path.read_bytes() != content:
            stale.append(path.relative_to(ROOT))
    if stale:
        for path in stale:
            print(f"FAIL stale generated label register: {path}", file=sys.stderr)
        return 1
    print(f"label registers {command} pass", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())