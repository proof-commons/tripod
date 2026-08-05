# Tripod Implementation Plans

This directory contains non-normative implementation planning for Tripod, the contract closure compiler, and for the attestation contract that proves its build-out. Plans sit below the specification, the typed architecture, and the executable model, and are not protocol, compiler, target, bundle, ABI, evidence, or release inputs.

## Authority · `rule:plans:authority`

When artifacts disagree:

| Subject | Owner |
|---|---|
| abstract economic interface | the Attestation specification |
| realization meaning, invariants, obligations, and residuals | the realization document |
| finite architecture registries and stable identifiers | typed architecture |
| current executable reference behavior | executable model |
| implemented repository engineering policy | ADRs |
| accepted future implementation direction | planning decisions |
| package boundaries | package contracts |
| sequencing | roadmap and phase cards |
| unresolved implementation alternatives | research notes |
| current execution | backlog |

The executable model is current behavioral evidence and remains authoritative
for its implemented reference behavior. It does not override the typed
architecture or realization contract.

A lower row never overrides an upper owner on that owner's subject.

## Current phase · `phase:plans:current`

```text
Current: Phase 2 - target-independent compiler analysis
```

Current work is indexed by [the backlog](backlog.md). Long-term ordering is
indexed by [the roadmap](roadmap.md).

## Planning labels · `rule:plans:labels`

Plan-local labels are backticked navigation labels:

```text
mint:      `pkg:realization:contract`
citation:  (`pkg:realization:contract`)
```

They are non-normative and non-identity-bearing. Their mints,
same-owner citations, and owner crossings are mechanically checked under
ADR-013 so planning navigation fails closed without becoming protocol,
compiler, target, ABI, deployment, or release identity.

Upstream citations use square brackets:

```text
(`[RZ-sec:realization:representation]`)
(`[A-def:model:classes]`)
(`[ADR011-rule:toolchain:target-compatibility]`)
```

Imported citations are linted under
(`[ADR012-rule:labels:external-citation]`).

## Directory index · `tbl:plans:directories`

| Path | Owner |
|---|---|
| [`labels/`](labels/README.md) | Generated upstream-label registers. |
| [`decisions/`](decisions/README.md) | Accepted cross-package implementation choices. |
| [`packages/`](packages/README.md) | Typed package contracts and boundaries. |
| [`registers/`](registers/README.md) | Cross-package identity and artifact ownership indexes. |
| [`phases/`](phases/README.md) | Phase entry conditions, deliverables, evidence, and exit gates. |
| [`research/`](research/README.md) | Unresolved prototype-driven questions. |
| [`reference/`](reference/README.md) | External technical reference material. |
| [`guides/`](guides/README.md) | Concept guides for upcoming implementation batches. |

## Root files · `tbl:plans:root-files`

| File | Owner |
|---|---|
| [roadmap.md](roadmap.md) | Phase order and dependency overview. |
| [backlog.md](backlog.md) | Current executable task queue only. |

## Planning laws · `rule:plans:laws`

1. Typed Rust is the first-party semantic source.
2. Generated artifacts are one-way derivatives.
3. Plans are never machine-consumed semantics.
4. One fact has one planning owner.
5. Other files cite that owner rather than repeating it.
6. Package dependencies remain explicit and acyclic.
7. Research does not freeze production interfaces.
8. Evidence classes remain separate.
9. Missing required semantics or evidence fails closed.
10. Canonical outputs derive only from explicit inputs.

Cross-cutting rationale lives in accepted decisions rather than package plans.

## Folder ownership · `rule:plans:nearest-readme`

Every planning directory has a `README.md`.

The nearest README indexes each child Markdown file and each child directory.
Ancestor READMEs do not repeat the complete descendant census.

## Weight budget · `rule:plans:weight`

The combined Markdown under:

```text
plans/
adr/
```

must remain below the accepted documentation budget.

The documentation checker reports total bytes, bytes by directory,
authored-file threshold warnings, and generated-register bytes. The budget is a
guardrail against accidental duplication, not a reason to omit useful rationale,
examples, or implementation detail.

## Updating · `rule:plans:update`

A planning change updates the smallest owning document.

Use an ADR for implemented repository policy; a decision for accepted
cross-package direction; a package contract for typed package boundaries; a
phase card for sequencing and gate; a research note for unresolved prototype;
a reference for external facts; and the backlog for current work.

Git history is the archive. Superseded long prose is deleted rather than copied
into another planning folder.

## Status vocabulary · `tbl:plans:status`

| Status | Meaning |
|---|---|
| active | Current maintained work or policy. |
| accepted | Implementation choice approved; implementation may remain. |
| planned | Intended future work not started. |
| prototype required | Interface waits for executable evidence. |
| open | Unresolved question. |
| blocked | Named dependency prevents progress. |
| parked | Deliberately outside the active path. |
| complete | Gate passed and evidence recorded. |
| superseded | Replaced by a named document. |
| dropped | No longer intended; rationale recorded. |

## Supersession · `rule:plans:supersession`

A replacement receives a new durable decision or ADR identity where needed,
names what it supersedes, updates the old status and nearest-folder indexes,
updates package and roadmap consumers, and removes stale active claims. Git
history preserves old prose; plans do not keep duplicate archives.

## Verification · `gate:plans:verification`

The planning tree passes when:

- every directory has a README;
- nearest-README indexes are complete;
- relative links resolve;
- upstream citations resolve;
- generated label registers are current;
- plan-local labels remain non-normative and non-identity-bearing while
  uniqueness and citation checks pass;
- weight limits pass;
- current phase declarations agree;
- no plan claims semantic authority;
- `git diff --check` passes.