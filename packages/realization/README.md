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

The declaration set covers `compact-ash`, `transfer-live-receipts`, `announce-maturity`, `burn`, and `clear`. `RealizationScope::phase1_pilots()` retains the two-operation compact-ASH and live-transfer pilot scope; an explicit `RealizationScope::from_operations` scope reaches `announce-maturity`, `burn`, or `clear` through the public `derive` entry point.

A scope containing fewer than all operations in the validated architecture is explicitly partial and cannot be converted into a complete realization until every operation is present.

### Burn declaration

Burn has 32 relations, 42 relation dependencies, 13 constructibility nodes with 12 edges, nine lifecycle nodes with ten edges, and 15 disclosure nodes with 16 edges and two seeds. Its expression list is empty because Burn has no stored-field assignment or unused expression. The complete shell implements (`[PLAN-tab:realization:burn-clear-relations]`), (`[PLAN-tab:realization:burn-clear-graphs]`) and the six architecture witnesses' obligation map at (`[PLAN-tab:realization:burn-clear-witnesses]`).

The architecture weld checks Burn's operation kind, receipt-owner authorization, each input authorization and cardinality, output cardinalities, empty roots/issuances/interface reads/writes, lateral U delta, `BurnRecord` schema, sponsor envelope, bounds, projections, six witnesses, value-flow classes and both ASH/live-receipt object lifecycles. Set-like collections accept permutations but retain exact multiplicity. Declaration validation separately compares every shell collection against the complete owner census, so coherently removing a required constructor fact and its edge still refuses. The activated and complete relation censuses share one exact comparison. Census failures are declaration errors, not observation refusals.

The activated Burn law-to-law census lives beside the relation census in `src/burn_clear_laws.rs`: input `SideRepresentation` → `BurnBinding` → `DataOutputs`, both `StaticRequirement`. The complete declaration adds family recognition/cardinality, authorization and conservation prerequisites; validation checks the activated subgraph and the complete dependency census. Literal tests refuse omitted, duplicated and changed law edges. Clear's complete declaration and activated fixture consume the same eighteen-edge law census.

Burn admits Explicit live inputs and Explicit or `PrivateCommitted` change. The lifecycle union retains both receipt modes and all three inherited exits; the private receipt → Burn edge still names the deferred direct-private route under ruling 191, and supported Burn scope does not discharge it. Public fresh ASH retains Explicit/`PublicCommitted` and both `CompactAsh`/Clear exits. Disclosure seeds the public ASH interface and Burn event; private change remains private and sponsor amounts are absent from constructor/disclosure facts.

The complete-declaration controls retain I=A+C, owner authorization, positive fresh ASH, typed event/data association, sponsor opacity, no STATE or destruction, and an external substrate-conservation requirement. A valid record overclaim remains a conformant Burn while credit independently returns `Overclaim`. Failed laws, blocked dependents and missing-operand errors remain separate. The compiler public control binds `AnnounceMaturity` from a realization that also declares Burn; adding Burn to compiler scope reaches `UnsupportedOperationPlan` at `bind_input`. No Burn compiler plan or model adapter is supplied; the independent adapter belongs to bite 2.5.

The nearest jointly declared compiler control uses Burn and `AnnounceMaturity`, whose lifecycle nodes are disjoint. The inherited assembler in `src/derive.rs` concatenates object-qualified lifecycle nodes, and `src/lifecycle.rs` rejects duplicates, so Burn with either pilot currently refuses with `DuplicateLifecycleNode`. The input/change mode-union test uses Burn's own side-qualified declarations; standalone live transfer independently retains its private Burn exit. No mixed-pilot scope support is claimed, and no shared graph owner is changed.

The vectors supported-operation census includes Burn in its non-STATE set and Clear as a STATE-declaring operation. The `candidate_leaf_and_transition_censuses_have_no_announced_state_spend` pin retains its candidate-leaf scope; `realization_operation_census_retains_only_the_announcement_state_transition` is a historical name for the declaration-scope pin and now asserts both `AnnounceMaturity` and `Clear` declare STATE succession. Neither census claims a newly emitted Clear leaf.

Independent tests expand the complete relation bodies and proof alternatives from (`[PLAN-tab:realization:burn-clear-relations]`) and (`[PLAN-tab:realization:burn-clear-graphs]`), and compare every lifecycle/disclosure node, edge and seed, including visibility and the exact retained-private set. Same-length substitutions and coherent requirement removals refuse; I=12, A=9, C=2 fails `CanonicalDeltaPolicy`, whose exact partition already enforces the value equation, and blocks Conservation, then binding and data outputs. A direct Conservation failure is therefore not the boundary of this complete-declaration subject. Zero fresh ASH refuses earlier at observation binding with `AshBinding`, so it cannot reach a complete-declaration Recognition verdict. An inserted expression also refuses. Every ASH/live object coordinate has an independent weld subject.

The Burn witness obligations below expand (`[PLAN-tab:realization:burn-clear-witnesses]`). Every identity has operation Burn; F(s,o) means `ObjectFamily { side: s, object: o }`, T(s) means `TransactionSide { side: s }`, A(U) means `Asset { asset: U }`, S(s) means `SideRepresentation { side: s, object: ReceiptLive }`, and L(o,e) means `LifecycleExit { object: o, exit: e }`. Each entry is Kind/Subject; unqualified kinds have subject Operation. These obligations name declaration coverage, not discharged external evidence.

| Witness | Exact Burn identities |
|---|---|
| `CanonicalDelta` | `CanonicalDeltaPolicy`; `AllowedObjectFamilies/T(Input)`, `AllowedObjectFamilies/T(Output)`; `Conservation/A(U)`; `BurnBinding` |
| `AshLineage` | `BurnBinding`; `Conservation/A(U)`; `Recognition/F(Output,Ash)`; `Lifecycle/L(Ash,CompactAsh)`, `Lifecycle/L(Ash,Clear)` |
| `ReceiptOwnerRouting` | `Authorization/F(Input,ReceiptLive)`; `Conservation/A(U)`; `BurnBinding`; `Recognition/F(Input,ReceiptLive)`, `Recognition/F(Output,ReceiptLive)` |
| `ReceiptClassClosure` | `Recognition/F(Input,ReceiptLive)`, `Recognition/F(Output,ReceiptLive)`; `AllowedObjectFamilies/T(Input)`, `AllowedObjectFamilies/T(Output)`; `SideRepresentation/S(Input)`, `SideRepresentation/S(Output)` |
| `AttestationAuthentication` | `BurnBinding`; `DataOutputs`; `ProjectionPolicy`; `Recognition/F(Output,Ash)` |
| `ValueFlowClosure` | `AllowedObjectFamilies/T(Input)`, `AllowedObjectFamilies/T(Output)`; `CanonicalDeltaPolicy`; `OpenFlowPolicy`; `SponsorIsolation/Sponsor`; `SponsorEnvelopeMultiplicity/Sponsor`; `Conservation/A(U)`; `BurnBinding` |

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
  - `evaluate_operation_with_burn_clear(&BoundBurnClearObservation) -> Result<ConformanceReport, RealizationError>`
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
  `CanonicalDeltaPolicy`, `OpenFlowPolicy`, `ExpressionPredicate`, `BurnClearLaw`.
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

The alternative was optional shared carriers on `OperationObservation`. At adoption it would migrate fourteen literal-initializer files plus the common struct; this bite adds its own separate test fixture. The adoption census includes the production maturity initializer in `packages/transaction/src/state_abi.rs`. The extension pays explicit binding checks and snapshot storage instead, leaving all those files and common normalization unchanged. The bound extension is consumed by `ScopedRealizationSpec::evaluate_operation_with_burn_clear`; Burn and Clear have complete declaration dispatch. Missing extension returns `MissingExtension`; it never supplies defaults. `BoundBurnClearObservation::resolve_state_fact` resolves only operation-qualified Clear `FactId::StateField` values from its carried STATE pair, using the existing exhaustive `StateField` vocabulary. Legacy maturity keeps its old entry and missing-carrier behavior.

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

Four closed refusal sums are separate public results: `MaturityTransitionRefusal` from `announce_maturity`, `ClearTransitionRefusal` from `clear_ash`, `StateMetadataRefusal` from `decode_state_metadata`, and `BurnClearRefusal` from the bound observation/projection checks. Each exposes `ALL` and `name()`. The first three describe STATE transition/codec outcomes; the fourth distinguishes missing facts, malformed binding and projection disagreement and is wrapped by `RealizationError::BurnClearObservation` at public relation evaluation. There is still no panicking alternative to any fallible call. Record overclaim is a successful `RecordCredit::Overclaim` result, not any of these refusals.

Which families arise where:

| Operation | Expect |
|---|---|
| `RealizationScope::from_operations` | `EmptyScope`, `DuplicateScopeOperation` |
| `RealizationScope::validate_against`, `try_complete` | `OperationOutsideArchitecture`, `IncompleteScope { missing }` |
| `ArchitectureBinding::from_architecture` | `ArchitectureValidationFailed { errors }` (wrapping `Vec<architecture::ManifestError>`), `ArchitectureHashUnavailable` |
| `derive` | the scope and binding families above, plus every declaration-integrity family: duplicate/unknown/cyclic expressions and relations, type mismatches (`ExpressionTypeMismatch`, `BinaryOperandTypeMismatch`, `InvalidSumType`, `InvalidOrderedType`), foreign-ownership violations (`ForeignExpressionOwnership`, `ForeignRelationOwnership`, `ForeignRelationDependency`, `ForeignProofAlternativeBinding`, `ForeignConstructibilityOwnership`, `ForeignDisclosureOwnership`), architecture-agreement violations (`MissingArchitectureOperation`, `ArchitectureOperationMismatch`, `MissingArchitectureRelation`, `SurplusArchitectureRelation`, `ArchitectureRelationMismatch`, `RelationKindMismatch`), and graph-policy violations (`ConstructibilityCycle`, `RelationDependencyCycle`, `PermissionlessPrivateDependency`, `SponsorValueRead`, `DuplicateLifecycleNode`); the last is reachable from valid Burn/pilot scope requests under the inherited assembler limitation |
| `ProtocolAmount` / `Count` / `Cycle` arithmetic; `AnnouncementLeadBounds::new` | `AmountOutOfDomain`, `AmountOverflow`, `AmountUnderflow`, `CountOverflow`, `CycleOverflow`, `InvalidAnnouncementLeadBounds { minimum, maximum }` |
| `announce_maturity` | `MaturityTransitionRefusal` |
| `clear_ash` | `ClearTransitionRefusal` |
| `decode_state_metadata` | `StateMetadataRefusal` |
| `BurnClearContext::bind_observation` | Existing common normalization errors; no extension facts are invented |
| `validate_burn_clear_observation` | `BurnClearRefusal`: missing extension, malformed observation, differing normalized common snapshot, transaction, certificate, operation, ASH/STATE binding, record bound/ordinal/structure, data association or destruction binding |
| `validate_burn_event`, `validate_clear_event`, `validate_state_succession`, `validate_accounting`, `validate_compact_silence`, `evaluate_record_credit` | `BurnClearRefusal`: wrong projection role, `AshBinding` for unreadable/non-U/nonpositive ASH, event/accounting/edge/silence disagreement, Clear transition refusal or arithmetic; credit overclaim remains a separate result |
| `evaluate_operation`, `validate_observation` | the observation families: `UnknownObservedObject`, `DuplicateObservedObject`, `DuplicateObservedReference`, `WrongObservedReferenceSide`, `ObservedCanonicalPartitionOverlap`, `ObservedOpenFlowOverlap`, `AnchorInObservedOpenFlow`, `DuplicateObservedRoot`, `MissingBoundValue`, plus `UnknownRelation` |
| `spec.constructibility_authorizations`, `spec.validate_against` | `OperationOutsideScope`, `ArchitectureBindingMismatch` |
| `burn_clear_laws`, `validate_burn_clear_law_census`, `BurnClearLawDeclaration::evaluate` | Census construction is infallible; validation returns `BurnClearLawError::Census`; evaluation returns `Identity`, `Operand` for missing operands or transition evaluation, or `Evaluation` for existing expression/arithmetic errors. Clamp sealed/zero-progress subjects are disagreements (`Ok(false)`), not errors. |

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

## Burn/Clear law bodies and public activation

`burn_clear_laws` owns five Burn laws and twelve Clear laws, including the existing Burn canonical policy. Its public API is `burn_clear_laws(operation)`, `validate_burn_clear_law_census(operation, declarations)`, `validate_burn_clear_relation_census(operation, declarations)`, `BurnClearLawDeclaration::evaluate(bound)` and `relation_declaration()`, `BurnClearLaw::key()` and `proof_kind()`, and `BurnClearLawError::name()`. The five helper types remain `BurnClearLawKey`, `BurnClearLawId`, `BurnClearLaw`, `BurnClearLawDeclaration`, and `BurnClearLawError`. `relation_declaration()` projects those bodies into the public `Relation`, `RelationKind` and `RelationSubject` vocabulary, including each copy assignment's owned expression ID. Exact activated census validation compares bodies, identities, proof alternatives and multiplicity. `ExpressionNode`, `FactId`, `StateField` and the common observation retain their existing vocabulary. Burn and Clear dispatch use their complete declarations; target plans remain Wave 3.

The public conversion `From<ClearTransitionRefusal> for BurnClearRefusal` maps `SupplyOutOfDomain` and `AshTotalOutOfDomain` to `Arithmetic`, and every other transition refusal to `ClearTransition`.

The staging error has four named cases: `identity` rejects a mismatched owner/key/body; `census` rejects omission, duplication, a foreign owner or changed body; `operand` wraps the existing `BurnClearRefusal` for missing operands or transition/arithmetic evaluation after binding; `evaluation` wraps the existing `RealizationError`, including expression facts and independent floor arithmetic. Sealed and zero-progress clamp subjects return `Ok(false)`, while a missing batch bound remains an error. A law disagreement returns `Ok(false)`; successful evaluation returns `Ok(true)`. This staging wrapper is additional to the four transition/codec/projection refusal sums described above. `RealizationError::BurnClearObservation` carries binding/operand refusals at the public evaluator; expression errors retain their existing variants. A failed public law carries `RelationFailure::BurnClearLaw`, and failed prerequisites block dependent relations. The local staging error converts into these public errors without turning an error into a disagreement.

Every evaluator is public and reachable in non-test builds. The singleton legacy graph invokes the existing canonical-partition, conservation and authorization evaluator with its actual operation-qualified identity; it does not copy the partition walker. Bindings reuse predicate-only projection helpers without constructing self-comparison snapshots; their standalone joins retain the census-required owner, conservation, flow and succession checks through the existing owners. The five copy laws invoke the existing Equal evaluator with both carried STATE facts. Public evaluation collects their owned expression closures and obtains STATE facts only from the bound carrier. The legacy public entry returns `BurnClearObservation(MissingExtension)` for Burn/Clear, inside the existing evaluation trace boundary. No per-relation tracing boundary is introduced, consistent with `TRACING.md`; transition refusals retain `clear_ash`'s existing event. No dead-code allowance, logging counter or local clock is added.

### Forms frozen for bite 2.3b

`ClearCanonicalDeltaPolicy` is a ninth body beyond the eight allocated in (`[PLAN-tab:realization:burn-clear-vocabulary]`), taken under the conditional-form delegation of (`[PLAN-tab:realization:burn-clear-relations]`). It shares kind `CanonicalDeltaPolicy` (engine code 5), as the three authorization bodies share kind 2; it allocates no kind or subject code and adds one arm to every exhaustive `Relation` match in the consumer table. Clear's architecture delta rows, including `PositiveAshResidual`, are never read by the body, so the 2.6 weld must compare both rows and their conditions literally.

The conditional delta body is `ClearCanonicalDeltaPolicy`, with existing kind `CanonicalDeltaPolicy` and subject `Operation`. It selects the unchanged `ExpectedCanonicalDelta` values: always U/Destruction/Recon; U/`OwnerlessLateral`/no-tag exactly when the derived residual is positive. It then invokes the existing canonical policy evaluator, preserving checked conservation, exact membership and no reused source. Burn retains its fixed U/Lateral/no-tag body. Thus neither captured `ExpectedCanonicalDelta` values nor their encoders change. Full clear and partial clear both have accepted controls; extra lateral without residual and residual without lateral return `Ok(false)`.

The clamp is the closed fieldless body `ClearClamp`. Its operands are the bound predecessor, authenticated ASH references/amounts and `AshBatchMax`; it calls `clear_ash` without pre-summing B. X is not a fact or expression value. `ClearAssignment { field: YL, expression: None }` resolves actual output YL through `FactId::StateField` and compares it with the same transition's derived successor YL. The other five assignments require `Some(ExprId::relation(assignment_id, Predicate))`, whose node is Equal of Input/Output `ExprId::fact(FactId::StateField { operation: Clear, side, field })`. Emit both sides for every field by iterating `StateField::ALL`; this is the only field census. Staging stores the Equal node itself because the new relation identity is not yet active; activation replaces that node field with the owned expression ID, with no change to its operands or meaning.

Compared alternative: a minimal expression node would need typed B, predecessor YL and YT, checked Y and Y−1, minimum/progress semantics, and a way for the YL assignment to consume X. Even a single specialized clamp node needs an additional checked decrement expression or a closed assignment consumer, expression type/dependency/evaluation support and compiler folding disposition. It provides no additional law expressiveness here. The closed body reuses the total transition and needs no new `ExpressionNode`, `FactId`, `StateField` or `DependencyEdge` member; that is the frozen smallest sufficient form. No compiler `fold.rs` activation is required.

| Staged body | Control and focused disagreement in `src/tests/burn_clear_relation_tests.rs` | Witness-map entry at (`[PLAN-tab:realization:burn-clear-witnesses]`) |
|---|---|---|
| Burn fixed / Clear conditional canonical policy | `every_staged_law_accepts_its_control`; wrong Burn movement and both conditional movement disagreements | `CanonicalDelta`; `ValueFlowClosure` |
| Two Burn side representations | Explicit input/private change; one side's disallowed mode in `side_admission_cannot_be_widened_by_change_modes` | `ReceiptClassClosure` |
| Burn `DataOutputs` | `burn_data_output_disagreement_is_a_binding_refusal`: valid overclaim remains accepted; wrong fresh-ASH association refuses at binding | `AttestationAuthentication` |
| Clear `DataOutputs` | Correct Recon data/leg; coherently changed data and leg quantity disagrees with clamp | `CanonicalDelta` |
| `BurnBinding` | Owner-authorized I=12, A=10, C=2, record claim=13; remove signer | `AshLineage`; `ReceiptOwnerRouting`; `AttestationAuthentication`; `ValueFlowClosure` |
| `ClearClamp` | B, live-supply and floor limiters, all ties; zero-progress and sealed predecessors disagree; missing bound remains an error | `CanonicalDelta`; `AshLineage`; `FloorNondecrease` |
| Six `ClearAssignments` | `each_assignment_uses_the_carried_field_and_its_owned_equal`: one changed successor coordinate per `StateField::ALL`; changed expression body refuses | `StateSuccession`; `FloorNondecrease` |
| `ClearFloor` | Positive nonincreasing successor supply and copied omega; zero/increased supply or changed omega | `FloorNondecrease` |
| `ClearFlow` | Full/partial residual; changed residual amount; same-tag split Recon refuses at normalization; a distinct second tag reaches `DestructionBinding` | `CanonicalDelta`; `AshLineage`; `ValueFlowClosure` |
| `ClearBinding` | Exact STATE succession; change root effect to termination | `StateSuccession`; `AshLineage` |

`literal_census_identity_and_completeness` fixes the 5/12 identity lists, checks permutations, and removes, duplicates and changes the owner of every declaration. `every_staged_law_accepts_its_control` additionally tests each law's owner mismatch. Missing extension, equal-total split Recon and wrong associations are exercised before a bound value can exist. Existing bite-2.2 tests retain transaction/certificate/region/source/STATE binding and missing-carrier controls. These subjects are primitive realization fixtures with literal expected arithmetic, not model executions. Independent execution is already the transition's bite-2.1 evidence and remains the adapter work assigned to bites 2.5 and 2.7; no model dependency or model acceptance change is introduced into realization.

### Exact activation and consumer packet

`activated_public_laws_accept_controls_and_require_the_extension` evaluates all five Burn and twelve Clear declarations through the public entry with bound controls and checks the legacy entry's missing-extension refusal. `activated_census_checks_every_identity_body_and_multiplicity` checks public identities, omissions, duplicates, foreign owners and permutation. The focused disagreements also use the public evaluator. `failed_public_clamp_blocks_dependents_instead_of_aborting_the_report` pairs sealed and no-progress predecessors with accepted controls one YL coordinate away and checks the blocked conditional delta explicitly. A missing clamp bound remains an error that aborts the report.

Burn `DataOutputs` is true for every bound Burn: bite 2.2's binding refuses each violation of its clause (`RecordBound`, `RecordOrdinal`, `RecordStructure`, `DataAssociation`), so its declared disagreement boundary is that binding. This reading of (`[PLAN-gate:wave2:bite-2-3a]`)'s focused-disagreement clause holds for this law because bite 2.2's gate requires binding to keep refusing a malformed carrier, so a malformed record family never becomes an evaluation subject. The `expected` rows are read by canonical body/identity validation before evaluation; changing a row refuses with the staged `Identity` error or public `ArchitectureRelationMismatch`, preserving the already-bound carrier guarantee.

The evaluated clamp operand graph is frozen below. These are existing fact identities, not new expression values. The raw source references remain necessary for authenticated membership and distinctness, and the checked aggregate is computed only after the predecessor supply/Sealed check. The closed body is opaque to generic expression folding; Wave 3 must lower it as a unit.

| Operand | Existing fact / binding | Consumer |
|---|---|---|
| Authenticated ASH family / B | `FactId::FamilyAmount { operation: Clear, side: Input, object: Ash }`, bound source references and per-source amounts | Checked aggregation inside the clamp |
| Live supply | `FactId::StateField { operation: Clear, side: Input, field: YL }` | Checked supply and clamp capacity |
| Time-locked supply | `FactId::StateField { operation: Clear, side: Input, field: YT }` | Checked supply and floor capacity |
| Batch bound | `FactId::BoundValue { bound: AshBatchMax }` | Structural bound before supply/aggregation |
| Derived X and residual | Closed transition result, no new `FactId` | YL assignment, flow, data quantity and conditional delta |

The eight new kind members follow `ExpressionPredicate`: `SideRepresentation`=16, `DataOutputs`=17, `BurnBinding`=18, `ClearClamp`=19, `ClearAssignment`=20, `ClearFloor`=21, `ClearFlow`=22, `ClearBinding`=23. The new subject `SideRepresentation` { side, object } has prefix 9, followed by `StateField` { field } with prefix 10. Field subcodes remain Omega=0, YL=1, YT=2, Q=3, Cycle=4, Maturity=5. Old kind 0–15, subject 0–8 and edge 0–12 codes are unchanged; the full literal lists and attestation namespace are tested in compiler `engine_attestation.rs`. These are owner engine encodings, not a new public serialization.

The bound public evaluator refuses undeclared Burn/Clear operations with `OperationOutsideScope`. Declaration census failures use `MissingArchitectureRelation`, `SurplusArchitectureRelation` and `ArchitectureRelationMismatch`; observation errors remain reserved for the carrier and its operands. The public ownership validator's five copy-expression refusals have accepted canonical declarations and subjects changing only the right operand's side. Operation-level census tests cover omission, duplication, changed bodies and changed proof alternatives, while identity tests pair every activated body with wrong-kind and wrong-subject refusals.

The complete Burn shell uses aggregate `ArchitectureRelationMismatch` anchors: expressions and relation dependencies use BurnBinding/Operation, constructor nodes and edges use Constructibility/Operation, and lifecycle/disclosure nodes, edges and seeds use Representation/Representation { object: Ash }. The last is a collection anchor, not a claim that every mismatch concerns ASH; the existing error vocabulary cannot identify every visibility, edge-kind or seed mismatch precisely. Individual relation mismatches retain their own identity. No new error variant is introduced.

The complete Clear shell declares one STATE pair, authenticated ASH input and conditional residual, the Recon destruction data output, six STATE assignments, the public permissionless constructor and both public ASH/STATE lifecycle modes. Its six architecture witnesses are obligation names, not private constructor credentials. Complete-shell census errors use the declared omega `ClearAssignment` identity for expression rows, `ClearBinding` for dependency rows, `Constructibility` for constructor rows, and `Representation` on ASH for lifecycle and disclosure rows. The omega identity is a collection anchor: a mismatched copy of another field is reported under that identity. Changed operation coordinates refuse through `ArchitectureOperationMismatch`; changed relation bodies and proofs retain their own relation identity. Clear alone and Clear with `TransferLive` derive. The current assembler refuses Clear with `CompactAsh`, Burn or `AnnounceMaturity` at overlapping lifecycle nodes; the shared union belongs to bite 2.7a. Declaration does not establish an emitted Clear leaf, public constructor bytes or substrate evidence.

Clear has 17 expressions, 44 relations, 55 relation dependencies, 15 constructibility nodes and 14 edges, 12 lifecycle nodes and 16 edges, and 32 disclosure nodes, 40 edges and four seeds. Its constructor requires the public STATE and ASH facts and both bounds; the sponsor-local authorization witness is optional. The disclosure clamp operands are input ASH amount, input YL and YT, and `AshBatchMax`, exactly as the frozen operand graph states. `ClearFloor` takes both sides' YL, YT and omega; `ClearFlow` takes input ASH amount. The event binding takes successor omega, YL and YT. The input ASH amount points to the ASH representation because Clear's residual may be absent and has no output amount fact.

The Clear witness identities below expand (`[PLAN-tab:realization:burn-clear-witnesses]`). Every identity has operation Clear. F(s,o) means `ObjectFamily { side: s, object: o }`, T(s) means `TransactionSide { side: s }`, L(o,e) means `LifecycleExit { object: o, exit: e }`, and S(f) means `StateField { field: f }`. Each entry is Kind/Subject; unqualified kinds have subject Operation. The optional sponsor witness is a constructibility node, not a relation identity.

| Clear architecture witness | Exact Clear declaration identities |
|---|---|
| `CanonicalDelta` | `CanonicalDeltaPolicy`; `ClearClamp`; `ClearFlow`; `DataOutputs`; `AllowedObjectFamilies/T(Input)`, `AllowedObjectFamilies/T(Output)` |
| `AshLineage` | `ClearClamp`; `ClearBinding`; `ClearFlow`; `Recognition/F(Input,Ash)`, `Recognition/F(Output,Ash)`; `Lifecycle/L(Ash,CompactAsh)`, `Lifecycle/L(Ash,Clear)` |
| `StateSuccession` | `Cardinality/F(Input,State)`, `Cardinality/F(Output,State)`; `Recognition/F(Input,State)`, `Recognition/F(Output,State)`; `RootPolicy`; `ClearBinding`; `ClearAssignment/S(f)` for every `StateField` |
| `FloorNondecrease` | `ClearClamp`; `ClearFloor`; `ClearAssignment/S(f)` for every `StateField` |
| `ValueFlowClosure` | `AllowedObjectFamilies/T(Input)`, `AllowedObjectFamilies/T(Output)`; `CanonicalDeltaPolicy`; `OpenFlowPolicy`; `SponsorIsolation/Sponsor`; `SponsorEnvelopeMultiplicity/Sponsor`; `ClearFlow` |
| `NativeFeeAuction` | `OpenFlowPolicy`; `SponsorIsolation/Sponsor`; `SponsorEnvelopeMultiplicity/Sponsor`; `Constructibility`; optional `SponsorAuthorization/SponsorLocal` constructor witness |

The first Clear mixed-scope refusal is an overlapping `DuplicateLifecycleNode`. Identical ASH lifecycle edges with `CompactAsh` or Burn, and STATE lifecycle edges with `AnnounceMaturity`, would next refuse as `DuplicateLifecycleDependency`; then the shared public `BoundValue` `AshBatchMax` with `CompactAsh` or `FeeSponsorInputMax` with Burn would refuse as `DuplicateDisclosureNode`. Bite 2.7a owns their union.

| Activated Relation body | Subject / kind | Exact body fields |
|---|---|---|
| `SideRepresentation` | `SideRepresentation` { side, object } / `SideRepresentation` | `side: TransactionSide`, `object: ObjectId`, `allowed: BTreeSet<RepresentationMode>` |
| `DataOutputs` | Operation / `DataOutputs` | `expected: Vec<architecture::DataOutputSpec>`; exact canonical schema rows including cardinality and condition |
| `BurnBinding` | Operation / `BurnBinding` | None |
| `ClearClamp` | Operation / `ClearClamp` | None |
| `ClearAssignment` | `StateField` { field } / `ClearAssignment` | `field: StateField`, `expression: Option<ExprId>`; None exactly for YL, canonical owned Equal otherwise |
| `ClearFloor` | Operation / `ClearFloor` | None |
| `ClearFlow` | Operation / `ClearFlow` | None |
| `ClearBinding` | Operation / `ClearBinding` | None |
| `ClearCanonicalDeltaPolicy` | Operation / existing `CanonicalDeltaPolicy` | None; only Clear may own this body |

No new `RelationEdge` is needed. `StaticRequirement` expands the operation-law endpoints: Burn input `ReceiptLive` recognition/cardinality, output `ReceiptLive` recognition/cardinality, output Ash recognition/cardinality, Authorization and Conservation → `BurnBinding` → `DataOutputs`; Burn input `SideRepresentation` → `BurnBinding`. Clear input State/Ash recognition/cardinality → `ClearClamp`; `ClearClamp` → each of the six assignments, `ClearFlow` and `DataOutputs`; `ClearClamp` → Clear's `ClearCanonicalDeltaPolicy`, because the conditional body selects its lateral family from the clamp's derived residual; this amends the operation-law dependencies of (`[PLAN-tab:realization:burn-clear-graphs]`) so that a failed clamp blocks the delta instead of aborting evaluation; each assignment → `ClearFloor`; `ClearFloor`, `ClearFlow`, `DataOutputs` and output State/Ash recognition/cardinality → `ClearBinding`. The remaining existing family, authorization, sponsor, root, projection, representation and lifecycle edge meanings and endpoints remain those in (`[PLAN-tab:realization:burn-clear-graphs]`), expanded by the complete Burn declaration and assigned to bite 2.6 for Clear. `PublicArithmetic` is frozen for clamp, assignments, floor and flow; `ManifestShape` for canonical policy, data outputs and bindings; side representation has no proof alternative.

| Consumer | Bite-2.3b action |
|---|---|
| realization `relation.rs`, `identity.rs`, `validate.rs` | Activate bodies/keys; derive exact subjects; validate all body fields and exact canonical multiplicity; include side admission's input/change mode union only in lifecycle census. |
| realization `expression.rs`, `evaluate.rs`, staged laws/observation/projection, `error.rs`, `lib.rs` | Bind required extension; feed STATE facts; include `ClearAssignment` in expression ownership/collection; evaluate reviewed bodies; legacy entry refuses missing extension; preserve old variants and controls. |
| realization `tests/burn_clear_relation_tests.rs`, `relation_identity_tests.rs`, `compact_ash_tests.rs` | Replace local identities with activated identities and retain controls, exact 5/12 staged census (including Burn's existing delta policy), code/subject isolation and old compact behavior. |
| compiler `engine_attestation.rs`, `engine_attestation/constructibility.rs` | Allocate exact new kind/subject codes; share the explicit `state_field_code` encoder and preserve old encodings. |
| compiler `coverage.rs`, `layout.rs`, `placement.rs`, `engine_attestation/{requirement,source,foundation}.rs` | Exhaustive new-body disposition; explicit unsupported behavior, never empty requirements or fabricated coverage. |
| compiler `engine_attestation/coverage.rs`, `tests/tracing_boundary_tests.rs` | Propagate unsupported coverage refusal at its caller; pin the count and grammar of 144 error trace names, including the operation and body refusals. |
| compiler `input.rs`, `error.rs`, `case.rs` and `tests/case_tests.rs` | Typed unsupported-plan refusal at `bind_input` and direct in-crate case entry; retain a reaching direct-entry subject. |
| compiler `tests/{coverage_tests,coverage_oracle_tests,analyzed_program_oracle_tests,source_tests}.rs` | Preserve old captures/identities and reaching unsupported behavior. |
| vectors `maturity_mutants/native_staging.rs`, `maturity_safety.rs` | Exhaustive relation handling and STATE-read classification; preserve candidate and dispatch pins until their assigned declaration bites. |
| compiler silent `engine_attestation/{lifecycle,proof}.rs`, carrier/case/sponsor/operation-plan selectors | Preserve old semantics; new paths remain unsupported in Wave 2. Wave 3 owns execution-case/proof meaning; a necessary extra edit stops for path expansion. |

The complete unchanged/active/downstream sweep remains (`[PLAN-tab:backlog:wave2-consumers]`); this table refines its activation inputs without granting edit authority. No expression change activates compiler `fold.rs`, `disclosure.rs`, or value/type consumers. The common `OperationObservation` and STATE constructors remain unchanged, including transaction's production maturity initializer. Compiler binding refuses Burn/Clear plans after owner validation and membership checking. Direct case, source, placement, layout, requirement, predicate and coverage entries also refuse unsupported laws rather than fabricate requirements. The typed boundaries are `UnsupportedOperationPlan` and `UnsupportedRelationPlan`; the latter also covers a new body retagged with a legacy operation at an in-crate synthetic entry. The full repository gate at import remains the downstream compilation verdict.

`src/relation.rs` is a pinned source of the Wave-1 freeze (`FrozenItem::Realization` in `packages/vectors/src/keyed_final_manifest/pins.rs`). Adding these bodies, kinds and subjects changes pinned source, so under (`[PLAN-rule:vectors:keyed-refreeze]`) bite 2.3b lands the change with its re-pin in one independently reviewed, gate-passing landing. That landing carries: the new digest in `pins.rs`; `relation.rs`'s capture-time digest in `RETAINED_CAPTURE` there, as the retained-capture construction requires for every path re-frozen after the capture; a freeze-history row in `packages/vectors/tests/keyed_final/freeze_history.rs` naming path, previous and new digest, source diff, behaviour classification and reopened evidence under (`[PLAN-rec:w1b16:adoption]`); and the matching plan record beside (`[PLAN-tab:vectors:dependency-refreeze]`) in `plans/packages/vectors.md`. The adopted path list and consumer census at (`[PLAN-tab:backlog:wave2-paths]`) and (`[PLAN-tab:backlog:wave2-consumers]`) include these three paths as the review-correction record, before bite 2.3b starts. The re-freeze is recorded in `REFREEZES[3]` and `ENCODING_DIGESTS[4]` in the freeze history, with the capture-time source identity retained in `RETAINED_CAPTURE`.

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

## Fixed-form declarations · `sec:realization-package:keyed-fixed`

The sibling `keyed_fixed`, `keyed_fixed_relation` and `declarations/keyed_fixed` owners derive four SOURCE and six MERGE rows from the schema-3 manifest. Handle families mean exact constant-program, explicit-asset and explicit-positive-value membership at birth position zero. SOURCE classification is complete-input evidence; its membership-reading row introduces the triple/handle singleton. MERGE adds no record and unions consumed sets. Its three spending clauses are born-first, own-qualification and successor qualification, with no accounting or surplus-destination constraint. Complete re-derivation rejects missing or changed rows, bindings and nonempty foreign stage fields. Local operation/object types cannot carry legacy or attestation identifiers. Relation encodings retain both namespaces and an explicit side-specific kind.

The paper weld maps every `KeyedFixedRelation` row to its obligation. Evidence rows require authenticated history or reader premises; runtime rows constrain each consumed member's spend.

| Operation | Exact relation | Class | Paper obligation |
|---|---|---|---|
| SOURCE | `HandleFamily { side: Input, cardinality: Zero }` | Evidence | Complete authenticated same-triple input classification under (`[A-op:kpcfixed-operations:source]`), (`[A-def:kpcfixed-model:keyed-membership]`) and Retrieval in (`[A-sec:kpcfixed:interface]`); ordinary funding inputs remain allowed. |
| SOURCE | `HandleFamily { side: Output, cardinality: One }` | Evidence | One same-triple output-0 handle under (`[A-def:kpcfixed-model:keyed-membership]`) and (`[A-op:kpcfixed-operations:source]`); later shaped outputs are not handles. |
| SOURCE | `SourceClassification` | Evidence | The empty consumed handle family classifies SOURCE under (`[A-def:kpcfixed-operations:classification]`); no covenant executes merely on creation. |
| SOURCE | `MembershipReading { recurrence: SourceSingleton }` | Evidence | Reader-derived singleton `(q, s)` under (`[A-def:kpcfixed-interface:membership-reading]`), (`[A-def:kpcfixed-operations:classification]`) and (`[A-op:kpcfixed-operations:source]`); no stored record field. |
| MERGE | `HandleFamily { side: Input, cardinality: OneOrMore }` | Evidence | Nonempty authenticated same-triple consumed family under (`[A-op:kpcfixed-operations:merge]`), with no covenant maximum. |
| MERGE | `HandleFamily { side: Output, cardinality: One }` | Evidence | Same-triple first successor under (`[A-def:kpcfixed-model:keyed-membership]`) and (`[A-def:kpcfixed-model:first-output]`); other outputs remain unconstrained. |
| MERGE | `BornFirst` | Per-member runtime | The consumed handle's authenticated birth position is zero under (`[A-def:kpcfixed-model:first-output]`). |
| MERGE | `OwnQualification` | Per-member runtime | The consumed output satisfies `Q_q`: constant program and exact explicit asset/value pair under (`[A-def:kpcfixed-model:keyed-membership]`) and (`[A-def:kpcfixed-model:first-output]`). |
| MERGE | `SuccessorQualification` | Per-member runtime | Output zero satisfies that same constant `Q_q` under (`[A-def:kpcfixed-model:first-output]`); no input/output accounting or surplus-destination constraint is added. |
| MERGE | `MembershipReading { recurrence: ConsumedUnion }` | Evidence | Reader-derived union under (`[A-def:kpcfixed-interface:membership-reading]`), (`[A-def:kpcfixed-operations:classification]`) and (`[A-op:kpcfixed-operations:merge]`), without a fresh record or runtime history traversal. |

SOURCE has four evidence obligations and no runtime carrier; MERGE has three evidence obligations and three per-member runtime obligations. Immutable history (`[A-def:kpcfixed-model:history]`), record evidence (`[A-rem:kpcfixed-interface:record-evidence]`) and membership correctness (`[A-thm:kpcfixed-interface:membership-correctness]`) retain their retrieval, inclusion, acceptance and finality premises. The declaration does not discharge them.

For the refusal census under rulings 158 and 168 at (`[PLAN-rule:phase7:wave1-home-rulings]`), `KeyedFixedRealizationError::ForeignOperation` is representational: the closed operation type has exactly the two variants required by a validated manifest. It has no well-typed foreign-operation subject and is not counted as a reached refusal. S4 must retain that distinction when consuming this owner.

`KeyedFixedValue` refuses zero; every positive u64 is semantically representable here. Ruling 212 at (`[PLAN-rule:phase7:fixed-form-rulings]`) assigns the target ceiling to S6 construction. `KeyedFixedParameters` supplies K,A,v without defaults. `is_keyed_fixed_member` compares supplied authenticated fields with that triple and its authenticated program; it does not read the key, authenticate the program-to-triple binding or restrict the nonce. Supplying the program authenticated for the same triple is the caller's obligation. It returns no record for a nonmember, only a membership boolean; the reader and record recurrence remain S7. No current fixed-form compiler, byte recipe, constructor or capture acceptance is claimed.
