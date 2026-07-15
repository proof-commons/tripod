# Phase 7 — Burn, ASH, and Clear · `phase:roadmap:burn-clear`

> **Status:** Planned
> **Entry:** (`gate:phase6:exit`) and accepted public-declassification policy
> **Packages:** tapscript, linker, transaction, vectors
> **Operations:** `burn`, `compact-ash`, `clear`

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
