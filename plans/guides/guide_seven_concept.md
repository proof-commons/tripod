# Conceptual plan — Guide 7: Complete Scoped Analyzed Programs and Phase-2 Exit

## Executive decision

Guide 7 should complete:

```text
P2-012  compact-ASH and live-transfer analyzed pilots
P2-013  Phase-2 evidence and exit
C1-014  compiler-preparation review and handoff
```

It should assemble the already implemented compiler stages into one internally validated, deterministic, **complete-for-its-explicit-scope** analyzed value.

The governing pipeline is:

```text
validated compiler input
    ↓
relation/expression foundation
    ↓
exact feasible proof plans
    ↓
constructibility, disclosure, and lifecycle analyses
    ↓
execution cases
    ↓
relation-case discharge plans
    ↓
factorized abstract placements
    ↓
layout requirements
    ↓
relation-indexed coverage requirements
    ↓
complete scoped analyzed program
```

Guide 7 should not add new planning algorithms unless integration reveals a real omission. Its central problem is **typed assembly and cross-component closure**, not another search.

The most important design decision is:

> Store placement and coverage per operation. Do not materialize the Cartesian product of independent operation placements in the analyzed-program value.

For proof plan \(p\) and operations \(o\), the complete global placement semantics remain:

\[\mathcal P_p=\prod_{o\in O}\mathcal P_{p,o}\]

but the analyzed program stores the factors \(\mathcal P_{p,o}\), not every member of the product.

---

# 1. Entry conditions

Guide 7 begins only after Guide 6 is complete and merged.

Expected entry state:

```text
P2-007  DONE  exact proof planning
P2-008  DONE  source, constructibility, disclosure
P2-009  DONE  representation lifecycle
P2-010  DONE  cases, placement, layout
P2-011  DONE  relation-indexed coverage

C1-008  DONE  exact proof-plan search
C1-009  DONE  execution-case placement
C1-010  DONE  typed coverage symbols and SCC policy
C1-013  DONE  placement and coverage oracles
```

Before implementation:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean and must contain the completed Guide-6 gate record.

If Guide 6 is not complete, Guide 7 remains conceptual and must not begin implementation.

---

# 2. Mission and non-goals

## 2.1 Mission

Guide 7 must produce one target-independent compiler value that proves, for its explicit scope:

- every source relation was retained;
- every proof-required relation has exact feasible alternatives;
- every selected proof has capabilities and authenticatable sources;
- every operation has its exact semantic cases;
- every relation-case has an explicit discharge disposition;
- every active runtime relation-case has feasible abstract placements;
- every selectable placement has complete layout requirements;
- every relation-case has complete coverage requirements;
- every external premise remains visibly unresolved;
- every lifecycle obligation is represented, including exits outside compiler scope;
- all cross-stage censuses agree;
- stable projections are deterministic;
- no local handle or target-specific detail leaks into the result.

## 2.2 Non-goals

Guide 7 must not implement:

- `tripod-target-elements`;
- a concrete target-capability adapter;
- tapscript emission;
- target programs or program identities;
- transaction input/output positions;
- stack, altstack, witness, tapleaf, or control-block positions;
- transaction ABI;
- concrete semantic fixtures or malformed target transactions;
- target execution;
- linked bundles;
- resource calibration;
- deployment reports;
- release manifests;
- a public realization identity;
- a public compiler-plan identity;
- a generated compiler publication;
- a cross-process cache;
- one selected “best” proof or placement plan.

No new dependency is expected.

---

# 3. Use “scoped analyzed program,” not an unqualified “complete program”

The realization covers only:

```text
compact-ash
transfer-live-receipts
```

The architecture declares thirteen operations.

Therefore Guide 7 should avoid a type or status suggesting full architecture
coverage.

Preferred terminology:

```rust
ScopedAnalyzedProgram
ScopedAnalyzedProgramProjection
AnalyzedProofPlan
AnalyzedOperationPlan
```

Avoid:

```rust
CompleteAnalyzedProgram
FinalAnalyzedProgram
ReleaseAnalyzedProgram
```

unless “complete” is explicitly qualified as complete for the requested scope.

The analyzed value should carry a typed scope status:

```rust
enum ArchitectureScopeStatus {
    Complete,
    Partial {
        missing: Vec<architecture::OperationId>,
    },
}
```

For the Phase-2 pilot scope, it must report `Partial` with the eleven
architecture operations outside the realization/compiler scope.

This is not an error. It is an honesty property.

---

# 4. Proposed analyzed-program structure

A useful conceptual structure is:

```rust
struct ScopedAnalyzedProgram {
    source: AnalyzedSourceBinding,
    foundation: CompilerAnalysisFoundationProjection,
    scope_status: ArchitectureScopeStatus,
    proof_plans: BTreeMap<ProofPlanCandidate, AnalyzedProofPlan>,
    lifecycle_status: LifecycleCompleteness,
    external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}
```

with:

```rust
struct AnalyzedProofPlan {
    operations: BTreeMap<architecture::OperationId, AnalyzedOperationPlan>,
}
```

and:

```rust
struct AnalyzedOperationPlan {
    operation: architecture::OperationId,
    execution_cases: BTreeSet<ExecutionCaseId>,
    relation_requirements:
        BTreeMap<RelationCaseKey, RelationRequirementBundle>,
    feasible_placements:
        BTreeSet<PlacementCandidateProjection>,
    layout_requirements:
        BTreeSet<LayoutRequirement>,
    coverage:
        OperationCoverageProjection,
}
```

Exact names and field boundaries remain implementation-owned. The conceptual
requirements are:

1. proof plans are keyed by their complete typed value;
2. operations are keyed by architecture-owned IDs;
3. relation-case entries are keyed by stable relation and case identities;
4. placement is stored per operation;
5. coverage is stored per operation;
6. vector/search order is not observable;
7. no digest is required.

## 4.1 Do not duplicate the combined placement product

Guide 5 found:

```text
compact ASH factor:
    36 placements per proof plan

live transfer factor:
    216 placements per proof plan

combined materialized product:
    36 × 216 = 7776
```

Guide 7 must not store all 7,776 combinations.

Instead:

```text
AnalyzedProofPlan
    ├── compact-ash
    │     └── 36 feasible operation placements
    └── transfer-live-receipts
          └── 216 feasible operation placements
```

The product remains derivable if a later target planner needs it.

This should be a permanent storage rule, not a pilot-only optimization.

## 4.2 Do not refer to proof plans by position

No analyzed value should use:

```text
plan 0
plan 1
candidate index
search rank
first feasible plan
```

Use the complete typed `ProofPlanCandidate` as the map key until ADR-016 admits
a persistent compiler-plan identity.

---

# 5. Source binding and provenance

## 5.1 The analyzed value must bind its exact typed source

The source binding should include enough typed information to prevent two
different scoped realizations from appearing to be the same analysis merely
because they share an architecture hash and operation list.

A suitable shape is:

```rust
struct AnalyzedSourceBinding {
    architecture: realization::ArchitectureBinding,
    realization: realization::ScopedRealizationProjection,
    compilation_scope: CompilationScope,
}
```

The complete realization projection may be retained directly or represented by
an equivalent complete typed source projection.

Do not replace it with:

- a source path;
- a source-tree hash;
- a generated publication;
- a realization digest that has no consumer;
- the architecture hash alone.

## 5.2 Analysis policy and work limits

Search limits are explicit inputs and should remain available as execution
provenance, but generous limits must not alter the semantic projection.

Distinguish:

```text
analysis result:
    feasible typed values

analysis report:
    limits, state counts, candidate counts, pruning diagnostics
```

Search statistics and work limits should normally be excluded from the stable
semantic projection because:

```text
larger sufficient limit
    ⇒ same complete feasible result
```

A limit may cause a typed failure. It must not change a successful complete
result.

Add a test showing that two sufficiently large limit configurations produce
equal analyzed-program projections.

---

# 6. Relation-indexed abstract target requirements

Guide 7 should close one remaining integration gap: candidate-level
capabilities and sources are currently aggregate sets. The complete analyzed
program should retain their relation provenance.

## 6.1 Requirement bundle

For each relation-case, derive a typed bundle such as:

```rust
struct RelationRequirementBundle {
    key: RelationCaseKey,
    activity: RelationActivity,

    selected_proof: Option<realization::ProofAlternativeId>,
    representation:
        Option<realization::RepresentationMode>,

    required_capabilities:
        BTreeSet<RequiredCapability>,
    source_requirements:
        BTreeSet<SourceRequirement>,

    compiler_requirements:
        Vec<CompilerStaticRequirement>,
    structural_requirements:
        Vec<BackendStructuralRequirement>,

    carrier_assignments:
        Option<CarrierAssignmentAlternatives>,
    layout_requirements:
        BTreeSet<LayoutRequirement>,

    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,
    coverage_requirements:
        BTreeSet<CoverageRequirementId>,
}
```

Equivalent factoring is acceptable.

This is still target-independent. It says what a later target or evidence
boundary must provide, not how.

## 6.2 Boundary behavior

### Proof-required runtime relation

Retain:

- selected proof alternative;
- required abstract capabilities;
- active source requirements;
- carrier assignment alternatives;
- layout requirements;
- coverage requirements.

### Compiler-static relation

Retain:

- compiler validation requirement;
- no runtime carrier;
- no target-execution claim.

### Backend-structural relation

Retain:

- structural requirement;
- coverage requirement;
- no claim that compiler analysis implemented the structure.

### External-evidence relation

Retain:

- approved proof class;
- abstract capability;
- typed external-evidence requirement;
- positive and negative evidence coverage;
- no runtime protocol carrier.

### Inactive relation-case

Retain:

- inactive-valid disposition;
- activation condition;
- no runtime carrier;
- no active source requirement;
- no active target-reject requirement.

## 6.3 Aggregate closure equations

For proof plan \(p\), relation-indexed requirements must exactly reconstruct the
candidate aggregates:

\[\operatorname{Capabilities}(p)=\bigcup_{r}\operatorname{Capabilities}(p,r)\]

\[\operatorname{Sources}(p)=\bigcup_{r}\operatorname{Sources}(p,r)\]

\[\operatorname{Evidence}(p)=\bigcup_{r}\operatorname{Evidence}(p,r)\]

These equalities should be validation rules, not assumptions.

This protects against a recurrence of T6: a fixed external-evidence capability
cannot disappear from the candidate aggregate if it remains present in the
relation-indexed requirement.

## 6.4 Selected proof closure

For every proof-required relation:

```text
exactly one selected proof in candidate
↔
exactly one selected proof in relation requirement
```

For compiler-static representation/lifecycle relations:

```text
no selected proof
```

For external evidence:

```text
approved external proof class retained
runtime proof completion absent
```

Unexpected proof entries fail.

---

# 7. Factorized operation analysis

## 7.1 Derive operation factors from one global proof plan

For each feasible global `ProofPlanCandidate`:

1. derive the plan’s execution cases;
2. partition cases by operation;
3. classify relation cases per operation;
4. derive carrier eligibility per operation;
5. enumerate feasible placements per operation;
6. derive layout requirements per operation;
7. derive coverage requirements per operation;
8. construct one `AnalyzedOperationPlan` per operation.

The proof plan remains global because it is already the complete typed feasible
candidate. Only placements and coverage are factorized.

## 7.2 Exact operation scope

For every proof plan:

```text
operation keys
=
CompilationScope operations
=
RealizationScope operations
```

Any missing, duplicate, or unexpected operation factor is a hard assembly
failure.

## 7.3 Preserve the product theorem as a regression

The production analyzed value should not materialize the combined placement
product, but a focused test should prove that factor storage preserves the
existing semantics.

For the pilot scope:

1. derive the old combined feasible placement projection through the existing
   complete search, if that helper remains available;
2. derive the Cartesian product of the per-operation factors in the test;
3. compare the two sets exactly;
4. store only the factors in `ScopedAnalyzedProgram`.

If the old global enumerator is retired, retain an independent test-only
product oracle.

Do not encode the numeric values `36`, `216`, or `7776` as semantic constants.
Exact set equality is stronger and survives legitimate relation-census changes.

---

# 8. Lifecycle and completeness status

## 8.1 Scope completeness and deployment completeness are different

Guide 7 should represent at least these distinct facts:

```text
analysis complete for requested compiler scope:
    yes

realization covers every architecture operation:
    no

all lifecycle exits available inside compiler scope:
    no

target selected:
    no

target evidence complete:
    no

deployment ready:
    no
```

Do not collapse them into one `complete: bool`.

## 8.2 Lifecycle status

Use existing lifecycle requirements to derive a typed status:

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
compact-ash:
    in compiler scope

clear:
    outside compiler scope
```

For live receipts:

```text
transfer-live-receipts:
    in compiler scope

burn:
    outside compiler scope

redeem:
    outside compiler scope
```

The scoped analysis remains valid. Deployment lifecycle remains incomplete.

## 8.3 External evidence status

The analyzed program should expose the required external evidence set as
requirements only.

It must not carry:

```text
passed
verified
complete
```

because no target or deployment evidence exists.

A suitable method is:

```rust
fn required_external_evidence(
    &self,
) -> impl Iterator<Item = &ExternalEvidenceRequirement>;
```

No raw report hashes should enter.

---

# 9. Complete assembly validator

The analyzed value should not be trusted merely because the assembler produced
it.

Implement a validator that rechecks the assembled object against its typed
source and component analyses.

## 9.1 Source and scope

Require:

- source realization validates against architecture;
- architecture binding matches;
- compilation scope is nonempty and canonical;
- operation factors exactly equal compiler scope;
- partial architecture status exactly names all missing architecture
  operations.

## 9.2 Relation and expression foundation

Require:

- relation graph census equals the scoped realization relation census;
- expression graph census equals the scoped realization expression closure;
- every relation dependency endpoint exists;
- every expression predicate binding remains valid;
- stable source declarations equal the validated foundation projection.

## 9.3 Proof-plan census

Require:

- every feasible proof plan appears exactly once;
- no infeasible or unexpected plan appears;
- every proof alternative is realization-approved;
- representation/proof compatibility holds;
- candidate capabilities, sources, disclosure, lifecycle, and evidence match
  their component analyses.

The validator should compare against the exact feasible-plan result, not assume
the assembled map is complete.

## 9.4 Per-operation plan closure

For every plan and operation:

- exact execution-case census;
- exact relation-case census;
- exact activity/discharge dispositions;
- exact carrier eligibility;
- every feasible placement validates;
- exact layout requirement census;
- exact relation-indexed coverage census;
- exact abstract target-requirement census;
- no cross-operation relation or case leaks into the factor.

## 9.5 Carrier and layout closure

Require:

- carrier coverage only for active runtime relations;
- every selectable assignment is Guide-5 feasible;
- every selected assignment’s layout dependencies are present;
- non-runtime relations have no runtime carrier;
- optional sponsor carriers never uniquely carry unconditional relations;
- owner authorization remains every-member;
- no combined placement product is stored.

## 9.6 Coverage closure

Require:

- every relation-case has coverage;
- active relations have applicable positive and negative requirements;
- inactive relations have inactive-valid coverage;
- runtime relations have accepted-projection requirements;
- conditional triplets are complete across the case set;
- coverage dependency graph is acyclic under current policy;
- coverage symbols resolve;
- coverage collateral matches the dependency closure.

## 9.7 Sponsor opacity

Traverse the complete analyzed value and fail if any component names:

- sponsor input amount;
- sponsor output amount;
- sponsor positivity;
- public sponsor sum;
- sponsor opening or blinding factor;
- exact sponsor value source.

Reuse `CompileError::SponsorValueRead` where appropriate.

Sponsor-related requirements may retain:

- family membership;
- cardinality;
- owner authorization;
- region disjointness;
- envelope multiplicity;
- external whole-transaction conservation evidence.

---

# 10. Constructor and mutation resistance

## 10.1 Private construction

The analyzed-program constructor should remain crate-private.

Fields should be private or crate-private so external callers cannot assemble a
partial analyzed program and present it as compiler output.

A future public constructor should be added only with the first real backend or
publication consumer.

## 10.2 Corruption tests

Inside compiler unit tests, clone and deliberately corrupt assembled values.

Required mutations include:

- remove one analyzed operation;
- add an unexpected operation;
- remove one feasible proof plan;
- duplicate one proof plan;
- remove one execution case;
- add a case for the wrong operation;
- remove one relation-case requirement;
- change a selected proof;
- remove one required capability;
- remove one source requirement;
- remove one external-evidence requirement;
- add a runtime carrier to an external relation;
- remove one feasible operation placement;
- insert a placement from another operation;
- remove one required layout requirement;
- remove one positive coverage requirement;
- remove one negative coverage requirement;
- remove one inactive-valid requirement;
- alter lifecycle incompleteness status;
- claim full architecture scope;
- add a sponsor amount requirement.

Each mutation should fail for a focused typed reason.

---

# 11. Stable projection

## 11.1 Projection contents

The stable projection should contain only typed semantic and analysis values:

- architecture binding;
- scoped realization projection;
- compilation scope;
- partial architecture status;
- relation/expression foundation projection;
- exact feasible proof-plan keys;
- per-operation execution-case sets;
- relation-case requirement bundles;
- per-operation feasible placement projections;
- per-operation layout requirements;
- per-operation coverage projections;
- lifecycle obligations;
- external-evidence requirements.

## 11.2 Projection exclusions

Exclude:

- `NodeIndex`;
- `EdgeIndex`;
- graph insertion order;
- topological traversal order unless semantically required;
- proof-search state count;
- placement-search state count;
- elapsed time;
- worker/thread scheduling;
- source file path or line;
- temporary path;
- candidate vector position;
- full materialized operation-placement product;
- diagnostics;
- hashes.

## 11.3 Canonical containers

Prefer:

```text
BTreeMap
BTreeSet
canonically sorted typed vectors
```

The stable projection should not depend on caller-provided vector order.

## 11.4 Determinism properties

Test equality under:

- proof-plan candidate permutation;
- operation ordering permutation;
- relation declaration permutation;
- expression declaration permutation;
- relation dependency permutation;
- execution-case permutation;
- placement candidate permutation;
- carrier ordering permutation;
- layout ordering permutation;
- coverage requirement permutation;
- repeated clean analysis.

Also test:

```text
larger sufficient proof-search limits
larger sufficient placement-search limits
    ⇒ equal stable projection
```

---

# 12. Public API and identity ruling

## 12.1 Recommended public boundary

Guide 7 should keep `ScopedAnalyzedProgram` crate-private.

The public compiler API should remain:

```text
validated input boundary
typed error root
stable architecture-owned IDs appearing in public errors
```

The complete analysis is demonstrated internally but not exposed until a real
consumer exists.

This is preferable because:

1. no target/backend package currently consumes it;
2. exact public fields would be frozen prematurely;
3. no cross-process or publication boundary exists;
4. ADR-016 rejects identities without consumers;
5. Phase 3 may reveal the narrow target-adapter API actually needed.

## 12.2 No compiler-plan identity

Do not add:

```text
CompilerPlanId
compiler_plan_hash
analysis_digest
candidate_digest
```

Typed comparison remains the boundary.

The completion record should state:

```text
compiler identity:
    not minted

reason:
    no persistent cross-process, cache, publication, or target consumer exists
```

## 12.3 No generated analysis artifact

Do not add a JSON or TOML compiler analysis publication merely to inspect the
result.

Tests may compare stable typed projections directly.

A publication is reconsidered only when a real consumer or review boundary
requires it.

---

# 13. Suggested source organization

A clean implementation may add:

```text
packages/compiler/src/requirement.rs
packages/compiler/src/analyzed.rs
```

or equivalent names.

## `requirement.rs`

Own:

- relation-indexed abstract target requirements;
- capability/source/evidence closure;
- proof and representation join;
- relation-case requirement bundles;
- aggregate-union validation.

## `analyzed.rs`

Own:

- `ScopedAnalyzedProgram`;
- `AnalyzedProofPlan`;
- `AnalyzedOperationPlan`;
- factorized assembly;
- completeness and lifecycle status;
- stable projection;
- full assembly validator;
- internal orchestration entry point.

Tests:

```text
packages/compiler/src/tests/requirement_tests.rs
packages/compiler/src/tests/analyzed_program_tests.rs
packages/compiler/src/tests/analyzed_program_oracle_tests.rs
```

Every new source file must be added to:

```text
packages/compiler/meson.build
packages/compiler/src/tests/mod.rs
```

in the same commit.

Do not create a generic shared “analysis graph” or “compiler IR” crate.

---

# 14. Suggested internal entry point

A crate-private entry point might be:

```rust
pub(crate) fn analyze_scoped_program(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<ScopedAnalyzedProgram, CompileError>;
```

The complete scoped analysis should use:

```rust
CapabilityView::Unconstrained
```

because its purpose is to retain every realization-approved abstract plan.

A target-constrained result is a later target-planning value, not the canonical
Phase-2 analyzed program.

If the function accepts a capability view, distinguish the result type:

```text
unconstrained complete analysis
target-pruned plan
```

Do not let a constrained view silently masquerade as the complete compiler
analysis.

---

# 15. Analysis orchestration

The production orchestration should be explicit and boring.

Conceptually:

```text
1. validate bound compiler input
2. analyze relation/expression foundation
3. build constructibility analysis
4. build lifecycle analysis
5. enumerate the complete unconstrained feasible proof-plan set
6. derive relation-indexed requirement bundles
7. for each proof plan:
       for each operation:
           derive execution cases
           classify relation cases
           derive carrier eligibility
           enumerate operation-local placements
           derive layout requirements
           derive relation-indexed coverage
           validate operation factor
8. derive scope and lifecycle status
9. aggregate external evidence
10. construct scoped analyzed program
11. validate the complete assembled value
12. return it
```

Do not hide stage failures behind generic “analysis failed.”

---

# 16. Independent assembly oracle

Guide 7 should not reimplement every prior algorithm. Guide 5 and Guide 6
already provide independent search and coverage oracles.

Guide 7 needs an independent **assembly census oracle**.

## 16.1 Oracle purpose

The oracle verifies that the final object contains exactly the outputs the
component analyses require.

It does not need to independently solve proof planning or placement again.

## 16.2 Direct expected censuses

Independently derive:

```text
expected operations:
    compilation scope

expected proof plans:
    exact feasible proof-plan set

expected cases:
    candidate × operation case derivation

expected relation-case keys:
    relation scope × applicable cases

expected runtime carrier keys:
    active runtime relation-case keys

expected layout references:
    union required by selectable operation-local assignments

expected coverage keys:
    complete relation-case keys

expected target-requirement keys:
    complete relation-case keys

expected external evidence:
    union of external-evidence dispositions
```

Compare direct sets with assembled program projections.

## 16.3 Do not use production assembler output as expectation

The oracle may call prior trusted component functions, but it should not derive
the expected census from the assembled object it is checking.

For example:

```text
wrong:
    expected relation cases = analyzed_program.relation_cases.keys()

right:
    expected relation cases =
        relation graph IDs × independently derived applicable case IDs
```

## 16.4 Product-factor oracle

For the two-pilot scope:

- independently calculate the Cartesian product of operation-local placement
  factors in the test;
- compare it with the old global placement result or a test-only global
  enumerator;
- assert the analyzed program stores only the factors.

---

# 17. Error vocabulary

Add only errors reached by real assembly validation paths.

Likely variants include:

```rust
AnalyzedOperationCensusMismatch {
    missing: Vec<architecture::OperationId>,
    unexpected: Vec<architecture::OperationId>,
}

AnalyzedProofPlanCensusMismatch {
    missing: usize,
    unexpected: usize,
}

AnalyzedRelationCaseCensusMismatch {
    operation: architecture::OperationId,
    missing: Vec<RelationCaseKey>,
    unexpected: Vec<RelationCaseKey>,
}

AnalyzedRequirementCensusMismatch {
    operation: architecture::OperationId,
    missing: Vec<RelationCaseKey>,
    unexpected: Vec<RelationCaseKey>,
}

AnalyzedCapabilityClosureMismatch {
    // Avoid carrying the full proof plan in the public diagnostic.
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

AnalyzedScopeStatusMismatch,
```

Because `CompileError` is public and non-exhaustive, avoid storing a huge
`ProofPlanCandidate` directly in a public error variant unless there is a
compelling downstream need.

A focused typed operation/relation/case is preferable to a generic
`InvalidAnalyzedProgram`.

---

# 18. Pilot acceptance matrix

## 18.1 Compact ASH

The analyzed operation must include, under every feasible compact-ASH proof
plan:

- explicit or public-committed ASH representation selected by the plan;
- sponsorless and sponsored cases;
- ASH input/output cardinality;
- sponsor input/output cardinality after Guide-6 correction;
- ASH input/output recognition;
- input/output family closure;
- permissionless constructibility;
- backend-structural secret-free path;
- ownerless `U` conservation;
- canonical ownerless-lateral delta;
- sponsor isolation and multiplicity;
- fee-sponsor-only open-flow policy;
- no-root policy;
- transition-certificate-only projection;
- compact and clear lifecycle obligations;
- whole-transaction-conservation capability;
- unresolved substrate-conservation evidence;
- operation-local placement factors;
- layout requirements;
- complete relation-indexed coverage.

## 18.2 Live transfer

The analyzed operation must include, under every feasible live-transfer proof
plan:

- explicit/public-arithmetic or private/confidential-conservation strategy;
- sponsorless and sponsored cases;
- live input/output cardinality and recognition;
- sponsor input/output cardinality and recognition;
- every-owner authorization;
- live input/output family closure;
- aggregate `U` conservation;
- canonical lateral delta;
- sponsor isolation and multiplicity;
- fee-sponsor-only open-flow policy;
- no-root policy;
- transition-certificate-only projection;
- owner constructibility;
- transfer, burn, and redeem lifecycle obligations;
- selected representation encoding requirement;
- whole-transaction-conservation capability;
- unresolved substrate-conservation evidence;
- operation-local placement factors;
- layout requirements;
- complete relation-indexed coverage.

## 18.3 Combined scoped program

The two-operation result must establish:

```text
operations:
    exactly compact-ash and transfer-live-receipts

proof plans:
    exact unconstrained feasible set

placements:
    factors stored per operation

coverage:
    stored per operation

cross-operation relation dependencies:
    none

architecture scope:
    explicit partial scope

deployment lifecycle:
    explicitly incomplete

external evidence:
    explicitly unresolved
```

---

# 19. Guide-7 implementation waves

## Wave 1 — Relation-indexed abstract requirements

Deliver:

- relation requirement bundle types;
- selected proof and representation join;
- per-relation capabilities and sources;
- external-evidence requirement retention;
- aggregate closure equations;
- focused tests for T6/T7 non-regression.

Suggested commit:

```text
compiler: derive relation-indexed target requirements
```

## Wave 2 — Factorized scoped analyzed-program types

Deliver:

- `ScopedAnalyzedProgram`;
- `AnalyzedProofPlan`;
- `AnalyzedOperationPlan`;
- source binding;
- partial architecture status;
- lifecycle completeness status;
- operation-local placement and coverage storage;
- stable projection.

Suggested commit:

```text
compiler: assemble factorized scoped analyses
```

## Wave 3 — Complete assembly validator

Deliver:

- scope validation;
- exact proof-plan census;
- operation-factor census;
- relation-case closure;
- capability/source/evidence union closure;
- placement/layout/coverage closure;
- sponsor-opacity traversal;
- focused corruption tests.

Suggested commit:

```text
compiler: validate analyzed-program closure
```

## Wave 4 — Independent assembly and factorization oracle

Deliver:

- direct assembly census oracle;
- product-factor oracle;
- sufficient-limit invariance;
- permutation determinism;
- synthetic missing/extra component tests.

Suggested commit:

```text
compiler: add analyzed-program assembly oracle
```

## Wave 5 — End-to-end pilot integration

Deliver:

- compact-ASH complete scoped analysis;
- live-transfer complete scoped analysis;
- combined two-pilot scoped analysis;
- exact projection equality under repeated analysis;
- explicit partial-scope and lifecycle status;
- unresolved evidence census.

Suggested commit:

```text
compiler: analyze both pilots end to end
```

## Wave 6 — Boundary review and documentation

Deliver:

- keep analyzed-program API crate-private;
- document why no compiler digest is admitted;
- update compiler Rustdoc and README;
- update compiler package contract;
- update Phase-2 card;
- update backlog;
- mark P2-012, C1-014, and P2-013 only after the required gate runs.

Suggested commit:

```text
plans: record the Phase-2 compiler exit
```

Each worker tranche should be committed as soon as it is green. Keep the
standing heartbeat loop active while delegated workers run and stop it at wave
boundaries.

---

# 20. Focused tests

## 20.1 Requirement closure

```sh
cargo test --locked -p tripod-compiler requirement
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler source
```

## 20.2 Factorized assembly

```sh
cargo test --locked -p tripod-compiler analyzed
cargo test --locked -p tripod-compiler pilot
```

## 20.3 Assembly oracle

```sh
cargo test --locked -p tripod-compiler analyzed_program_oracle
cargo test --locked -p tripod-compiler oracle
```

Use actual test names. Verify that a filter did not accidentally match zero
tests.

## 20.4 Compiler package and Rustdoc

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-compiler --no-deps
```

## 20.5 Realization non-regression

```sh
cargo test --locked -p tripod-realization
```

---

# 21. Working cadence

After each coherent wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Do not run the full Meson/document surface after every small edit.

Every new Rust source must be added to:

```text
packages/compiler/meson.build
```

in the same commit.

No generated compiler output should be introduced.

---

# 22. Phase-2 exit gate

Because Guide 7 closes Phase 2 rather than merely completing one intermediate
task, P2-013 should use the stronger phase-exit evidence surface.

## 22.1 Required Rust toolchains

Run the complete Rust gate under:

```text
declared MSRV:
    Rust 1.88

current stable:
    current stable toolchain
```

Record exact toolchain versions.

For each required toolchain:

```sh
CI_REQUIRE_MESON=1 scripts/ci.sh
```

If running the mocked Meson contract twice is unnecessarily expensive, the
phase record may separate the Rust-only duplicated lanes from one required
Meson-contract run, but it must state the exact commands and not describe an
unrun lane as passed.

## 22.2 Canonical Meson build

Using only the canonical build directory:

```sh
meson compile -C build
meson test -C build --print-errorlogs
```

If `build/` does not exist:

```sh
meson setup build
```

Do not create another production build directory.

## 22.3 Document reproducibility

Because this is a phase-exit gate, run:

```sh
scripts/check-document-reproducibility.sh
```

even if paper inputs were unchanged, provided the required TeX environment is
available.

If it cannot run, P2-013 should not be described as a completely green
repository gate. Record it as deferred and decide explicitly whether Phase-2
policy permits that deferral. The safest rule is to leave P2-013 blocked until
the phase-exit reproducibility run is recorded.

## 22.4 Advisory lane

Run:

```sh
cargo audit
```

when installed.

If unavailable:

```text
SKIPPED
```

not passed. ADR-011 permits a loud skip, but the result is partial rather than
fully green.

## 22.5 Dependency evidence

Because Guide 7 should add no dependency:

```sh
cargo tree --locked -e features
cargo metadata --locked
git diff -- Cargo.toml Cargo.lock
```

Expected result:

```text
no dependency or lockfile change
```

## 22.6 Documentation and generated artifacts

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Expected generated-publication impact:

```text
architecture.json:
    unchanged

architecture.toml:
    unchanged

declassification.json:
    unchanged

model_labels.json:
    unchanged unless deliberate new Rust labels were minted

Specification and realization registers:
    unchanged unless labels were deliberately added
```

## 22.7 Final clean tree

After commits:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

---

# 23. Documentation result

After verification, update current documentation to say:

```text
Compiler input:
    implemented

Relation/expression foundation:
    implemented

Checked folding:
    implemented

Exact proof planning:
    implemented

Source/constructibility/disclosure/lifecycle:
    implemented

Cases/placement/layout:
    implemented

Relation-indexed coverage:
    implemented

Complete scoped pilot analysis:
    implemented internally

Public complete-analysis API:
    not exposed

Target adapter/backend:
    absent

Compiler-plan identity:
    not minted
```

Update:

```text
packages/compiler/src/lib.rs
packages/compiler/README.md
plans/packages/compiler.md
plans/phases/02-compiler.md
plans/backlog.md
plans/roadmap.md if the current phase advances
plans/phases/README.md if the phase status advances
```

If P2-013 fully passes, advance:

```text
Phase 2:
    Active → Complete

Phase 3:
    Planned → Active

Current gate:
    Phase 3 — Elements target and foundational prototypes
```

The active phase card and all current-phase declarations must move together
under the phase-declaration weld.

Do not advance Phase 3 if P2-013 remains partial or deferred under a policy that
requires complete phase-exit evidence.

---

# 24. Identity and schema impact

Expected Guide-7 impact:

```text
architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

architecture schema:
    unchanged

generated architecture publications:
    unchanged

realization public identity:
    none exists

compiler-plan identity:
    not minted

compiler public schema:
    no complete-analysis schema introduced

dependency graph:
    unchanged
```

The analyzed-program type is internal and typed. It does not create a
publication or identity merely by existing.

The completion record should cite ADR-016 explicitly:

```text
no persistent consumer
    ⇒ no digest admission
```

---

# 25. Guide-7 exit checklist

## Assembly

- [ ] one scoped analyzed-program type exists;
- [ ] construction is crate-private;
- [ ] source realization projection is bound;
- [ ] architecture scope status is explicit;
- [ ] proof-plan set is exact;
- [ ] every proof plan has every operation factor;
- [ ] no combined placement product is stored;
- [ ] coverage is stored per operation;
- [ ] stable projection is deterministic.

## Relation requirements

- [ ] every relation-case has one requirement bundle;
- [ ] selected proof matches candidate;
- [ ] representation matches candidate;
- [ ] relation capabilities union to candidate capabilities;
- [ ] relation sources union to candidate sources;
- [ ] relation evidence unions to candidate evidence;
- [ ] compiler-static, structural, runtime, external, and inactive boundaries
      remain distinct.

## Placement and layout

- [ ] operation-local feasible placements are exact;
- [ ] product-factor theorem remains tested;
- [ ] selectable carrier alternatives are preserved;
- [ ] layout requirements are complete;
- [ ] every-member authorization remains quantified;
- [ ] non-runtime relations have no runtime carrier.

## Coverage

- [ ] relation-case coverage census is exact;
- [ ] positive requirements are complete;
- [ ] negative requirements are complete;
- [ ] inactive-valid requirements are complete;
- [ ] accepted semantic projections are complete;
- [ ] conditional triplets are complete;
- [ ] coverage graph remains acyclic;
- [ ] typed symbols resolve.

## Completeness status

- [ ] analysis is complete for requested scope;
- [ ] architecture scope remains explicitly partial;
- [ ] lifecycle obligations outside scope remain explicit;
- [ ] external evidence remains unresolved;
- [ ] no target support is claimed;
- [ ] no deployment readiness is claimed.

## Security and identity

- [ ] sponsor amounts remain absent;
- [ ] no secret-bearing field is added;
- [ ] no target position enters compiler core;
- [ ] no local graph handle enters stable projection;
- [ ] no new digest is minted;
- [ ] no generated analysis publication is added;
- [ ] no new dependency enters.

## Evidence

- [ ] focused tests pass;
- [ ] independent assembly oracle agrees;
- [ ] workspace format passes;
- [ ] workspace Clippy passes;
- [ ] workspace debug tests pass;
- [ ] workspace release tests pass;
- [ ] required MSRV and stable lanes are recorded;
- [ ] Meson compile passes;
- [ ] Meson tests pass;
- [ ] generated and documentation checks pass;
- [ ] document reproducibility is passed or explicitly blocks full phase exit;
- [ ] advisory lane is passed or loudly skipped;
- [ ] final tree is clean.

---

# 26. Completion report template

```text
Guide 7 result
==============

Scoped analyzed-program model:
    source binding:
    architecture scope status:
    proof-plan census:
    operation-factor storage:
    stable projection:

Relation-indexed target requirements:
    proof closure:
    capability closure:
    source closure:
    evidence closure:
    boundary handling:

Placement factorization:
    compact ASH factors:
    live-transfer factors:
    product equivalence:
    materialized combined product stored:
        no

Coverage closure:
    relation-case census:
    positive requirements:
    negative requirements:
    inactive requirements:
    carrier requirements:
    projection requirements:

Lifecycle and evidence status:
    architecture scope:
    compiler scope:
    lifecycle completeness:
    required external evidence:
    target selected:
    deployment readiness:

Independent assembly oracle:
    direct census:
    factorization oracle:
    corruption cases:
    permutation equality:
    sufficient-limit invariance:

Pilot results:
    compact ASH:
    live transfer:
    combined scoped analysis:
    sponsor opacity:

Planning result:
    P2-012:
    P2-013:
    C1-014:
    Phase 2:
    next phase:

Identity impact:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:
    compiler publication:

Dependency impact:
    new dependencies:
    Cargo.lock:

Verification:
    MSRV toolchain:
    stable toolchain:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
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

# 27. What follows Guide 7

If Guide 7 and P2-013 fully pass, Phase 2 is complete.

The next implementation guide should begin Phase 3:

```text
Guide 8 — Typed Elements Target Contract and Backend Foundation
```

Its initial scope should be narrow:

```text
tripod-target-elements crate boundary
typed target-definition and deployment-instance separation
execution domain and leaf version
backend-used opcode and encoding registry
abstract compiler-capability adapter
target evidence requirement registry
development deployment binding
no operation emission yet
```

The first Phase-3 implementation should not immediately attempt STATE,
wide arithmetic, or a complete compact-ASH script. It should first replace
target assumptions with one validated typed target contract, preserving the
compiler/target dependency boundary demonstrated by Phase 2.
