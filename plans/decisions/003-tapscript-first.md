# D003: Preserve Multiple-Backend Boundaries; Implement Tapscript First · `dec:backend:tapscript-first`

> **Status:** Accepted
> **Class:** Backend architecture
> **Depends on:** (`dec:architecture:realization-layer`)
> **Imports:** (`[RZ-rem:overview:conformance-claim]`),
> (`[RZ-sec:oracle:boundary]`)
> **Supersedes:** none

## Choice · `rule:backend:tapscript-first`

Keep realization semantics and compiler analysis independent of one execution
target.

Implement Elements tapscript as the first production backend.

Keep Simplicity parked until the first backend has demonstrated the semantic,
linking, transaction, and evidence boundaries.

```text
realization
    ↓
compiler analysis
   ├──────────────────┐
   ▼                  ▼
Elements target     future target
   ↓                  ↓
tapscript           Simplicity
```

## Boundary · `rule:backend:boundary`

Above the backend boundary:

- semantic facts and expressions;
- relation IDs;
- public observables;
- constructibility;
- lifecycle;
- representation capabilities;
- proof alternatives;
- disclosure provenance;
- target capability requirements;
- semantic vector definitions.

Below the boundary:

- opcodes or combinators;
- instruction selection;
- stack or target-program scheduling;
- concrete constructors;
- concrete witness encoding;
- target transaction layout;
- resource formulas;
- relocatable programs.

## Tapscript-first rationale · `sec:backend:rationale`

Tapscript provides the shortest path to testing the realization against the
intended Liquid capability set.

It exposes the concrete limits that matter early:

- transaction introspection;
- no general loop assumption;
- stack scheduling;
- bounded collection lowering;
- metadata-dependent constructors;
- wide arithmetic;
- transaction layout;
- target resource accounting;
- confidential transaction construction.

A second backend before those boundaries exist would multiply unresolved
interfaces rather than test them.

## Consequences · `sec:backend:consequences`

- `realization` contains no target detail.
- Compiler relation identity is backend-independent.
- Target packages expose typed capabilities.
- Backends select only compiler-approved proof alternatives.
- Unsupported target relations fail closed.
- Every backend emits separate bundle, ABI, resource, and execution identities.
- Semantic vectors may be shared; concrete target vectors remain
  backend-specific.
- The initial deployment may be tapscript-only.

## Simplicity reactivation · `gate:backend:simplicity-reactivation`

Simplicity becomes active only after:

- complete target-independent realization coverage for the selected scope;
- demonstrated compiler proof/disclosure/placement boundaries;
- at least one complete linked tapscript operation;
- understood transaction ABI and evidence interfaces;
- an exact reviewed Simplicity target can be typed;
- a concrete assurance or deployment need justifies the work.

Its first milestone should be one already implemented semantic pilot, not the
complete protocol.

## Does not authorize · `sec:backend:limits`

This decision does not authorize:

- tapscript concepts in `RealizationSpec`;
- a stack-shaped “universal” low-level IR;
- lowering semantic strength to the intersection of all targets;
- implementing two production backends simultaneously;
- claiming backend diversity before a second backend exists;
- describing one target’s execution evidence as another’s;
- treating target-language formal semantics as proof of compiler, linker, or
  transaction correctness.

## Supersession · `rule:backend:supersession`

This choice may be replaced if the project deliberately selects one permanent
backend or another primary target.

A replacement must address semantic identity, package boundaries, ABI,
evidence, and deployment migration.

## Verification · `gate:backend:boundary`

The boundary is demonstrated when tapscript emits a complete pilot without
target-specific fields appearing in architecture, realization, or compiler
relation identity.
