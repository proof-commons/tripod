# Phase 10 — Settlement Prototype and Implementation · `phase:roadmap:settlement`

> **Status:** Planned; prototype-driven
> **Entry:** (`gate:phase5:exit`), (`gate:phase8:exit`),
> (`gate:phase9:exit`)
> **Research:** [`settlement layout`](../research/settlement-layout.md)
> **Operation:** `settle-distribution`

## Goal · `sec:phase10:goal`

Select and implement one canonical bounded settlement layout that preserves
per-entitlement floors, owner/class routing, control/vault accounting, terminal
residue, permissionless construction, and target feasibility.

No final layout or `SETTLEMENT_BATCH_MAX` is accepted before a complete
batch-size-2 prototype.

## Entry conditions · `sec:phase10:entry`

Required:

- exact wide arithmetic pattern accepted;
- entitlement/control/vault constructors available;
- transaction ABI foundation proven;
- explicit public initial settlement representation selected;
- model settlement and accounting fixtures green.

## Prototype gate · `gate:phase10:prototype`

Build and compare at least:

### Candidate A

```text
global distribution-control coordinator
+
canonical per-entitlement positive receipt outputs
+
no owner aggregation
```

### Candidate B

```text
entitlement-local floor/routing checks
+
global control aggregate and closure checks
```

Candidate B is accepted only if global uniqueness and completeness hold without
duplicating essentially all arithmetic for no benefit.

## Required semantic relation · `sec:phase10:semantics`

For each entitlement `i` and class `c`:

```text
m_c(i) = floor(δ_i * D_c / P)
```

Floors occur per entitlement before physical output aggregation.

Batch totals update:

```text
remaining principal
remaining live value
remaining time-locked value
```

Continuing settlement creates exact successor control and conditional vault.

Terminal settlement:

- creates no successor control or vault;
- destroys control;
- destroys positive remaining `U` residue;
- projects live and time-locked residue separately;
- does not feed residue into monetary or attestation computation.

## Permissionless rule · `rule:phase10:permissionless`

Construction uses:

- public control;
- public vault;
- public entitlements;
- public arithmetic witnesses;
- public bundle/ABI;
- constructor-local sponsor funds.

No entitlement-owner or operator secret.

## Required vectors · `sec:phase10:vectors`

### Batch structure

- one and two entitlements;
- cross-cycle entitlement;
- omitted or duplicated entitlement;
- wrong batch count/rank;
- sponsor overlap.

### Arithmetic and routing

- exact and remainder floors;
- aggregate-before-floor substitution;
- owner swap;
- class swap;
- one-unit redistribution;
- missing/extra/duplicate/unclaimed receipt output;
- zero-valued placeholder;
- wrong output mapping.

### Control and vault

- continuing exact successor;
- terminal exact closure;
- wrong predecessor/successor vault;
- missing/unexpected vault;
- wrong counters;
- wrong branch;
- duplicate control/vault.

### Residue

- zero/live/time-locked/both residues;
- omitted/excess residue;
- class component swap;
- wrong tag;
- residue routed to receipt;
- false event projection.

### Closed assets and programs

- wrong/confidential `ENT`, `DIST_CTL`, or `U`;
- flow claimed twice;
- local program bound to another control;
- mixed operation program;
- unreachable relation carrier.

## Resource evidence · `sec:phase10:resources`

Measure complete batch-size-2 transactions for:

- continuing with both classes;
- mixed zero draws;
- repeated owner;
- terminal zero residue;
- each residue class;
- no predecessor vault;
- maximum sponsor candidate;
- deepest control paths.

Compare candidates by:

- script bytes;
- witness bytes;
- stack;
- operation cost;
- transaction weight;
- proof locality;
- relation coverage clarity;
- generalized cost in batch size.

## Decision handoff · `rule:phase10:handoff`

The research result must create or update an accepted decision fixing:

- family order;
- coordinator;
- local/global placement;
- output mapping;
- witness ABI;
- continuing/terminal program roles;
- resource formula;
- calibration interface.

If no target candidate succeeds, record target infeasibility. Do not weaken
semantic settlement or make it owner-assisted silently.

## Exit gate · `gate:phase10:exit`

Phase 10 exits when:

- batch-size-2 target-native evidence exists;
- one layout strategy is accepted;
- every entitlement is checked exactly once;
- per-entitlement floors and owner/class routing are exact;
- control/vault continuing and terminal relations match the model;
- residue event/accounting projection is exact;
- permissionless construction has no private witness;
- every relation has reachable positive and negative coverage;
- a useful calibrated bound appears feasible;
- final bound remains release calibration output;
- reports are deterministic and the tree remains clean.
