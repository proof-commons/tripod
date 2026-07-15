#!/bin/sh
# Planning-tree checker (P0-010): census, links, and hygiene for
# plans/. A documentation checker only — never a semantic input to any
# compiler, linker, or release tool.
#
# Checks:
#   - every planning directory has a README that indexes its direct Markdown
#     children and direct child directories;
#   - every relative Markdown link resolves;
#   - no placeholder URL (example.invalid) remains;
#   - no old package path (packages/tripod-*) remains;
#   - no confidence percentage remains;
#   - no chat-style draft scaffolding remains ("## File N", "Below is a
#     complete draft", "Notes for applying this file", "The next file
#     should be");
#   - every document opens with a real top-level heading on line 1 (a
#     whole-document code fence or wrapper prose fails this);
#   - the current-phase declarations in backlog.md (authoritative),
#     roadmap.md, toolchain-architecture.md, and README.md agree;
#   - git diff --check passes (staged and unstaged).
#
# Deleted-plan references inside backlog task descriptions and
# supersession records are historical text, not active links, and are
# deliberately not flagged here.
set -eu

cd "$(dirname "$0")/.."

python3 scripts/doc-labels.py check

python3 - <<'EOF'
import os, re, glob, sys

fail = []

md_files = sorted(glob.glob('plans/**/*.md', recursive=True))
directories = sorted(
    path for path, _, _ in os.walk('plans')
)

for directory in directories:
    readme_path = os.path.join(directory, 'README.md')
    if not os.path.isfile(readme_path):
        fail.append(f"ownership: {directory} has no README.md")
        continue
    readme = open(readme_path).read()
    for child in sorted(os.listdir(directory)):
        child_path = os.path.join(directory, child)
        if child == 'README.md':
            continue
        if os.path.isfile(child_path) and child.endswith('.md'):
            if child not in readme:
                fail.append(f"ownership: {child_path} not indexed in {readme_path}")
        elif os.path.isdir(child_path):
            child_readme = f"{child}/README.md"
            if not os.path.isfile(os.path.join(child_path, 'README.md')):
                fail.append(f"ownership: {child_path} has no README.md")
            elif child_readme not in readme:
                fail.append(
                    f"ownership: {child_path} not indexed in {readme_path}"
                )

link_re = re.compile(r'\[[^\]]*\]\(([^)#\s]+)(#[^)\s]*)?\)')
for f in md_files:
    text = open(f).read()
    base = os.path.dirname(f)
    for m in link_re.finditer(text):
        target = m.group(1)
        if target.startswith(('http://', 'https://', 'mailto:')):
            continue
        if not os.path.exists(os.path.normpath(os.path.join(base, target))):
            fail.append(f"broken link: {f} -> {target}")

# Chat-style draft scaffolding: an assistant-response wrapper around a
# document is a defect anywhere, including inside code fences.
scaffolding = [
    (re.compile(r'^## File \d+', re.M), 'assistant draft header ("## File N")'),
    (re.compile(r'Below is a complete draft'), 'assistant draft preamble'),
    (re.compile(r'^#+ Notes for applying this file', re.M),
     'assistant application notes'),
    (re.compile(r'The next file should be'), 'assistant continuation note'),
]

for f in md_files:
    text = open(f).read()
    if 'example.invalid' in text:
        fail.append(f"placeholder URL in {f}")
    if 'packages/tripod-' in text:
        fail.append(f"old package path in {f}")
    for m in re.finditer(r'(?i)\bconfidence[^.\n]{0,20}\d{1,3}\s*%', text):
        fail.append(f"confidence percentage in {f}: {m.group(0)!r}")
    for pattern, label in scaffolding:
        if pattern.search(text):
            fail.append(f"draft scaffolding in {f}: {label}")
    first_line = text.split('\n', 1)[0]
    if not first_line.startswith('# '):
        fail.append(
            f"heading shape: {f} does not open with a top-level heading "
            f"(starts with {first_line[:40]!r})"
        )

# Decision records have stable numeric identities. This intentionally checks
# only record structure; decision-local labels remain unlinted planning aids.
for f in sorted(glob.glob('plans/decisions/[0-9][0-9][0-9]-*.md')):
    text = open(f).read()
    number = os.path.basename(f)[:3]
    if not re.match(rf'^# D{number}:', text):
        fail.append(f"decision heading: {f} must open with '# D{number}:'")
    if not re.search(r'^> \*\*Status:\*\*\s+\S+', text, re.M):
        fail.append(f"decision status: {f} has no status field")

# ADR-011 target policy: the active target documents must not
# reintroduce consensus-implementation source pinning as identity or
# release vocabulary. (Other plans may mention the dropped policy
# historically; these four are where Phase-3 identity and release
# design reads.) "exact commit(?!ment)" catches commit/release
# pinning while sparing cryptographic-commitment vocabulary.
TARGET_POLICY_DOCS = [
    'plans/packages/target-elements.md',
    'plans/reference/elements-tapscript.md',
    'plans/packages/vectors.md',
    'plans/packages/release.md',
]
prohibited = re.compile(
    r'(?i)source[ -]pin|exact upstream'
    r'|upstream repository identity|exact commit(?!ment)'
)
for f in TARGET_POLICY_DOCS:
    for number, line_text in enumerate(open(f), start=1):
        if prohibited.search(line_text):
            fail.append(f"ADR-011 pin vocabulary in {f}:{number}: {line_text.strip()!r}")

# Current-phase consistency. backlog.md is the authoritative execution
# queue; every other current-phase declaration must agree with it.
def declared_phase(path, pattern):
    m = re.search(pattern, open(path).read(), re.M)
    return int(m.group(1)) if m else None

declarations = {
    'plans/backlog.md': declared_phase(
        'plans/backlog.md', r'^> \*\*Current gate:\*\* Phase (\d+)'),
    'plans/roadmap.md': declared_phase(
        'plans/roadmap.md', r'^Current: Phase (\d+) - '),
    'plans/toolchain-architecture.md': declared_phase(
        'plans/toolchain-architecture.md',
        r'^> \*\*Current phase:\*\* Phase (\d+)'),
    'plans/README.md': declared_phase(
        'plans/README.md',
        r'^## Current phase[\s\S]*?^Phase (\d+) - '),
}

authoritative = declarations['plans/backlog.md']
if authoritative is None:
    fail.append("phase: plans/backlog.md declares no '> **Current gate:** Phase N'")
else:
    for path, phase in declarations.items():
        if phase is None:
            fail.append(f"phase: {path} declares no current phase")
        elif phase != authoritative:
            fail.append(
                f"phase drift: {path} says Phase {phase}, "
                f"backlog.md says Phase {authoritative}"
            )

total_bytes = sum(os.path.getsize(path) for path in md_files)
bytes_by_directory = {}
for path in md_files:
    directory = os.path.dirname(path)
    bytes_by_directory[directory] = bytes_by_directory.get(directory, 0) + os.path.getsize(path)
generated_register_bytes = sum(
    os.path.getsize(path)
    for path in md_files
    if path.startswith('plans/labels/') and os.path.basename(path) != 'README.md'
)

print(f"checked {len(md_files)} plan documents ({total_bytes} bytes)", file=sys.stderr)
for directory in sorted(bytes_by_directory):
    print(f"bytes {directory}: {bytes_by_directory[directory]}", file=sys.stderr)
print(f"generated register bytes: {generated_register_bytes}", file=sys.stderr)
if fail:
    for item in fail:
        print(f"FAIL {item}", file=sys.stderr)
    sys.exit(1)
print("plan tree checks pass", file=sys.stderr)
EOF

git diff --check
git diff --cached --check
echo "==> plans tree is valid" >&2
