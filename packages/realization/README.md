# `tripod-realization`

`realization` is the target-independent typed semantic layer between the finite
attestation architecture and two separate consumers:

- executable-model conformance; and
- future compiler analysis.

## Input

The crate consumes a validated `architecture::Architecture` and
realization-owned typed Rust declarations.

It does not parse:

- generated architecture JSON or TOML;
- generated declassification;
- model source or tests;
- realization Markdown;
- Attestation LaTeX;
- plans or ADRs;
- target reference prose;
- target programs;
- filesystem or environment state.

## Phase-1 scope

The initial scope contains exactly:

```text
compact-ash
transfer-live-receipts
```

A Phase-1 value is explicitly partial. It cannot be converted into a complete
realization until every operation in the validated architecture is present.

## Ownership

Architecture continues to own finite identifiers such as operation, object,
asset, root, bound, tag, quantity, witness, and invariant-clause IDs.

This crate owns target-independent typed keys for semantic facts, expressions,
relations, observables, constructibility, lifecycle, representation, and proof
alternatives.

Local graph positions and arena handles are never semantic identity.

## Model boundary

The executable model runs independently.

Model conformance projects concrete predecessor, request, successor, and
certificate facts into a realization observation and checks every active
realization relation. Model execution must not ask the realization evaluator
whether a transition should be accepted, because doing so would make the
conformance comparison circular.

## Compiler boundary

The future compiler consumes validated realization relations, proof
alternatives, constructibility, lifecycle, representation, disclosure
provenance, and evidence requirements.

The realization crate contains no target opcodes, stack indexes, target
transaction positions, tapleaves, control blocks, target commitment prefixes,
or deployment network values.

## Publications and identity

Phase 1 publishes no realization hash or generated realization file.

Typed Rust values are consumed directly. A publication is added only when a
real consumer or review need exists and after its schema and identity policy
are reviewed.