# ADR-018: Upstream Elements Workspace and Gripe Tracking

**Status:** Decided; adoption proceeds in the Elements checkout itself
**Scope:** The local upstream Elements checkout consumed by target review and
native execution — its branch taxonomy, worktree layout, observation
tracking, and the provenance this repository records about it
**Coordinates with:** the target-compatibility policy of
(`[ADR011-rule:toolchain:target-compatibility]`) and the execution boundary
of (`[ADR015-rule:security:untrusted-source]`)
**Does not establish:** upstream governance, an upstream release pin, or any
claim that local fixes are accepted upstream

---

## Context · `sec:upstream:context`

Target review and native conformance execute against a local checkout of the
upstream Elements implementation. That checkout accumulates three distinct
kinds of local state: upstreamable fixes discovered during review, standing
observations about upstream behavior that are not patches, and the built
binaries the native lane executes. Without a declared shape, these mix into
one drifting branch and the executed revision becomes unstatable — the first
native gate recorded a local fix branch tip as an upstream revision because
nothing distinguished them.

## Branch taxonomy · `rule:upstream:branches`

The checkout carries exactly four branch roles:

- `master` is a pristine upstream mirror. It advances only by fast-forward
  from the upstream default branch and never carries a local commit.
- `fix/` branches carry one upstreamable change each, based on `master`.
  Every fix branch is a candidate upstream contribution; its subject stays
  small enough to rebase cheaply when `master` advances.
- `notes` is a documentation-only branch owning the local observation
  register and its tooling. Because it touches no implementation file, it
  merges cleanly against any upstream state.
- `merged` is a derived integration branch: `master` plus every `fix/`
  branch plus `notes`, recreated from scratch at every synchronization. It
  is never committed to directly; a conflict is resolved in the owning
  topic branch and `merged` is rebuilt.

## Worktree layout · `rule:upstream:worktrees`

The primary checkout stays on `master`, so the default view of the
repository is the upstream head. Two sibling worktrees accompany it. The
`merged` worktree checks out the derived integration branch; builds run
there, and the native executor is pointed at its binaries, so every local
run includes the local fixes by construction. The `work` worktree is the
editing surface: new fixes and register entries are developed there on
whichever `fix/` or `notes` branch owns them, so neither the pristine
mirror nor the derived integration state is ever the branch being edited.

## Gripe register · `rule:upstream:gripes`

Observations about upstream that are not yet patches live in one register on
the `notes` branch. Each entry carries a stable identifier, the upstream
location observed, the observed behavior, why it matters to this project, a
disposition — patch, report upstream, or live with it — and its status. When
a gripe graduates into a fix, its entry names the `fix/` branch and remains
as the record of why the fix exists. The register is the single collection
point: this repository's reference and gate records cite gripes by
identifier rather than restating them.

## Synchronization ritual · `rule:upstream:sync`

One script on the `notes` branch performs the whole synchronization: fetch
upstream and fast-forward `master`; rebase each `fix/` branch and `notes`
onto the new `master`; recreate `merged` from `master` by merging every
topic branch; rebuild in the merged worktree. A synchronization that cannot
complete — a rebase conflict, a failed build — leaves `master` advanced and
the derived state visibly stale rather than half-merged.

## Provenance statement · `rule:upstream:provenance`

A gate or evidence record in this repository never describes an executed
revision as bare upstream identity. It states the executed tip, the upstream
base — the merge base with the upstream default branch — and the census of
local branches included. A binary whose embedded revision does not match the
intended tip is refused for evidence and rebuilt, as the first native gate
already practiced. Per (`[ADR011-rule:toolchain:target-compatibility]`), none
of this is protocol identity; it is test provenance about one run.

## Boundary · `rule:upstream:boundary`

Automation operating from this repository treats the Elements checkout as
read-only source plus runnable build outputs. Creating branches, writing the
register, and committing there are deliberate acts in that repository, under
this shape — never side effects of an attestation batch. The disposable
chains the native lane creates remain governed by
(`[ADR015-rule:security:test-material]`).

## Consequences · `sec:upstream:consequences`

The default view of upstream is always genuinely upstream; local work is
enumerable as the difference between `merged` and `master`; every gripe has
one home with a lifecycle instead of living in scattered worker reports; and
executed-revision statements name exactly what ran. The costs are one extra
worktree, a synchronization script to maintain, and the discipline of never
committing to `master` or `merged` directly.
