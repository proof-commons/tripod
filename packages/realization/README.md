# `tripod-realization`

## Wave-1 independence evidence

Keyed declarations and sealed owner binding express permissionless SOURCE/MERGE and their external reading obligations beside the attestation declaration set. The compiler consumes those declarations through shared analysis; keyed identity is not an extra attestation operation. SOURCE's four and MERGE's three evidence rows remain explicit, while the two runtime clauses apply to consumed keyed members. Existing attestation projections stay exact under the ten realization/compiler pins and complete eligibility controls (`[PLAN-rec:w1b12:gate]`).

The final public pipeline, six families and exact-byte native joins have the evidence indexed at (`[PLAN-rec:w1b16:packet]`). The reader reconstructs SOURCE singleton and MERGE union sets; no record is introduced by MERGE, no accumulator is carried and no reader predicate becomes a covenant clause. One reviewed template accepts any 32-byte key as data with conditional commitment success or typed refusal; per-key premises are never inherited from the representative native pair. This is the bounded realization account in (`[PLAN-sec:w1b16:packet-domain]`), not unconditional security or deployment finality.

`realization` is the target-independent typed semantic layer between the finite
attestation architecture and two separate consumers:

- executable-model conformance; and
- analysis by the `compiler` crate.

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

## Declared operations

The declaration set covers `compact-ash`, `transfer-live-receipts`, and `announce-maturity`. `RealizationScope::phase1_pilots()` retains the two-operation compact-ASH and live-transfer pilot scope; an explicit `RealizationScope::from_operations` scope reaches `announce-maturity` through the public `derive` entry point.

A scope containing fewer than all operations in the validated architecture is explicitly partial and cannot be converted into a complete realization until every operation is present.

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
    ObservedObjectKind, ObservedObjectRef, ObservedOperationRegion, ObservedSide, ObservedValue,
    OperationObservation,
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
    // The observation interface explicitly claims the operation's objects.
    region: ObservedOperationRegion {
        members: BTreeSet::from([input0, input1, output0]),
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
    },
    other_regions: Vec::new(),
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
  - `evaluate_operation_with_operator_membership(&OperationObservation, &ObservedOperatorMembership) -> Result<ConformanceReport, RealizationError>`
    — supplies the operator-membership decision that the witness-free form
    leaves external
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
`operation()`, `kind()`, `subject()`; `ProofAlternativeId::new(RelationId, ProofKind)` with `relation()`, `proof()`; `ExprId::fact(FactId)` and
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
- `Cycle` — an ordinal over the full `u64` domain; `Cycle::new(value: u64) -> Self`, `Cycle::ZERO`, `Cycle::MAX`, `Cycle::get(self) -> u64`, and `Cycle::checked_add(self, other: Self) -> Result<Self, RealizationError>`, which fails closed with `CycleOverflow`. A cycle is neither a count nor an amount: the three integer domains never convert implicitly, preserving cardinality, value, and sequence-position semantics.
- `BurnAddress([u8; 32])` — realization-owned record-address bytes with equality semantics, no target encoding or authentication claim.
- `RepresentationMode` — explicit or private-committed value representation.
- `SemanticType`, `SemanticValue` (with `semantic_type()`, `as_bool()`,
  `as_count()`, `as_amount()`, `as_owner_set()`), `OwnerId([u8; 32])`.

### STATE metadata, transition, and canonical bytes (`state`, `state_codec`)

- `StateMetadata` — public fields `omega: ProtocolAmount`, `y_l: ProtocolAmount`, `y_t: ProtocolAmount`, `q: ProtocolAmount`, `cycle: Cycle`, and `maturity: Maturity`; `with_maturity(self, maturity: Maturity) -> Self` copies all five other fields and replaces only `maturity`.
- `Maturity` — the closed variants `Unannounced`, `Announced { cycle: Cycle }`, and `Complete`.
- `AnnouncementLeadBounds` — `new(minimum: Cycle, maximum: Cycle) -> Result<Self, RealizationError>` refuses a zero minimum or an inverted pair with `InvalidAnnouncementLeadBounds { minimum, maximum }`; `minimum(self) -> Cycle` and `maximum(self) -> Cycle` return the inclusive bounds, while `window(self, current: Cycle) -> Result<(Cycle, Cycle), MaturityTransitionRefusal>` returns the inclusive earliest and latest announceable cycles and refuses checked-arithmetic overflow with `CycleArithmeticOverflow`.
- `announce_maturity(predecessor: &StateMetadata, announced_cycle: Cycle, bounds: AnnouncementLeadBounds) -> Result<StateMetadata, MaturityTransitionRefusal>` — total: every input yields either one semantic successor or one refusal; the successor copies `omega`, `y_l`, `y_t`, `q`, and `cycle` and replaces only `maturity`.
- `MaturityTransitionRefusal` — the closed members, in declaration order, are `PredecessorAlreadyAnnounced`, `PredecessorMaturityComplete`, `AnnouncementBelowMinimum`, `AnnouncementAboveMaximum`, and `CycleArithmeticOverflow`; `ALL: &'static [Self]` enumerates them in that order and `name(self) -> &'static str` returns each stable kebab-case name.
- `clear_ash<I: Ord>(predecessor: &StateMetadata, ash: &[(I, ProtocolAmount)], batch_max: usize) -> Result<ClearTransitionResult, ClearTransitionRefusal>` — total over its argument types: every input yields X, the residual and one successor that differs from the predecessor in `y_l` alone, or one refusal; its authenticated-family domain and check order are stated under Clear below.
- `ClearTransitionResult` — public fields `clear_amount: ProtocolAmount`, `residual: ProtocolAmount` and `successor: StateMetadata`.
- `ClearTransitionRefusal` — the closed members, in declaration and check order, are `EmptyBatch`, `BatchTooLarge`, `DuplicateInput`, `SupplyOutOfDomain`, `Sealed`, `ZeroAshAmount`, `AshTotalOutOfDomain` and `ZeroProgress`; `ALL: &'static [Self]` enumerates them in that order and `name(self) -> &'static str` returns each stable kebab-case name.
- Codec constants — `STATE_METADATA_DOMAIN: &[u8] = b"tripod/state-metadata"`, `STATE_METADATA_SCHEMA: u32 = 1`, and `STATE_METADATA_BYTES: usize = 86`; the width is 21 domain bytes + 4 schema bytes + four 8-byte amounts + 8 current-cycle bytes + 1 maturity-discriminant byte + 8 announced-cycle bytes + 4 nonce bytes + 8 reserved bytes = 86. The canonical encoding fixes its domain separator, schema revision, field order, field widths, integer byte order, maturity discriminants, representation-nonce position, reserved-field behavior, and rejection of trailing bytes. `STATE_METADATA_LAYOUT` and `StateMetadataRegionClass` classify its ten rows; `STATE_METADATA_VARIABLE_RANGE` and `STATE_METADATA_VARIABLE_BYTES` derive the contiguous 53-byte variable span; `state_metadata_variable_region` extracts that span and `rebuild_state_metadata` restores the canonical 86-byte layout from it (`0.6.234-dev`).
- `StateRepresentationNonce` — `ZERO`, `new(value: u32) -> Self`, `get(self) -> u32`, and `next(self) -> Option<Self>`, which advances deterministically and returns `None` at `u32::MAX`.
- `EncodedStateMetadata` — public fields `semantic: StateMetadata` and `representation: StateRepresentationNonce`.
- `encode_state_metadata(semantic: &StateMetadata, representation: StateRepresentationNonce) -> Vec<u8>` — always returns exactly `STATE_METADATA_BYTES` bytes.
- `decode_state_metadata(bytes: &[u8]) -> Result<EncodedStateMetadata, StateMetadataRefusal>` — strict and returns one refusal per rejected input, checking in decoder order for an input shorter than the domain (`WrongLength`), a wrong domain (`WrongDomain`), a truncated schema (`WrongLength`), an unsupported schema (`UnsupportedSchema`), a truncated later fixed-width field (`WrongLength`), an out-of-domain amount (`AmountOutOfDomain`), a malformed zero-payload maturity (`MaturityPayloadMalformed`) or unknown maturity discriminant (`UnknownMaturityDiscriminant`), a nonzero reserved field (`ReservedFieldNonzero`), and trailing bytes (`TrailingBytes`).
- `StateMetadataRefusal` — the closed members, in declaration order, are `WrongDomain`, `UnsupportedSchema`, `WrongLength`, `AmountOutOfDomain`, `UnknownMaturityDiscriminant`, `MaturityPayloadMalformed`, `ReservedFieldNonzero`, and `TrailingBytes`; `ALL: &'static [Self]` enumerates them in that order and `name(self) -> &'static str` returns each stable kebab-case name.

Integers are big-endian, and the nonce belongs to representation rather than semantics: semantic projection erases it.

The three STATE refusal sums are closed, distinct from each other and from `RealizationError`; constructor-layer names reserved by the guide are not decode refusals. The separate observation extension adds a fourth sum, `BurnClearRefusal`, described below.

#### Clear

`clear_ash(&StateMetadata, &[(input_identity, ProtocolAmount)], batch_max)` returns `ClearTransitionResult { clear_amount, residual, successor }` or a `ClearTransitionRefusal`. It derives `X=min(B,y_l,Y−1)`, returns residual B−X, and changes only stored `y_l`. Zero residual denotes absence of residual ASH, not a zero-valued output. The existing `StateField::ALL` owns the six-field census; Y is derived and nonce/encoding remain outside semantic metadata.

The function consumes an authenticated family with faithful input identities and a resolved batch bound. Scalar domains are enforced by `ProtocolAmount`; batch and duplicate checks precede derived supply and Sealed; then, for each input in family order, positivity and then checked addition; then progress. Zero ASH defensively refuses at the model's `WrongShape` boundary. Whole-world lookup, asset and STATE recognition, fee validation and transaction finishing are outside function equivalence. The helper does not authenticate an offered family itself. The batch bound is a caller-supplied `usize`; bite 2.1 does not bind it to the architecture's `AshBatchMax`. Bite 2.2's projection validators read it from the bound snapshot.

`src/tests/clear_state_tests.rs` covers all refusals with one-coordinate accepted controls, all three clamp limiters and four ties, both residual branches, field copying and independent wide arithmetic. The model's `src/tests/state_metadata_conformance_tests.rs` retains the refusal-to-Guard agreement table and function comparisons that execute the model's `ClearAsh::apply` on branch-local worlds; `src/tests/ash_clear_tests.rs` compares accepted executed successors, including nonzero q/cycle and announced maturity. Branch-local probes are explicitly distinct from reachable histories. Maturity's existing implementation and tests are preserved. This is bite 2.1 of (`[PLAN-step:wave2:bite-2-1]`): declaration, observation binding and emitted-path evidence remain later work.

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

`AnnounceMaturity` is available through an explicit `RealizationScope::from_operations` scope and the public `derive` entry point; its declaration covers State and sponsor structure, evidence-required operator authorization, the public representation pair, the six-exit State lifecycle census, and the fifteen public facts; the typed State transition owns the maturity law and the declaration carries no cycle-window expression; `phase1_pilots()` remains the original two-operation scope.

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
- `ObservedOperatorMembership` — carries the attested operation, a two-valued
  `OperatorMembershipDisposition` (`Member` or `NonMember`), and provenance text
  naming the deciding component and run. Its fields are private;
  `new(OperationId, OperatorMembershipDisposition, impl Into<String>) -> Result<Self, EmptyOperatorMembershipProvenance>`
  refuses empty or whitespace-only provenance, and its accessors are
  `operation() -> OperationId`,
  `disposition() -> OperatorMembershipDisposition`, and
  `provenance() -> &str`. Construction authenticates neither the decision nor
  its association with observation bytes, and no key, digest, identity, or
  profile enters the realization. The producer that verifies a deployment
  authorization and maps it to the model's abstract operator, and the binding
  to finalized bytes, live in later waves.
- `RelationFailure` — the focused runtime failure classes:
  `CardinalityBelowMinimum`, `CardinalityAboveMaximum`,
  `UndeclaredObjectFamily`, `ObjectRecognition`, `AmountConservation`,
  `MissingOwnerAuthorization`, `OperatorMembership`,
  `UnexpectedProtocolAuthorization`,
  `SponsorIsolation`, `SponsorEnvelopeMultiplicity`, `RootPolicy`,
  `ProjectionPolicy`, `Constructibility`, `Representation`,
  `CanonicalDeltaPolicy`, `OpenFlowPolicy`, `ExpressionPredicate`.
- `ExternalEvidenceRequirement` records the two premises the evaluator cannot
  establish itself: `SubstrateConservation { operation, asset }` and
  `OperatorAuthorization { operation }`. A runtime pass over the sponsor-erased
  observation is not evidence that the substrate accepted whole-transaction
  value conservation, so the substrate requirement stays visible in the report
  for the model kernel or the target to discharge. The operator requirement is
  discharged only by a witness supplied through the new entry point, which
  fails both operator relations closed on a rejecting or mismatched witness.

### Bound Burn/Clear projections

`BurnClearContext::read_observation()` returns its immutable normalized common snapshot; `BoundBurnClearObservation::read_observation()` returns the immutable checked extension. Neither accessor allows a successful binding to be mutated afterwards.

`BurnClearObservation` is a separate internal extension, with `BurnClearContext` and a `BurnClearPayload::{Burn, Clear, Compact}`. `BurnClearContext::bind_observation(common, transaction, certificate)` normalizes and retains the complete common snapshot. Transaction and certificate are opaque producer-supplied equality tokens, not hashes published by realization, signatures or proof of authentication. A trusted independent producer must supply their provenance and primitive STATE fields. `validate_burn_clear_observation(common, expected_context, extension)` compares that independently expected context, normalizes the current common observation and checks exact region/object/data associations, returning an immutable `BoundBurnClearObservation`. Keeping the entire snapshot makes unrelated-context changes conservative binding refusals; no digest or external serialization is introduced.

The alternative was optional shared carriers on `OperationObservation`. At adoption it would migrate fourteen literal-initializer files plus the common struct; this bite adds its own separate test fixture. The adoption census includes the production maturity initializer in `packages/transaction/src/state_abi.rs`. The extension pays explicit binding checks and snapshot storage instead, leaving all those files and common normalization unchanged. It does not activate Burn/Clear declarations or the old evaluator: the declaration/evaluator integrations belong to bites 2.3b, 2.4 and 2.6. Missing extension returns `MissingExtension`; it never supplies defaults. `BoundBurnClearObservation::resolve_state_fact` resolves only operation-qualified Clear `FactId::StateField` values from its carried STATE pair, using the existing exhaustive `StateField` vocabulary. Legacy maturity keeps its old entry and missing-carrier behavior.

The primitive carriers are `ObservedState { reference, metadata }`, `BurnRecord { ordinal, address, amount, tag, fresh_ash }`, and `Destruction { asset, amount, tag, flow }`. `BurnAddress([u8; 32])` belongs to realization. Records retain order, permit repeated addresses, require positive scalar amounts and a configured `BurnRecordMax`, and use contiguous zero-based ordinals. Destruction names a flow in the normalized snapshot and must match its sole U/Recon leg. The contextual snapshot gives each record its transaction/ordinal/fresh-ASH identity and each destruction its owning region. These checks establish typed binding, not whole-operation acceptance or native carrier authentication.

The public projection API has separate roles:

- `BurnEventProjection` and `validate_burn_event(bound, offered)` compare fresh ASH, ordered records and context, independently of credit.
- `evaluate_record_credit(bound)` returns `RecordCredit::Credited { total }` or `Overclaim { total, fresh_amount }`. Totals use checked `u128`; beyond-scalar-domain overclaim voids the whole record family's credit without refusing the extension or Burn event.
- `ClearEventProjection` and `validate_clear_event(bound, offered)` compare the STATE pair, ASH sources, B, recomputed X, destruction pinned to X, residual references, omega and derived successor Y; foreign root effects refuse.
- `StateSuccessionProjection` and `validate_state_succession(bound, offered)` check the exact edge and complete successor against the existing `clear_ash` transition.
- `AccountingProjection`, `AccountingValues` and `validate_accounting(bound, offered)` cover Burn record totals/fresh amount, compact input/output preservation, Clear B/X/actual STATE decrement/destruction including tag/residual, event type and scoped absence of distribution residue. Event equality never supplies accounting acceptance. Later-history noninterference still needs bite 2.8's separately bound composition evidence.
- `CompactSilenceProjection` and `validate_compact_silence(bound, offered)` check no Burn/Clear event, STATE effect, issuance or destruction; compact cannot receive fresh record credit.

Projection validators feed `clear_ash` the bound snapshot's `AshBatchMax`. Missing/exceeded bounds, sealed predecessors and zero progress return `ClearTransition`; supply and ASH-total domain failures remain `Arithmetic`. `BurnClearRefusal` has 21 members with stable kebab-case `name()` and matching error strings. Burn events require consumed live U receipts and only live U change besides the fresh ASH. Clear binding requires its associated leg to be the region's sole destruction and forbids issuance.

These are internal typed comparison subjects, not the report producers assigned to Waves 10–12. The tests in `src/tests/burn_clear_projection_tests.rs` retain literal arithmetic controls and one-coordinate binding/projection subjects. Clear's fixture is a primitive projection control with B=10, `y_l=6`, `y_t=3`, X=6 and residual=4; it makes no claim to a model-executed history. Arithmetic refusals remain distinct from credit overclaim. Two-fault subjects pin transaction-before-certificate and record-content-before-ordinal precedence within each record.

## Error handling

`RealizationError` remains the general error root for construction, scoping, architecture binding, derivation, common evaluation, and arithmetic. It derives `thiserror::Error`, so it implements `Display` and `std::error::Error`, and it is `PartialEq` — variants can be matched or compared directly, as the public API test does. The operation-specific projection checks use their separate closed refusal sum below.

Four closed refusal sums are separate public results: `MaturityTransitionRefusal` from `announce_maturity`, `ClearTransitionRefusal` from `clear_ash`, `StateMetadataRefusal` from `decode_state_metadata`, and `BurnClearRefusal` from the bound observation/projection checks. Each exposes `ALL` and `name()` and is never wrapped into `RealizationError`. The first three describe STATE transition/codec outcomes; the fourth distinguishes missing facts, malformed binding and projection disagreement. There is still no panicking alternative to any fallible call. Record overclaim is a successful `RecordCredit::Overclaim` result, not any of these refusals.

Which families arise where:

| Operation | Expect |
|---|---|
| `RealizationScope::from_operations` | `EmptyScope`, `DuplicateScopeOperation` |
| `RealizationScope::validate_against`, `try_complete` | `OperationOutsideArchitecture`, `IncompleteScope { missing }` |
| `ArchitectureBinding::from_architecture` | `ArchitectureValidationFailed { errors }` (wrapping `Vec<architecture::ManifestError>`), `ArchitectureHashUnavailable` |
| `derive` | the scope and binding families above, plus every declaration-integrity family: duplicate/unknown/cyclic expressions and relations, type mismatches (`ExpressionTypeMismatch`, `BinaryOperandTypeMismatch`, `InvalidSumType`, `InvalidOrderedType`), foreign-ownership violations (`ForeignExpressionOwnership`, `ForeignRelationOwnership`, `ForeignRelationDependency`, `ForeignProofAlternativeBinding`, `ForeignConstructibilityOwnership`, `ForeignDisclosureOwnership`), architecture-agreement violations (`MissingArchitectureOperation`, `ArchitectureOperationMismatch`, `MissingArchitectureRelation`, `RelationKindMismatch`), and graph-policy violations (`ConstructibilityCycle`, `RelationDependencyCycle`, `PermissionlessPrivateDependency`, `SponsorValueRead`) |
| `ProtocolAmount` / `Count` / `Cycle` arithmetic; `AnnouncementLeadBounds::new` | `AmountOutOfDomain`, `AmountOverflow`, `AmountUnderflow`, `CountOverflow`, `CycleOverflow`, `InvalidAnnouncementLeadBounds { minimum, maximum }` |
| `announce_maturity` | `MaturityTransitionRefusal` |
| `clear_ash` | `ClearTransitionRefusal` |
| `decode_state_metadata` | `StateMetadataRefusal` |
| `BurnClearContext::bind_observation` | Existing common normalization errors; no extension facts are invented |
| `validate_burn_clear_observation` | `BurnClearRefusal`: missing extension, malformed observation, differing normalized common snapshot, transaction, certificate, operation, ASH/STATE binding, record bound/ordinal/structure, data association or destruction binding |
| `validate_burn_event`, `validate_clear_event`, `validate_state_succession`, `validate_accounting`, `validate_compact_silence`, `evaluate_record_credit` | `BurnClearRefusal`: wrong projection role, `AshBinding` for unreadable/non-U/nonpositive ASH, event/accounting/edge/silence disagreement, Clear transition refusal or arithmetic; credit overclaim remains a separate result |
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

The `compiler` crate consumes validated realization relations, proof alternatives, constructibility, lifecycle, representation, disclosure provenance, and evidence requirements.

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

The compiler states symbolic laws between the six input and six output STATE facts; executable declaration relations remain unchanged.

Both STATE sides carry a public confidentiality baseline; that baseline does not specify when an output becomes available.

## Stable keyed declarations

`keyed::KeyedRealizationDraft` derives and validates SOURCE and MERGE from `architecture::keyed::KeyedManifest::contract()` without a feature flag. The normative nine-row paper weld, evidence obligations and refusal boundaries are in (`[PLAN-sec:realization:keyed-declarations]`). The default compiler library now consumes these declarations through production keyed preparation, publication and independent validation. The discharged bite-1.2 obligations and explicit later-bite handoffs are recorded at (`[PLAN-rec:w1b12:gate]`).

## Offline keyed evidence packet

The bite-1.5 packet is closed at (`[PLAN-rec:w1b15:closed]`). Vectors supplies a graph-only model, byte/evidence reader and six internal report validators; these do not change this package's declarations or add a production chain reader. SOURCE has its singleton record and MERGE only the consumed union: no extra clause, carried accumulator, STATE metadata or owner-authorization law is introduced. The comparison-source and premise inventories distinguish local model evidence from native transaction validity.

The published finality form is a named external premise. No deployment declaration exists here; hypothetical fixtures never satisfy Required, and birth, continuation and resources make no finality claim. Four per-input parent facts and the per-key inclusion assumption remain conditional. Exact byte-bound relation evidence and every absent final-subject measurement pass to bite 1.6 at (`[PLAN-obl:guide15-exec:measurement-home]`); this packet makes no full-wave realization claim.
