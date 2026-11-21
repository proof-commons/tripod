# `tripod-compiler`

## Wave-1 independence evidence

The moved seam is semantic ownership: opaque per-position keys, owner encoders/restoration and the stable `keyed_plan` facade let keyed declarations use the same seven analysis stages and fresh independent revalidation. The shared kernels acquire no new operation-identity semantic decision; the seven-site baseline and separate six-site ledger remain unchanged. SOURCE/MERGE/joint scope publishes 4/5/9 relation nodes and 28/33/61 coverage nodes, while complete eligibility controls, ten attestation projection pins and canonical publication remain exact (`[PLAN-rec:w1b12:gate]`). Final construction and native joins are downstream evidence at (`[PLAN-rec:w1b16:packet]`), not compiler verdicts.

The retained default-consumer campaign measures an approximately threefold attestation-derivation regression, including independent validation. Profiling remains open without an invented threshold or established cause; the final default-consumer campaign's 7.239723-second maximum has no attributed delivery failure. Historical build walls remain valid while build CPU/stall deltas remain unadmitted because their cgroup identities were not retained. The baseline disposition at (`[PLAN-rec:w1b16:baseline]`) makes no new build-cost comparison and preserves the runner-repair prerequisite.

`compiler` analyzes a validated realization into a deterministic
target-independent compilation plan. It emits no target program.

The package contract is [plans/packages/compiler.md](../../plans/packages/compiler.md).

## Input

The crate consumes typed values only: a validated scoped realization, an
explicit compilation scope, a typed analysis policy, and optionally abstract
target capabilities.

It does not consume:

- generated architecture, realization, or declassification publications;
- model source, labels, or tests;
- plans, ADR, or target reference prose;
- target bytecode or disassembly;
- environment or filesystem state.

Nothing in this crate opens a file or reads the environment, and that absence
is a contract rather than an accident.

## Output

The analyzed value will carry architecture and realization bindings, explicit
scope, the normalized relation graph, source provenance, proof alternatives,
disclosure analysis, fact-source requirements, constructibility, lifecycle,
placement and layout requirements, target capability requirements, and coverage
requirements.

No target opcode, stack index, tapleaf, transaction position, or target byte
enters compiler core. Concrete positions are backend output.

## Identity

The crate mints no public compiler digest. An analysis identity activates only
once a real cross-process, cached, or published consumer exists; until then
typed comparison is the boundary.

## Quickstart

The analysis workflow below binds input and projects target requirements.

```rust
use std::num::NonZeroU64;

use architecture::{ARCHITECTURE, OperationId};
use compiler::{AnalysisPolicy, CompilationScope, CompileError, ProofSearchLimits, bind_input};
use compiler::target::{
    ExternalEvidenceRole, PlacementSearchLimits, RequiredCapability, analyze_target_requirements,
};

// 0. The realization is the upstream owner's value; the compiler never
//    builds one, it only re-validates what it is handed.
let realization = realization::derive(
    &ARCHITECTURE,
    realization::RealizationScope::phase1_pilots(),
)
.expect("phase-1 pilots derive");

// 1. State the scope explicitly. It may be a subset of the realization's
//    scope, never a superset. Order does not matter: the scope is
//    canonicalized into stable architecture-code order.
let scope = CompilationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
    .expect("nonempty, no duplicates");
assert_eq!(
    scope.operations(),
    [OperationId::TransferLive, OperationId::CompactAsh],
);

// 2. State the policy and its search limits explicitly. There is no
//    ambient default: a silent limit is a silent truncation.
let policy = AnalysisPolicy::strict(ProofSearchLimits::new(
    NonZeroU64::new(1_000_000).unwrap(),
    NonZeroU64::new(10_000).unwrap(),
));

// 3. Bind. This re-runs the realization owner's validation, checks the
//    architecture binding, and checks scope containment.
let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("bindable");
assert_eq!(input.policy(), policy);
assert!(input.realization().operation(OperationId::CompactAsh).is_some());

// 4. Analyze. The complete scoped analysis plus its independent validator
//    run inside this call; what comes back is an opaque projection.
let requirements = analyze_target_requirements(
    &input,
    PlacementSearchLimits::new(
        NonZeroU64::new(10_000_000).unwrap(),
        NonZeroU64::new(1_000_000).unwrap(),
    ),
)
.expect("the pilot analysis completes");

// 5. Consume: two canonical censuses, and nothing else.
let capabilities: Vec<RequiredCapability> = requirements.capabilities().collect();
assert!(capabilities.iter().all(|c| RequiredCapability::ALL.contains(c)));
assert!(capabilities.windows(2).all(|pair| pair[0] < pair[1]));
assert_eq!(
    requirements.external_evidence().collect::<Vec<_>>(),
    [
        ExternalEvidenceRole::ConfidentialValueConservation,
        ExternalEvidenceRole::SubstrateConservation,
    ],
);

// Typed comparison is the whole comparison mechanism.
assert_eq!(requirements, requirements.clone());

// 6. A scope member the realization does not declare is a typed refusal.
let wider = CompilationScope::from_operations([OperationId::CompactAsh, OperationId::Burn])
    .expect("nonempty, no duplicates");
let pilots = realization::derive(
    &ARCHITECTURE,
    realization::RealizationScope::phase1_pilots(),
)
.unwrap();
assert_eq!(
    bind_input(&ARCHITECTURE, pilots, wider, policy).unwrap_err(),
    CompileError::IncompleteRealizationScope { operation: OperationId::Burn },
);

// 7. And a search budget too small to finish is a typed failure, never a
//    smaller requirement set. Partial requirements are precisely the
//    weakening a target assessment exists to prevent.
let truncated = PlacementSearchLimits::new(
    NonZeroU64::new(1).unwrap(),
    NonZeroU64::new(1).unwrap(),
);
assert!(matches!(
    analyze_target_requirements(&input, truncated),
    Err(CompileError::PlacementSearchStateLimitExceeded { .. }
        | CompileError::PlacementCandidateLimitExceeded { .. }),
));
```

## Public-API tour

The public modules are `burn_operation_plan`, `clear_operation_plan`, `error`, `input`, `live_transfer_plan`,
`maturity_announcement_plan`, `operation_plan`, `keyed_plan`, `keyed_fixed_plan`, and `target`; all are available in the default library.
Symbolic announcement requirements are also re-exported at the crate root; the
internal analysis containers remain crate-private.

`burn_operation_plan::plan_burn_target_operation` retains the public explicit
Burn route. `plan_burn_target_operation_with_output_selection` accepts an
offered output-change mode: `Explicit` delegates to that route, while
`PrivateCommitted` runs complete analysis, the mixed constructor and fresh
validation. Other offered modes refuse. Both return the same private-field
validated requirement type; derivation and validators remain crate-private.
The type carries report
obligations for Burn event anchors; it does not assert that a report was
produced or authenticated. Burn conservation now has an internally lowered
confidential alternative for Explicit input, Explicit Ash and
`PrivateCommitted` output change. The mixed route requires target-owned commitment-conservation
evidence, and its private change amount is never a public arithmetic operand.
For this tuple the change total equals explicit input minus explicit Ash, so the
total is public by arithmetic; the change output's own amount and split stay
outside the public operands.

The mixed projection retains distinct input/output choices, source rows,
external evidence, carrier alternatives, layout, coverage and lifecycle
obligations. Burn's optional change family has zero minimum, so both alternatives
admit a Burn without change; the mixed alternative then still requires its
commitment-relation source and the open commitment-conservation obligation
over an empty change family, without claiming that a target proof has run.
The target-owned commitment-conservation requirement is an open obligation,
not a completed target proof or emitted bytes. The focused corruption tests
select layout and coverage rows from the produced plan as omission pins;
they do not claim an independent layout or coverage oracle. The mixed oracle
forms the side choices and conservation source/evidence table from literal
declaration coordinates, and crosses four literal placement option sets; it
does not derive the full layout or coverage censuses independently. Its removed
carrier row is a publication pin. Four fresh-ASH relation bodies are compared
with explicit Burn after translating only the output-mode case identity.

`clear_operation_plan::plan_clear_target_operation` publicly constructs a
validated private-field Clear facade after complete product analysis and fresh
validation. Its 44 relation requirements include all six stored STATE assignments.
The independent oracle checks the analysis's local factors, carrier routing
and per-option layout without materializing Clear's enormous placement set.
It compares the facade's carrier alternatives with the explicit candidate's
expected product.

### `keyed_plan` — stable owner-typed plans

`keyed_plan::plan(&BoundCompilerInput<KeyedOwner>, PlacementSearchLimits)` returns
an immutable `KeyedPlan`; `project()` borrows its complete typed publication.
`revalidate(input, limits, &plan)` independently prepares the retained source and
reruns every shared stage before comparing the complete publication, including
scope, binding, obligations, coverage and execution reports. `plan` performs this
independent validation before returning. No producer cache is accepted.

The projection contains factorized proof/operation plans, per-relation declarations,
carrier alternatives and placement assignments, positive/negative coverage rows,
the coverage graph, explicit evidence premises, seven stage censuses, and proof
and placement limits/reports. SOURCE has four evidence rows and no runtime
carrier; MERGE has three evidence rows and two universally quantified
per-consumed-handle clauses, each with a `RuntimeClause` restored from case facts
that explicitly names `TransactionSide::Input` and `CarrierMultiplicity::EveryMember`; publication checks the clause object against its alternative carriers.
Reading retains inclusion, retrieval and finality
premises; publishing an obligation does not certify its witness or a target script.

`KeyedPlanError` preserves stage and defect distinctions with all restorable owner
identities. `Refused` and `CoverageGraph` include `unrestored_key_count`; a foreign
key does not erase the original refusal. The count is of distinct unrestored keys. Census subjects are the sorted,
deduplicated union of missing and unexpected relations. Complete proof-source,
no-feasible-plan and region refusals use dedicated variants; partial restoration
uses `Refused` with the same stage and named defect. Publication registry failures
use class-3 `CompileError::MissingOwnerIdentity`; lowering refusals retain their named defect even if their relation is unknown.
This distinguishes partial diagnostics from broken publication registries, as the
attestation adapter does. Unexpected layouts, unsupported runtime scopes and
foreign evidence receive named typed defects rather than that generic error.
The test-only opaque graph Owner symbol has no keyed reverse registry and remains
a documented class-3 publication site. Neither successful
plans nor errors expose raw engine records. The sealed public API cannot construct
synthetic stage defects; paired internal controls enter the production preparation
and analysis path, while `tests/keyed_plan.rs` is the external consumer. The raw
record entry remains a unit-test oracle. No feature or self dev-dependency is needed by a keyed consumer. `CarrierMultiplicity` is non-exhaustive; consumers must allow future variants, while the current keyed runtime clauses require `EveryMember`.

### `input` — the validated input boundary (P2-004)

Owner storage lives in private `src/input/owner.rs`; its public re-export paths
remain `compiler::input::{AttestationOwner, KeyedOwner}`. Lowering implementations
stay in private `engine_attestation` and `engine_keyed`. The owner/binding
public-item inventory in the default library is:

| Surface | Default owner/binding inventory |
|---|---|
| Attestation | `AttestationOwner` with one read-only method, `source`; `BoundCompilerInput<AttestationOwner>` and its five existing readers. |
| Keyed | `KeyedOwner` with three read-only methods, `manifest`, `source`, `scope`; `KeyedBindingError` (four variants). Keyed bound inputs expose `policy`; preparation, publication and independent validation are available through `keyed_plan`. |
| Binding | Sealed `InputBinding` (four associated types and `bind_owned`) and generic `bind_input`, statically fixing the owner. |
| Both owners | Zero public owner constructors, mutable owner accessors, engine-record types, lowering callbacks or public generic analysis callbacks. Both owner types implement `Clone` and `Debug`. |

The owner/input source lock checks explicit public fields, constructors, return
types, mutable receivers and re-export signatures. It is not a Rust trait-system
proof: mutation exposed indirectly by a future `DerefMut` or `AsMut` implementation
requires review of that implementation and is not covered by this text scanner.

`tests/public_api.rs` compiles the retained paths and method signatures as an
external consumer, checks that owner/source types cannot be substituted, and
locks the keyed facade without a feature guard. `tests/keyed_plan.rs` compiles all
three scopes and their public-boundary refusals in the default build. The compiler
has no feature table and no self dev-dependency. The `cargo-owner-public-consumers`
lint runs Clippy with warnings denied on architecture, realization and compiler
default libraries and on compiler's two public consumer test targets, then runs
those targets with the gate profile.
Private `engine_identity` delegates semantic identifier spelling to
architecture, takes `SemanticProfileId` for namespace encoding, and owns the
single transaction-side encoding (input 0, output 1); both lowerers use it.

- `CompilationScope::from_operations(impl IntoIterator<Item = OperationId>) -> Result<Self, CompileError>`
  — sorts into stable architecture-code order. `EmptyCompilationScope` for an
  empty scope; `DuplicateScopeOperation { operation }` for a repeat. A scope is
  a set, so a duplicate is a caller defect and is never silently normalized
  away. Two permutations of the same members compare equal.
- `CompilationScope::operations(&self) -> &[OperationId]` — canonical order.
- `ProofSearchLimits::new(maximum_states: NonZeroU64, maximum_candidates: NonZeroU64) -> Self`
  — public fields of the same names. `NonZeroU64` rather than `u64` because a
  zero limit is a search that never starts.
- `AnalysisPolicy::strict(ProofSearchLimits) -> Self` — the only reviewed
  policy, and the only constructor. Strict means: preserve every in-scope
  realization relation, never weaken an unsupported relation, retain explicit
  external-evidence requirements, and fail rather than emit a partial plan.
  Public field `proof_search_limits`.
- `bind_input(&architecture::Architecture, realization::ScopedRealizationSpec, CompilationScope, AnalysisPolicy) -> Result<BoundCompilerInput, CompileError>` — the only constructor of a bound input. It re-runs the realization owner's validation (`InvalidRealization`), requires the realization's architecture binding to equal the supplied architecture (`ArchitectureBindingMismatch`), and requires every scope member to be declared by the realization (`IncompleteRealizationScope { operation }`). `UnsupportedOperationPlan { operation }` is reserved for a future declared operation without a compiler plan.
- `BoundCompilerInput` — immutable, private fields, no mutable accessor, no
  serializer, no digest. Readers: `realization() -> &ScopedRealizationSpec`,
  `scope() -> &CompilationScope`, `policy() -> AnalysisPolicy`,
  `architecture_binding() -> &realization::ArchitectureBinding`,
  `architecture_operations() -> &BTreeSet<OperationId>` (the *complete*
  architecture census, so an analysis can state which operations lie outside
  its scope without guessing).

### `target` — the abstract target requirement boundary (Guide-8 §15)

This module is the whole vocabulary a target adapter needs from the compiler,
and deliberately nothing else. The compiler depends on no target package, so a
target-specific type is not merely discouraged here — it is unnameable.

- `RequiredCapability` — one abstract requirement an approved proof places on
  some target. Thirteen members in three groups: authenticated reading
  (`AuthenticatedObjectRecognition`, `AuthenticatedFamilyCardinality`,
  `AuthenticatedCanonicalPartition`, `AuthenticatedOpenFlowPartition`,
  `AuthenticatedRootEffects`, `AuthenticatedProjectionSet`), value arithmetic
  (`ExactPublicAmountArithmetic`, `ConfidentialValueConservation`,
  `WholeTransactionValueConservation`), and authorization
  (`OwnerAuthorization`, `OperatorAuthorization`, `RefundAuthorization`,
  `PublicConstructibility`).
  - `RequiredCapability::ALL: &'static [Self]` — the complete census in
    canonical order, generated with the enum from one declaration by
    `census_enum!`, so the two cannot disagree; the projection boundary
    still re-checks canonical order on every use. The order is a census
    order; it ranks nothing.
- `ExternalEvidenceRole` — the *class* of external claim a requirement carries.
  Three members. `ConfidentialValueConservation`: the plan holds a protocol
  asset's amounts as commitments, so nothing the analysis emits reads them and
  only the target's own confidential-transaction rules relate them.
  `SubstrateConservation`: no analysis, and no program an analysis could emit,
  discharges it — only the target's own consensus rules do. Neither implies the
  other — a target could conserve the whole transaction while carrying a
  protocol asset's amounts in the clear. `OperatorAuthorization`: the operator's
  authorization is an external claim because the realization carries no
  operator identity. Target-side operator-key encoding and signature control
  discharge it, never anything the analysis emits. `ExternalEvidenceRole::ALL`
  is the census. This is a compiler-owned projection of
  `realization::ExternalEvidenceRequirement` with the operation and asset
  identities dropped, because those are architecture-owned values a target
  adapter has no business reading, and because the adapter's package contract
  admits no realization dependency. The projection is total and exhaustive: a
  new realization requirement class fails to compile here until this module
  states its role.
- `PlacementSearchLimits::new(maximum_states: NonZeroU64, maximum_candidates: NonZeroU64) -> Self`
  — same explicit-configuration rule as `ProofSearchLimits`, for the placement
  search. The candidate limit bounds candidates actually materialized; it does
  not bound the symbolic cardinality of a checked placement product. Product
  construction charges the root, every factor and every option to the state
  limit. Explicit bounded expansion checks its full candidate count first and
  returns no partial set on exhaustion. The existing enumerator still applies
  both limits to its bounded search as before. Internal Clear analysis selects
  the checked product through the shared assembly; active attestation and both
  keyed owners retain enumerated members. Public Clear binding is supported.
  The placement-storage test's literal carrier table and test-side layout rule
  are independent of product construction, but its source routings and family
  scope come from `carrier::relation_case_eligibility`, which attestation
  lowering also calls. The Clear activation oracle derives expected routing
  from typed source classes and literal carrier roles without that function.
- `TargetRequirementSet` — read-only and opaque: private fields, no public
  constructor, no `Default`, no builder. `capabilities() -> impl Iterator<Item = RequiredCapability>`
  and `external_evidence() -> impl Iterator<Item = ExternalEvidenceRole>`, both
  in canonical census order. A set assembled from arbitrary capabilities would
  be a *request* rather than an analysis, and nothing downstream could tell the
  two apart once they shared a type.
- `analyze_target_requirements(&BoundCompilerInput, PlacementSearchLimits) -> Result<TargetRequirementSet, CompileError>`
  — the only route to the type. It assembles the scoped analyzed program, runs
  the complete re-derivation validator inside that assembly, checks the derived
  censuses for duplicates and canonical order, derives the requirement set, and
  returns an opaque projection. A caller receives the requirements or a typed
  failure; there is no partial result and no route to the analyzed program
  itself.

A requirement set is the **union** of what the retained proof alternatives
require, never the intersection. Phase 2 selects no proof plan, and the
intersection would silently drop every capability that only one retained
alternative needs — precisely the weakening a target assessment exists to
prevent.

### `live_transfer_plan` — the live-transfer target-operation projection (Guide-13 §8)

`plan_live_transfer_target_operation` projects one operation out of a bound
input and returns a `ValidatedLiveTransferOperationPlan`: the class closure,
the owner family, the value projection, the admitted representation plans and
their per-representation coverage, the candidate lifecycle, and the sponsor
case vocabulary.

The projected operation is fixed to `TransferLive` and is not a parameter. An
analyzed program may carry several operation factors; this boundary commits to
exactly one before any target program is planned, which is what makes a
program mixing operations something no later layer is ever handed rather than
something a later layer refuses.

Burn and redeem appear in the plan only as outstanding lifecycle exits. No
constructor is derivable from a plan whose lifecycle is complete, and no
pretend leaf is emitted for an exit that is not implemented.

### `maturity_announcement_plan` — the maturity-announcement target-operation projection (Guide-14 Wave 2)

`plan_maturity_announcement_target_operation(&BoundCompilerInput, PlacementSearchLimits) -> Result<ValidatedMaturityAnnouncementOperationPlan, CompileError>`
projects the fixed `AnnounceMaturity` operation. The operation is not a
parameter: a successful result has committed to that one operation before
target planning begins.

The validated container carries the complete typed source and exposes only
readers: `operation`, `source`, `representation`, `state`, `operator`,
`canonical`, `sponsor`, `roots`, `certificate`, `lifecycle`,
`constructibility`, `public_facts`, `transition`, `root_history`,
`public_recovery`, `representations`, and `projection`. The representation
policy admits exactly `Explicit` and `PublicCommitted`; `PrivateCommitted` has
no announcement plan.

The STATE projection retains separate input and output recognition and
cardinality relations. Its `input_closure` and `output_closure` each admit
exactly STATE and the ordinary L-BTC sponsor family, and its forbidden set is
every other object family. It carries no owner family and states no conserved
sum. The canonical projection likewise requires an empty delta: the operation
updates STATE rather than moving a receipt value.

The lifecycle has one implemented exit, `AnnounceMaturity`, and five
outstanding exits: `AdmitDeposits`, `Cycle`, `Redeem`, `ReceiptRelabel`, and
`Clear`. No public field carries a transaction position, target encoding,
digest, byte count, or sponsor amount. The announcement-specific errors are
`MaturityAnnouncementContractDefect { clause }` and
`MissingMaturityAnnouncementRepresentation { operation, representation }`.

The crate-root
`validate_maturity_announcement_plan(&ValidatedMaturityAnnouncementOperationPlan, &BoundCompilerInput) -> Result<(), CompileError>`
re-checks a plan against its complete input and returns either a complete
verdict or a typed error, never a partial verdict. The planning entry is the
only route to the validated type: there is no public constructor, `Default`,
or builder.

### Crate root

- `CompileError` — re-exported from `error`; see below.
- `ExpressionCycleComponent { members: Vec<realization::ExprId>, internal_edges: Vec<realization::ExpressionDependencyProjection> }`
  and `RelationCycleComponent { members: Vec<realization::RelationId>, internal_edges: Vec<realization::RelationDependencyProjection> }`
  — one canonically ordered cyclic strongly connected component each, carried
  by the two cycle error variants so a caller can report the actual cycle
  rather than its existence.
- `OperandId` — the stable identity of one relation operand, with
  `relation() -> &realization::RelationId`. Read the caveat under
  *Residuals* below before depending on it.
- `validate_required_dependency` — checks whether one authorization case
  discharges one required constructibility dependency, returning the typed
  permissionless or unavailable-witness refusal when it does not; a node with
  no availability class is discharged by every case.
- `validate_maturity_announcement_plan` — re-checks the complete input as the
  maturity-announcement section above describes.
- The announcement requirement re-exports form one symbolic vocabulary: the
  `Announcement*` roles and duties, the `StateField*` law vocabulary,
  `StateSuccessionRequirement`, `PublicRecoveryRequirement`,
  `ConstructorContinuityRequirement`, `RootHistoryRequirement`, and the
  `AnnouncementRequirementMutation` mutation census. The residual below
  describes what those values do and do not establish.

## Error handling

`CompileError` is the error root for compilation analysis. It derives
`thiserror::Error` (so `Display` and
`std::error::Error`) and is `PartialEq`, so variants can be compared directly.
It carries **architecture-owned identifiers** — `OperationId`, `ObjectId` —
rather than compiler-local restatements of them, so an error never duplicates
an upstream identity or obscures its owner.

The input-boundary entry points report the following failures.

### From `CompilationScope::from_operations`

`EmptyCompilationScope`, `DuplicateScopeOperation { operation }`.

### From `bind_input`

`InvalidRealization` (the realization failed its owner's re-run validation), `ArchitectureBindingMismatch` (the realization binds a different architecture identity than the one supplied), `IncompleteRealizationScope { operation }` (a scope member the realization does not declare), `UnsupportedOperationPlan { operation }` (reserved for a future declared operation without a compiler plan), and `UnsupportedRealizationSchema { schema }`. A Clear plus `TransferLive` realization binds with Clear in compiler scope. A realization lacking Clear still refuses that scope as `IncompleteRealizationScope`.

### From `analyze_target_requirements`

Everything else — the analysis is complete, so any stage can fail. The
variants group by stage, and a caller normally distinguishes three classes
rather than a hundred names:

1. **Search budget exhausted** — the analysis genuinely started and genuinely
   did not finish: `ProofSearchStateLimitExceeded`, `ProofCandidateLimitExceeded`,
   `PlacementSearchStateLimitExceeded`, `PlacementCandidateLimitExceeded`.
   These are the variants to match when you might retry with larger limits. A
   truncated search is never reported as a smaller requirement set.
2. **The realization does not admit a plan** — a real finding about the input,
   not a defect in the compiler: `NoFeasibleProofPlan`, `NoEligibleCarrier`,
   `MissingProofAlternative`, `MissingRepresentationChoice`,
   `MissingLifecyclePath`, `PermissionlessPrivateDependency`,
   `PermissionlessPrivateFact`, `SponsorValueRead`, `SponsorDependencyEscaped`,
   `UndecidableSponsorRegion`, `GlobalRelationHasOnlyLocalCarrier`,
   `ConstructibilityWitnessUnavailable`, the `MissingSourceRequirement` /
   `UnauthenticatedSourceRequirement` pair, and the `Missing*Coverage` family.
3. **Structural defects in the scoped source or the assembled program** — the
   graph, census, closure, and mismatch families: `Duplicate*`, `Unknown*`,
   `*CensusMismatch`, `*ClosureMismatch`, `*Cycle` (carrying the cycle
   components above), `*EscapesScope`, `CrossOperation*`, `ConstantFold*`,
   `NonBooleanPredicateExpression`, `Analyzed*Mismatch`,
   `NoncanonicalCapabilityCensus`, `NoncanonicalEvidenceRoleCensus`. Most of
   these are raised by the *independent validators* that re-derive the analysis
   and compare, so seeing one means a corruption-resistance check fired rather
   than that a user did something wrong.
   `MissingOwnerIdentity` is also class 3: a compiler-internal reverse-registry
   defect, not a claim that the realization admits no plan. It exposes neither
   an engine key nor a free-text domain in the public API.

There is no partial-success return anywhere in the crate: every failure is an
`Err`, and no `Ok` value is a truncated version of a larger one.

## What this package deliberately does not do

- It emits **no target program**, and no target opcode, stack index, tapleaf,
  control block, transaction position, or target byte enters compiler core.
  Concrete positions are backend output.
- It exposes **no public complete-analysis result**. The analyzed program, its
  proof plans, placements, coverage graph, search reports, and every graph
  handle inside them stay crate-private. The target boundary publishes a
  projection of the result rather than the result.
- It mints **no compiler-plan identity** and no analysis digest. Under the
  recorded identity policy an analysis identity activates only once a real
  cross-process, cached, or published consumer exists; until then typed
  comparison is the boundary, and a field reserved for a future digest would
  itself be a speculative identity.
- It provides **no target capability adapter** and names no target-specific
  type. It depends on no target package, so it could not.
- It performs **no I/O**: no file is opened, no environment variable is read.
- It does not select a proof plan. Requirement sets are unions over retained
  alternatives.
- It does not construct or validate a realization from scratch; it re-runs the
  realization owner's validation on a value it was handed.

Diagnostic search statistics exist internally but are deliberately absent from
every stable projection: a state count is an artifact of how the search walked
its options, not a property of what it found.

## State

Implemented: typed input binding (P2-004); exact scoped relation and
expression DAGs (C1-005/P2-005); conservative checked constant folding
(P2-006); proof-obligation classification, exact feasible proof-plan
enumeration under explicit search limits, authenticatable source
requirements, authorization-case constructibility analysis, plan-specific
disclosure analysis, and representation lifecycle analysis
(C1-008/P2-007/P2-008/P2-009), oracle-checked; typed execution cases,
relation discharge classification, carrier eligibility, exact feasible
placement, and target-independent layout requirements (C1-009/P2-010);
relation-indexed coverage requirements, typed coverage dependencies with a
forbidden-cycle SCC policy, and independent placement and coverage oracles
(P2-011/C1-010/C1-013); relation-indexed requirement bundles with exact
aggregate closure and complete scoped analyzed programs for the pilot scope,
factorized per operation, with a corruption-resistant assembly validator and
an independent assembly census oracle (P2-012); the abstract target
requirement projection, derivable only from a completely validated analyzed
program (Guide-8 §15); the validated maturity-announcement plan boundary, its
evidence-closure, sponsor-erasure-traversal, and requirement-precedence
corruption validators and corruption catalogue, the independent announcement
census oracle, and consumer-only public-API coverage of the symbolic
announcement requirement vocabulary and of the validated plan boundary
(Guide-14 Wave 2).

Not implemented: target capability adapter, concrete target layout, and
target program emission. The analysis structures are crate-private, no
complete analyzed program is exposed publicly, and no compiler-plan identity
exists — typed comparison remains the boundary. No public value this crate
produces today can be mistaken for a completed analysis.

## Relationship to neighbors

- **`tripod-architecture`** supplies the typed manifest `bind_input`
  checks against, and owns every identifier that appears in `CompileError`.
- **`tripod-realization`** supplies the `ScopedRealizationSpec` that
  is the analysis subject. The compiler re-runs realization validation rather
  than trusting the value, and its `CompilationScope` must be a subset of the
  realization's own scope. Realization types appear in the compiler's *inputs*
  and in the cycle-component diagnostics, but the target boundary deliberately
  projects realization evidence requirements into a compiler-owned role so no
  realization type reaches a future adapter's signature.
- **A future target adapter** consumes `target::TargetRequirementSet` and
  nothing else from this crate. That package does not exist yet; the boundary
  is stated ahead of it so the adapter's contract can forbid realization and
  architecture dependencies.
- The crate depends on **no target package** — not `target-elements`, not
  `tapscript` — and that absence is what makes target-freedom checkable by the
  package graph rather than by review.

## Residuals

The operation-plan API re-exports `BurnClearCoordinate` through `OperandRole`
and the corresponding `RelationMutation`. These are closed semantic field
identities for future Burn/Clear source and coverage rows. The source key uses
an appended operand role and a fact subject outside the existing fact domain;
the mutation uses an appended purpose code. They carry no transaction position
or observed value. Explicit Burn now binds publicly and returns requirements;
Clear is admitted by public input binding. Neither facade asserts a
target observation or completed report evidence.

Two notes for a consumer reading this as a manual rather than as prose:

- `OperandId` is re-exported at the crate root, but its constructor
  `OperandId::new` and its `role()` accessor both mention `OperandRole`, which
  is *not* exported. Externally the type is therefore effectively opaque: it can
  be received, compared, ordered, hashed, and asked for its `relation()`, but
  not constructed and not fully destructured. Treat it as a read-only handle.
- No public API returns an analyzed program, so there is currently no way for a
  consumer to inspect *why* a given capability is required. The requirement set
  answers what, not why, by design.

### Symbolic maturity-announcement requirements

`AnnouncementMetadataRequirement::required()` returns six `StateFieldRequirement`
rows with input/output `FactId` keys and a `StateFieldLaw`. Its `public_facts()`
returns fifteen keys in realization declaration order. `StateFieldLawKind`
provides eleven law signatures; `StateLawOperand` names either a fact or a
realization-owned `StateLawParameter`. `StateFieldRequirement::validate()` checks
subject sides, matching operation and field, and operand count, returning
`StateFieldRequirementError` for a structural defect. It executes no transition.

`AnnouncementRequirementMutation` is the payload-free census of seventeen
announcement requirement mutations: a requirement row names an obligation for
future validation, while a mutation names a way that obligation can be missing
or altered. Neither is discharged evidence, and constructing or naming one
claims no witness.

The five copy laws have no operands; the maturity law names the requested cycle.
The separate checked inclusive window names the input cycle, request, and two
lead selectors, whose `bound_id()` mappings identify the calibrated architecture
bounds. Lead bounds are not field-law operands.

`StateSuccessionRequirement` carries optional complete input/output field sets.
`PublicRecoveryRequirement` carries seven source facts, six result facts, six
ancillary roles, and six recovery steps. `ConstructorContinuityRequirement` and
`RootHistoryRequirement` retain constructor and history duties. These are
requirements for future validation; constructing them claims no validated plan.

## Fixed-form keyed plans

Under (`[PLAN-sec:compiler:fixed-form]`), `keyed_fixed_plan::plan` consumes a sealed `BoundCompilerInput<input::KeyedFixedOwner>` and placement limits. The owner retains the complete schema-3 manifest, validated realization and explicit scope through read-only accessors. `KeyedFixedBindingError` names invalid realization, empty scope, duplicate operation and foreign operation refusals. `KeyedFixedPlan` exposes an immutable `KeyedFixedPlanProjection`; `revalidate` performs fresh preparation and all shared stages from the retained source. Projection clones confer no construction authority. The external consumer is `tests/keyed_fixed_plan.rs`; downstream S5 consumes the same public facade.

The projection publishes typed evidence obligations and premises, runtime clauses/carriers, relation/operation/proof plans, complete coverage requirements, symbols/nodes/dependencies, stage censuses, limits and search reports. `KeyedFixedPlanError` retains stage, defect and restored owner identities; unrestorable identities retain their count. SOURCE/MERGE/joint scopes have 4/6/10 relations, 4/3/7 evidence obligations, 0/3/3 per-member runtime clauses and 28/39/67 coverage nodes. SOURCE has no runtime carrier. The three MERGE clauses retain Input/EveryMember execution scope without accounting, a maximum family size or any constraint on surplus destinations.

Lowering reuses `engine_records::analyze_relations`, `engine_foundation::validate_predicates`, `engine_source::analyze_sources`, `engine_constructibility::analyze_constructibility`, `engine_lifecycle::analyze_lifecycle`, `engine_proof::enumerate`, `engine_requirement::proof_tables`, `engine_cases::select`, `engine_placement`, `engine_coverage`, `engine_coverage_join` and `engine_coverage_graph` through `analyzed::analyze_bound_prepared`. Preparation and restoration are fixed-form siblings; no graph algorithm or analyzer is copied. The opaque source arm retains complete values and selects no policy.

The first combined owner consumption compares the manifest clause multiset with the declaration clause multiset, before lowering. Relation-kind codes come directly from realization's five-component encoding. Architecture still owns namespace 3. Own-qualification sources require the authenticated spent object, including program, asset/value fields and forms, and symbolic K/A/v public construction data. Successor qualification additionally requires the authenticated output object. S5 owns the concrete field-introspection recipe; this compiler supplies no deployment values. The member predicate's key/program authentication obligation belongs to S5/S7, and target amount admission to S6 under ruling 212, not to symbolic compilation.

The ten S4 reopenings are recorded at (`[PLAN-tab:phase7:fixed-form-legacy-rulings]`). Their dispositions are below; these controls establish symbolic compiler behavior, not target execution or evidence witnesses.

| Ruling and source | S4 count or check and receiver |
|---|---|
| 158, (`[PLAN-rule:phase7:wave1-realization-rulings]`) | One sealed binding implementation, one opaque source arm and one lowering-error arm. One test-only attestation exhaustive-match forwarding arm is compile-forced. The binder's `ForeignOperation` is representational and unreachable for the closed validated operation census. In `program.rs`, `premises` has one compile-forced foreign-evidence refusal arm with a reaching negative; `source_evidence` has three representational foreign-evidence encoding arms that this owner never lowers, with no reaching subject. |
| 166, (`[PLAN-rule:phase7:wave1-engine-rulings]`) | `InputBinding` fixes `KeyedFixedOwner` once. `revalidate` freshly prepares retained source and reruns every shared stage; `production_keyed_publication_equals_record_oracle_and_revalidates` checks changed publication rejection. |
| 167, (`[PLAN-rule:phase7:wave1-engine-rulings]`) | All nine ancillary declaration fields have read/refusal subjects in `program/tests.rs`. SOURCE's four evidence obligations survive proof, placement and coverage in the independent external projection. Representation and lifecycle record subjects are refused in `program/negative_tests.rs`; empty stages are derived, not skipped. |
| 168, (`[PLAN-rule:phase7:wave1-engine-rulings]`) | Seven relation-vocabulary sites: `check_clauses` (four exhaustive arms), `source_table` (four), `lower_placements` (two), `lower_sources` (two exhaustive arms), two `qualification_sources` matches and one mandatory-input-family match. They read variants and fields, not identities to choose policy. No attestation law enters this owner; the historical seven-site baseline and separate six-site ledger remain unchanged. |
| 171, (`[PLAN-rule:phase7:wave1-home-rulings]`) | Sibling preparation/publication remains in the shared compiler home, with no new analyzer or crate. Frozen first-edition owner/plans and their regression tests remain unchanged; both-direction foreign-attestation controls and the ten attestation projection pins retain their boundaries. |
| 172, (`[PLAN-rule:phase7:wave1-home-rulings]`) | Three namespace-qualified case/carrier/purpose encoders retain complete payloads and owner-held reverse registries. `program/tests.rs` pins ten relation encodings and checks same-local-payload collisions against profiles 1 and 2 and reverse-registry refusals. No engine-to-owner policy callback is added. |
| 174, (`[PLAN-rule:phase7:wave1-home-rulings]`) | `tests/public_api.rs` locks seven facade re-exports, the sole permitted engine-path alias, private immutable publication and absence of constructors/builders/mutable accessors/graph handles. Typed read-only owner paths and both-direction owner/source substitution controls cover the normal library boundary. |
| 175, (`[PLAN-rule:phase7:wave1-home-rulings]`) | `tests/keyed_fixed_plan.rs` consumes SOURCE, MERGE and joint scopes through the normal library, compares full independent publication expectations and freshly revalidates. Sibling stage/source/placement/coverage negatives and unchanged attestation pins run beside it. |
| 176, (`[PLAN-rule:phase7:wave1-home-rulings]`) | Two pass-through operation/object helpers consume architecture's encoding; runtime Input uses `side_code`. `engine_identity.rs` retains old side/namespace pins and adds namespace 3. Cross-owner collision controls introduce no semantic kernel match. |
| 178, (`[PLAN-rule:phase7:wave1-home-rulings]`) | `src/tests/engine_cases_tests.rs` compares complete canonical eligibility from direct lowering and shared case selection with literal rows in every scope: SOURCE zero, MERGE/joint three `EveryMember` rows with one `PerMember` carrier, empty layout, and `NoEligible`. Equal-length key, multiplicity, entry, option, carrier and empty-cause subjects change one coordinate. The existing canonical comparison destructures every row/entry field and its layout-coordinate control remains in place. |

The clause join has reaching owner-level controls in `program/tests.rs`: removing each manifest clause beside a validated-source control refuses both `lower_relations` and `lower_sources` with exactly `ClauseCensus`. The `lowering_rows` refusal table includes its complete public `CensusMismatch` payload.
