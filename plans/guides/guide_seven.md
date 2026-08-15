# Guide 7 — Complete Scoped Analyzed Programs and Phase-2 Exit

> **Status:** Implementation guide
> **Primary tasks:** `P2-012`, `P2-013`, `C1-014`
> **Entry evidence:** completed Guide-6 coverage gate
> **Current phase:** Phase 2 — target-independent compiler analysis
> **Next phase:** Phase 3 — Elements target and foundational prototypes
> **Primary package:** `tripod-compiler`
> **Supporting packages:** `tripod-realization`, `tripod-architecture`, `tripod-model`, `tripod-labels`
> **No new dependency expected**

---

## 1. Mission

Guide 7 completes the Phase-2 compiler boundary by assembling the existing internal analyses into one validated, deterministic, target-independent value that is complete for its explicit compiler scope.

The production pipeline is:

```text
validated compiler input
    ↓
relation/expression foundation
    ↓
exact unconstrained feasible proof-plan set
    ↓
relation-indexed capabilities, sources, and evidence
    ↓
execution cases
    ↓
relation-case discharge plans
    ↓
operation-local carrier eligibility
    ↓
operation-local exact feasible placements
    ↓
operation-local layout requirements
    ↓
operation-local relation-indexed coverage
    ↓
complete scoped analyzed program
```

Guide 7 must complete:

```text
P2-012  compact-ASH and live-transfer analyzed pilots
P2-013  Phase-2 evidence and exit
C1-014  compiler-preparation review and Phase-3 handoff
```

The central implementation task is **typed assembly and cross-stage closure**. Guide 7 should not add a new optimizer, graph abstraction, target package, backend, identity, or publication.

The central storage ruling is:

> Proof plans remain global typed candidates. Placement, layout, and coverage are stored per operation. The analyzed program must not materialize the Cartesian product of independent operation placements.

For proof plan \(p\) and operation set \(O\), the complete placement semantics remain:

\[\mathcal P(p,O)=\prod_{o\in O}\mathcal P(p,o)\]

but the analyzed value stores the factors \(\mathcal P(p,o)\), not every member of that product.

After Guide 6, each pilot currently has 216 placements per proof plan. Materializing the combined two-operation product would therefore produce:

\[216\times216=46{,}656\]

combined placements per proof plan. Guide 7 must preserve the exact product semantics without storing or routinely enumerating those 46,656 combinations.

---

# 2. Guide-6 entry record

Guide 7 starts from the following accepted Guide-6 result.

## 2.1 Sponsor-cardinality ruling

The compact-ASH realization omission was confirmed and repaired.

The operation now declares:

```text
PLAIN_LBTC input cardinality:
    minimum 0
    maximum FEE_SPONSOR_INPUT_MAX

PLAIN_LBTC output cardinality:
    minimum 0
    maximum exactly 1
```

Recognition-before-cardinality dependencies are present, and realization validation derives the expected family-relation census from architecture operation rows.

Both pilots now expose the same optional sponsor-family shape.

## 2.2 Coverage state

Per proof plan:

```text
compact ASH:
    46 relation-cases

live transfer:
    48 relation-cases
```

The accepted scope equality is:

```text
realization relations
=
compiler relations
=
relation-case plans
=
coverage relation-cases
```

Coverage includes:

- positive requirements;
- focused negative requirements;
- exactly four inactive sponsor relation-cases in the unsponsored case;
- compressed carrier assignment alternatives;
- boundary-specific accepted projections;
- typed dependency collateral;
- separate external-evidence obligations.

## 2.3 Typed coverage graph

Guide 6 delivered:

- two-pass symbol census and resolution;
- duplicate-symbol rejection;
- unknown-symbol rejection;
- cross-operation dependency rejection;
- direct Petgraph `DiGraph` storage;
- canonical typed graph projection;
- canonical SCC diagnostics;
- a forbidden-cycle policy with no generic escape.

## 2.4 Independent oracle

Guide 6 delivered independent checks for:

- relation × case coverage;
- dependency closure by repeated complete scan;
- SCCs by mutual reachability;
- bounded generated instances;
- declaration and insertion-order determinism.

## 2.5 Residuals carried into Guide 7

Guide 7 must account for these recorded residuals:

1. the combined-scope coverage regression adds roughly 92 seconds to the compiler debug suite;
2. an external boundary naming several independent reports will need a set-valued evidence role;
3. coverage-stack `dead_code` allowances await the analyzed program as their first real consumer;
4. no compiler identity exists;
5. no persistent compiler publication exists;
6. target and deployment evidence remain absent.

---

# 3. Mandatory preflight corrections

Guide 7 closes a phase. It must not build the final analyzed-program boundary over known validation or gate gaps.

These corrections should land before, or as Wave 0 of, the analyzed-program work.

## 3.1 Ensure CI actually runs the complete census and tracked-mode audit

The complete tracked-entry audit must run in the runner-agnostic CI path.

Required changes:

```text
top-level lint target:
    include census-audit

mocked Meson contract:
    explicitly build lint or census-audit

mocked contract evidence:
    verify census.ok
    verify census-audit.json
    verify report.valid = true
```

The audit must retain complete rejection of tracked modes other than:

```text
100644
100755
```

including tracked symlinks and gitlinks in paths excluded from ordinary lint subjects.

A Phase-2 exit report must not call the prior Guide-6 `ci.sh` result fully green until this production graph edge has been corrected and rerun.

## 3.2 Separate observed root effects from root-use policy

Do not encode an actual root succession or termination using `architecture::RootUse`.

Introduce an effect type such as:

```rust
enum ObservedRootEffectKind {
    Succession,
    Termination,
}
```

Compatibility must be:

```text
Forbidden:
    no observed effect

Succession:
    succession only

SuccessionOrTermination:
    succession or termination
```

Add focused tests covering normal RESV succession and sealing termination. This is not exercised by the two current root-free pilots, but it is part of the target-independent model/realization boundary Phase 2 is preparing for Phase 3 and later operation work.

## 3.3 Weld lifecycle relations to lifecycle graph declarations

The realization validator must establish exact closure among:

```text
Relation::Representation
Relation::LifecycleExit
LifecycleNodeId::Representation
LifecycleNodeId::RequiredExit
LifecycleEdge::RequiresExit
```

Require:

- every allowed representation has exactly one graph representation node;
- every lifecycle relation has exactly one required-exit node;
- every allowed representation reaches every declared required exit;
- every graph representation is declared by a representation relation;
- every graph exit is declared by a lifecycle relation;
- every lifecycle edge connects one allowed representation to one declared exit of the same object;
- no extra lifecycle nodes or edges exist.

A coherent omission from both relation dependencies and compiler coverage must not validate merely because a hard-coded pilot graph path remains.

## 3.4 Strengthen placement validation

Before analyzed-program assembly relies on placement validation:

- reject duplicate placed carriers;
- reject nonminimal carrier supersets;
- require exactly-one obligations to carry exactly one carrier;
- require every-member obligations to select exactly one valid quantified alternative;
- require at-least-one obligations to use the accepted inclusion-minimal policy;
- require deliberate duplication to carry exactly the complete admitted carrier set;
- require exact placement-local layout equality;
- reject extra, unrelated layout obligations.

Production enumeration and validation must describe the same candidate language.

## 3.5 Validate the exact placed proof-plan set

`validate_placed_proof_plans` must compare:

```text
offered proof-plan set
=
placed proof-plan set
```

It must not rely only on execution-case union equality, because two proof plans may select different proofs while producing the same execution-case identities.

## 3.6 Make representation-dependent activation object-specific

Replace activation forms that carry only a representation mode with forms carrying both object and mode:

```rust
WhenRepresentation {
    object: ObjectId,
    mode: RepresentationMode,
}
```

Resolve against the named object:

```rust
case.representations.get(&object) == Some(&mode)
```

A representation choice for one object must never activate a condition belonging to another object.

## 3.7 Enforce open-flow reference sides and uniqueness

Observation normalization must require:

```text
open-flow source:
    input reference

open-flow destination:
    output reference
```

It must reject:

- a source used by two open flows;
- a destination used by two open flows;
- a reference on the wrong side;
- an unclaimed ordinary open-value member where complete partitioning is required;
- a CPFP anchor entering the ordinary sponsor family.

This should be structural observation validation rather than an assumption made only by selected current relation evaluators.

## 3.8 Decide the deployment-profile hash boundary

The deployment-profile identity is dormant, but the public hash API should not make invalid profiles appear identity-bearing.

Preferred correction:

```rust
struct ValidatedDeploymentProfile<'a> {
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
}
```

with hashing available only for that validated wrapper.

If this change is deliberately deferred because no release consumer exists, record that explicitly under ADR-016 and keep the unchecked helper nonpublic until activation.

## 3.9 Reconcile current compiler status prose

Update the package index to say that placement, layout, and coverage are implemented internally and the complete analyzed pilots remain open.

The following sources must agree:

```text
packages/compiler/README.md
packages/compiler/src/lib.rs
plans/packages/compiler.md
plans/packages/README.md
plans/phases/02-compiler.md
plans/backlog.md
```

---

# 4. Scope and non-goals

## 4.1 In scope

Guide 7 implements:

- relation-indexed target-independent requirement bundles;
- complete scoped analyzed-program types;
- exact proof-plan census retention;
- factorized operation analysis;
- exact operation-local placements;
- exact operation-local layout requirements;
- exact operation-local coverage projections;
- lifecycle and architecture-scope completeness status;
- external-evidence requirement aggregation;
- complete assembly validation;
- independent assembly census oracle;
- factorization oracle;
- corruption tests;
- permutation tests;
- sufficient-limit invariance tests;
- complete compact-ASH analysis;
- complete live-transfer analysis;
- combined two-pilot scoped analysis;
- Phase-2 exit evidence;
- Phase-3 planning handoff.

## 4.2 Out of scope

Guide 7 must not implement:

- `tripod-target-elements`;
- `tripod-tapscript`;
- target capability adaptation;
- target opcode or stack types;
- target transaction positions;
- input/output ABI positions;
- tapleaves or control blocks;
- target proof patterns;
- target program emission;
- linked bundles;
- transaction ABI;
- target execution;
- target evidence reports;
- deployment calibration;
- release profile migration;
- release manifest;
- compiler serialization;
- generated compiler JSON or TOML;
- a compiler-plan digest;
- a realization digest;
- a public complete-analysis constructor;
- a selected “best” proof plan;
- a selected “best” placement;
- a new dependency.

---

# 5. Terminology and completeness classes

## 5.1 Use “scoped analyzed program”

The architecture has thirteen operations.

The realization and compiler pilot scope contains:

```text
compact-ash
transfer-live-receipts
```

Therefore the implementation should use names such as:

```rust
ScopedAnalyzedProgram
ScopedAnalyzedProgramProjection
AnalyzedProofPlan
AnalyzedOperation
RelationRequirements
RelationCaseRequirements
```

Avoid unqualified names such as:

```text
CompleteAnalyzedProgram
FinalAnalyzedProgram
ReleaseAnalyzedProgram
```

The result is complete for the explicit compiler scope, not complete for the architecture or deployment.

## 5.2 Keep completeness dimensions separate

The analyzed value must distinguish at least:

```text
analysis complete for requested compiler scope:
    yes

realization covers every architecture operation:
    no

all lifecycle exits are inside compiler scope:
    no

target selected:
    no

backend proof patterns implemented:
    no

external target evidence complete:
    no

deployment ready:
    no
```

Do not represent these claims with one `complete: bool`.

## 5.3 Architecture-scope status

A useful type is:

```rust
enum ArchitectureScopeStatus {
    Complete,

    Partial {
        missing: BTreeSet<architecture::OperationId>,
    },
}
```

For the Phase-2 pilots, the status must be `Partial`, naming the eleven architecture operations outside the compiler scope.

The missing set must be derived from the validated architecture operation census, not hard-coded.

## 5.4 Lifecycle status

A useful type is:

```rust
enum LifecycleCompleteness {
    Complete,

    Incomplete {
        obligations: BTreeSet<LifecycleRequirement>,
    },
}
```

For compact ASH:

```text
available in compiler scope:
    compact-ash

declared outside compiler scope:
    clear
```

For live receipts:

```text
available in compiler scope:
    transfer-live-receipts

declared outside compiler scope:
    burn
    redeem
```

Lifecycle incompleteness is expected and valid for the pilot scope. It must remain explicit.

## 5.5 Evidence status

The analyzed program carries requirements only:

```text
required external evidence:
    present

verified evidence:
    absent
```

Do not attach mutable pass/fail status or report digests.

---

# 6. Source binding

## 6.1 Bind the exact typed source

The analyzed program must bind the exact typed source from which it was derived.

A suitable internal value is:

```rust
struct AnalyzedSource {
    architecture: realization::ArchitectureBinding,
    realization: realization::ScopedRealizationProjection,
    compilation_scope: CompilationScope,
}
```

The exact structure may differ, but it must retain:

- architecture binding;
- complete scoped realization projection;
- explicit compilation scope.

The source binding must not be replaced by:

- source path;
- source line;
- Git revision alone;
- generated architecture publication;
- generated declassification;
- architecture hash alone;
- speculative realization hash;
- planning document;
- vector index.

## 6.2 Retain architecture operation census

The input boundary should retain enough typed architecture information to derive the scope status.

Options include:

```rust
architecture_operations: BTreeSet<OperationId>
```

captured during `bind_input`, or another immutable typed architecture census.

Do not infer full architecture scope from source files or planning prose.

## 6.3 Analysis policy and search limits

The production analysis uses explicit proof and placement limits.

Separate:

```text
semantic result:
    complete feasible typed values

execution report:
    limits, state counts, candidate counts, pruning counts
```

A successful semantic projection should not change merely because a larger sufficient search limit was supplied.

Search limits may turn a complete result into a typed complexity failure. They must not change one successful complete result into a different successful result.

A suitable execution report is:

```rust
struct AnalysisExecutionReport {
    proof_search_limits: ProofSearchLimits,
    placement_search_limits: PlacementSearchLimits,

    proof_search: ProofSearchReport,
    operation_placement_search:
        BTreeMap<OperationId, PlacementSearchReport>,
}
```

This report is diagnostic provenance and is excluded from the stable semantic projection.

## 6.4 Canonical capability view

The canonical Phase-2 analyzed program uses:

```rust
CapabilityView::Unconstrained
```

It retains every realization-approved abstract proof plan.

A target-pruned plan is a later downstream value. It must not be represented as the complete compiler analysis.

---

# 7. Relation-indexed requirements

Guide 7 must transform candidate-level aggregate capability, source, and evidence sets into relation-owned typed requirements.

## 7.1 Why relation provenance is required

A complete analyzed program cannot safely carry only:

```text
candidate.required_capabilities
candidate.source_requirements
candidate.external_evidence
```

without recording which relation introduced each item.

The relation-indexed structure prevents:

- a recurrence of an external-evidence capability disappearing from the aggregate;
- one relation’s witness being mistaken for another’s;
- target requirements losing semantic ownership;
- coverage and placement being joined only by operation-wide coincidence;
- a future backend being unable to report which relation remains blocked.

## 7.2 Relation-level requirement type

A useful structure is:

```rust
struct RelationRequirements {
    relation: realization::RelationId,

    proof: ProofDisposition,

    required_capabilities:
        BTreeSet<RequiredCapability>,

    source_requirements:
        BTreeSet<SourceRequirement>,

    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,

    representation:
        Option<realization::RepresentationMode>,

    lifecycle:
        BTreeSet<LifecycleRequirement>,
}
```

## 7.3 Proof disposition

Use an explicit typed distinction:

```rust
enum ProofDisposition {
    Selected {
        proof: realization::ProofAlternativeId,
    },

    StaticallyValidated,

    ExternalEvidence {
        approved_proof:
            realization::ProofAlternativeId,

        requirement:
            realization::ExternalEvidenceRequirement,
    },
}
```

Equivalent factoring is acceptable.

Required behavior:

### Proof-required relation

```text
candidate contains exactly one selected proof
relation requirements contain the same selected proof
proof is realization-approved
```

### Representation and lifecycle relation

```text
candidate contains no proof
relation requirement is statically validated
```

### External-evidence relation

```text
approved proof class retained
required capability retained
source requirement retained
external evidence requirement retained
no runtime proof completion claimed
```

## 7.4 Source requirements

For each relation and selected proof:

1. derive the complete source rows;
2. validate them against constructibility;
3. retain them in canonical order;
4. retain their activation conditions;
5. reject sponsor amount operands;
6. attach the rows to the relation that owns them.

For an execution case, derive:

```text
active source requirements
=
relation source requirements filtered by case activation
```

The relation-level set is the complete set. The case-level set is its active subset.

## 7.5 Representation ownership

When a relation names one representation-sensitive object, retain the object-specific selected mode.

Do not search the case’s representation values by mode alone.

For example:

```rust
struct RepresentationSelection {
    object: ObjectId,
    mode: RepresentationMode,
}
```

A relation requiring private-committed live receipts must not activate because some unrelated object happens to use the same mode.

## 7.6 Candidate aggregate closure

For each proof plan \(p\), validate:

\[\operatorname{Capabilities}(p)=\bigcup_{r}\operatorname{Capabilities}(p,r)\]

\[\operatorname{Sources}(p)=\bigcup_{r}\operatorname{Sources}(p,r)\]

\[\operatorname{Evidence}(p)=\bigcup_{r}\operatorname{Evidence}(p,r)\]

The unions are exact set equality.

A missing relation-owned item and an unexpected aggregate item are both errors.

## 7.7 Disclosure closure

The candidate’s disclosure analysis remains the authoritative plan-specific analysis.

Validate that:

- every added public fact has a typed reason;
- representation disclosure corresponds to the selected representation;
- inherited public facts remain inherited;
- retained-private facts remain consistent;
- no sponsor amount appears in any disclosure class;
- relation-indexed requirements do not require a fact that disclosure analysis says is unavailable.

## 7.8 Lifecycle closure

The candidate’s lifecycle requirements must equal the requirements selected by its representations.

For each selected representation:

- retain every required exit;
- preserve whether the exit is available in compiler scope;
- preserve future obligations outside compiler scope;
- reject missing or unexpected lifecycle rows.

---

# 8. Relation-case requirements

## 8.1 Relation-case bundle

A useful internal structure is:

```rust
struct RelationCaseRequirements {
    key: RelationCaseKey,

    activity: RelationActivity,
    activation: ActivationCondition,

    boundaries:
        BTreeSet<DischargeBoundary>,

    active_sources:
        BTreeSet<SourceRequirement>,

    compiler_requirements:
        BTreeSet<CompilerStaticRequirement>,

    structural_requirements:
        BTreeSet<BackendStructuralRequirement>,

    carrier_assignments:
        BTreeSet<CarrierAssignmentAlternative>,

    layout_requirements:
        BTreeSet<LayoutRequirement>,

    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,

    coverage:
        RelationCoverageProjection,
}
```

The exact structure may avoid fields already contained in `RelationCoverageProjection`, but the owning relation-case key and boundary distinctions must remain explicit.

## 8.2 Active runtime relation-case

Retain:

- active status;
- selected proof inherited from relation requirements;
- active sources;
- runtime requirement;
- semantic scope;
- carrier multiplicity;
- every feasible carrier assignment alternative;
- assignment-specific layout dependencies;
- positive and focused negative coverage;
- accepted semantic projection.

## 8.3 Compiler-static relation-case

Retain:

- active status;
- compiler validation requirement;
- relevant positive and negative compiler-analysis coverage;
- no runtime carrier;
- no runtime layout obligation;
- no target-execution claim.

## 8.4 Backend-structural relation-case

Retain:

- structural requirement;
- structural positive and negative coverage;
- no claim that the structure has been emitted;
- no runtime relation carrier.

Permissionless authorization belongs here as a secret-free emitted-path obligation.

## 8.5 External-evidence relation-case

Retain:

- approved external proof class;
- required abstract capability;
- typed external evidence requirement;
- positive and negative evidence coverage;
- no runtime carrier;
- no target verdict;
- no completed report.

## 8.6 Inactive relation-case

Retain:

- explicit vacuous activity;
- activation condition;
- inactive-valid coverage;
- no active sources;
- no runtime carrier;
- no active rejecting target mutation;
- no accepted runtime semantic projection.

An inactive relation-case is present in the census. It is never omitted.

---

# 9. Factorized operation analysis

## 9.1 Global proof plans, local operation factors

Proof plans remain global because one plan may select compatible strategies across several operations.

After selecting one global `ProofPlanCandidate`, analyze each operation independently for:

- execution cases belonging to that operation;
- relation-case plans belonging to that operation;
- carrier eligibility;
- feasible placements;
- layout requirements;
- relation-indexed coverage;
- coverage dependency graph.

## 9.2 Analyzed operation type

A useful internal type is:

```rust
struct AnalyzedOperation {
    operation: architecture::OperationId,

    execution_cases:
        BTreeSet<ExecutionCaseId>,

    relation_cases:
        BTreeMap<
            RelationCaseKey,
            RelationCaseRequirements,
        >,

    feasible_placements:
        BTreeSet<PlacementCandidateProjection>,

    layout_requirements:
        BTreeSet<LayoutRequirement>,

    coverage:
        OperationCoverageProjection,

    coverage_dependencies:
        CoverageGraphProjection,
}
```

## 9.3 Exact operation census

For every proof plan:

```text
analyzed operation keys
=
compilation scope operations
=
realization scope operations
```

Reject:

- missing operation;
- unexpected operation;
- duplicate operation factor;
- operation factor containing a case for another operation;
- relation-case key owned by another operation;
- cross-operation coverage edge;
- cross-operation layout requirement.

## 9.4 Operation-local execution cases

For operation \(o\):

```text
operation cases
=
all candidate cases whose operation is o
```

The representation is fixed by the proof plan.

The only current case expansion is:

```text
sponsor absent
sponsor present
```

Counts, denominations, owners, object order, and sponsor denominations remain vector dimensions, not compiler semantic case dimensions.

## 9.5 Operation-local relation-case census

For operation \(o\):

\[\operatorname{RelationCases}(p,o)=\operatorname{Relations}(o)\times\operatorname{Cases}(p,o)\]

Every pair appears exactly once.

This includes:

- inactive sponsor relations;
- compiler-static relations;
- backend-structural relations;
- external-evidence relations;
- runtime relations.

## 9.6 Operation-local placement

Enumerate placements over only the active runtime relation-cases of one operation.

Do not call the existing global product enumerator from the canonical analyzed-program path if it materializes combinations across operations.

A suitable internal function is:

```rust
fn analyze_operation_placements(
    relations: &CompilerRelationAnalysis,
    candidate: &ProofPlanCandidate,
    operation: OperationId,
    limits: PlacementSearchLimits,
) -> Result<OperationPlacementAnalysis, CompileError>;
```

The result contains:

- operation cases;
- operation relation-case plans;
- operation carrier eligibility;
- operation placements;
- operation layout census;
- placement search report.

## 9.7 Exact placement validation

For every placement:

- relation-case assignment census is exact;
- carrier list is nonempty;
- carrier list is duplicate-free;
- each carrier is eligible;
- scope permits the carrier;
- quantification matches multiplicity;
- optional sponsor carriers cannot uniquely carry unconditional relations;
- global relations do not rely on member-local carriers;
- carrier assignment is one accepted inclusion-minimal alternative;
- layout requirements equal the exact selected-assignment dependencies;
- no surplus requirement exists.

## 9.8 Factorization conditions

The product theorem is valid only if:

1. relation-case keys are operation-owned;
2. no relation dependency crosses operations;
3. no carrier spans operations;
4. no layout requirement spans operations;
5. no coverage dependency crosses operations;
6. proof plan is already fixed globally;
7. operation placements do not mutate proof selection;
8. operation placements do not consume a shared target resource model in Phase 2.

The analyzed-program validator must check these conditions.

## 9.9 Factorization regression

Replace ordinary repeated materialization of the full pilot product with:

1. exact validation of each operation factor;
2. a structural cross-operation-independence check;
3. a small synthetic exhaustive product oracle;
4. one explicit, optionally ignored/manual pilot product comparison.

The expensive existing combined-scope regression may be marked ignored only after equivalent signal exists through the factor oracle and cross-operation closure checks.

A manual regression can remain available:

```sh
cargo test --locked -p tripod-compiler \
  combined_scope_product -- --ignored --nocapture
```

Use the actual final test name.

For the Phase-2 exit record, run the ignored full pilot product test once if retained.

The ordinary compiler suite should not spend approximately 92 seconds repeatedly materializing a product the production analyzed value deliberately does not store.

---

# 10. Coverage integration

## 10.1 Per-operation coverage

Coverage remains stored per operation.

For each operation factor:

```text
coverage keys
=
operation relation-case keys
```

The operation coverage projection retains:

- activity;
- boundaries;
- positive requirements;
- negative requirements;
- carrier alternatives;
- accepted projections;
- external evidence;
- typed dependency graph.

## 10.2 Exact requirement validation

The existing coverage shape validator is not sufficient by itself for corruption-resistant analyzed-program validation.

The complete assembly validator must compare the stored coverage projection against a freshly and independently derived expected projection from:

- relation declaration;
- execution case;
- relation-case plan;
- placement alternatives;
- layout requirements;
- typed dependency closure.

Reject:

- missing mutation class;
- unexpected mutation class;
- wrong boundary;
- wrong evidence role;
- wrong positive purpose;
- wrong negative purpose;
- incorrect collateral policy;
- incorrect dependency closure;
- runtime carrier on a nonruntime relation;
- missing accepted projection;
- target-execution role assigned to a compiler-static, structural, or external boundary.

## 10.3 External evidence roles

The current pilots have one external report requirement per external relation.

Implement the analyzed structure so several requirements remain representable without forcing one report into a scalar role.

Preferred structure:

```rust
struct ExternalEvidenceCoverage {
    requirements:
        BTreeSet<ExternalEvidenceRequirement>,

    roles:
        BTreeSet<EvidenceRole>,
}
```

or an equivalent set-valued representation.

Do not freeze a scalar role model that becomes ambiguous as soon as one boundary names two reports.

## 10.4 Conditional triplets

Across each proof plan’s complete case set, every conditional relation must have:

```text
inactive valid
active valid
active invalid
```

The triplet is a plan-set property, not one relation-case property.

## 10.5 Representation census

Across the complete feasible proof-plan set, every realization-approved representation mode must receive coverage.

Each individual proof plan fixes one mode. Do not create an impossible case that simultaneously carries several incompatible modes merely to satisfy the census.

---

# 11. Scoped analyzed-program model

## 11.1 Principal type

A useful internal shape is:

```rust
struct ScopedAnalyzedProgram {
    source: AnalyzedSource,

    foundation:
        CompilerAnalysisFoundationProjection,

    architecture_scope:
        ArchitectureScopeStatus,

    proof_plans:
        BTreeMap<
            ProofPlanCandidate,
            AnalyzedProofPlan,
        >,

    required_external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,

    execution_report:
        AnalysisExecutionReport,
}
```

Fields should remain private or crate-private.

## 11.2 Analyzed proof plan

```rust
struct AnalyzedProofPlan {
    proof_plan: ProofPlanCandidate,

    relation_requirements:
        BTreeMap<
            realization::RelationId,
            RelationRequirements,
        >,

    operations:
        BTreeMap<
            architecture::OperationId,
            AnalyzedOperation,
        >,

    lifecycle:
        LifecycleCompleteness,
}
```

The map key and retained `proof_plan` field may be deduplicated internally. The stable projection must represent the typed plan exactly once.

## 11.3 No vector-position identity

Do not use:

```text
plan 0
plan 1
candidate index
search rank
first plan
best plan
```

The complete typed `ProofPlanCandidate` remains the map key until a persistent consumer justifies an identity under ADR-016.

## 11.4 No selected canonical plan

Guide 7 retains the exact feasible plan set.

It does not choose:

- one preferred proof plan;
- one preferred representation;
- one preferred placement;
- one target;
- one backend;
- one resource objective.

Selection belongs to a later target-planning boundary under explicit policy.

## 11.5 Required external evidence

The top-level external-evidence set must equal the union over every proof plan and relation requirement.

For the current pilots it must retain whole-transaction L-BTC conservation requirements and must not mark them complete.

---

# 12. Stable projection

## 12.1 Projection type

A suitable projection is:

```rust
struct ScopedAnalyzedProgramProjection {
    source: AnalyzedSourceProjection,

    foundation:
        CompilerAnalysisFoundationProjection,

    architecture_scope:
        ArchitectureScopeStatus,

    proof_plans:
        BTreeMap<
            ProofPlanCandidate,
            AnalyzedProofPlanProjection,
        >,

    required_external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,
}
```

## 12.2 Projection contents

The stable projection may contain:

- architecture binding;
- scoped realization projection;
- compilation scope;
- relation and expression graph projections;
- architecture-scope status;
- exact proof-plan typed keys;
- relation-indexed requirements;
- per-operation execution-case sets;
- per-operation relation-case requirements;
- per-operation feasible placement projections;
- per-operation layout requirements;
- per-operation coverage projections;
- coverage dependency projections;
- lifecycle completeness;
- external-evidence requirements.

## 12.3 Projection exclusions

The stable projection must exclude:

- Petgraph `NodeIndex`;
- Petgraph `EdgeIndex`;
- insertion order;
- topological traversal order where several orders are valid;
- proof-search state count;
- placement-search state count;
- candidate vector position;
- full cross-operation placement product;
- elapsed time;
- thread schedule;
- temporary path;
- source path;
- source line;
- diagnostics;
- raw floating values;
- report hash;
- compiler digest;
- ambient environment;
- filesystem state.

## 12.4 Canonical containers

Use:

```text
BTreeMap
BTreeSet
canonically sorted typed vectors
```

Set-like source vectors must not remain order-sensitive in the stable projection.

## 12.5 Determinism properties

Require projection equality under:

- proof-plan candidate permutation;
- operation ordering permutation;
- relation declaration permutation;
- relation dependency permutation;
- expression declaration permutation;
- expression dependency permutation;
- execution-case permutation;
- carrier ordering permutation;
- placement ordering permutation;
- layout-requirement permutation;
- coverage-requirement permutation;
- coverage dependency insertion permutation;
- repeated clean analysis.

Also require:

```text
larger sufficient proof-search limits
+
larger sufficient placement-search limits
⇒
equal successful stable projection
```

The execution reports may differ in state counts. The semantic projection must not.

---

# 13. Complete assembly validator

The analyzed program must not be trusted solely because its assembler produced it.

Implement one complete validator over the assembled value.

## 13.1 Source validation

Require:

- supplied architecture validates;
- scoped realization validates against architecture;
- analyzed architecture binding equals realization binding;
- compilation scope is canonical and nonempty;
- scope is a subset of realization scope;
- source projection equals the input’s exact typed source;
- architecture-scope status exactly names missing operations.

## 13.2 Foundation validation

Require:

- relation graph census equals scoped realization relation census;
- expression graph census equals scoped realization expression census;
- every relation dependency resolves;
- every expression dependency resolves;
- every expression predicate is bound to a same-operation Boolean expression;
- folded values preserve source expressions and dependencies;
- stable foundation projection equals the validated component projection.

## 13.3 Proof-plan census

Require exact equality:

```text
fresh exact feasible proof-plan set
=
analyzed proof-plan key set
```

Reject:

- missing plan;
- unexpected plan;
- duplicate plan;
- infeasible plan;
- plan altered after exact search;
- representation/proof mismatch;
- external-evidence proof treated as a runtime selected proof.

## 13.4 Relation requirement census

For every proof plan:

```text
relation requirement keys
=
compiler relation IDs
```

Require exactly one relation requirement per relation.

Reject:

- missing relation;
- unexpected relation;
- proof disposition inconsistent with relation class;
- selected proof absent from candidate;
- selected proof not realization-approved;
- static relation carrying a proof;
- external relation missing approved proof class;
- relation capability/source/evidence closure mismatch.

## 13.5 Operation-factor census

For every proof plan:

```text
analyzed operation keys
=
compilation scope
```

Require every factor to contain only its operation’s:

- cases;
- relation-case keys;
- placements;
- layouts;
- coverage;
- dependency graph.

## 13.6 Case census

For every operation and proof plan:

```text
stored cases
=
independently derived applicable cases
```

Reject duplicate, missing, unexpected, or foreign-operation cases.

## 13.7 Relation-case census

For every operation factor:

```text
stored relation-case keys
=
operation relations × operation cases
```

Every relation-case must have exactly one disposition.

## 13.8 Carrier and placement closure

Require:

- carrier eligibility recomputes exactly;
- only active runtime relations receive carriers;
- every stored placement is valid;
- exact accepted carrier alternatives are represented;
- no placement is missing;
- no unexpected placement exists;
- no duplicate/nonminimal carrier assignment exists;
- exact selected layout dependencies are present;
- no surplus layout requirement exists.

## 13.9 Coverage closure

Require exact equality with independently re-derived coverage:

- positive;
- negative;
- inactive-valid;
- carrier alternatives;
- accepted projections;
- external evidence;
- dependency collateral.

The validator must reject extra requirements, not only missing requirements.

## 13.10 Lifecycle closure

Require:

- relation/lifecycle graph weld passes;
- candidate lifecycle rows match selected representations;
- top-level lifecycle status matches candidate rows;
- exits inside compiler scope are marked available;
- exits outside compiler scope remain explicit;
- no target support is inferred.

## 13.11 Evidence closure

Require:

```text
top-level evidence
=
union of plan evidence
=
union of relation evidence
```

No report status or digest is permitted.

## 13.12 Sponsor opacity

Traverse the complete assembled value.

Reject any occurrence of:

- sponsor input amount;
- sponsor output amount;
- public sponsor aggregate;
- sponsor positivity;
- sponsor opening;
- sponsor blinding factor;
- exact sponsor value source;
- sponsor amount in coverage projection;
- sponsor amount in layout;
- sponsor amount in evidence identity.

Retain only:

- sponsor family recognition;
- sponsor cardinality;
- sponsor owner authorization;
- sponsor/protocol disjointness;
- envelope multiplicity;
- substrate-conservation evidence.

## 13.13 Factorization closure

Require:

- no cross-operation relation dependency;
- no cross-operation carrier;
- no cross-operation layout requirement;
- no cross-operation coverage dependency;
- no combined placement-product field;
- operation placement factors independently validate.

---

# 14. Independent assembly oracle

Guide 5 and Guide 6 already independently verify proof planning, placement, coverage, closure, and SCC behavior.

Guide 7 needs an independent **assembly census oracle**, not another implementation of every prior algorithm.

## 14.1 Oracle purpose

The oracle answers:

> Does the complete analyzed object contain exactly the component results and cross-stage joins required by its typed source?

It does not need to reimplement:

- proof-search recursion;
- carrier availability;
- placement search;
- coverage mutation catalogue;
- Petgraph SCC implementation.

Those already have independent oracles.

## 14.2 Direct expected censuses

Independently derive:

```text
expected architecture operation census
expected compiler scope
expected architecture-scope missing set
expected proof-plan set
expected relations per operation
expected cases per proof plan and operation
expected relation-case keys
expected active runtime relation-case keys
expected compiler-static keys
expected structural keys
expected external-evidence keys
expected placement factors
expected layout requirement sets
expected coverage keys
expected external evidence union
expected lifecycle obligations
```

Compare each set with the assembled projection.

## 14.3 Oracle independence rule

The oracle must not use an analyzed-program field to derive the expectation for that same field.

Wrong:

```text
expected relation cases =
    analyzed_program.relation_cases.keys()
```

Correct:

```text
expected relation cases =
    relation graph IDs × independently derived applicable case IDs
```

Wrong:

```text
expected proof plans =
    analyzed_program.proof_plans.keys()
```

Correct:

```text
expected proof plans =
    exact feasible proof-planning result
```

## 14.4 Requirement-union oracle

Independently union relation-owned:

- capabilities;
- source rows;
- external evidence.

Compare exactly with the candidate aggregates.

This is a permanent regression against capability/evidence loss.

## 14.5 Factorization oracle

Use two levels.

### Ordinary test level

Prove structurally:

- every placement constraint is operation-owned;
- no dependency crosses operations;
- each operation factor is exact;
- the analyzed value stores the factors only.

Use a small synthetic problem to enumerate both:

```text
global product
per-operation factor product
```

and compare exact sets.

### Manual or phase-exit pilot level

For the real two-pilot scope, enumerate the old combined placement product once and compare it with the test-only product of the two factors.

This may remain ignored during normal development after the ordinary factorization tests cover the same property generically.

## 14.6 Corruption oracle

Mutate one assembled projection at a time and require:

1. the production validator rejects it;
2. the independent expected projection differs.

A corruption detected only by one side indicates a blind spot in the other.

---

# 15. Required corruption tests

The complete analyzed-program validator needs focused mutation coverage.

## 15.1 Source and scope

- change architecture binding;
- change realization projection;
- drop one scope operation;
- add an unexpected operation;
- claim complete architecture scope;
- omit one missing architecture operation;
- add an operation that is not missing.

## 15.2 Proof plans

- remove one proof plan;
- add an infeasible plan;
- duplicate one plan;
- replace one selected proof;
- add a proof to a static relation;
- remove the external proof class;
- change a representation without updating its proof;
- use two different plans with identical cases and remove one.

## 15.3 Relation requirements

- remove one relation requirement;
- add an unexpected relation;
- remove one capability;
- add one capability;
- remove one source row;
- add an unauthenticated source row;
- remove external evidence;
- add evidence to the wrong relation;
- change one relation’s proof disposition.

## 15.4 Execution cases

- remove sponsorless case;
- remove sponsored case;
- add duplicate case;
- add case for another operation;
- change representation on one case only;
- activate an object-specific representation condition through another object.

## 15.5 Placement

- remove one feasible placement;
- add one infeasible placement;
- add a placement from another operation;
- duplicate one carrier;
- add redundant second carrier;
- replace per-member carrier with coordinator;
- select optional sponsor carrier for unconditional relation;
- remove one selected layout requirement;
- add an unrelated layout requirement.

## 15.6 Coverage

- remove positive coverage;
- remove one required negative while leaving another;
- add an unrelated mutation;
- move a mutation to the wrong boundary;
- assign target-execution role to compiler-static coverage;
- assign target-execution role to external evidence;
- remove inactive-valid coverage;
- add a rejection to an inactive relation;
- remove accepted projection;
- change dependency collateral;
- introduce a coverage dependency cycle;
- add a cross-operation coverage edge.

## 15.7 Lifecycle

- remove one future exit;
- mark an outside-scope exit available;
- add an undeclared exit;
- remove one allowed representation;
- leave graph node after removing semantic relation;
- leave semantic relation after removing graph node.

## 15.8 Sponsor opacity

- add sponsor amount operand;
- add public sponsor sum;
- add sponsor positivity mutation;
- add sponsor amount source;
- add sponsor amount layout routing;
- add sponsor amount to accepted projection;
- add sponsor amount to evidence subject.

Each must fail with `CompileError::SponsorValueRead` or a narrower typed error.

---

# 16. Proposed source organization

A clean implementation may add:

```text
packages/compiler/src/requirement.rs
packages/compiler/src/analyzed.rs
```

and tests such as:

```text
packages/compiler/src/tests/requirement_tests.rs
packages/compiler/src/tests/analyzed_program_tests.rs
packages/compiler/src/tests/analyzed_program_oracle_tests.rs
packages/compiler/src/tests/factorization_tests.rs
```

Exact factoring may differ.

## 16.1 `requirement.rs`

Own:

- `ProofDisposition`;
- relation-indexed capability derivation;
- relation-indexed source derivation;
- relation-indexed evidence derivation;
- object-specific representation selection;
- relation-level requirement bundles;
- candidate aggregate closure;
- relation-case active-source filtering;
- sponsor-erasure traversal.

## 16.2 `analyzed.rs`

Own:

- `ArchitectureScopeStatus`;
- `LifecycleCompleteness`;
- `AnalyzedSource`;
- `AnalyzedOperation`;
- `AnalyzedProofPlan`;
- `ScopedAnalyzedProgram`;
- stable projections;
- factorized orchestration;
- complete assembly validation;
- analysis execution report.

## 16.3 No generic graph wrapper

Do not create:

```text
AnalysisGraph
CanonicalGraph
CompilerGraph
GraphAdapter
```

The existing direct Petgraph ownership remains in the relevant modules.

## 16.4 Meson census

Every new Rust source must be added to:

```text
packages/compiler/meson.build
```

Every new test module must be added to:

```text
packages/compiler/src/tests/mod.rs
```

in the same commit.

---

# 17. Internal orchestration entry point

A suitable crate-private entry point is:

```rust
pub(crate) fn analyze_scoped_program(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ScopedAnalyzedProgram, CompileError>;
```

The exact signature may separate execution configuration:

```rust
pub(crate) struct AnalysisExecutionPolicy {
    pub placement_limits: PlacementSearchLimits,
}
```

The canonical proof-search limits already live in `AnalysisPolicy`.

The orchestrator should be explicit:

```text
1. revalidate compiler input
2. analyze relation/expression foundation
3. build constructibility analysis
4. build lifecycle analysis
5. enumerate exact unconstrained feasible proof plans
6. derive relation-indexed requirements for every plan
7. for each plan:
       for each operation:
           derive operation cases
           classify relation cases
           derive carrier eligibility
           enumerate operation-local placements
           derive operation-local layout
           derive operation-local coverage
           resolve operation-local coverage dependencies
           validate operation factor
8. derive architecture-scope status
9. derive lifecycle completeness
10. aggregate external evidence
11. construct scoped analyzed program
12. run complete assembly validator
13. return result
```

No stage failure should be converted into a generic partial result.

---

# 18. Error vocabulary

Add only errors reached by real validation branches.

Possible additions include:

```rust
AnalyzedOperationCensusMismatch {
    missing: Vec<architecture::OperationId>,
    unexpected: Vec<architecture::OperationId>,
}

AnalyzedProofPlanCensusMismatch {
    missing: usize,
    unexpected: usize,
}

DuplicateAnalyzedProofPlan,

AnalyzedRelationRequirementCensusMismatch {
    missing: Vec<realization::RelationId>,
    unexpected: Vec<realization::RelationId>,
}

AnalyzedRelationCaseCensusMismatch {
    operation: architecture::OperationId,
    missing: Vec<RelationCaseKey>,
    unexpected: Vec<RelationCaseKey>,
}

AnalyzedCapabilityClosureMismatch {
    relation: realization::RelationId,
}

AnalyzedSourceClosureMismatch {
    relation: realization::RelationId,
}

AnalyzedEvidenceClosureMismatch {
    relation: realization::RelationId,
}

AnalyzedPlacementFactorMismatch {
    operation: architecture::OperationId,
}

AnalyzedLayoutCensusMismatch {
    operation: architecture::OperationId,
}

AnalyzedCoverageCensusMismatch {
    operation: architecture::OperationId,
}

AnalyzedLifecycleStatusMismatch,

AnalyzedArchitectureScopeStatusMismatch,

UnexpectedCombinedPlacementProduct,
```

Because `CompileError` is public and non-exhaustive:

- do not expose local graph handles;
- do not expose vector positions;
- do not add speculative digests;
- avoid embedding an entire proof plan in every error;
- prefer operation, relation, and case context;
- keep error ordering deterministic.

---

# 19. Pilot acceptance requirements

## 19.1 Compact ASH

For every feasible compact-ASH proof plan, retain and validate:

### Scope and cases

```text
operation:
    compact-ash

representations across plan set:
    Explicit
    PublicCommitted

cases per plan:
    sponsor absent
    sponsor present
```

### Relations

- ASH input cardinality;
- ASH output cardinality;
- sponsor input cardinality;
- sponsor output cardinality;
- ASH input recognition;
- ASH output recognition;
- sponsor input recognition;
- sponsor output recognition;
- input family closure;
- output family closure;
- permissionless authorization;
- public constructibility;
- ownerless `U` conservation;
- canonical ownerless-lateral delta;
- fee-sponsor-only open-flow policy;
- sponsor isolation;
- sponsor multiplicity;
- no-root policy;
- transition-certificate-only projection;
- representation selection;
- compact lifecycle;
- clear lifecycle;
- substrate conservation.

### Requirements

- public arithmetic proof for ASH conservation;
- authenticated object recognition;
- authenticated family cardinality;
- authenticated canonical partition;
- authenticated open-flow partition;
- public constructibility;
- whole-transaction substrate conservation evidence;
- no owner/operator witness;
- no sponsor amount.

### Factorized outputs

- exact compact-ASH execution cases;
- exact compact-ASH relation-case plans;
- exact compact-ASH carrier alternatives;
- exact compact-ASH placements;
- exact compact-ASH layout requirements;
- exact compact-ASH coverage;
- exact compact-ASH coverage dependency graph.

## 19.2 Live transfer

For every feasible live-transfer proof plan, retain and validate:

### Scope and cases

```text
operation:
    transfer-live-receipts

representations across plan set:
    Explicit
    PrivateCommitted

cases per plan:
    sponsor absent
    sponsor present
```

### Relations

- live input cardinality;
- live output cardinality;
- sponsor input cardinality;
- sponsor output cardinality;
- live input recognition;
- live output recognition;
- sponsor input recognition;
- sponsor output recognition;
- every-owner authorization;
- input family closure;
- output family closure;
- aggregate `U` conservation;
- canonical lateral delta;
- fee-sponsor-only open-flow policy;
- sponsor isolation;
- sponsor multiplicity;
- no-root policy;
- transition-certificate-only projection;
- owner constructibility;
- representation selection;
- transfer lifecycle;
- burn lifecycle;
- redemption lifecycle;
- substrate conservation.

### Proof strategies

```text
Explicit:
    PublicArithmetic

PrivateCommitted:
    ConfidentialConservation
```

Reject:

```text
PrivateCommitted + PublicArithmetic
Explicit + ConfidentialConservation
```

unless another future reviewed relation explicitly permits such a pairing.

### Requirements

- every-owner witness tied to the live input family;
- explicit closed `U` identity;
- output-family closure;
- exact aggregate semantic conservation;
- representation-specific amount source;
- sponsor isolation;
- whole-transaction substrate conservation evidence;
- no sponsor amount.

### Factorized outputs

- exact live-transfer cases;
- exact relation-case plans;
- exact owner-per-member placement;
- exact global conservation placement;
- exact layout requirements;
- exact coverage;
- exact dependency graph.

## 19.3 Combined scoped program

The combined result must establish:

```text
compiler operations:
    compact-ash
    transfer-live-receipts

architecture scope:
    partial, with eleven operations missing

proof-plan set:
    exact unconstrained feasible set

operation storage:
    factorized

combined placement product stored:
    no

coverage:
    per operation

cross-operation dependencies:
    none

target:
    absent

deployment lifecycle:
    incomplete

external substrate evidence:
    required and unresolved

compiler identity:
    not minted
```

---

# 20. Performance policy

## 20.1 Remove the first real consumer’s `dead_code` allowances

When `ScopedAnalyzedProgram` consumes the coverage stack, remove `dead_code` allowances from modules and items that now have production internal consumers.

Do not remove allowances blindly. The compiler should still keep partial internal types private where the complete analyzed program does not consume them.

## 20.2 No combined product in production

The production Guide-7 path must never enumerate the two-operation placement product.

Its work should approximate:

\[\sum_{p}\sum_{o}|\mathcal P(p,o)|\]

rather than:

\[\sum_{p}\prod_{o}|\mathcal P(p,o)|\]

## 20.3 Demote the expensive global-product regression only after replacement

The roughly 92-second combined-scope coverage test may be ignored or moved to a manual/phase-exit lane only after:

- operation factor exactness is tested;
- synthetic product equality is tested;
- cross-operation independence is validated;
- one manual full pilot product comparison remains available.

Record the ordinary compiler-suite wall time before and after the change.

## 20.4 Complexity failures remain typed

If proof or placement search exceeds explicit limits:

- return a typed error;
- return no partial analyzed program;
- do not retain the first plans seen;
- do not select a greedy plan;
- do not drop a relation;
- do not claim exactness.

---

# 21. Implementation waves

## Wave 0 — Close preflight validation and gate gaps

Deliver:

- census audit in `lint`;
- census audit exercised by mocked Meson contract;
- root effect/policy separation;
- lifecycle relation/graph closure;
- exact placement validation;
- exact offered/placed proof-plan equality;
- object-specific representation activation;
- open-flow side/uniqueness checks;
- deployment-profile hash decision;
- compiler status reconciliation.

Focused tests must accompany each repair.

Suggested commit sequence:

```text
build: run the complete tracked-entry audit in CI
realization: separate root effects from root-use policy
realization: weld lifecycle relations to lifecycle graphs
compiler: make placement validation exact
compiler: validate the exact placed proof-plan set
compiler: key representation activation by object
realization: validate open-flow reference structure
architecture: restrict deployment hashing to validated profiles
plans: reconcile compiler and review status
```

A smaller commit series is acceptable if each commit remains coherent and green.

## Wave 1 — Relation-indexed requirements

Deliver:

- `ProofDisposition`;
- `RelationRequirements`;
- relation proof join;
- relation capability derivation;
- relation source derivation;
- relation external-evidence derivation;
- representation selection;
- lifecycle rows;
- candidate aggregate closure;
- sponsor-erasure validation.

Required regressions:

- external substrate capability retained;
- external source retained;
- representation relation has no proof variable;
- live proof/mode compatibility;
- missing capability fails;
- missing source fails;
- missing evidence fails;
- sponsor amount fails.

Suggested commit:

```text
compiler: derive relation-indexed requirements
```

## Wave 2 — Operation-local factor analysis

Deliver:

- operation-specific execution case extraction;
- operation-specific relation-case classification;
- operation-specific carrier eligibility;
- operation-local placement enumeration;
- operation-local layout derivation;
- operation-local coverage derivation;
- operation-local coverage dependency resolution;
- `AnalyzedOperation`;
- operation factor projection.

Required tests:

- compact ASH factor;
- live-transfer factor;
- exact operation-owned censuses;
- no cross-operation dependency;
- placement exactness;
- layout exactness;
- coverage exactness;
- repeated equality.

Suggested commit:

```text
compiler: derive factorized operation analyses
```

## Wave 3 — Scoped analyzed-program assembly

Deliver:

- `AnalyzedSource`;
- `ArchitectureScopeStatus`;
- `LifecycleCompleteness`;
- `AnalyzedProofPlan`;
- `ScopedAnalyzedProgram`;
- execution report;
- stable projection;
- internal orchestration.

Required tests:

- complete source binding;
- exact proof-plan map;
- exact operation factors;
- partial architecture status;
- lifecycle incompleteness;
- unresolved external evidence;
- no combined placement product.

Suggested commit:

```text
compiler: assemble complete scoped analyzed programs
```

## Wave 4 — Complete assembly validator

Deliver:

- source validation;
- proof-plan census;
- relation-requirement census;
- capability/source/evidence closure;
- operation-factor census;
- case and relation-case census;
- placement/layout exactness;
- coverage exactness;
- lifecycle exactness;
- sponsor-opacity traversal;
- corruption tests.

Suggested commit:

```text
compiler: validate analyzed-program closure
```

## Wave 5 — Independent assembly and factorization oracle

Deliver:

- direct expected-census oracle;
- requirement-union oracle;
- factorization oracle;
- synthetic product comparison;
- manual real-pilot product comparison;
- permutation tests;
- sufficient-limit invariance;
- performance replacement for the 92-second ordinary test.

Suggested commit:

```text
compiler: add analyzed-program assembly oracles
```

## Wave 6 — End-to-end pilot acceptance

Deliver:

- complete compact-ASH analyzed program;
- complete live-transfer analyzed program;
- combined two-pilot scoped analyzed program;
- exact stable projections;
- explicit partial architecture status;
- explicit lifecycle incompleteness;
- external evidence census;
- sponsor opacity over the complete projection.

Suggested commit:

```text
compiler: analyze both pilots end to end
```

## Wave 7 — Documentation and Phase-2 gate

Deliver:

- updated compiler Rustdoc;
- updated compiler README;
- updated compiler package contract;
- updated package index;
- updated Phase-2 card;
- compact Guide-7 gate record in backlog;
- Phase-2 completion only after the complete gate;
- Phase-3 activation only after all phase declarations move together.

Suggested commit:

```text
plans: record the Phase-2 compiler exit
```

---

# 22. Focused verification during implementation

## 22.1 Preflight fixes

```sh
cargo test --locked -p tripod-labels
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-model realization_conformance
cargo test --locked -p tripod-compiler placement
scripts/test-meson-mock.sh .
```

Use actual final test filters and verify they match nonzero tests.

## 22.2 Requirement analysis

```sh
cargo test --locked -p tripod-compiler requirement
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler source
cargo test --locked -p tripod-compiler disclosure
cargo test --locked -p tripod-compiler lifecycle
```

## 22.3 Factorized operation analysis

```sh
cargo test --locked -p tripod-compiler factorization
cargo test --locked -p tripod-compiler placement
cargo test --locked -p tripod-compiler layout
cargo test --locked -p tripod-compiler coverage
```

## 22.4 Complete analyzed program

```sh
cargo test --locked -p tripod-compiler analyzed_program
cargo test --locked -p tripod-compiler pilot
```

## 22.5 Independent oracles

```sh
cargo test --locked -p tripod-compiler analyzed_program_oracle
cargo test --locked -p tripod-compiler oracle
```

## 22.6 Complete package

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-compiler --no-deps
```

## 22.7 Realization non-regression

```sh
cargo test --locked -p tripod-realization
```

---

# 23. Working cadence

After each coherent wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Read `git status` after formatting.

Do not run the full Meson/document/release surface after every small edit.

Do not create another production build directory. The canonical directory is:

```text
build/
```

Every new tracked source joins its nearest `meson.build` census in the same commit.

Commit each coherent green wave promptly.

---

# 24. Phase-2 exit verification

Guide 7 closes Phase 2. The final evidence must be stronger than an ordinary working batch.

## 24.1 Declared MSRV lane

Run under Rust 1.88:

```sh
rustc --version
cargo --version
CI_REQUIRE_MESON=1 scripts/ci.sh
```

Record exact toolchain versions.

## 24.2 Current stable lane

Run under current stable:

```sh
rustc --version
cargo --version
CI_REQUIRE_MESON=1 scripts/ci.sh
```

Record exact toolchain versions.

The Rust lanes may share Cargo’s normal target directory only when the toolchain workflow safely distinguishes artifacts. Follow the repository’s existing toolchain practice.

## 24.3 Canonical Meson build

Using only `build/`:

```sh
meson compile -C build
meson test -C build --print-errorlogs
```

If `build/` does not exist:

```sh
meson setup build
```

Do not configure another production build directory.

## 24.4 Full factorization regression

If the real combined-product test is ignored in ordinary development, run it explicitly for the Phase-2 gate:

```sh
cargo test --locked -p tripod-compiler \
  combined_scope_product -- --ignored --nocapture
```

Use the actual final test name.

Record:

- operation factor sizes;
- product size;
- exact equality result;
- wall time;
- confirmation that production analyzed storage contains factors only.

## 24.5 Document reproducibility

Because this is a phase exit, run:

```sh
scripts/check-document-reproducibility.sh
```

Paper inputs being unchanged is not sufficient reason to call this passed without running it.

If the required TeX environment is unavailable, record:

```text
document reproducibility:
    DEFERRED
```

and leave `P2-013` blocked unless the project adopts an explicit phase-exit policy allowing that deferral.

The safest and recommended result is:

```text
P2-013 closes only after document reproducibility passes.
```

## 24.6 Advisory lane

Run:

```sh
cargo audit
```

when installed.

If unavailable, record:

```text
cargo-audit:
    SKIPPED — tool unavailable
```

not passed.

ADR-011 permits a loud skip. The overall run must be described as partial in the advisory dimension, not fully green.

## 24.7 Dependency evidence

Guide 7 expects no dependency change.

Run:

```sh
cargo tree --locked -e features
cargo metadata --locked
git diff -- Cargo.toml Cargo.lock
```

Expected result:

```text
new dependency:
    none

Cargo.lock:
    unchanged
```

Any change receives separate dependency review.

## 24.8 Documentation and generated artifacts

Run:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Expected generated-publication result:

```text
architecture.json:
    unchanged

architecture.toml:
    unchanged

declassification.json:
    unchanged

model_labels.json:
    changes only for deliberate participating Rust labels

Specification register:
    unchanged

Realization register:
    unchanged
```

## 24.9 Clean repository

After all commits:

```sh
git status --porcelain=v1 --untracked-files=all
```

The output must be empty.

---

# 25. Phase and planning updates

## 25.1 Before the exit gate passes

Keep:

```text
Phase 2:
    Active

Phase 3:
    Planned
```

Mark:

```text
P2-012:
    DONE
```

only when the complete analyzed pilots and focused evidence exist.

Keep:

```text
P2-013:
    BLOCKED
```

until the full phase gate is recorded.

## 25.2 After the complete exit gate passes

Update all current-phase declarations atomically:

```text
plans/backlog.md
plans/README.md
plans/roadmap.md
plans/phases/02-compiler.md
plans/phases/03-target-foundation.md
plans/phases/README.md
```

Expected state:

```text
Phase 2:
    Complete

Phase 3:
    Active

Current gate:
    Phase 3 — Elements target and foundational prototypes
```

Exactly one numbered phase card may be Active.

## 25.3 Compiler status after Guide 7

Documentation should state:

```text
validated compiler input:
    implemented

relation/expression foundation:
    implemented

checked folding:
    implemented

exact proof planning:
    implemented

source/constructibility/disclosure/lifecycle:
    implemented

execution cases:
    implemented

placement/layout:
    implemented

relation-indexed coverage:
    implemented

complete scoped pilot analysis:
    implemented internally

public complete-analysis API:
    not exposed

compiler identity:
    not minted

target adapter:
    absent

backend:
    absent
```

---

# 26. Public API ruling

## 26.1 Keep the complete analyzed program crate-private

Guide 7 should not expose `ScopedAnalyzedProgram` publicly.

Reasons:

1. no target or backend package exists;
2. there is no external consumer;
3. the first real consumer may need only a narrow target-requirement projection;
4. exposing the complete type now would freeze internal factorization and validation choices;
5. ADR-016 rejects speculative identities and publications;
6. Phase 3 should reveal the smallest honest downstream API.

## 26.2 Public compiler API remains narrow

The public boundary remains:

- validated input types;
- explicit scope;
- strict policy;
- typed error root;
- architecture-owned IDs used by public errors.

Guide 8 may later expose `RequiredCapability` and a narrow target-requirement projection when the tapscript adapter becomes the first real consumer.

## 26.3 No public constructor

There must be no public API allowing external callers to assemble a purported complete analysis from independent fields.

Construction remains through the crate-private orchestrator and complete validator.

Compile-fail documentation may be used to demonstrate that private fields cannot be assigned externally once the type becomes reachable inside crate documentation.

---

# 27. Identity and schema impact

Expected Guide-7 identity impact:

```text
Attestation version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

architecture publications:
    byte-identical

realization public identity:
    none

compiler-plan identity:
    not minted

target identity:
    absent

linked-bundle identity:
    absent

transaction ABI identity:
    absent

deployment profile identity:
    remains dormant

release identity:
    absent
```

## 27.1 Why no compiler identity is minted

The analyzed value has no persistent:

- cross-process consumer;
- cache;
- publication;
- signature;
- target package;
- release binding.

Therefore:

```text
typed comparison is sufficient
```

under ADR-016.

Do not add:

```text
CompilerPlanId
compiler_plan_hash
analysis_digest
candidate_digest
```

## 27.2 No generated compiler publication

Do not add:

```text
analyzed-program.json
compiler-plan.toml
coverage-publication.json
```

Tests compare typed stable projections directly.

A publication is reconsidered only when a real external consumer appears and its schema, identity recipe, migration policy, and non-writing checker are reviewed.

---

# 28. Dependency impact

Expected:

```text
new first-party package:
    none

new third-party dependency:
    none

Cargo.lock:
    unchanged

Petgraph features:
    unchanged

parallelism:
    unchanged
```

If implementation suggests another dependency, stop and perform ADR-011 review rather than adding it opportunistically.

The analyzed-program assembly is ordinary typed Rust over existing component values. It does not require a new graph, solver, serialization, or concurrency dependency.

---

# 29. Exit criteria

Guide 7 is complete only when every applicable assertion below holds.

## 29.1 Preflight correctness

- [ ] CI explicitly runs the complete census and tracked-mode audit.
- [ ] `meson compile -C build lint` includes the census audit.
- [ ] mocked Meson contract proves the audit edge executes.
- [ ] root policy and observed root effect are separate.
- [ ] lifecycle relations and lifecycle graph declarations are exactly welded.
- [ ] placement validation rejects duplicate and nonminimal carrier assignments.
- [ ] placed proof-plan validation compares exact offered and placed sets.
- [ ] representation activation is object-specific.
- [ ] open-flow references validate side and uniqueness.
- [ ] deployment-profile hash boundary is validated or explicitly kept dormant and nonpublic.
- [ ] compiler status documentation agrees.

## 29.2 Relation-indexed requirements

- [ ] every compiler relation has exactly one relation requirement;
- [ ] every selected proof equals the proof-plan candidate;
- [ ] static relations carry no proof variable;
- [ ] external evidence retains its approved proof class;
- [ ] relation capabilities union exactly to candidate capabilities;
- [ ] relation sources union exactly to candidate sources;
- [ ] relation evidence unions exactly to candidate evidence;
- [ ] selected representations are object-specific;
- [ ] disclosure and lifecycle rows agree with the candidate;
- [ ] sponsor amounts are absent.

## 29.3 Factorized analysis

- [ ] proof plans remain global typed candidates;
- [ ] operation factors exactly equal compiler scope;
- [ ] execution cases are operation-local;
- [ ] relation-case plans are operation-local;
- [ ] placements are operation-local;
- [ ] layout requirements are operation-local;
- [ ] coverage is operation-local;
- [ ] coverage dependencies are operation-local;
- [ ] no combined placement product is stored.

## 29.4 Placement and layout

- [ ] every active runtime relation-case has an eligible carrier;
- [ ] every stored placement validates exactly;
- [ ] every accepted carrier assignment is inclusion-minimal under policy;
- [ ] every-member authorization remains quantified per member;
- [ ] global relations have complete carriers;
- [ ] unconditional relations do not rely on optional sponsor carriers;
- [ ] exact selected layout requirements are present;
- [ ] no surplus layout requirements are present;
- [ ] every feasible operation placement is retained.

## 29.5 Coverage

- [ ] coverage relation-case census is exact;
- [ ] positive requirements are complete;
- [ ] negative mutation classes are exact;
- [ ] inactive-valid requirements are complete;
- [ ] accepted semantic projections are complete;
- [ ] carrier alternatives match placements;
- [ ] evidence roles match their boundaries;
- [ ] dependency collateral matches exact closure;
- [ ] coverage symbols resolve;
- [ ] coverage graph remains acyclic;
- [ ] conditional triplets are complete;
- [ ] representation census is complete across the plan set.

## 29.6 Scoped analyzed program

- [ ] source binding is exact;
- [ ] architecture-scope status is explicit and correct;
- [ ] proof-plan set is exact;
- [ ] every proof plan has every operation factor;
- [ ] every relation has a relation requirement;
- [ ] every relation-case has a requirement bundle;
- [ ] lifecycle incompleteness is explicit;
- [ ] external evidence is unresolved and explicit;
- [ ] stable projection is deterministic;
- [ ] execution statistics are excluded from semantic projection;
- [ ] no target-specific type enters the value.

## 29.7 Independent evidence

- [ ] direct assembly census oracle agrees;
- [ ] requirement-union oracle agrees;
- [ ] synthetic product-factor oracle agrees;
- [ ] manual full pilot product comparison agrees;
- [ ] corruption cases are rejected by validator and disagree with oracle;
- [ ] permutation tests pass;
- [ ] sufficient-limit invariance passes;
- [ ] ordinary compiler suite no longer pays the unnecessary combined-product cost.

## 29.8 Identity and security

- [ ] no compiler digest is minted;
- [ ] no generated compiler publication is added;
- [ ] no local graph handle enters stable projection;
- [ ] no vector position becomes identity;
- [ ] no source path or line becomes identity;
- [ ] no sponsor amount appears;
- [ ] no secret-bearing field is added;
- [ ] no target or deployment readiness is claimed.

## 29.9 Repository gate

- [ ] MSRV `scripts/ci.sh` run recorded;
- [ ] current-stable `scripts/ci.sh` run recorded;
- [ ] mocked Meson contract is required, not skipped;
- [ ] canonical Meson compile passes;
- [ ] canonical Meson tests pass;
- [ ] document reproducibility passes;
- [ ] advisory lane passes or is loudly recorded as skipped;
- [ ] generated checks pass;
- [ ] plan and label checks pass;
- [ ] `git diff --check` passes;
- [ ] final tree is clean.

---

# 30. Completion report template

```text
Guide 7 result
==============

Preflight corrections:
    census audit in lint:
    mocked Meson census edge:
    root effect/policy separation:
    lifecycle relation/graph weld:
    exact placement validation:
    exact placed-plan validation:
    representation activation:
    open-flow observation validation:
    deployment hash boundary:
    documentation reconciliation:

Scoped analyzed-program model:
    source binding:
    architecture scope status:
    foundation projection:
    proof-plan census:
    operation-factor storage:
    lifecycle status:
    external evidence:
    stable projection:
    public visibility:

Relation-indexed requirements:
    relation census:
    proof disposition:
    capability closure:
    source closure:
    evidence closure:
    representation closure:
    disclosure closure:
    lifecycle closure:
    sponsor opacity:

Operation factorization:
    compact-ASH cases:
    compact-ASH placements:
    compact-ASH layout:
    compact-ASH coverage:

    live-transfer cases:
    live-transfer placements:
    live-transfer layout:
    live-transfer coverage:

    combined product semantics:
    combined product stored:
        no
    structural factorization checks:
    synthetic product oracle:
    manual full pilot product comparison:
    ordinary-suite wall-time change:

Coverage closure:
    relation-case census:
    positive requirements:
    negative requirements:
    inactive-valid requirements:
    carrier alternatives:
    accepted projections:
    external evidence:
    dependency graph:
    collateral closure:
    cycle policy:

Independent assembly oracle:
    direct operation census:
    direct proof-plan census:
    direct case census:
    direct relation-case census:
    requirement-union oracle:
    corruption cases:
    permutation equality:
    sufficient-limit invariance:

Pilot results:
    compact ASH:
    live transfer:
    combined scoped analysis:
    architecture scope:
    lifecycle completeness:
    target selected:
    deployment readiness:

Planning result:
    P2-012:
    P2-013:
    C1-014:
    Phase 2:
    Phase 3:
    next task:

Identity impact:
    Attestation version:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:
    compiler publication:
    deployment profile identity:

Dependency impact:
    new dependencies:
    Cargo.toml:
    Cargo.lock:
    Petgraph features:

Verification:
    MSRV rustc:
    MSRV cargo:
    MSRV CI_REQUIRE_MESON=1 scripts/ci.sh:

    stable rustc:
    stable cargo:
    stable CI_REQUIRE_MESON=1 scripts/ci.sh:

    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:

    compiler package tests:
    compiler Rustdoc:
    realization package tests:
    model conformance tests:
    labels package tests:

    full factorization regression:
    scripts/check-plans.sh:
    meson compile -C build lint:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    scripts/check-document-reproducibility.sh:
    cargo audit:
    cargo tree --locked -e features:
    cargo metadata --locked:
    git diff --check:
    git diff --cached --check:
    final git status:

Skipped or deferred:
    cargo audit:
    document reproducibility:
    other:

Residuals:
```

---

# 31. Phase-2 completion statement

If every exit criterion passes, the durable result should say:

```text
Phase 2 result
==============

The target-independent compiler now constructs one deterministic,
validated scoped analyzed program for the complete Phase-1 pilot scope.

The result retains:
    exact feasible proof plans;
    relation-owned capabilities, sources, and evidence;
    operation-local cases, placements, layout, and coverage;
    explicit partial architecture scope;
    explicit lifecycle obligations outside compiler scope;
    unresolved external substrate evidence.

The result does not contain:
    target opcodes;
    target positions;
    target programs;
    a linked bundle;
    a transaction ABI;
    deployment evidence;
    a compiler digest;
    a compiler publication.

Placement and coverage are factorized per operation. The global Cartesian
product remains mathematically exact and independently tested but is not
materialized in the analyzed-program value.

Phase 2 is complete.
Phase 3 may begin.
```

---

# 32. Next guide

After Guide 7 and the complete Phase-2 exit gate pass, begin:

```text
Guide 8 — Typed Elements Target Contract and Capability-Adapter Foundation
```

Guide 8 should initially implement:

- `tripod-target-elements`;
- typed execution domain and leaf version;
- reviewed opcode, encoding, failure, and resource contracts;
- target evidence requirements;
- explicit development deployment declaration;
- the smallest reviewed compiler target-requirement public boundary;
- `tripod-tapscript`;
- a compiler-to-target capability adapter;
- no attestation-contract operation emission yet;
- no target hash without a real consumer;
- no production activation claim.

The first Phase-3 work must consume the Guide-7 analyzed boundary without forcing target types back into compiler core.
