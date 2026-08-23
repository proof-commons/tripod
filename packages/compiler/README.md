# `tripod-compiler`

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

The whole public workflow is two calls, and there is no route between them.

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

Three public modules, and that narrowness is the design. Everything between
the input boundary and the target boundary is crate-private.

### `input` — the validated input boundary (P2-004)

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
- `bind_input(&architecture::Architecture, realization::ScopedRealizationSpec, CompilationScope, AnalysisPolicy) -> Result<BoundCompilerInput, CompileError>`
  — the only constructor of a bound input. It re-runs the realization owner's
  validation (`InvalidRealization`), requires the realization's architecture
  binding to equal the supplied architecture (`ArchitectureBindingMismatch`),
  and requires every scope member to be declared by the realization
  (`IncompleteRealizationScope { operation }`).
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
  Two members. `SubstrateConservation`: no analysis, and no program an analysis
  could emit, discharges it — only the target's own consensus rules do.
  `ConfidentialValueConservation`: the plan holds a protocol asset's amounts as
  commitments, so nothing the analysis emits reads them and only the target's
  own confidential-transaction rules relate them. Neither implies the other — a
  target could conserve the whole transaction while carrying a protocol asset's
  amounts in the clear. `ExternalEvidenceRole::ALL` is its census. This is a compiler-owned
  projection of `realization::ExternalEvidenceRequirement` with the operation
  and asset identities dropped, because those are architecture-owned values a
  target adapter has no business reading, and because the adapter's package
  contract admits no realization dependency. The projection is total and
  exhaustive: a new realization requirement class fails to compile here until
  this module states its role.
- `PlacementSearchLimits::new(maximum_states: NonZeroU64, maximum_candidates: NonZeroU64) -> Self`
  — same explicit-configuration rule as `ProofSearchLimits`, for the placement
  search.
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

## Error handling

`CompileError` is the single error root for the whole crate; every public
fallible function returns it. It derives `thiserror::Error` (so `Display` and
`std::error::Error`) and is `PartialEq`, so variants can be compared directly.
It carries **architecture-owned identifiers** — `OperationId`, `ObjectId` —
rather than compiler-local restatements of them, so an error never duplicates
an upstream identity or obscures its owner.

Only two public entry points exist, so the practical question is which
variants each can raise.

### From `CompilationScope::from_operations`

`EmptyCompilationScope`, `DuplicateScopeOperation { operation }`.

### From `bind_input`

`InvalidRealization` (the realization failed its owner's re-run validation),
`ArchitectureBindingMismatch` (the realization binds a different architecture
identity than the one supplied), `IncompleteRealizationScope { operation }` (a
scope member the realization does not declare), and
`UnsupportedRealizationSchema { schema }`.

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
program (Guide-8 §15).

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

Two notes for a consumer reading this as a manual rather than as prose:

- `OperandId` is re-exported at the crate root, but its constructor
  `OperandId::new` and its `role()` accessor both mention `OperandRole`, which
  is *not* exported. Externally the type is therefore effectively opaque: it can
  be received, compared, ordered, hashed, and asked for its `relation()`, but
  not constructed and not fully destructured. Treat it as a read-only handle.
- No public API returns an analyzed program, so there is currently no way for a
  consumer to inspect *why* a given capability is required. The requirement set
  answers what, not why, by design.
