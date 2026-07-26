# ADR-017: Path Scope and Host Filesystem Trust

**Status:** Decided; implementation required in the same series
**Scope:** First-party repository inputs, build inputs and outputs,
source-derived references, explicit path arguments, reports, stamps, generated
publications, and publication mirrors
**Coordinates with:** the tracked-entry census of
(`[ADR014-rule:build:tracked-entry-modes]`) and the execution boundary of
(`[ADR015-rule:security:filesystem-paths]`)
**Does not establish:** a sandbox, hostile-filesystem containment, or protection
against time-of-check/time-of-use races

---

## Context · `sec:path:context`

The repository has accumulated checks for lexical aliases, symlinks, symlinked
ancestors, dangling links, hard links, device/inode identity, and missing path
suffixes.

Those checks do not establish filesystem security. A host may replace a path
after validation, remount a directory, expose aliases through bind mounts or
FUSE, or otherwise change what an operating-system path resolves to. Detecting
some aliases while leaving those cases open adds complexity without defining a
real trust boundary.

The repository therefore checks semantic path derivation and honest build
configuration. The host owns filesystem integrity and path resolution.

## Trusted roots · `rule:path:roots`

Let:

```text
R = repository root supplied by the canonical build
B = canonical build root, R/build
```

Canonical repository path claims apply only to paths lexically beneath `R` or
`B`.

The repository does not claim to authenticate the host mapping of either root.
If the host replaces, remounts, aliases, or mutates a root or one of its
components, the result is outside the repository trust boundary.

The mocked Meson tree under `R/build/mocks` remains part of `B`.

## Repository shape · `rule:path:repository-shape`

The tracked repository contains only ordinary Git blobs.

Allowed tracked modes are:

```text
100644
100755
```

The repository contains no:

```text
tracked symlinks
gitlinks
submodules
```

Git and the ADR-014 census audit enforce this centrally over the complete
tracked set. Individual tools do not repeat symlink, ancestor, hard-link, or
inode checks for build-supplied repository paths.

Untracked nonignored entries are rejected by the clean-tree gate. Ignored build
state is host-controlled.

## Build-supplied paths · `rule:path:build-paths`

The canonical Meson graph supplies repository inputs beneath `R` and build
outputs beneath `B` or another explicit publication destination beneath `R`.

A first-party tool receiving those paths validates the required content,
schema, and semantic role. It does not re-establish repository membership,
walk ancestors for symlinks, compare device/inode identity, or prove that the
host has not changed the path since the central audit.

The tracked-source claim belongs to Git and ADR-014, not to every reader.

## Source-derived references · `rule:path:derived-references`

A path derived from parsed source text is untrusted until constrained.

It must resolve through:

- an explicit caller-supplied allowlist; or
- an explicit declared root and documented resolution rule.

Absolute references, parent traversal, ambiguity, undeclared members, and
include cycles fail closed.

An allowlist constrains what source text may select. It does not authenticate
the host filesystem beneath an explicitly supplied allowlist entry.

## Explicit path arguments · `rule:path:explicit-paths`

An explicit path argument is a caller-granted filesystem capability.

An explicit input grants read authority to the object the host resolves from
that path.

An explicit output grants write authority to the object or directory entry the
host resolves from that path.

When an explicit path is not lexically beneath `R` or `B`, its containment,
aliasing, permissions, persistence, mount behavior, and pathname integrity are
entirely the caller's and host's responsibility.

The command still validates content and reports I/O failures. It makes no path
security claim outside the trusted roots.

## Output-role uniqueness · `rule:path:output-roles`

A command with several output roles rejects two roles naming the same lexically
normalized absolute path.

Normalization is lexical:

1. resolve a relative path against its documented base;
2. make it absolute;
3. remove `.` components;
4. resolve `..` components without filesystem access.

Generic output validation does not use:

- filesystem canonicalization;
- symlink resolution;
- device/inode comparison;
- hard-link detection;
- mount identity.

Callers must not alias distinct output roles through symlinks, hard links,
mounts, namespaces, or other host mechanisms.

A hard link is not semantic identity. First-party outputs are identified by
role, path, schema, and bytes.

## Publication correctness · `rule:path:publication`

Path trust and byte-publication correctness are separate.

First-party writers still:

1. receive every output destination explicitly;
2. derive no unrelated destination from source content;
3. stage complete bytes in the destination directory;
4. flush staged bytes before publication;
5. publish by rename where supported;
6. preserve unchanged output through compare-if-changed;
7. stage every member of a multi-output publication before publishing the
   first, where practical;
8. report honestly that separate final renames are not one transaction.

These are honest-tool correctness properties. They do not protect against a
host replacing a path before, during, or after publication.

## Stamps · `rule:path:stamps`

The two ADR-014 stamp classes remain distinct.

A checker `--stamp` is an empty freshness fact:

- its report is published first;
- an existing nonempty stamp is refused without truncation;
- failure never freshly dates the stamp.

An always-stale generator, Cargo, or publication marker is not a freshness
oracle and may use ordinary marker creation or `touch`.

Neither class attempts filesystem-alias detection beyond
(`rule:path:output-roles`).

## TOCTOU boundary · `rule:path:toctou`

The repository does not defend against time-of-check/time-of-use races.

In particular, it does not claim safety if the host or another process:

- replaces a file after validation;
- replaces an ancestor directory;
- changes a symlink target;
- creates or removes a hard link;
- changes a mount;
- swaps an inode;
- mutates a destination during publication.

Code must not describe preflight path checks as closing those races.

Untrusted code runs in an externally established, credential-free and
appropriately isolated environment under ADR-015.

## Operation-specific checks · `rule:path:local-checks`

A package may retain a stronger local check only for a concrete operation-level
correctness hazard, such as incompatible concurrent writers targeting one log.

The package must document:

- the exact hazard;
- the additional check;
- the remaining host race;
- why lexical output-role uniqueness is insufficient.

No current package-level exception becomes a repository-wide filesystem
security claim.

## Consequences · `sec:path:consequences`

- Repository shape is checked once by Git and the census audit.
- The repository carries no symlinks or submodules.
- Build-supplied source paths do not receive repeated alias analysis.
- Source-derived references remain strictly constrained.
- Explicit external paths remain caller and host capabilities.
- Generic output-role checks are lexical, not inode-based.
- Hard-link and symlink-parent tests are removed from generic helpers.
- Atomic staging and compare-if-changed remain required.
- TOCTOU and hostile filesystem mutation are explicitly outside scope.

## Rejected alternatives · `sec:path:alternatives`

### Detect every filesystem alias

Rejected because the detection is incomplete and cannot establish a security
boundary.

### Repeat repository checks in every tool

Rejected because Git mode and census membership have one central owner.
Repeated checks create divergent policy without protecting against host
replacement.

### Use inode identity as semantic identity

Rejected because inode identity is local, mutable, platform-specific, and
unrelated to semantic or artifact identity.

### Perform no path validation

Rejected because source-derived traversal, undeclared references, duplicate
output roles, stale census members, and partial publication are first-party
correctness defects.

## Verification · `gate:path:verification`

This decision is implemented when:

- the census audit reads the complete tracked Git mode census;
- every tracked entry has mode `100644` or `100755`;
- tracked symlinks and gitlinks fail the canonical gate;
- no submodule is present;
- canonical build inputs are lexically beneath `R`;
- canonical build outputs are lexically beneath `B` or an explicitly declared
  publication destination beneath `R`;
- source-derived references reject absolute paths, traversal, ambiguity,
  undeclared members, and cycles;
- shared output-role validation compares lexically normalized paths only;
- generic path helpers do not resolve symlinks or compare device/inode identity;
- documentation makes no TOCTOU or hostile-filesystem containment claim;
- any stronger package-local check names its exact hazard and residual race;
- checker stamps retain their empty-stamp contract;
- always-stale markers remain outside that contract;
- focused tests, the mocked Meson contract, the complete repository gate, and
  the clean-tree check pass.
