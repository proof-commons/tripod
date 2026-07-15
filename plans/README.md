# Tripod Implementation Plans

This directory contains non-normative implementation planning for Tripod, the contract closure compiler, and for the attestation contract that proves its build-out. Plans sit below the specification, the typed architecture, and the executable model, and are not protocol, compiler, target, bundle, ABI, evidence, or release inputs.

## Authority · `rule:plans:authority`

When artifacts disagree:

1. Layer-0 economic specification;
2. Realization conformance contract;
3. typed architecture;
4. implemented ADRs;
5. accepted planning decisions;
6. package contracts;
7. phase cards;
8. research candidates;
9. active backlog.

The executable model is current behavioural evidence and remains authoritative
for its implemented reference behavior. It does not override the typed
architecture or realization contract.

## Current phase · `phase:plans:current`

```text
Phase 1 - typed realization foundation
```

Current work is indexed by [the backlog](backlog.md). Long-term ordering is
indexed by [the roadmap](roadmap.md).

## Planning labels · `rule:plans:labels`

Plan-local labels are backticked navigation labels:

```text
mint:      `pkg:realization:contract`
citation:  (`pkg:realization:contract`)
```

They are non-normative and unlinted.

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
| [`phases/`](phases/README.md) | Phase entry conditions, deliverables, evidence, and exit gates. |
| [`research/`](research/README.md) | Unresolved prototype-driven questions. |
| [`reference/`](reference/README.md) | External technical reference material. |

## Root files · `tbl:plans:root-files`

| File | Owner |
|---|---|
| [roadmap.md](roadmap.md) | Phase order and dependency overview. |
| [backlog.md](backlog.md) | Current executable task queue only. |
| [toolchain-architecture.md](toolchain-architecture.md) | Existing cross-package plan pending migration into decisions and package contracts. |

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
authored-file threshold warnings, and generated-register bytes.

Density is preferred over repeated explanation.

## Updating · `rule:plans:update`

A planning change updates the smallest owning document.

Use an ADR for implemented repository policy; a decision for accepted
cross-package direction; a package contract for typed package boundaries; a
phase card for sequencing and gate; a research note for unresolved prototype;
a reference for external facts; and the backlog for current work.

Git history is the archive. Superseded long prose is deleted rather than copied
into another planning folder.

## Verification · `gate:plans:verification`

The planning tree passes when:

- every directory has a README;
- nearest-README indexes are complete;
- relative links resolve;
- upstream citations resolve;
- generated label registers are current;
- plan-local labels remain outside linting;
- weight limits pass;
- current phase declarations agree;
- no plan claims semantic authority;
- `git diff --check` passes.