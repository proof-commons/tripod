#!/usr/bin/env python3
"""Non-semantic documentation structure and hygiene checks."""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parent.parent
BASELINE_MARKDOWN_BYTES = 1_295_616
HARD_CAP_BYTES = 768 * 1024
SOFT_TARGET_BYTES = 512 * 1024
GENERATED_REGISTERS = {
    Path("plans/labels/specification.md"),
    Path("plans/labels/realization.md"),
}
OLD_PATHS = (
    "plans/toolchain-architecture.md",
    "plans/decisions/001-typed-rust-is-normative.md",
    "plans/decisions/002-target-independent-realization-layer.md",
    "plans/decisions/003-multiple-backends-tapscript-first.md",
    "plans/decisions/004-translation-validation-over-compiler-trust.md",
    "plans/decisions/005-value-parametric-asset-rigid.md",
    "plans/decisions/006-canonical-transaction-layout-abi.md",
    "plans/research/state-object-constructor.md",
)
LINK = re.compile(r"\[[^]]*\]\(([^)#\s]+)(?:#[^)\s]+)?\)")
SCAFFOLDING = (
    "Below is a complete draft",
    "The next file should be",
    "Notes for applying this file",
)
PROHIBITED_MACHINE_INPUT = (
    "Machine-consumed by the toolchain: yes",
    "plans are compiler input",
    "this plan is normative protocol input",
)
PROHIBITED_SOURCE_IDENTITY = (
    "source revision is target identity",
    "exact Elements revision is protocol identity",
    "deployment release pins the Elements source revision",
    "target identity binds the upstream implementation commit",
)


def markdown_files() -> list[Path]:
    return sorted(
        path
        for root in (ROOT / "adr", ROOT / "plans")
        for path in root.rglob("*.md")
    )


def warn_threshold(relative: Path) -> int | None:
    if relative in GENERATED_REGISTERS:
        return None
    if relative.name == "README.md":
        return 16 * 1024
    if relative.parts[0] == "adr" and relative.name != "README.md":
        return 14 * 1024
    if len(relative.parts) > 1 and relative.parts[1] == "decisions":
        return 12 * 1024
    if len(relative.parts) > 1 and relative.parts[1] == "packages":
        return 24 * 1024
    if len(relative.parts) > 1 and relative.parts[1] == "phases":
        return 16 * 1024
    if len(relative.parts) > 1 and relative.parts[1] == "research":
        return 32 * 1024
    if len(relative.parts) > 1 and relative.parts[1] == "reference":
        return 40 * 1024
    return None


def main() -> int:
    failures: list[str] = []
    warnings: list[str] = []
    files = markdown_files()

    directories: list[Path] = []
    for root in (ROOT / "adr", ROOT / "plans"):
        for directory, _, _ in __import__("os").walk(root):
            directories.append(Path(directory))
    for directory in sorted(directories):
        readme = directory / "README.md"
        if not readme.is_file():
            failures.append(f"ownership: {directory.relative_to(ROOT)} has no README.md")
            continue
        contents = readme.read_text()
        for child in sorted(directory.iterdir()):
            if child.name == "README.md":
                continue
            if child.is_file() and child.suffix == ".md" and child.name not in contents:
                failures.append(f"ownership: {child.relative_to(ROOT)} not indexed in {readme.relative_to(ROOT)}")
            if child.is_dir() and f"{child.name}/README.md" not in contents:
                failures.append(f"ownership: {child.relative_to(ROOT)} not indexed in {readme.relative_to(ROOT)}")

    for path in files:
        relative = path.relative_to(ROOT)
        text = path.read_text()
        first_line = text.splitlines()[0] if text else ""
        if not first_line.startswith("# "):
            failures.append(f"heading: {relative} does not start with a top-level heading")
        for target in LINK.findall(text):
            if target.startswith(("http://", "https://", "mailto:")):
                continue
            if not (path.parent / target).resolve().exists():
                failures.append(f"broken link: {relative} -> {target}")
        for marker in SCAFFOLDING:
            if marker in text:
                failures.append(f"draft scaffolding: {relative}: {marker}")
        if re.search(r"^## File \d+", text, re.M):
            failures.append(f"draft scaffolding: {relative}: ## File N")
        if any(marker in text for marker in ("example.invalid", "TODO_URL", "INSERT_HASH", "TBD_PATH")):
            failures.append(f"placeholder data: {relative}")
        if re.search(r"(?i)(confidence\s+\d{1,3}%|\d{1,3}%\s+confident)", text):
            failures.append(f"confidence percentage: {relative}")
        if any(old in text for old in OLD_PATHS):
            failures.append(f"deleted path reference: {relative}")
        if any(marker in text for marker in PROHIBITED_MACHINE_INPUT):
            failures.append(f"machine-input overclaim: {relative}")
        if relative.parts[0] == "plans" and any(marker in text for marker in PROHIBITED_SOURCE_IDENTITY):
            failures.append(f"source-identity drift: {relative}")
        threshold = warn_threshold(relative)
        if threshold is not None and path.stat().st_size > threshold:
            warnings.append(f"weight warning: {relative} is {path.stat().st_size} bytes (threshold {threshold})")

    backlog = ROOT / "plans/backlog.md"
    active = re.search(r"^> \*\*Current gate:\*\* Phase (\d+)", backlog.read_text(), re.M)
    if active is None:
        failures.append("phase: backlog declares no current gate")
    else:
        active_card = ROOT / f"plans/phases/{active.group(1).zfill(2)}-"
        cards = list(active_card.parent.glob(f"{active_card.name}*.md"))
        if len(cards) != 1 or not re.search(r"^> \*\*Status:\*\* Active", cards[0].read_text(), re.M):
            failures.append("phase: backlog current gate does not point to one active phase card")
        other_active = [path for path in (ROOT / "plans/phases").glob("[0-9][0-9]-*.md") if re.search(r"^> \*\*Status:\*\* Active", path.read_text(), re.M)]
        if len(other_active) != 1:
            failures.append("phase: exactly one phase card must be Active")

    sizes: dict[str, int] = {"adr": 0, "plans": 0}
    for path in files:
        sizes[path.relative_to(ROOT).parts[0]] += path.stat().st_size
    combined = sizes["adr"] + sizes["plans"]
    print(f"adr bytes: {sizes['adr']}", file=sys.stderr)
    print(f"plans bytes: {sizes['plans']}", file=sys.stderr)
    print(f"combined bytes: {combined}", file=sys.stderr)
    print(f"hard cap: {HARD_CAP_BYTES}", file=sys.stderr)
    print(f"soft target: {SOFT_TARGET_BYTES}", file=sys.stderr)
    for warning in warnings:
        print(f"WARNING {warning}", file=sys.stderr)
    if combined > HARD_CAP_BYTES:
        failures.append(f"weight: combined Markdown {combined} exceeds hard cap {HARD_CAP_BYTES}")
    if combined > SOFT_TARGET_BYTES:
        print("WARNING combined Markdown exceeds soft target", file=sys.stderr)
    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())