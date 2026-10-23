# Phase 7 — Burn, ASH, and Clear · `phase:roadmap:burn-clear`

> **Status:** Active — Guide 15 is chartered as the execution guide at `plans/guides/guide_fifteen.md`; Wave 0 is the first authorized wave and opens on its backlog entry record with its provisioned pinned target and other entry obligations recorded; implementation of new semantics is held until its exit (`gate:guide15-exec:wave0`). Both phase-entry conditions hold on the amended Phase-6 exit (`sec:phase6:exit-assessment`) and accepted public policy (`sec:public-opening:result`); chartering the guide also required disposition of the accepted compact-ASH candidate at its recorded scope and the concept's detailed entry and preflight requirements through (`tab:guide15-exec:preflight`).
> **Entry:** (`gate:phase6:exit`) on its amended assessment and accepted public-declassification policy; these remain the two phase-entry conditions, distinct from the execution guide's compact-candidate and detailed entry/preflight dispositions.
> **Packages:** architecture, realization, compiler, target-elements, tapscript, linker, transaction, vectors; target-elements-conformance only for required target-generic evidence support, with model conformance and labels verification.
> **Operations:** `burn`, `compact-ash`, `clear`

## Change statement · `sec:phase7:change-statement`

Phase 7 carries destruction lineage as a second compiled contract for independence evidence inside Guide 15, with no change to phase order or dependencies; inherited debt and preflight close in Wave 0, the second contract runs in Wave 1, and burn, ASH, and clear follow in Waves 2–14 (`rule:roadmap:change`); (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`).

The supporting evidence is `plans/drafts/guide15-admission-study.md` and (`abst:lineage:scope`), (`sec:lineage:requirements`), (`sec:lineage:measurements`) and (`sec:lineage:decisions`); the latter states the contract's own BURN and MERGE laws, six measurement obligations, package responsibilities, and nine decisions, with no assumed positive relay minimum or input bound.

Affected package contracts are architecture, realization, compiler, target-elements, tapscript, linker, transaction, and vectors, as mapped in the exemplar study's §5; labels retains registration and citation checking, target-elements-conformance changes only for a demonstrated target-generic evidence need, and model evidence remains independent of compiler-facing semantics.

Affected research notes are `plans/research/public-declassification.md`, `plans/research/state-constructor.md`, `plans/research/wide-arithmetic.md`, `plans/research/compiler-algorithms.md`, `plans/research/linker-algorithms.md`, and `plans/research/test-performance.md`; later findings enter those notes at their own scope without promoting candidate results into calibration.

The backlog effect is the Phase-7 task table in `plans/backlog.md`, with a DONE charter import and TODO Wave-0 bites carrying the four inherited debt rows; the next gate is (`gate:guide15-exec:wave0`). Versioning impact is none: the workspace remains `0.7.0-dev`, with `0.7.N-dev` steps named as landed when citations require them under the existing identity rules.

## Goal · `sec:phase7:goal`

Implement the share-nothing owner-authorized burn path and the permissionless
public ASH maintenance path while preserving event provenance, closed-asset
closure, and conservative accounting.

## Deliverables · `sec:phase7:deliverables`

### Burn

Implement:

- nonempty bounded live-receipt inputs;
- every-owner authorization;
- no ASH or root input;
- exactly one fresh positive ASH;
- optional positive live receipt change;
- exact `U` relation;
- canonical burn records;
- optional isolated sponsor flow;
- burn event projection;
- no clear or residue projection.

### Public ASH

Implement the accepted ASH representation:

- explicit or public-committed value;
- authenticated public opening where needed;
- explicit `U` asset;
- durable public construction data;
- compact and clear lifecycle.

### Clear

Implement:

```text
B = sum(ASH inputs)
X = min(B, Y_L, Y - 1)
```

Require:

- nonempty bounded ASH input family;
- public/openable ASH values;
- canonical STATE succession;
- no RESV edge;
- `X > 0`;
- exact `tag-recon` destruction;
- residual ASH iff `B-X > 0`;
- clear event projection;
- all unaffected state fields unchanged;
- optional isolated sponsor flow;
- permissionless construction.

## Event authentication · `rule:phase7:event-authentication`

Burn credit requires both:

1. event-type anchor:
   a canonical burn transition with live inputs and one fresh ASH;

2. value anchor:
   accepted record total does not exceed fresh ASH.

This implements (`[RZ-sec:ledger:authentication]`).

Compaction and clear must never produce a burn event.

## Required vectors · `sec:phase7:vectors`

### Burn

- valid full/partial burn;
- multi-input and multi-owner;
- missing owner;
- time-locked input;
- ASH input;
- wrong change class;
- undeclared `U` output;
- no/two ASH outputs;
- wrong ASH amount;
- record under-claim;
- record over-claim;
- duplicate/skipped record ordinal;
- record mutation after signing;
- bare record payload without burn event;
- confidential closed asset.

### Compact ASH

- value-preserving compaction;
- no burn event;
- public opening use;
- malformed/missing opening rejection.

### Clear

- exact clear;
- oversized ASH partial clear;
- residual ASH;
- no clear to zero supply;
- zero-progress rejection;
- wrong decrement;
- wrong destruction amount/tag;
- RESV input/output substitution;
- wrong STATE successor;
- private owner secret requirement;
- residual opening unavailable to a fresh constructor.

## Representation evidence · `sec:phase7:representation`

If private source receipts are supported:

- compare explicit and private burn semantic projections;
- prove fresh ASH public authentication;
- prove residual blinding closure;
- prove unrelated future compaction and clear;
- keep explicit `U` asset identity.

If direct private burn is not accepted, require the documented normalization or
explicit-only path and do not claim direct support.

## Resource evidence · `sec:phase7:resources`

Measure:

- maximum burn inputs/change/records candidates;
- maximum ASH compact batch;
- maximum ASH clear batch;
- public-opening verification;
- residual branch;
- maximum sponsor candidate;
- deepest control paths.

## Exit gate · `gate:phase7:exit`

Phase 7 exits when:

- burn event type and record value anchors are independent and complete;
- over-claiming invalidates records without erasing burn provenance;
- compact ASH is attestation-silent;
- clear uses actual authenticated ASH values;
- clear preserves the `Y-1` floor and RESV non-use;
- ASH lifecycle is publicly constructible;
- no owner recovery or closed-asset escape exists;
- model/target event projections agree;
- all representation, resource, and relation reports pass;
- output remains reproducible and the tree remains clean.
