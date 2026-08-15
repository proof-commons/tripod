# Guide 6 — Relation-Indexed Coverage and Typed Coverage Closure

## Mission

Complete the remaining Phase-2 coverage work:

```text
P2-011  relation-indexed coverage requirements
C1-010  typed symbol resolution and SCC policy
C1-013  independent coverage oracle
```

Guide 6 consumes the exact feasible proof plans, execution cases, abstract
placements, and layout requirements delivered by Guide 5. It must derive a
complete, deterministic, target-independent evidence contract for every
relation in every applicable semantic case.

The intended pipeline is:

```text
validated compiler input
    ↓
exact feasible proof plans
    ↓
execution cases
    ↓
relation-case discharge plans
    ↓
feasible abstract placements
    ↓
target-independent layout requirements
    ↓
relation-indexed coverage requirements
    ↓
independent coverage oracle
```

Guide 6 must also resolve the Guide-5 finding that compact ASH declares sponsor
recognition without sponsor cardinality relations. The default ruling from the
current architecture and package contracts is that this is a **realization
omission**, not an intentional exception: the architecture already declares
the sponsor input/output cardinalities, and relation-indexed coverage must not
certify an incomplete realization relation census.

At completion, Phase 2 should be ready for:

```text
Guide 7 — P2-012 complete analyzed pilots and internal analyzed-program value
```

Do not begin target/backend work in this guide.

---

# 1. Baseline and authority

## 1.1 Starting point

Guide 6 begins from:

```text
branch:
    main

starting commit:
    0.2.2-dev

Guide-5 state:
    C1-009 DONE
    P2-010 DONE
    placement oracle delivered
    P2-011 next TODO
```

Before editing, verify:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean and the starting commit must be the intended Guide-5
merge.

## 1.2 Required reading

Read these implementation owners before changing source:

```text
AGENTS.md

adr/011-toolchain-and-dependency-policy.md
adr/013-global-label-graph.md
adr/014-meson-lint-census-and-stamps.md
adr/016-semantic-identities-and-evidence-binding.md

packages/compiler/src/relation.rs
packages/compiler/src/case.rs
packages/compiler/src/carrier.rs
packages/compiler/src/placement.rs
packages/compiler/src/layout.rs
packages/compiler/src/proof.rs
packages/compiler/src/source.rs
packages/compiler/src/error.rs

packages/compiler/src/tests/pilot_placement_tests.rs
packages/compiler/src/tests/placement_oracle_tests.rs

packages/realization/src/declarations/compact_ash.rs
packages/realization/src/declarations/transfer_live.rs
packages/realization/src/relation.rs
packages/realization/src/validate.rs

plans/backlog.md
plans/packages/compiler.md
plans/phases/02-compiler.md
plans/research/compiler-algorithms.md
```

## 1.3 Ownership rules

Guide 6 preserves these boundaries:

```text
realization:
    owns semantic relations and approved proof alternatives

compiler:
    owns analysis, placement, layout, and required coverage

backend:
    later selects concrete target programs and positions

vectors:
    later materializes coverage requirements into concrete fixtures,
    mutations, target transactions, and reports

release:
    later verifies coverage completion for exact bundle and ABI identities
```

Compiler coverage output says:

```text
evidence is required
```

It does not say:

```text
evidence has been produced
```

---

# 2. Scope and non-goals

## 2.1 In scope

- resolve compact-ASH sponsor-cardinality asymmetry;
- add missing realization relations if confirmed;
- prevent recurrence with typed realization coverage validation;
- define stable relation-indexed coverage requirement types;
- derive coverage per proof plan, operation, execution case, and relation;
- distinguish active-valid, active-invalid, and inactive-valid coverage;
- derive boundary-specific positive and negative requirements;
- derive carrier-execution requirements without concrete target positions;
- derive accepted semantic-projection requirements;
- derive dependency-collateral closure;
- implement two-pass typed coverage-symbol resolution;
- implement a direct Petgraph coverage-dependency graph;
- normalize SCC diagnostics by stable typed key;
- reject unsupported coverage cycles;
- validate exact equality among relation, placement, layout, and coverage
  scopes;
- implement an independent direct relation × case coverage oracle;
- integrate both pilots;
- update compiler/package/phase/backlog status after verification.

## 2.2 Out of scope

- target opcodes;
- tapscript programs;
- tapleaves or taptrees;
- stack or witness positions;
- transaction input/output ordinals;
- concrete coordinator indexes;
- target capability adapters;
- transaction ABI;
- concrete semantic fixtures;
- concrete malformed target transactions;
- target execution;
- resource calibration;
- public complete analyzed-program API;
- compiler-plan serialization;
- realization or compiler digest;
- generated compiler publication;
- release evidence envelopes;
- external solver dependency.

No new dependency is expected.

---

# 3. Conceptual model

## 3.1 Coverage is relation-indexed and case-indexed

Coverage identity begins with the stable pair:

```rust
RelationCaseKey {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}
```

Every relation in every applicable execution case appears exactly once in the
coverage analysis, including when its disposition is:

- active runtime requirement;
- compiler-static;
- backend-structural;
- external evidence;
- inactive but valid.

A relation must never disappear merely because no runtime carrier executes it.

## 3.2 Coverage has several evidence boundaries

Coverage must preserve the discharge boundary from Guide 5.

A useful typed vocabulary is:

```rust
enum CoverageBoundary {
    CompilerAnalysis,
    BackendStructure,
    RuntimeTarget,
    ExternalEvidence,
}
```

Equivalent reuse of `DischargeBoundary` is acceptable if the semantics remain
clear.

The coverage expected at each boundary differs:

| Boundary | Positive evidence | Negative evidence |
|---|---|---|
| compiler analysis | valid typed analysis accepted | malformed or unsupported typed input rejected |
| backend structure | required program/layout property present | missing, hidden, or contradictory structure rejected |
| runtime target | target accepts valid operation and carrier executes | focused target mutation rejects |
| external evidence | typed report verifies exact subject | missing, failed, stale, mismatched, or malformed report rejects |
| inactive case | valid case accepted without activating relation | unexpected activation or carrier execution rejects |

Do not force all relation kinds into “target accepts / target rejects.”

## 3.3 Coverage requirement is not evidence completion

A coverage requirement names:

- subject relation;
- execution case;
- evidence boundary;
- expected disposition;
- required carrier behavior where applicable;
- semantic facts or operands to compare;
- mutation class;
- expected dependent collateral;
- representation condition;
- evidence role.

It carries no mutable completion status.

Later evidence packages produce reports answering those requirements.

---

# 4. Tranche A — Resolve compact-ASH sponsor cardinality

## 4.1 Finding

Guide 5 found:

```text
compact ASH:
    sponsor input recognition declared
    sponsor output recognition declared
    sponsor input cardinality absent
    sponsor output cardinality absent

live transfer:
    sponsor recognition and cardinality both declared
```

The architecture row for compact ASH declares:

```text
PLAIN_LBTC input:
    minimum 0
    maximum FEE_SPONSOR_INPUT_MAX

PLAIN_LBTC output:
    minimum 0
    maximum 1
```

Therefore the compiler relation graph currently has no cardinality relation
owning those architecture facts for compact ASH.

## 4.2 Ruling

Treat this as a realization omission.

Rationale:

1. architecture owns the cardinalities;
2. realization is required to expose the semantic relations compiler analysis
   consumes;
3. live transfer already exposes the same sponsor cardinality shape;
4. Guide-5 carrier derivation correctly refused to invent a sponsor family from
   an absent cardinality relation;
5. Guide 6 must not claim coverage equality over an incomplete realization
   census;
6. adding the relations makes existing architecture semantics explicit—it does
   not change the architecture or accepted operation behavior.

If implementation evidence disproves this ruling, stop and record a typed,
reviewed exception. Do not silently retain the asymmetry.

## 4.3 Add compact-ASH sponsor cardinality relations

In:

```text
packages/realization/src/declarations/compact_ash.rs
```

add stable relation IDs for:

```text
PLAIN_LBTC input cardinality
PLAIN_LBTC output cardinality
```

The relations should match the architecture exactly:

```rust
Relation::Cardinality {
    side: ObservedSide::Input,
    object: ObjectId::PlainLbtc,
    minimum: Count::ZERO,
    maximum: CardinalityMaximum::Bound(
        BoundId::FeeSponsorInputMax,
    ),
}
```

and:

```rust
Relation::Cardinality {
    side: ObservedSide::Output,
    object: ObjectId::PlainLbtc,
    minimum: Count::ZERO,
    maximum: CardinalityMaximum::Exact(Count::ONE),
}
```

Both use:

```rust
ProofKind::ManifestShape
```

Add dependencies:

```text
sponsor input recognition
    → sponsor input cardinality

sponsor output recognition
    → sponsor output cardinality
```

using:

```rust
RelationEdge::RecognitionBeforeCardinality
```

Do not add sponsor amount operands.

## 4.4 Strengthen realization relation-coverage validation

Add a typed validation rule for the scoped pilot operations:

> Every architecture-declared input and output family in realization scope has
> exactly one matching cardinality relation and one matching recognition
> relation.

The check should compare:

- operation;
- side;
- object;
- minimum;
- maximum;
- object asset.

The architecture remains the owner of the values.

Do not implement this by operation-name matching. Derive the expected census
from the architecture operation row and compare it with the realization
relations.

A suitable failure shape is:

```rust
MissingArchitectureRelation {
    operation: OperationId,
    kind: RequiredRelationKind,
    subject: RelationSubject,
}
```

or a focused extension of existing architecture-mismatch errors.

Only add variants that real validation branches produce.

## 4.5 Focused tests

Add tests establishing:

- compact ASH has sponsor input cardinality;
- compact ASH has sponsor output cardinality;
- sponsor input maximum is `FEE_SPONSOR_INPUT_MAX`;
- sponsor output maximum is exactly one;
- sponsorless cases mark sponsor-family cardinality inactive/vacuous;
- sponsored cases mark sponsor-family cardinality active;
- missing sponsor input cardinality fails derivation;
- missing sponsor output cardinality fails derivation;
- transposed input/output cardinality fails;
- wrong bound fails;
- no sponsor amount appears in source, placement, layout, or coverage
  requirements.

Update Guide-5 count/placement assertions only where the new valid relation
census changes them. Prefer exact set assertions over unexplained magic counts.

Historical Guide-5 measurements in its gate record remain historical and should
not be rewritten.

---

# 5. Tranche B — Coverage vocabulary

## 5.1 Stable requirement identity

Define a stable typed coverage key.

One suitable design is:

```rust
struct CoverageRequirementId {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    purpose: CoveragePurpose,
}
```

with a purpose vocabulary such as:

```rust
enum CoveragePurpose {
    ActiveAccept,
    InactiveAccept,
    FocusedReject(RelationMutation),
    CarrierExecution(PlacedCarrier),
    AcceptedProjection,
    StructuralPresence,
    StructuralRejection,
    ExternalEvidenceVerified,
    ExternalEvidenceRejected,
}
```

Equivalent factoring is acceptable.

Requirements:

- complete typed key;
- canonical ordering;
- no digest;
- no graph handle;
- no candidate vector index;
- no target program identity;
- no target position.

## 5.2 Coverage disposition

Every relation-case entry should state whether the relation is:

```rust
enum CoverageActivity {
    Active,
    InactiveValid,
}
```

Guide-5 `RelationActivity` may be reused.

Inactive is an explicit valid disposition, never omission.

## 5.3 Coverage-plan shape

A suitable internal value is:

```rust
struct RelationCoveragePlan {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    activity: RelationActivity,
    boundaries: BTreeSet<CoverageBoundary>,

    positive: Vec<PositiveCoverageRequirement>,
    negative: Vec<NegativeCoverageRequirement>,

    carrier: Option<CarrierCoverageRequirement>,
    accepted_projection:
        Option<SemanticProjectionRequirement>,

    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,
}
```

Exact field names remain implementation-owned.

The type must support hybrid relations. Representation and lifecycle relations
can produce both compiler-static and backend-structural requirements.

## 5.4 Operation-local aggregation

Coverage should be stored and validated per operation.

Guide 5 proved that combined placement factorizes across operations. Guide 6
must not create another cross-operation Cartesian product.

A suitable outer structure is:

```rust
struct OperationCoverageAnalysis {
    operation: architecture::OperationId,
    cases: BTreeSet<ExecutionCaseId>,
    requirements:
        BTreeMap<RelationCaseKey, RelationCoveragePlan>,
}
```

A proof-plan coverage result may contain one operation analysis per operation.

This does not yet require refactoring all Guide-5 placement storage. Coverage
may project and deduplicate per-operation assignments from the existing placed
candidate.

Record for Guide 7 that the complete analyzed-program value should store
placement per operation rather than materialize the cross-operation product.

---

# 6. Tranche C — Derive positive, negative, and inactive requirements

## 6.1 Active runtime relation

An active runtime relation requires:

1. valid target acceptance;
2. selected carrier assignment execution;
3. relation verdict success;
4. accepted semantic projection comparison;
5. at least one focused rejecting mutation;
6. expected rejection at the runtime target boundary;
7. dependency-collateral reporting.

This is a future evidence requirement only. Guide 6 does not construct a target
transaction.

## 6.2 Compiler-static relation

A compiler-static relation requires:

- positive: valid typed declaration/selection accepted;
- negative: invalid or unsupported typed declaration/selection rejected;
- no runtime carrier;
- no target execution claim.

Examples:

```text
constructibility:
    required availability is dischargeable / unavailable witness rejects

representation:
    selected mode is allowed / unsupported mode rejects
```

## 6.3 Backend-structural relation

A backend-structural relation requires:

- positive: required structural property is present in emitted bundle or ABI;
- negative: missing or contradictory structural property rejects;
- no claim that compiler analysis alone implemented it.

Examples:

```text
permissionless authorization:
    no hidden owner/operator secret gate

representation:
    selected representation encoded and authenticated

lifecycle:
    required exit remains present and reachable
```

## 6.4 External-evidence relation

An external-evidence relation requires:

- exact typed evidence role;
- exact relation subject;
- exact operation and asset;
- required capability;
- positive verified report;
- negative missing/failed/stale/mismatched report;
- no runtime carrier;
- no protocol amount reconstruction from erased sponsor values.

For substrate conservation:

```text
capability:
    WholeTransactionValueConservation

evidence:
    remains unresolved until target/deployment report exists
```

## 6.5 Inactive-valid relation

An inactive relation-case requires:

- valid operation/case accepted;
- relation not activated;
- no runtime carrier executed for that relation;
- no private witness demanded by the inactive branch;
- no false specialized projection emitted.

Conditional relation coverage across the complete case set must include:

```text
inactive valid
active valid
active invalid
```

---

# 7. Tranche D — Typed mutation catalogue

## 7.1 Mutation requirements are semantic classes

Guide 6 defines required mutation classes. It does not mutate model worlds or
target transactions.

Use an exhaustive match over `realization::Relation`, with no wildcard arm.

A suitable typed vocabulary is:

```rust
enum RelationMutation {
    CardinalityBelowMinimum,
    CardinalityAboveMaximum,

    UndeclaredObjectFamily,
    WrongRecognizedAsset,
    WrongRecognizedObject,

    AmountMismatch,
    MissingRequiredOwner,
    UnexpectedProtocolSecret,

    SponsorProtocolOverlap,
    MissingSponsorAuthorization,
    SponsorEnvelopeMultiplicityExceeded,

    WrongRootEffect,
    MissingRequiredProjection,
    ForbiddenProjectionPresent,

    MissingCanonicalDeltaFamily,
    UnexpectedCanonicalDeltaFamily,
    DuplicateCanonicalSourceOrDestination,

    UndeclaredOpenFlow,

    ConstructibilityWitnessUnavailable,
    PermissionlessPrivateDependency,

    UnsupportedRepresentation,
    UnauthenticatedRepresentation,

    RequiredLifecycleExitMissing,

    ExpressionPredicateFalse,

    ExternalEvidenceMissing,
    ExternalEvidenceFailed,
    ExternalEvidenceIdentityMismatch,
}
```

Exact names may differ.

Do not create mutation classes that cannot be materialized later.

## 7.2 Relation-specific mapping

The minimum mapping is:

| Relation | Required negative class |
|---|---|
| cardinality | below minimum when meaningful; above maximum |
| allowed families | undeclared object family |
| recognition | wrong asset and/or wrong object shape |
| amount conservation | one-unit or exact amount mismatch |
| owner authorization | omit one required owner |
| permissionless authorization | add hidden protocol secret gate |
| sponsor isolation | overlap sponsor/protocol membership or omit sponsor authorization |
| sponsor multiplicity | exceed declared maximum |
| root policy | wrong root use/effect |
| projection policy | missing required or forbidden present |
| canonical delta policy | missing/extra family; duplicate partition member where applicable |
| open-flow policy | undeclared open-flow kind |
| constructibility | unavailable required witness or private dependency |
| representation | unsupported mode; unauthenticated encoding |
| lifecycle exit | required exit missing |
| expression predicate | false predicate |
| substrate conservation | missing, failed, or identity-mismatched external evidence |

Where a relation has several independent boundary failures, retain several
negative requirements.

## 7.3 Cardinality edge cases

A minimum of zero has no “below minimum” mutation.

Derive:

```text
minimum > 0:
    below-minimum requirement

all finite maxima:
    above-maximum requirement
```

For architecture-bound maxima, the later vector materializer uses the exact
bound value plus one. Guide 6 records the bound-owned relation; it does not
copy a draft runtime value.

## 7.4 Focused does not mean isolated

A mutation is focused when it names one intended relation and changes the
smallest known semantic fact set.

It may still cause dependent relations to fail or block.

Do not claim strict independence unless proved.

Each negative requirement should record:

```text
intended relation
mutation class
dependency-collateral set
actual additional collateral must be reported later
```

A suitable policy value is:

```rust
enum CollateralPolicy {
    RequireIntendedAndDependencyClosure,
    ReportAdditional,
}
```

or equivalent explicit fields.

---

# 8. Tranche E — Dependency collateral and typed coverage graph

## 8.1 Coverage dependency graph

Build a direct Petgraph graph over stable coverage symbols.

Graph-shaped semantics in this tranche include:

- relation-case dependency;
- positive requirement dependency;
- negative collateral dependency;
- carrier requirement dependency;
- accepted projection dependency;
- external-evidence dependency.

Use a concrete Petgraph type directly:

```rust
petgraph::graph::DiGraph<
    CoverageNode,
    CoverageEdge,
    u32,
>
```

A domain-specific analysis struct containing this graph and typed lookup maps is
permitted.

Do not introduce a generic graph wrapper.

## 8.2 Stable node identities

A useful node identity vocabulary is:

```rust
enum CoverageNodeId {
    RelationCase(RelationCaseKey),

    Requirement(CoverageRequirementId),

    Carrier {
        relation_case: RelationCaseKey,
        carrier: PlacedCarrier,
    },

    Layout(LayoutRequirement),

    ExternalEvidence(
        ExternalEvidenceRequirement,
    ),
}
```

Complete typed values may be used directly where they already provide stable
identity.

No string symbol names are needed.

## 8.3 Edge vocabulary

Possible typed edges:

```rust
enum CoverageEdge {
    RelationPrerequisite,
    RequiresCarrier,
    RequiresLayout,
    RequiresProjection,
    RequiresExternalEvidence,
    DependencyCollateral,
}
```

Every edge endpoint is resolved through the typed definition census.

## 8.4 Two-pass typed symbol resolution

Implement C1-010 in a narrow compiler-coverage scope.

### Pass 1 — definition census

Collect and validate definitions for:

- in-scope relations;
- execution cases;
- relation-case entries;
- carrier alternatives;
- layout requirements;
- external-evidence requirements;
- coverage requirements.

Reject duplicate stable keys.

### Pass 2 — reference resolution

Resolve every coverage reference against the complete census.

Reject:

- missing relation;
- missing case;
- missing relation-case entry;
- missing carrier alternative;
- missing layout requirement;
- missing external-evidence requirement;
- cross-operation dependency;
- malformed boundary reference.

Do not resolve by display string or insertion order.

## 8.5 Dependency collateral closure

For a negative mutation of relation \(r\) in case \(c\), derive the relations
that must become blocked because they depend transitively on \(r\).

With relation dependency edge:

```text
prerequisite → dependent
```

the required dependency collateral is the strict descendant closure of \(r\)
within the same operation and execution case.

Conceptually:

\[
\operatorname{Collateral}(r,c)=\{d\mid r\leadsto d,\ d\ne r,\ \operatorname{Active}(d,c)\}
\]

Inactive dependents need not be claimed as blocked.

The later vector report may contain additional collateral caused by a concrete
mutation. It must always include the intended relation and the typed dependency
closure.

## 8.6 SCC policy

Coverage dependencies are expected to be acyclic in Phase 2.

Use Petgraph SCC analysis to detect:

- self-loops;
- multi-node cycles.

Normalize:

- SCC members by stable `CoverageNodeId`;
- SCC collection lexicographically;
- internal edges by typed source, target, and edge role.

Policy:

```text
current coverage cycle strategy:
    forbidden
```

Finding an SCC does not authorize it.

A future accepted cycle requires a new typed resolution strategy and policy
review. Do not add a generic “allowed cycle” escape.

## 8.7 Stable graph projection

Project to:

```text
sorted stable node IDs
sorted typed edges
```

Exclude:

- `NodeIndex`;
- `EdgeIndex`;
- graph insertion order;
- SCC library return order;
- topological evaluation order unless a consumer requires a canonical schedule.

---

# 9. Tranche F — Carrier coverage requirements

## 9.1 Do not bind coverage to the combined placement product

Guide 5’s multi-operation result factorizes.

Guide 6 should compress feasible placement information per relation-case:

```rust
struct CarrierCoverageRequirement {
    relation_case: RelationCaseKey,

    allowed_assignments:
        BTreeSet<BTreeSet<PlacedCarrier>>,
}
```

Equivalent canonical vectors are acceptable.

The requirement means:

1. a later target plan selects one allowed assignment;
2. every carrier in the selected assignment must be reachable and execute in
   the case;
3. no carrier outside an allowed assignment may be used to claim discharge;
4. evidence binds the selected assignment.

Do not require target evidence for every unselected feasible alternative.

## 9.2 Derive assignment alternatives exactly

For each relation-case:

1. inspect all feasible placements of the proof plan;
2. extract that relation-case’s selected carrier set;
3. canonicalize it;
4. deduplicate it;
5. retain the complete set of feasible assignment alternatives.

This avoids duplicating the complete placement product in coverage output.

If the same per-relation assignments appear in 7,776 combined placements, they
must still appear once in the coverage requirement.

## 9.3 Multiplicity preservation

Coverage must preserve:

- exactly one carrier;
- every-member quantified carrier;
- complete-family proof;
- deliberate duplication;
- other Guide-5 multiplicity semantics.

A future report must not satisfy:

```text
EveryMember
```

by showing one arbitrary member executed.

A complete-family proof remains distinct from per-member execution.

## 9.4 No runtime carrier for non-runtime relations

Coverage validation must reject a runtime carrier requirement for:

- compiler-static relation;
- backend-structural-only relation;
- external-evidence relation;
- inactive relation-case.

This is a focused negative test.

---

# 10. Tranche G — Accepted semantic projection requirements

## 10.1 Compare semantic facts, not only verdicts

A target accepting a transaction is insufficient.

For each active runtime relation, derive an accepted semantic projection
requirement that includes:

- relation ID;
- execution case;
- active operands;
- active source requirements;
- expected relation verdict;
- applicable public semantic projection components.

Prefer reusing:

```rust
OperandId
SourceRequirement
```

rather than creating a second fact vocabulary.

A suitable shape is:

```rust
struct SemanticProjectionRequirement {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    operands: Vec<OperandId>,
    sources: Vec<SourceRequirement>,
    compare_relation_verdict: bool,
}
```

## 10.2 Sponsor erasure

Projection requirements must never include:

- sponsor input amount;
- sponsor output amount;
- sponsor positivity;
- public sponsor total;
- sponsor opening or blinding factor.

They may include:

- sponsor family membership;
- sponsor owner authorization;
- sponsor/protocol disjointness;
- envelope multiplicity;
- external whole-transaction-conservation evidence.

Reuse the existing sponsor-value guard and add a coverage-specific traversal
assertion.

## 10.3 Boundary-specific projections

For compiler-static coverage:

- compare compiler selection/validation result;
- do not require target projection.

For backend-structural coverage:

- compare bundle/ABI structural fact;
- do not call it target execution.

For external evidence:

- compare evidence role and exact subject identity later;
- do not reconstruct sponsor values.

---

# 11. Tranche H — Exact coverage census validation

## 11.1 Relation scope equality

For each proof-plan candidate, require exact equality among:

```text
realization relation scope
compiler relation scope
relation-case plan scope
coverage relation-case scope
```

The expected set is:

```text
every in-scope relation
×
every applicable execution case of its operation
```

No relation disappears due to its boundary or inactivity.

## 11.2 Placement scope equality

Require:

```text
active runtime relation-case keys
=
carrier coverage relation-case keys
```

Compiler-static, backend-structural, external, and inactive cases must have no
carrier coverage key.

## 11.3 Layout scope coverage

Every layout requirement required by a selectable carrier assignment must be
referenced by at least one coverage requirement.

Unexpected layout references fail.

The coverage analysis does not require all eligible-carrier layout
requirements to execute simultaneously. It requires the selected carrier
assignment’s layout requirements.

## 11.4 Positive and negative coverage

Every active relation-case has:

- at least one positive requirement at each applicable boundary;
- at least one negative requirement at each applicable boundary;
- accepted projection requirement when runtime-target acceptance is claimed;
- carrier requirement when runtime-carried.

Every inactive relation-case has:

- one inactive-valid requirement;
- no runtime carrier;
- no active target-reject mutation requirement in that inactive case.

## 11.5 Conditional triplet validation

Across the complete applicable case set, every conditional relation must have:

```text
inactive valid
active valid
active invalid
```

For current sponsor-conditional relations:

```text
sponsor absent:
    inactive valid

sponsor present:
    active valid
    active invalid
```

Representation-specific coverage is distributed across proof-plan candidates,
because each candidate fixes one mode.

## 11.6 Representation census

Across the complete feasible plan set:

```text
compact ASH:
    Explicit
    PublicCommitted

live transfer:
    Explicit
    PrivateCommitted
```

must each have positive and negative coverage requirements at their applicable
boundaries.

Do not create impossible cross-mode cases inside one proof plan.

---

# 12. Tranche I — Independent coverage oracle

## 12.1 Independence requirement

The coverage oracle must not call production:

- coverage derivation;
- coverage census validator;
- mutation mapping;
- dependency-collateral closure;
- coverage symbol resolver;
- carrier-alternative compression.

It may share stable typed values and Guide-5 input analyses.

## 12.2 Direct relation × case oracle

For each proof-plan candidate:

1. derive relation IDs directly from the compiler relation graph;
2. derive case IDs directly from the candidate and typed sponsor activation;
3. form the expected relation × applicable-case set;
4. classify active/inactive independently;
5. restate expected coverage boundary rules;
6. derive direct expected positive/negative requirement classes;
7. compare with production coverage.

Do not infer the expected census from production coverage output.

## 12.3 Independent collateral oracle

Implement a slower repeated-complete-scan closure over stable relation IDs:

```text
start with intended relation
repeatedly add every dependent whose prerequisite is present
stop when unchanged
```

Compare with the production Petgraph descendant closure.

This satisfies the independent closure requirement without duplicating the
production traversal.

## 12.4 Independent SCC oracle

For generated small graphs, determine SCC equivalence through mutual
reachability:

\[
u\sim v\iff u\leadsto v\land v\leadsto u
\]

Compare normalized components with Petgraph SCC output.

The oracle is test-only and need not scale.

## 12.5 Required oracle cases

Cover:

- complete pilot relation × case matrix;
- missing coverage entry;
- duplicate coverage entry;
- unexpected coverage entry;
- active relation missing positive requirement;
- active relation missing negative requirement;
- inactive relation with runtime carrier;
- runtime relation missing carrier requirement;
- external relation assigned runtime carrier;
- selected carrier missing layout reference;
- sponsor amount introduced into coverage;
- conditional relation missing inactive valid case;
- conditional relation missing active invalid case;
- representation mode omitted from complete plan set;
- relation dependency chain;
- relation dependency diamond;
- self-loop;
- multi-node SCC;
- cross-operation edge;
- declaration permutations;
- case permutations;
- placement permutations;
- repeated analysis equality.

## 12.6 Property tests

Generate bounded synthetic instances with:

- 1–8 relations;
- 1–4 cases;
- acyclic dependency graphs;
- optional sponsor activations;
- mixed discharge boundaries;
- mixed carrier multiplicities;
- small placement alternatives.

Compare:

```text
production coverage projection
=
independent oracle projection
```

For cyclic generated graphs, both should reject with canonical SCC
diagnostics.

Use explicit small case counts. Do not turn this into an unbounded test-time
search.

---

# 13. Recommended source organization

Add internal modules such as:

```text
packages/compiler/src/coverage.rs
packages/compiler/src/coverage_graph.rs
```

or:

```text
packages/compiler/src/coverage.rs
packages/compiler/src/symbol.rs
```

A reasonable split is:

## `coverage.rs`

- coverage identities and requirement values;
- relation-to-mutation mapping;
- positive/negative/inactive derivation;
- carrier-alternative compression;
- accepted projection requirements;
- exact coverage census validation;
- per-operation coverage analysis.

## `coverage_graph.rs` or `symbol.rs`

- typed definition census;
- typed reference resolution;
- direct Petgraph graph;
- dependency-collateral closure;
- SCC normalization and rejection;
- stable graph projection.

Keep both modules crate-private.

Add tests:

```text
packages/compiler/src/tests/coverage_tests.rs
packages/compiler/src/tests/coverage_oracle_tests.rs
```

Update:

```text
packages/compiler/src/tests/mod.rs
packages/compiler/meson.build
```

Every new Rust source must join the explicit Meson census in the same commit.

---

# 14. Error vocabulary

Add only errors reached by real branches. Likely variants include:

```rust
DuplicateCoverageRequirement {
    requirement: CoverageRequirementId,
}

CoverageCensusMismatch {
    missing: Vec<RelationCaseKey>,
    unexpected: Vec<RelationCaseKey>,
}

MissingPositiveCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    boundary: CoverageBoundary,
}

MissingNegativeCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
    boundary: CoverageBoundary,
}

MissingInactiveCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

UnexpectedRuntimeCarrierCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

MissingCarrierCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

MissingProjectionCoverage {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

MissingCoverageLayoutRequirement {
    relation: realization::RelationId,
    case: ExecutionCaseId,
}

UnknownCoverageSymbol {
    symbol: CoverageNodeId,
}

DuplicateCoverageDependency {
    source: CoverageNodeId,
    target: CoverageNodeId,
    edge: CoverageEdge,
}

CoverageDependencyCycle {
    components: Vec<Vec<CoverageNodeId>>,
}

CrossOperationCoverageDependency {
    source: CoverageNodeId,
    target: CoverageNodeId,
}
```

Reuse `SponsorValueRead` if a coverage requirement names an erased sponsor
amount.

Do not expose target-specific error variants.

---

# 15. Real-pilot acceptance matrix

## 15.1 Compact ASH

Coverage must include:

| Relation | Cases | Boundary |
|---|---|---|
| ASH input cardinality | all compact-ASH cases | runtime |
| ASH output cardinality | all compact-ASH cases | runtime |
| sponsor input cardinality | active when sponsored; inactive valid otherwise | runtime |
| sponsor output cardinality | active when sponsored; inactive valid otherwise | runtime |
| ASH input recognition | all | runtime |
| ASH output recognition | all | runtime |
| sponsor recognition | active when sponsored; inactive valid otherwise | runtime |
| input/output closure | all | runtime |
| ownerless `U` conservation | all | runtime |
| canonical delta policy | all | runtime |
| sponsor isolation | all; absent case proves no sponsor region | runtime |
| sponsor envelope multiplicity | all | runtime |
| open-flow policy | all | runtime |
| no-root policy | all | runtime |
| transition-certificate-only projection | all | runtime |
| public permissionless constructibility | all | compiler-static |
| secret-free permissionless path | all | backend-structural |
| representation selection | per plan-fixed mode | compiler-static |
| representation encoding | per plan-fixed mode | backend-structural |
| compact and clear lifecycle exits | all | compiler-static plus backend-structural |
| L-BTC substrate conservation | all | external evidence |

Every active runtime relation receives:

- active acceptance;
- focused rejection;
- carrier requirement;
- accepted projection requirement.

## 15.2 Live transfer

Coverage must include:

| Relation | Cases | Boundary |
|---|---|---|
| live input/output cardinality | all live-transfer cases | runtime |
| sponsor input/output cardinality | active sponsored; inactive valid otherwise | runtime |
| live input/output recognition | all | runtime |
| sponsor recognition | active sponsored; inactive valid otherwise | runtime |
| every-owner authorization | all | runtime, every member |
| input/output closure | all | runtime |
| aggregate `U` conservation | all | runtime |
| canonical delta policy | all | runtime |
| sponsor isolation/multiplicity | all | runtime |
| open-flow policy | all | runtime |
| no-root policy | all | runtime |
| transition-certificate-only projection | all | runtime |
| owner constructibility | all | compiler-static |
| representation selection | explicit or private, fixed by plan | compiler-static |
| representation encoding | matching selected mode | backend-structural |
| transfer/burn/redeem lifecycle exits | all | compiler-static plus backend-structural |
| L-BTC substrate conservation | all | external evidence |

Owner authorization positive coverage must require:

```text
every consumed live-receipt owner
```

A single coordinator execution cannot satisfy it unless the selected carrier is
typed as a complete all-owner proof, which the current eligibility does not
permit.

## 15.3 Sponsor opacity

For both pilots, assert over the complete coverage projection:

```text
no OperandRole::ObjectFamilyAmount {
    object: ObjectId::PlainLbtc,
    ...
}

no source requirement:
    AuthenticatedConsensusValue for sponsor amount

no mutation:
    sponsor positivity

no projection:
    public sponsor total
```

---

# 16. Stable projection and determinism

## 16.1 Projection contents

The stable coverage projection may contain:

- proof-plan typed value as outer map key;
- operation ID;
- execution-case ID;
- relation ID;
- coverage boundary;
- requirement purpose;
- mutation class;
- carrier assignment alternatives;
- layout requirements;
- operand/source projection checks;
- external evidence role;
- dependency collateral;
- stable coverage graph nodes and edges.

## 16.2 Projection exclusions

Exclude:

- Petgraph indices;
- search-state counts;
- full combined placement product order;
- test fixture IDs not yet owned by vectors;
- target program IDs;
- target transaction positions;
- timestamps;
- file paths;
- source lines;
- process-local diagnostics;
- hashes.

## 16.3 Determinism tests

Test equality under:

- relation declaration permutation;
- relation dependency permutation;
- proof-plan candidate permutation;
- execution-case permutation;
- placement candidate permutation;
- carrier order permutation;
- layout requirement permutation;
- graph insertion permutation;
- repeated clean analysis.

The projected output must remain equal.

---

# 17. Complexity policy

Coverage derivation should be polynomial over existing finite analyses.

Expected main costs:

```text
relation × case census:
    O(R·C)

carrier-alternative compression:
    O(P·R·C·log A)

dependency closure:
    O((V+E) per relation) initially,
    or one reusable transitive analysis if justified

SCC:
    O(V+E)
```

where:

- \(R\) = relations;
- \(C\) = cases;
- \(P\) = feasible placements;
- \(A\) = distinct carrier assignments.

Guide 6 should avoid introducing a new exponential search.

Scanning existing feasible placements is acceptable for the pilots. Coverage
output must deduplicate relation-case carrier assignments so combined placement
products do not propagate into the stable coverage value.

If a practical limit is needed for synthetic or future inputs, add an explicit
typed limit and a typed failure. Do not silently truncate coverage.

---

# 18. Suggested implementation waves

## Wave 0 — Sponsor-cardinality ruling and realization repair

Deliver:

- compact-ASH sponsor input/output cardinality relations;
- dependencies;
- generic architecture-family relation census validation;
- focused realization tests;
- compiler tests updated for the new complete relation set.

Suggested commit:

```text
realization: complete compact-ash sponsor cardinality
```

## Wave 1 — Coverage types and relation matrix

Deliver:

- coverage IDs;
- boundary and activity types;
- per-relation exhaustive coverage derivation;
- typed mutation catalogue;
- exact relation-case coverage census;
- active/negative/inactive tests.

Suggested commit:

```text
compiler: derive relation-indexed coverage requirements
```

## Wave 2 — Carrier and projection coverage

Deliver:

- compressed carrier assignment alternatives;
- selected-carrier execution requirements;
- layout references;
- accepted semantic-projection requirements;
- sponsor-opacity traversal;
- runtime/non-runtime carrier separation tests.

Suggested commit:

```text
compiler: bind coverage to placements and projections
```

## Wave 3 — Typed coverage graph and SCC policy

Deliver:

- two-pass coverage definition/reference resolution;
- direct Petgraph graph;
- dependency collateral closure;
- SCC normalization;
- cycle rejection;
- stable graph projection;
- synthetic graph tests.

Suggested commit:

```text
compiler: resolve typed coverage dependencies
```

## Wave 4 — Independent coverage oracle

Deliver:

- direct relation × case oracle;
- repeated-scan collateral oracle;
- mutual-reachability SCC oracle;
- adversarial synthetic instances;
- property-generated small instances;
- complete production/oracle projection equality.

Suggested commit:

```text
compiler: add independent coverage oracle
```

## Wave 5 — Pilot integration

Deliver:

- complete compact-ASH coverage analysis;
- complete live-transfer coverage analysis;
- complete feasible-plan-set validation;
- per-operation factorization;
- stable projection determinism;
- no sponsor value reads;
- exact scope equalities.

Suggested commit:

```text
compiler: analyze pilot coverage end to end
```

## Wave 6 — Documentation and gate

Deliver:

- compiler Rustdoc updated;
- compiler package contract updated;
- Phase-2 card updated;
- backlog gate record;
- P2-011 DONE;
- C1-010 DONE;
- C1-013 DONE;
- P2-012 next TODO.

Suggested commit:

```text
plans: record relation-indexed coverage gate
```

Workers should commit each green tranche immediately. While delegated workers
are active, keep the standing heartbeat loop armed; stop it at wave boundaries.

---

# 19. Focused verification

## 19.1 Wave 0

```sh
cargo test --locked -p tripod-realization compact_ash
cargo test --locked -p tripod-compiler case
cargo test --locked -p tripod-compiler placement
```

## 19.2 Coverage derivation

```sh
cargo test --locked -p tripod-compiler coverage
```

## 19.3 Coverage graph and SCC

```sh
cargo test --locked -p tripod-compiler coverage_graph
cargo test --locked -p tripod-compiler scc
```

Use actual test filters produced by the test names; do not rely on a filter
that matches nothing.

## 19.4 Oracle

```sh
cargo test --locked -p tripod-compiler coverage_oracle
cargo test --locked -p tripod-compiler oracle
```

## 19.5 Complete compiler package

```sh
cargo test --locked -p tripod-compiler
cargo doc --locked -p tripod-compiler --no-deps
```

---

# 20. Working cadence

After each coherent implementation wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Do not run both the debug Cargo lane and the full Meson lane after every
individual edit.

Read `git status` after formatting.

Every new Rust source must be added to:

```text
packages/compiler/meson.build
```

in the same commit.

---

# 21. Generated artifacts and labels

No architecture or model-generated publication is expected to change.

Run:

```sh
meson compile -C build lint
```

If `build/` does not exist:

```sh
meson setup build
```

Do not create another production build directory.

Expected identity/publication impact:

```text
architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

architecture JSON/TOML:
    unchanged

model declassification:
    unchanged

model labels:
    unchanged unless new acute Rust labels are deliberately minted

realization public identity:
    none exists

compiler-plan identity:
    none exists
```

The realization relation set changes internally by adding missing compact-ASH
sponsor cardinality relations. No public realization hash exists, and no new
one should be minted.

If generated architecture bytes move, stop and investigate.

---

# 22. Full batch gate

After all Guide-6 waves are complete:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Record skipped lanes honestly.

In particular:

```text
cargo-audit unavailable:
    SKIPPED, not passed

document reproducibility not run:
    DEFERRED, not passed
```

Paper inputs should remain unchanged, so document byte reproducibility may be
deferred for this batch unless release policy requires it.

Finally:

```sh
git diff --check
git diff --cached --check
git status --porcelain=v1 --untracked-files=all
```

The final status must be empty after commits.

---

# 23. Documentation and backlog update

Update status only after the recorded verification has actually run.

## 23.1 Compiler Rustdoc

Update:

```text
packages/compiler/src/lib.rs
packages/compiler/README.md
```

State that these are implemented internally:

- cases;
- discharge classification;
- carrier eligibility;
- exact placement;
- layout requirements;
- relation-indexed coverage;
- typed coverage dependencies;
- independent placement and coverage oracles.

Keep absent:

- complete pilot analyzed-program value;
- public complete-analysis result;
- target adapter;
- target emission;
- compiler identity.

## 23.2 Compiler package contract

Update:

```text
plans/packages/compiler.md
```

Mark:

```text
milestone:compiler:coverage
    Implemented internally
```

Keep:

```text
milestone:compiler:pilots
    Open
```

The next issue is assembling the complete analyzed pilot value, not deriving
more coverage.

## 23.3 Phase card

Update:

```text
plans/phases/02-compiler.md
```

Move coverage from open deliverable to implemented internal foundation.

Keep the Phase-2 exit gate open until P2-012 and P2-013 pass.

## 23.4 Backlog

Add a compact Guide-6 gate record after completion.

Expected status changes:

```text
P2-011:
    TODO → DONE

C1-010:
    TODO → DONE

C1-013:
    TODO → DONE

P2-012:
    BLOCKED → TODO
```

Record the compact-ASH sponsor-cardinality ruling:

```text
result:
    omission confirmed and repaired

architecture identity:
    unchanged

realization relation census:
    expanded to match architecture input/output cardinality

compiler coverage:
    now includes sponsor cardinality for compact ASH
```

Do not rewrite the historical Guide-5 gate measurements.

---

# 24. Required exit assertions

Guide 6 is complete only when all assertions below hold.

## Sponsor cardinality

- [ ] compact ASH declares sponsor input cardinality;
- [ ] compact ASH declares sponsor output cardinality;
- [ ] both relations match architecture values exactly;
- [ ] missing family cardinality fails realization validation;
- [ ] sponsorless cases are inactive-valid;
- [ ] sponsored cases are active;
- [ ] no sponsor amount enters analysis.

## Coverage census

- [ ] every relation appears in every applicable case;
- [ ] no duplicate relation-case coverage entry exists;
- [ ] active relations have positive and negative requirements;
- [ ] inactive relations have inactive-valid requirements;
- [ ] runtime relations have carrier coverage;
- [ ] non-runtime relations have no runtime carrier;
- [ ] runtime accepted cases have semantic projection requirements;
- [ ] external evidence remains external.

## Conditional coverage

- [ ] each conditional relation has inactive valid coverage;
- [ ] each conditional relation has active valid coverage;
- [ ] each conditional relation has active invalid coverage;
- [ ] representation modes are covered across the complete proof-plan set;
- [ ] impossible representation combinations are not manufactured.

## Mutation and collateral

- [ ] mutation mapping is exhaustive over relation variants;
- [ ] cardinality minimum-zero cases do not generate impossible below-minimum
      mutations;
- [ ] dependency collateral uses exact active descendant closure;
- [ ] later additional collateral is required to be reported;
- [ ] no strict-independence claim is made without proof.

## Carrier and layout

- [ ] selectable carrier assignment alternatives are exact;
- [ ] coverage does not duplicate the combined placement product;
- [ ] every selected carrier’s layout dependencies are referenced;
- [ ] every-member multiplicity remains quantified;
- [ ] complete-family proof remains distinct from per-member execution;
- [ ] optional sponsor carriers never carry unconditional relations alone.

## Typed symbols and SCCs

- [ ] definition census precedes reference resolution;
- [ ] duplicate symbols fail;
- [ ] unknown references fail;
- [ ] cross-operation references fail;
- [ ] direct Petgraph graph is used;
- [ ] local graph handles stay private;
- [ ] SCC members and diagnostics are canonical;
- [ ] every current cycle is rejected;
- [ ] SCC membership never authorizes a cycle.

## Oracle

- [ ] direct relation × case oracle agrees with production;
- [ ] repeated-scan collateral oracle agrees with production;
- [ ] mutual-reachability SCC oracle agrees with Petgraph;
- [ ] adversarial synthetic cases pass;
- [ ] property-generated small instances agree;
- [ ] insertion and case permutations preserve projections.

## Project boundary

- [ ] no target opcode enters compiler core;
- [ ] no transaction position enters compiler core;
- [ ] no tapleaf, stack slot, or witness position enters compiler core;
- [ ] no target execution is claimed;
- [ ] no concrete vector is generated;
- [ ] no new digest is minted;
- [ ] no new dependency is added;
- [ ] no generated architecture publication changes;
- [ ] full gates pass or skipped/deferred lanes are reported honestly;
- [ ] final tree is clean.

---

# 25. Completion report template

Use this exact structure when Guide 6 is complete:

```text
Guide 6 result
==============

Sponsor-cardinality ruling:
    finding:
    ruling:
    repair:
    focused tests:

Coverage model:
    relation-case census:
    positive requirements:
    negative requirements:
    inactive requirements:
    carrier requirements:
    projection requirements:

Typed symbols / SCC:
    definition census:
    reference resolution:
    graph:
    cycle policy:
    diagnostics:

Independent oracle:
    relation × case oracle:
    collateral oracle:
    SCC oracle:
    generated cases:

Pilot results:
    compact ASH:
    live transfer:
    combined scope:
    sponsor opacity:

Planning result:
    P2-011:
    C1-010:
    C1-013:
    next task:

Identity impact:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:

Dependency impact:
    new dependencies:
    lockfile:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Residuals:
```

---

# 26. Next guide after Guide 6

Once Guide 6 passes, the next guide should be:

```text
Guide 7 — Complete Pilot Analyzed Programs and Phase-2 Exit
```

Its scope should be:

```text
P2-012
P2-013
C1-014
```

Guide 7 should:

- assemble per-operation proof, case, placement, layout, target-requirement, and
  coverage analyses;
- store placement per operation rather than as a materialized cross-operation
  Cartesian product;
- define one complete crate-private analyzed-program value;
- validate compact ASH and live transfer end to end;
- decide the smallest honest public boundary, if any;
- keep target-specific values absent;
- retain lifecycle incompleteness explicitly;
- mint no compiler digest without a real consumer;
- run and record the complete Phase-2 exit gate.

Do not expose a partial public analyzed-program value before Guide 7 proves its
complete census and ownership invariants.
