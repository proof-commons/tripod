# `tripod-realization`

`realization` is the target-independent typed semantic layer between the finite
attestation architecture and two separate consumers:

- executable-model conformance; and
- future compiler analysis.

It fills the gap between the architecture's finite registries and the formulas,
dependencies, constructibility, lifecycle, representation, and observables
needed below them. The package contract is
[plans/packages/realization.md](../../plans/packages/realization.md).

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

## Quickstart

One derivation, then either consumption. This example does both: derives the
Phase-1 pilots, reads the typed structure, and evaluates a concrete compact-ASH
observation against it.

```rust
use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId,
};
use realization::{
    Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObject,
    ObservedObjectKind, ObservedObjectRef, ObservedSide, ObservedValue, OperationObservation,
    ProtocolAmount, RealizationScope, RepresentationMode, derive,
};

// 1. Derive. One pure, deterministic call: validates the architecture,
//    derives every declaration, builds and validates the graphs.
let spec = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
    .expect("the two pilots derive");

// The scope is explicit; a value outside it is a typed absence.
assert!(spec.scope().contains(OperationId::CompactAsh));
assert!(spec.operation(OperationId::Burn).is_none());

// 2a. Compiler-style consumption: read the typed structure. The
//     projection carries no graph handles, so it compares stably.
let projection = spec.project();
assert_eq!(projection.operations.len(), 2);
assert_eq!(projection.relations.nodes.len(), spec.relations().count());
assert_eq!(
    spec.constructibility_authorizations(OperationId::CompactAsh)
        .unwrap(),
    &[realization::ConstructibilityAuthorization::Permissionless],
);

// 2b. Conformance-style consumption: build a primitive observation of one
//     concrete operation and evaluate every active relation against it.
//     Two ASH inputs of 40 and 60 compact into one output of 100.
let input0 = ObservedObjectRef { side: ObservedSide::Input, ordinal: 0 };
let input1 = ObservedObjectRef { side: ObservedSide::Input, ordinal: 1 };
let output0 = ObservedObjectRef { side: ObservedSide::Output, ordinal: 0 };

let ash = |side, ordinal, value| ObservedObject {
    reference: ObservedObjectRef { side, ordinal },
    kind: ObservedObjectKind::Declared(ObjectId::Ash),
    asset: ObservedAsset::Declared(AssetId::U),
    value: ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
    owner: None,
    representation: RepresentationMode::Explicit,
};

let observation = OperationObservation {
    operation: OperationId::CompactAsh,
    objects: vec![
        ash(ObservedSide::Input, 0, 40),
        ash(ObservedSide::Input, 1, 60),
        ash(ObservedSide::Output, 0, 100),
    ],
    // Compact ASH is permissionless: no protocol signer is required.
    protocol_signers: BTreeSet::new(),
    sponsor_signers: BTreeSet::new(),
    canonical_partition: ObservedCanonicalPartition {
        issuances: Vec::new(),
        flows: vec![ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: vec![input0, input1],
            destinations: vec![output0],
            movement_kind: Some(DeltaKind::OwnerlessLateral),
            destructions: Vec::new(),
        }],
    },
    open_flows: Vec::new(),
    root_effects: Vec::new(),
    projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
    bounds: BTreeMap::from([
        (BoundId::AshBatchMax, Count::new(64)),
        (BoundId::FeeSponsorInputMax, Count::new(16)),
    ]),
};

let report = spec.evaluate_operation(&observation).unwrap();

// 3. Read the report. Conformance and evidence completion are two
//    separate questions, and the report answers them separately.
assert!(report.is_conformant());
assert!(!report.has_semantic_failure());
assert_eq!(report.failed_relations().count(), 0);
```

Note the last three assertions. `is_conformant()` means no relation failed or
was blocked; it does **not** mean every proof obligation is discharged. A
conformant report can still carry `EvidenceRequired` verdicts whose proof
belongs to the model kernel or the target — ask `is_evidence_complete()` for
that, and `required_external_evidence()` for the list.

## Public-API tour

Everything below is re-exported at the crate root.

### Scope and binding — start here

- `RealizationScope::phase1_pilots() -> Self` — the two-operation pilot scope.
- `RealizationScope::from_operations(impl IntoIterator<Item = OperationId>) -> Result<Self, RealizationError>`
  — rejects an empty scope (`EmptyScope`) and a repeated operation
  (`DuplicateScopeOperation`).
- `RealizationScope::operations(&self) -> &[OperationId]`,
  `contains(&self, OperationId) -> bool`.
- `RealizationScope::validate_against(&self, &Architecture) -> Result<(), RealizationError>`
  — every scoped operation must exist in that architecture.
- `RealizationScope::try_complete(..) -> Result<CompleteRealizationScope, RealizationError>`
  — the only route to `CompleteRealizationScope`; fails with `IncompleteScope`
  listing every missing operation. A pilot scope can never reach it.
  `CompleteRealizationScope::as_scope(&self) -> &RealizationScope`.
- `ArchitectureBinding::from_architecture(&Architecture) -> Result<Self, RealizationError>`
  — validates the architecture and records its identity. Readers:
  `architecture_schema_version() -> u32`, `realization_version() -> &str`,
  `semantic_hash() -> [u8; 32]`.

### Derivation — the single entry point

- `derive(&Architecture, RealizationScope) -> Result<ScopedRealizationSpec, RealizationError>`
  — the one pure deterministic derivation. It performs architecture validation,
  deterministic indexing, fact construction, expression typing and dependency
  analysis, relation construction, authorization/observability/constructibility/
  lifecycle/representation construction, declassification derivation,
  bidirectional validation, and explicit scope validation, in that order.
- `ScopedRealizationSpec` — validated on construction and internally consistent
  afterwards. Every invariant-bearing field is private; the `compile_fail`
  doctests on the type are the standing proof that no external consumer can
  desynchronize the declarations from the graphs. Readers:
  - `architecture() -> &ArchitectureBinding`
  - `scope() -> &RealizationScope`
  - `operations() -> impl Iterator<Item = (OperationId, &OperationRealization)>`
  - `operation(OperationId) -> Option<&OperationRealization>` — `None` outside
    the scope
  - `relations() -> impl Iterator<Item = &RelationDeclaration>` and
    `relation(&RelationId) -> Option<&RelationDeclaration>`
  - `declassification() -> &DeclassificationAnalysis`
  - `constructibility_authorizations(OperationId) -> Result<&[ConstructibilityAuthorization], RealizationError>`
    — `Err(OperationOutsideScope)` outside the scope, never a silent default
  - `validate_against(&Architecture) -> Result<(), RealizationError>` — the
    binding must still match
  - `evaluate_operation(&OperationObservation) -> Result<ConformanceReport, RealizationError>`
  - `project() -> ScopedRealizationProjection`
- `project_scoped_realization(&ScopedRealizationSpec) -> ScopedRealizationProjection`
  — the free-function form of `project`.
- `ScopedRealizationProjection` — public fields `architecture`, `scope`,
  `operations`, `expressions`, `relations`, `constructibility`, `lifecycle`,
  `disclosure`, `declassification`. Evaluation order is deliberately absent:
  among mutually independent nodes no unique topological order exists, so
  publishing the one the graph library happened to pick would put a traversal
  artifact into a value meant for stable comparison. A consumer needing a
  schedule derives one from the projected graph under its own stated rule.

### Stable keys (`identity`)

`RelationId::new(OperationId, RelationKind, RelationSubject)` with readers
`operation()`, `kind()`, `subject()`; `ProofAlternativeId::new(RelationId,
ProofKind)` with `relation()`, `proof()`; `ExprId::fact(FactId)` and
`ExprId::relation(RelationId, ExpressionRole)`. Supporting enums: `FactId`,
`RelationKind`, `RelationSubject`, `ExpressionRole`, `ProofKind`,
`TransactionSide`. These are complete typed keys — never graph positions,
source locations, declaration ordinals, target identities, or backend handles.

### Semantic domains (`domain`, `value`)

- `ProtocolAmount` — opaque; `new(u64) -> Result<Self, RealizationError>`
  (`AmountOutOfDomain` outside `0 <= v < 2^51`), `ZERO`, `ONE`, `get()`,
  `is_zero()`, `checked_add`, `checked_sub`, `checked_sum` (all fail closed
  with `AmountOverflow` / `AmountUnderflow`).
- `PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE: u64` — the `2^51` bound.
- `Count` — cardinality, deliberately a distinct type; `new(u64) -> Self`
  (infallible), `ZERO`, `ONE`, `get()`, `is_zero()`, `checked_add`
  (`CountOverflow`). A count is not an amount merely because both are integers.
- `RepresentationMode` — explicit or private-committed value representation.
- `SemanticType`, `SemanticValue` (with `semantic_type()`, `as_bool()`,
  `as_count()`, `as_amount()`, `as_owner_set()`), `OwnerId([u8; 32])`.

### Declarations and graphs

Each analysis is a typed declaration set plus a canonical projection; the
projections are what a consumer compares.

- Expressions: `ExpressionNode`, `ExpressionDeclaration`, `DependencyEdge`,
  `ExpressionDependencyProjection`, `ExpressionGraphProjection`, plus the
  evaluator inputs/outputs `FactValues` (`insert`, `get`, `iter`) and
  `EvaluatedExpressions` (`get`, `bool`, `iter`).
- Relations: `Relation` (the relation bodies), `RelationDeclaration`,
  `RelationEdge`, `RelationDependencyDeclaration`,
  `RelationDependencyProjection`, `RelationGraphProjection`, with
  `ConstructibilityClass`, `CardinalityMaximum`, `ExpectedCanonicalDelta`.
- Operations: `OperationRealization` and `OperationRealizationProjection`
  (`project()`).
- Constructibility: `ConstructibilityNodeId`, `ConstructibilityNode`,
  `ConstructibilityEdge`, `ConstructibilityEdgeRole`, `RequirementStrength`,
  `WitnessRole`, `ConstructibilityDependencyDeclaration`,
  `ConstructibilityDependencyProjection`, `ConstructibilityGraphProjection`,
  and the discharge pair `AvailabilityClass` /
  `ConstructibilityAuthorization::discharges(AvailabilityClass) -> bool`.
  `Public` and `SponsorLocal` availability are always dischargeable —
  sponsor confinement is a separate rule — while a private class must be
  named by the authorization case.
- Lifecycle: `LifecycleNodeId`, `LifecycleNode`, `LifecycleEdge`,
  `LifecycleDependencyDeclaration`, `LifecycleDependencyProjection`,
  `LifecycleGraphProjection`.
- Declassification: `DeclassificationAnalysis`, `DisclosureNodeId`,
  `DisclosureNode` (`id()`), `DisclosureEdge`, `DisclosureSeed`,
  `DisclosureReason`, `InitialVisibility`,
  `DisclosureDependencyDeclaration`, `DisclosureDependencyProjection`,
  `DisclosureGraphProjection`, and `phase1_declassification() -> DeclassificationAnalysis`.
  Disclosure cycles are permitted: the analysis is the monotone least fixed
  point of seeded reasons over incoming dependencies, so a cycle discloses
  exactly the facts reachable from seeded public requirements and no unseeded
  component.

### Observation and evaluation

- `OperationObservation` — public fields `operation`, `objects`,
  `protocol_signers`, `sponsor_signers`, `canonical_partition`, `open_flows`,
  `root_effects`, `projections`, `bounds`. Methods:
  `validate_and_normalize(self) -> Result<Self, RealizationError>`,
  `object(ObservedObjectRef) -> Option<&ObservedObject>`,
  `flow_role(ObservedObjectRef) -> ObservedFlowRole`, `declared_objects(..)`.
  `validate_observation(..)` is the free-function form of the check.
- Component types: `ObservedObject`, `ObservedObjectRef`, `ObservedSide`,
  `ObservedObjectKind`, `ObservedAsset`, `ObservedValue` (`is`, `is_zero`),
  `ObservedCanonicalFlow`, `ObservedCanonicalPartition`,
  `ObservedDestructionLeg`, `ObservedIssuance`, `ObservedOpenFlow`,
  `ObservedFlowRole`, `ObservedRootEffect`, `ObservedRootEffectKind`.
- `ConformanceReport` — public fields `operation` and `verdicts` (sorted by
  relation). Methods `is_conformant()`, `has_semantic_failure()`,
  `is_evidence_complete()`, `required_external_evidence()`,
  `failed_relations()`, `verdict(&RelationId) -> Option<&RelationVerdict>`
  (binary search over the sorted verdicts).
- `RelationVerdict { relation, status }` with `RelationStatus`:
  - `Passed` — evaluated and satisfied;
  - `StaticallyValidated` — settled at derivation, nothing left to observe;
  - `EvidenceRequired { requirement }` — well-formed but only external evidence
    can discharge it; it never becomes `Passed` at this boundary;
  - `Blocked { prerequisites }` — a prerequisite relation did not pass;
  - `Failed { reason }` — carries a `RelationFailure`.
- `RelationFailure` — the focused runtime failure classes:
  `CardinalityBelowMinimum`, `CardinalityAboveMaximum`,
  `UndeclaredObjectFamily`, `ObjectRecognition`, `AmountConservation`,
  `MissingOwnerAuthorization`, `UnexpectedProtocolAuthorization`,
  `SponsorIsolation`, `SponsorEnvelopeMultiplicity`, `RootPolicy`,
  `ProjectionPolicy`, `Constructibility`, `Representation`,
  `CanonicalDeltaPolicy`, `OpenFlowPolicy`, `ExpressionPredicate`.
- `ExternalEvidenceRequirement::SubstrateConservation { operation, asset }` —
  the one premise the evaluator cannot establish itself. A runtime pass over
  the sponsor-erased observation is not evidence that the substrate accepted
  whole-transaction value conservation, so the requirement stays visible in the
  report for the model kernel or the target to discharge.

## Error handling

`RealizationError` is the single error root for the whole crate. It derives
`thiserror::Error`, so it implements `Display` and `std::error::Error`, and it
is `PartialEq` — variants can be matched or compared directly, as the public
API test does. Every fallible function in the crate returns it.

There are no error subtypes and no panicking alternative to any fallible call.
Which families arise where:

| Operation | Expect |
|---|---|
| `RealizationScope::from_operations` | `EmptyScope`, `DuplicateScopeOperation` |
| `RealizationScope::validate_against`, `try_complete` | `OperationOutsideArchitecture`, `IncompleteScope { missing }` |
| `ArchitectureBinding::from_architecture` | `ArchitectureValidationFailed { errors }` (wrapping `Vec<architecture::ManifestError>`), `ArchitectureHashUnavailable` |
| `derive` | the scope and binding families above, plus every declaration-integrity family: duplicate/unknown/cyclic expressions and relations, type mismatches (`ExpressionTypeMismatch`, `BinaryOperandTypeMismatch`, `InvalidSumType`, `InvalidOrderedType`), foreign-ownership violations (`ForeignExpressionOwnership`, `ForeignRelationOwnership`, `ForeignRelationDependency`, `ForeignProofAlternativeBinding`, `ForeignConstructibilityOwnership`, `ForeignDisclosureOwnership`), architecture-agreement violations (`MissingArchitectureOperation`, `ArchitectureOperationMismatch`, `MissingArchitectureRelation`, `RelationKindMismatch`), and graph-policy violations (`ConstructibilityCycle`, `RelationDependencyCycle`, `PermissionlessPrivateDependency`, `SponsorValueRead`) |
| `ProtocolAmount` / `Count` arithmetic | `AmountOutOfDomain`, `AmountOverflow`, `AmountUnderflow`, `CountOverflow` |
| `evaluate_operation`, `validate_observation` | the observation families: `UnknownObservedObject`, `DuplicateObservedObject`, `DuplicateObservedReference`, `WrongObservedReferenceSide`, `ObservedCanonicalPartitionOverlap`, `ObservedOpenFlowOverlap`, `AnchorInObservedOpenFlow`, `DuplicateObservedRoot`, `MissingBoundValue`, plus `UnknownRelation` |
| `spec.constructibility_authorizations`, `spec.validate_against` | `OperationOutsideScope`, `ArchitectureBindingMismatch` |

An important distinction: a `RealizationError` from `evaluate_operation` means
the *observation itself* was malformed — the evaluator could not run. A
relation that ran and did not hold is not an error; it is a `Failed` verdict
inside an `Ok(ConformanceReport)`. Callers must check the report, not just the
`Result`.

## What this package deliberately does not do

- It does not decide whether a model transition is accepted. Model conformance
  is a **post-execution observation check**; asking the realization evaluator
  whether a transition should be accepted would make the comparison circular.
- It does not establish model behavior, target capability, backend correctness,
  transaction construction, or deployment evidence.
- It does not emit or name anything target-specific: no opcodes, stack indexes,
  transaction positions, tapleaves, control blocks, commitment prefixes, or
  deployment network values.
- It does not perform I/O, and it does not ingest any generated publication —
  including the generated declassification JSON, which is derivative only.
- It does not publish a realization hash or a generated realization file, and
  it exposes no local graph handle or mutable registry.
- It does not accept a partial value as a complete one. `CompleteRealizationScope`
  is unreachable from a pilot scope.

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

## Relationship to neighbors

```text
architecture
    ↓
realization
   ├────────▶ model conformance
   └────────▶ compiler
```

- **`tripod-architecture`** is the only permitted first-party
  dependency. It owns the finite identifier registries; this crate cites them
  rather than restating them, and records which architecture it derived from in
  `ArchitectureBinding`.
- **`tripod-model`** consumes this crate for conformance only, in one
  direction: it projects executed transitions into `OperationObservation`
  values and checks the resulting `ConformanceReport`.
- **`tripod-compiler`** takes a `ScopedRealizationSpec` straight into
  `compiler::bind_input`, along with a `CompilationScope` that must be a subset
  of this realization's scope.
- Forbidden dependencies (per the package card): `model`, `compiler`,
  `target-elements`, `tapscript`, `simplicity`, `linker`, `transaction`,
  `vectors`, `release`, `artifacts`. No target or evidence package may become
  necessary to construct a realization.

Graph-shaped internals use the workspace-owned Petgraph dependency directly
(D007). Petgraph indices remain local graph handles, never semantic identity,
and no graph wrapper type crosses the public boundary.

## Publications and identity

Phase 1 publishes no realization hash or generated realization file.

Typed Rust values are consumed directly. A publication is added only when a
real consumer or review need exists and after its schema and identity policy
are reviewed.
