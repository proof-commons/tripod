# Phase 1 — Typed Realization Foundation · `phase:roadmap:realization`

> **Status:** Complete
> **Completed:** 2026-07-23
> **Entry:** (`phase:roadmap:baseline`)
> **Package:** [`realization`](../packages/realization.md)
> **Decisions:** D001, D002, D005
> **Target/backend work:** isolated research only

## Completion evidence · `sec:phase1:completion`

Phase 1 exited when every remediation finding (F1-001 through F1-016 and F1-018 through F1-034) closed and the full gate ran green: `scripts/ci.sh` under MSRV 1.88.0 and under stable, the Meson document lane (`meson compile`/`meson test`), the mocked Meson contract, and document reproducibility. The `cargo audit` advisory lane is skipped where the tool is absent (ADR-011). The durable record of that run is the tool-independent gate summary in the backlog archive (`gate:backlog:phase1`), which states what ran and what it established. The toolchain versions, per-lane results, architecture identities, and reproducibility hashes it omits are properties of one machine's run rather than of the phase that run closed, and the phase is what a later reader is asking about.

## Goal · `sec:phase1:goal`

Create the target-independent typed semantic layer shared by model conformance
and compiler analysis.

The phase proves the vocabulary on two operations:

```text
compact-ash
transfer-live-receipts
```

The package boundary implements the target-independent conformance direction of
(`[RZ-rem:oracle:thesis]`) without parsing normative prose or model source.

## Entry conditions · `sec:phase1:entry`

Required before substantive work:

- Phase-0 baseline exists;
- active baseline-review findings are assigned;
- architecture and model checks are green;
- D001 and D002 remain accepted;
- no target-specific field is required by the proposed realization API.

Baseline correctness findings may be fixed in parallel, but all release-relevant
ones must close before Phase 1 exits.

## Deliverables · `sec:phase1:deliverables`

### Crate

Create:

```text
packages/realization
tripod-realization
library: realization
```

Direct first-party dependency:

```text
architecture
```

### Typed identities and domains

Define deterministic typed identities for:

- facts;
- expressions;
- relations;
- observables;
- constructibility requirements;
- lifecycle requirements;
- proof alternatives.

Distinguish:

- amount;
- count;
- boolean;
- asset;
- object;
- owner;
- input reference;
- output reference.

### Expression graph

Provide:

- typed arena or DAG;
- checked arithmetic;
- equality and comparison;
- bounded family aggregation required by the pilots;
- dependency traversal;
- small reference evaluator;
- deterministic structural identity.

### Relation graph

Provide the minimum vocabulary required by the pilots:

- cardinality;
- object/asset recognition;
- class closure;
- authorization;
- equality/conservation;
- output-family closure;
- sponsor isolation;
- projection policy;
- constructibility;
- representation;
- lifecycle.

### Operation adjuncts

Represent:

- input and output families;
- semantic authorization;
- public observables;
- witness availability;
- constructibility;
- lifecycle exits;
- representation capabilities;
- proof alternatives;
- architecture bound references.

### Deterministic derivation

Derive one explicitly scoped `RealizationSpec` from:

```text
architecture
+
typed realization declarations
```

The derivation is pure, deterministic, and filesystem-independent.

## Pilot A — compact ASH · `op:phase1:compact-ash`

Declare:

- at least two ASH inputs;
- maximum `ASH_BATCH_MAX`;
- one ASH output;
- ownerless exact semantic `U` conservation;
- permissionless authorization;
- no owner/operator witness;
- optional isolated sponsor flow;
- no roots;
- transition-certificate projection only;
- public ASH constructibility;
- compact and clear lifecycle exits.

## Pilot B — live transfer · `op:phase1:live-transfer`

Declare:

- nonempty bounded live-receipt input/output families;
- every-owner authorization;
- live-class closure;
- exact aggregate semantic `U` conservation;
- authorized destination freedom;
- optional isolated sponsor flow;
- no roots;
- transition-certificate projection;
- explicit and private-committed value alternatives;
- future transfer, burn, and redemption exits.

## Declassification · `rule:phase1:declassification`

Derive semantic declassification from typed dependencies.

Expected pilot results:

```text
compact-ash:
    public ASH facts remain publicly usable;
    no owner-private disclosure is introduced

live transfer:
    no numerical receipt value disclosure is required by the abstract
    lateral relation
```

Any JSON publication remains derivative. The compiler will consume the typed
result.

## Model conformance · `rule:phase1:model-conformance`

For each pilot:

1. construct valid model worlds and transitions through public APIs;
2. project typed realization facts;
3. evaluate every active relation;
4. compare model certificate/projection with declared observables;
5. mutate one semantic relation or fact at a time;
6. require conformance failure;
7. preserve model and realization as distinct evidence roles.

Shared implementation code must be identified and not described as independent
evidence.

## Evidence · `sec:phase1:evidence`

Required evidence includes:

- ID determinism tests;
- expression type and domain tests;
- dependency traversal tests;
- duplicate and unresolved reference rejection;
- architecture omission/transposition/bound mutations;
- positive model conformance;
- negative model conformance;
- declassification provenance;
- public API integration test;
- repeated derivation equality;
- debug and release workspace checks.

## Non-goals · `sec:phase1:non-goals`

Phase 1 does not define:

- target opcodes;
- transaction positions;
- stack behavior;
- target constructors;
- compiler proof selection;
- linked bundle;
- stable public realization hash;
- complete thirteen-operation realization.

## Exit gate · `gate:phase1:exit`

Phase 1 exits when:

- reopened baseline findings required by the gate are closed;
- the crate is a workspace member;
- only architecture is a direct first-party dependency;
- both pilots are complete and architecture-welded;
- typed domains, expressions, and relations validate;
- declassification is dependency-derived;
- constructibility and lifecycle are explicit;
- model transitions satisfy the declarations;
- focused semantic mutations fail;
- pilot scope cannot be mistaken for release-complete scope;
- repeated derivation is equal;
- no generated file, model source, target type, or planning file is consumed;
- all repository lanes pass and leave the tree clean.
