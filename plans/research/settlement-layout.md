# Research Question: Bounded Settlement Layout · `q:settlement:layout`

> **Status:** Open; prototype and measurement required
> **Blocks:** settlement backend, ABI, calibration, and cycle-to-settlement lifecycle
> **Affected packages:** compiler, tapscript, linker, transaction, vectors, release
> **Depends on:** accepted wide arithmetic and transaction ABI foundation
> **Decisions:** D004, D006
> **Imports:** (`[RZ-sec:operations:settle]`),
> (`[RZ-inv:invariant:no-starve]`),
> (`[RZ-trap:branches:control-vault]`),
> (`[RZ-pin:pins:settle]`)
> **Expected handoff:** accepted canonical settlement layout and placement decision

## Question · `sec:settlement-layout:question`

Which bounded Elements transaction layout and target obligation placement can
enforce settlement exactly while remaining permissionlessly constructible and
within target limits?

The first decisive prototype uses:

```text
settlement batch size = 2
```

Batch 2 is sufficient to expose:

- repeated per-entitlement floors;
- output mapping;
- zero and positive class draws;
- owner repetition;
- local/global obligation placement;
- continuing and terminal branches;
- output uniqueness;
- residue accounting.

No final layout or calibrated `SETTLEMENT_BATCH_MAX` is accepted before this
prototype.

## Semantic relation · `sec:settlement-layout:semantics`

For control:

```text
cycle                   k
original principal      P
original live allocation D_L
original locked allocation D_T
remaining principal     R_P
remaining live          R_L
remaining locked        R_T
```

For entitlement `i`:

```text
owner_i
target_i = k
principal_i = δ_i > 0
```

Per-entitlement draws are:

```text
l_i = floor(δ_i * D_L / P)
t_i = floor(δ_i * D_T / P)
```

Batch totals:

```text
B_P = Σδ_i
B_L = Σl_i
B_T = Σt_i
```

Successor remainders:

```text
R_P' = R_P - B_P
R_L' = R_L - B_L
R_T' = R_T - B_T
```

Every floor occurs before any physical receipt aggregation.

## Branches · `rule:settlement-layout:branches`

### Continuing

When:

```text
R_P' > 0
```

require:

- exactly one successor control;
- unchanged original principal and allocations;
- exact successor remainders;
- successor vault iff `R_L'+R_T'>0`;
- successor vault value exactly that sum;
- no terminal control close;
- no residue projection.

### Terminal

When:

```text
R_P' = 0
```

require:

- no successor control;
- no successor vault;
- control close destruction;
- positive `R_L'+R_T'` destroyed as `U` residue;
- residue projection carries class components separately;
- no residue effect on attestation or monetary computation.

The caller cannot select the branch independently.

## Closed-asset partitions · `rule:settlement-layout:partitions`

Require exact:

```text
ENT inputs
    = entitlement destruction

DIST_CTL input
    = successor control          continuing
    = control-close destruction  terminal

U vault input
    = live receipts
    + locked receipts
    + successor vault
    + terminal residue destruction
```

under active branch terms.

Every source and destination is claimed exactly once.

Sponsor L-BTC is a separate open flow.

## Permissionless rule · `rule:settlement-layout:permissionless`

Construction uses:

- public control;
- public vault;
- public entitlements;
- public arithmetic witnesses;
- public linked bundle and ABI;
- constructor-local sponsor funds.

No entitlement-owner or operator secret is permitted.

The initial profile uses public/explicit entitlement, control, and vault values.

## Architecture shape · `sec:settlement-layout:shape`

Inputs:

```text
one control
vault iff predecessor remainder sum is positive
one..SETTLEMENT_BATCH_MAX entitlements
optional bounded sponsor inputs
```

Outputs:

```text
optional successor control
optional successor vault
bounded live receipts
bounded time-locked receipts
optional sponsor change
activated destruction outputs
```

Settlement uses no global root.

The control is the natural transaction coordinator.

## Candidate matrix · `tbl:settlement-layout:candidates`

| Mint | Candidate | Strength | Main risk |
|---|---|---|---|
| (`candidate:settlement:global-control`) | Control verifies complete batch and outputs | Strong global completeness | Large control program and stack |
| (`candidate:settlement:distributed`) | Entitlements verify local floors/routing; control verifies aggregates | Distributes arithmetic | Cross-input consistency and uniqueness |
| `candidate:settlement:per-entitlement-outputs` | Positive outputs in entitlement order; no aggregation | Direct mapping | Variable output ranks |
| `candidate:settlement:owner-aggregation` | Aggregate after floors by owner/class | Fewer outputs | Loopless grouping and uniqueness |
| `candidate:settlement:provenance-metadata` | Receipt output carries entitlement provenance | Easier mapping | May alter semantic object schema |
| `candidate:settlement:zero-placeholders` | Fixed live/locked slot per entitlement | Simple indexes | Conflicts with positive protocol objects; rejected |
| `candidate:settlement:proof-accumulator` | One compact batch proof | Potential scale | No approved target proof; deferred |

Required prototypes combine:

1. (`candidate:settlement:global-control`) with
   (`candidate:settlement:per-entitlement-outputs`);

2. (`candidate:settlement:distributed`) with the same output policy.

Owner aggregation is a later optimization unless the primary candidates fail.

## Candidate A · `candidate:settlement:global-control`

Proposed input layout:

```text
input 0: control coordinator
input 1 if required: predecessor vault
next N: entitlements in canonical outpoint order
suffix: sponsor inputs
```

Proposed output-family order:

```text
optional successor control
optional successor vault
positive live receipts in entitlement order
positive locked receipts in entitlement order
optional sponsor change
canonical destruction-data families
```

The coordinator:

1. authenticates control and layout;
2. derives vault presence;
3. checks all entitlements and target cycles;
4. verifies both floors per entitlement;
5. derives positive-output presence and ranks;
6. checks receipt owners/classes/values;
7. sums batch values;
8. derives successor counters;
9. selects continuing or terminal branch;
10. checks control/vault outputs or absence;
11. checks all closed-asset partitions;
12. checks destruction outputs and residue projection;
13. checks sponsor boundary.

Local entitlement and vault programs authenticate participation and constructor
identity.

## Candidate B · `candidate:settlement:distributed`

Each entitlement program verifies:

- own canonical entitlement;
- own target cycle;
- own principal;
- own two floor proofs;
- own positive receipt outputs and routing.

The control verifies:

- complete entitlement family;
- aggregate principal and class values;
- output uniqueness and completeness;
- successor/terminal relation;
- closed-asset closure.

Candidate B is accepted only if the control can prove all local terms belong to
one consistent batch without repeating substantially all arithmetic or adding
an unsupported proof object.

## Positive-output mapping · `q:settlement:positive-output-mapping`

Zero draws create no receipt output.

For batch 2, statically derive presence bits:

```text
l_0>0, l_1>0, t_0>0, t_1>0
```

and canonical output ranks through verified prefix counts.

For general calibrated `N`, the backend may statically unroll prefix counts.

Caller-supplied indexes or masks are untrusted and must agree exactly with
derived draw positivity and ABI ranges.

## Threat model · `sec:settlement-layout:threats`

Required failures:

| Mutation | Result |
|---|---|
| cross-cycle entitlement | reject |
| entitlement omitted or counted twice | reject |
| aggregate-before-floor values | reject |
| owner or class swap | reject |
| one-unit redistribution | reject |
| missing or extra receipt | reject |
| two entitlements claim one output | reject |
| unclaimed receipt output | reject |
| zero-valued placeholder | reject |
| wrong/missing/unexpected vault | reject |
| wrong successor counters | reject |
| wrong continuing/terminal branch | reject |
| omitted/excess/class-swapped residue | reject |
| sponsor value masks U mismatch | reject |
| confidential/unclassified `ENT`, `DIST_CTL`, or `U` | reject |
| local program bound to another control | reject |
| private owner opening required | compile/ABI rejection |

## Prototype fixtures · `sec:settlement-layout:fixtures`

Batch-size-2 fixtures include:

1. continuing, both classes positive;
2. continuing, mixed zero draws;
3. repeated owner;
4. continuing with no successor vault;
5. terminal, zero residue;
6. terminal, live residue;
7. terminal, locked residue;
8. terminal, both residue classes;
9. no predecessor vault and zero draws;
10. partial principal;
11. exact terminal principal boundary;
12. sponsorless and maximum sponsor candidate.

Each fixture derives exact expected semantic results from model/realization
before target materialization.

## Prototype · `sec:settlement-layout:prototype`

### Stage 1 — semantic fixture generator

Produce control, vault, entitlement, output, delta, and residue expectations.

### Stage 2 — Candidate A ABI and programs

Implement readable unoptimized batch-2 programs and complete target
transactions.

### Stage 3 — Candidate A adversarial vectors

Run every arithmetic, routing, range, control/vault, residue, sponsor, and
mixed-program mutation.

### Stage 4 — Candidate A measurements

Measure local programs, coordinator, witnesses, complete continuing and
terminal transactions.

### Stage 5 — Candidate B

Implement only enough to establish whether global consistency is sound and
materially cheaper.

Stop if soundness requires duplicating all arithmetic without meaningful
benefit.

### Stage 6 — scaling formula

For the selected candidate, derive symbolic cost in batch size `N`.

Validate against at least:

```text
N=1
N=2
```

and larger counts when feasible.

### Stage 7 — calibration handoff

Provide candidate resource formulas and valid worst-case fixture generation.

The final bound is selected by release calibration, not this research note.

## Vectors · `sec:settlement-layout:vectors`

Required vector groups:

- exact floor/remainder and quotient mutations;
- aggregate-before-floor counterexample;
- owner/class/value routing;
- mapping collision and unclaimed outputs;
- input/output range mutations;
- continuing and terminal control/vault faults;
- all residue cases;
- exact `ENT`, `DIST_CTL`, and `U` partition faults;
- sponsor overlap/interference;
- private-witness constructibility fault;
- mixed local/global programs;
- wrong bundle/constructor/control;
- confidential closed-asset escape.

Accepted transactions compare the complete semantic projection:

- consumed/created objects;
- receipt owner/class/value multiset;
- successor control/vault;
- canonical deltas;
- data outputs;
- residue event;
- fee/open flow.

## Measurements · `sec:settlement-layout:measurements`

For Candidates A and B record:

| Dimension | Candidate A | Candidate B |
|---|---:|---:|
| control program bytes | | |
| local program bytes | | |
| witness bytes | | |
| peak stack | | |
| target operation cost | | |
| transaction weight | | |
| policy verdict | | |
| positive/negative vector locality | | |
| symbolic cost in `N` | | |

Measure separate worst cases for:

- continuing;
- terminal;
- maximum receipt outputs;
- maximum residue/data outputs;
- maximum sponsor candidate;
- deepest control paths.

## Acceptance · `gate:settlement-layout:accept`

Accept only when:

- every entitlement is authenticated and counted once;
- each class floor is exact before aggregation;
- every positive draw reaches the exact owner/class;
- zero outputs are canonically absent;
- output uniqueness/completeness holds;
- control/vault predecessor and successor relations are exact;
- continuing/terminal choice is derived;
- residue destruction and class projection are exact;
- sponsor is isolated;
- every relation has a reachable carrier;
- fresh-process permissionless construction succeeds;
- complete batch-2 target transactions pass;
- predicted and observed resources agree;
- one canonical deterministic ABI exists;
- scaling and calibration handoff are documented.

## Rejection · `gate:settlement-layout:reject`

Reject a candidate if:

- totals can hide owner/class error;
- aggregate-before-floor passes;
- output mappings collide or remain unclaimed;
- local programs can use inconsistent control facts;
- a required relation has no carrier;
- permissionless construction needs private owner data;
- batch 2 exceeds hard limits;
- ABI mapping is ambiguous;
- only transaction-builder checks enforce a target-required relation;
- candidate requires semantic object metadata changes without normative review.

## Result · `sec:settlement-layout:result`

Pending.

## Handoff · `sec:settlement-layout:handoff`

A successful result creates an accepted layout/placement decision and updates:

- compiler placement and layout requirements;
- tapscript programs;
- linker roles;
- transaction ABI;
- permanent vectors;
- resource formulas;
- release calibration;
- [Phase 10](../phases/10-settlement.md).

If no candidate succeeds, record exact target infeasibility. Do not make
settlement owner-assisted or weaken its semantic relation silently.
