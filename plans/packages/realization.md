# Typed Realization Package · `pkg:realization:contract`

> **Status:** Active
> **Phase:** [Phase 1](../phases/01-realization.md)
> **Package:** `tripod-realization`
> **Library:** `realization`
> **Decisions:** [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D005](../decisions/005-value-representation.md),
> [D007](../decisions/007-petgraph-graph-substrate.md)

## Purpose · `sec:realization:purpose`

`realization` declares the target-independent semantic relation shared by model
conformance and compiler analysis.

```text
architecture
    ↓
realization
   ├────────▶ model conformance
   └────────▶ compiler
```

It fills the gap between the architecture’s finite registries and the formulas,
dependencies, constructibility, lifecycle, representation, and observables
needed below them.

The package implements typed semantics corresponding to
(`[RZ-rem:oracle:thesis]`) and the representation boundary of
(`[RZ-sec:realization:representation]`).

## Dependencies · `sec:realization:dependencies`

Allowed first-party direct dependency:

```text
architecture
```

Graph-shaped realization internals use the workspace-owned Petgraph dependency
directly under [D007](../decisions/007-petgraph-graph-substrate.md). Petgraph
indices remain local graph handles, not semantic identity.

Forbidden dependencies:

```text
model
compiler
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

No target or evidence package may become necessary to construct a realization.

## Typed inputs · `sec:realization:inputs`

The package consumes:

- a validated `architecture::Architecture`;
- realization-owned typed Rust semantic declarations;
- architecture IDs and bound references.

It does not consume concrete deployment values during Phase 1.

## Typed outputs · `sec:realization:outputs`

The principal output is a validated, explicitly scoped realization containing:

- architecture binding;
- semantic facts;
- typed expressions;
- semantic relations;
- operation declarations;
- authorization requirements;
- state and root effects;
- public observables;
- constructibility requirements;
- witness availability;
- lifecycle requirements;
- representation capabilities;
- proof alternatives;
- derived declassification.

Exact Rust fields remain provisional until the two pilots pass.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn derive(
   architecture: &architecture::Architecture,
   scope: RealizationScope,
) -> Result<RealizationSpec, RealizationError>;
```

## Forbidden inputs · `sec:realization:forbidden`

The package must not parse or scrape:

- generated architecture JSON or TOML;
- generated declassification;
- model source or tests;
- realization Markdown;
- Attestation LaTeX;
- plans or ADR prose;
- target references;
- target programs;
- environment variables;
- filesystem state.

This follows [D001](../decisions/001-typed-rust-source.md).

## Identity ownership · `sec:realization:identity`

Architecture continues to own:

```text
AssetId
RootId
ObjectId
OperationId
QuantityId
WitnessId
InvariantClauseId
BoundId
TagId
```

Realization may own typed IDs for:

```text
Fact
Expression
Relation
Observable
Constructibility requirement
Lifecycle requirement
Proof alternative
```

IDs must be deterministic and independent of:

- source paths;
- line numbers;
- insertion order where order is not semantic;
- target selection;
- deployment values;
- wall-clock time.

No stable realization hash is published until its canonical projection and
migration policy receive a separate decision.

## Semantic domains · `sec:realization:domains`

The type system must distinguish at least:

- boolean;
- count;
- protocol amount;
- asset;
- object;
- owner;
- input reference;
- output reference.

Later operations may add cycle, block age, ratio, root, address, tag, and
canonical-order domains.

A count is not an amount merely because both are integers.

Arithmetic is checked and fail-closed.

## Derivation · `rule:realization:derive`

The package exposes one pure deterministic derivation from architecture and
typed declarations.

Derivation performs:

1. architecture validation;
2. deterministic indexing;
3. fact construction;
4. expression typing and dependency analysis;
5. relation construction;
6. authorization, observability, constructibility, lifecycle, and
   representation construction;
7. declassification derivation;
8. bidirectional validation;
9. explicit scope validation.

A pilot-scoped value must not be accepted as a full realization.

## Pilot A — compact ASH · `op:realization:compact-ash`

The declaration must cover:

- ASH input minimum and `ASH_BATCH_MAX`;
- exactly one ASH output;
- explicit semantic `U` conservation;
- permissionless authorization;
- optional isolated sponsor flow;
- no root use;
- transition-certificate projection;
- no burn, clear, or residue projection;
- public constructibility;
- future ASH compaction and clear lifecycle.

## Pilot B — live transfer · `op:realization:live-transfer`

The declaration must cover:

- nonempty bounded live-receipt inputs and outputs;
- owner authorization for every input;
- live-class closure;
- aggregate semantic `U` conservation;
- caller-selected authorized destinations;
- optional isolated sponsor flow;
- no root use;
- transition-certificate projection;
- explicit and private-committed value alternatives;
- future transfer, burn, and redemption lifecycle.

## Declassification · `rule:realization:declassification`

Declassification derives from typed dependencies.

Seed facts are those required by:

- public state;
- public events or interface output;
- permissionless construction.

The result records reason and provenance.

The compiler consumes the typed result. A JSON publication, if retained, is
derivative only.

## Assurance boundary · `sec:realization:assurance`

This package establishes:

- typed semantic completeness for its declared scope;
- architecture binding;
- expression and relation validity;
- dependency-derived disclosure;
- declared constructibility and lifecycle.

It does not establish:

- model behavior;
- target capability;
- backend correctness;
- transaction construction;
- deployment evidence.

Model conformance is a separate test boundary.

## Milestones · `tbl:realization:milestones`

| Label | Deliverable |
|---|---|
| `milestone:realization:crate` | Crate and architecture dependency |
| `milestone:realization:identity` | Typed IDs and domains |
| `milestone:realization:expressions` | Expression arena and evaluator |
| `milestone:realization:relations` | Pilot relation vocabulary |
| `milestone:realization:derive` | Deterministic derivation and validation |
| `milestone:realization:compact-ash` | First pilot |
| `milestone:realization:live-transfer` | Second pilot |
| `milestone:realization:declassification` | Typed derived disclosure |
| `milestone:realization:model-check` | Model conformance |

## Exit gate · `gate:realization:exit`

Phase 1 exits when:

- both pilots are complete;
- architecture and realization coverage agree bidirectionally;
- model transitions satisfy the declarations;
- focused semantic mutations fail;
- declassification is derived;
- pilot scope is explicit;
- output is deterministic;
- no target type or generated-file input appears;
- workspace checks pass without modifying tracked files.

## Research boundary · `sec:realization:research`

Phase 1 is not blocked by target research.

Later operation declarations must import accepted results from:

- [STATE constructor](../research/state-constructor.md);
- [wide arithmetic](../research/wide-arithmetic.md);
- [public declassification](../research/public-declassification.md);
- [settlement layout](../research/settlement-layout.md).

Research may refine proof alternatives and lifecycle requirements. It must not
insert target opcodes into realization types.

## Error vocabulary · `sec:realization:errors`

See [`errors/realization.md`](errors/realization.md).

## Open questions · `sec:realization:open`

- How is partial pilot scope represented?
- Does the amount domain remain realization-owned until a second consumer exists?
- Are fact, expression, and relation identities explicit keys, structural hashes, or both?
- How do structurally shared relations retain operation-specific provenance?
- When is a public realization identity safe to mint?
