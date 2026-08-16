# D002: Introduce a Target-Independent Realization Layer · `dec:architecture:realization-layer`

> **Status:** Accepted
> **Class:** Package architecture
> **Depends on:** (`dec:source:typed-rust`)
> **Imports:** (`[RZ-rem:overview:conformance-claim]`),
> (`[RZ-rem:oracle:thesis]`),
> (`[RZ-sec:realization:representation]`)
> **Supersedes:** none

## Choice · `rule:realization-layer:choice`

Add:

```text
packages/realization
Cargo package: tripod-realization
Rust library: realization
```

The package sits between architecture and compiler:

```text
architecture
    ↓
realization
   ├──────────────▶ model conformance
   └──────────────▶ compiler analysis
```

Architecture continues to own finite identifiers and enumerated relations.

Realization owns the target-independent semantic detail needed by the model
conformance boundary and compiler.

## Ownership · `tab:realization-layer:ownership`

| Architecture owns | Realization owns |
|---|---|
| assets, roots, objects | semantic facts |
| operations and IDs | typed expressions |
| cardinalities and bounds | formulas and state assignments |
| authorization classes | semantic relations |
| quantities and readers | public observables |
| tags and projections | constructibility requirements |
| dependencies and decisions | witness availability |
| stable discriminants | lifecycle requirements |
| architecture identities | representation capabilities and proof alternatives |
| deployment-profile schema | derived declassification |

Realization reuses architecture IDs. It does not create a parallel operation or
object registry.

## Constraints · `rule:realization-layer:constraints`

`realization` depends initially on:

```text
architecture
```

It must not depend on:

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
generated publications
```

It contains no:

- target opcode;
- stack index;
- target transaction position;
- tapleaf or control block;
- target commitment prefix;
- Simplicity combinator;
- deployment network value.

## Consequences · `sec:realization-layer:consequences`

The package provides:

- deterministic fact, expression, and relation identities;
- typed arithmetic and semantic domains;
- operation relation graphs;
- public observable definitions;
- dependency-derived declassification;
- explicit constructibility and lifecycle requirements;
- target-independent proof alternatives;
- validation against architecture coverage;
- a small reference evaluator for semantic tests.

The model is checked against these declarations. The compiler consumes them.

Neither role replaces the other.

## Initial scope · `rule:realization-layer:pilots`

The vocabulary is proven first on:

1. `compact-ash`;
2. `transfer-live-receipts`.

The first tests bounded ownerless permissionless aggregation.

The second tests owner authorization, class closure, sponsor isolation, and
value-representation latitude.

A pilot realization must carry explicit partial scope. It must not be accepted
as a full release realization.

## Does not authorize · `sec:realization-layer:limits`

This decision does not authorize:

- moving target layout into architecture;
- parsing the realization document as a DSL;
- compiling directly from model callbacks;
- publishing a stable realization hash before its canonical projection is
  reviewed;
- designing a universal theorem language before the pilots;
- collapsing model and compiler into one shared implementation;
- introducing speculative target abstractions.

## Supersession · `rule:realization-layer:supersession`

The package boundary may be replaced only by another typed target-independent
semantic source that supports:

- architecture binding;
- model conformance;
- compiler consumption;
- dependency-derived disclosure;
- constructibility;
- lifecycle;
- representation latitude;
- deterministic identity.

## Verification · `gate:realization-layer:exit`

The decision is demonstrated when both pilot operations are completely
declared, architecture-welded, model-checked, deterministically derived, and
free of target-specific types.
