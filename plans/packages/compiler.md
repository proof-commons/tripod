# Target-Independent Compiler Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/compiler`
> **Planned Cargo package:** `tripod-compiler`
> **Planned Rust library name:** `compiler`
> **Implementation phase:** Phase 2 — target-independent compiler analysis
> **Depends on package:** `tripod-realization`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies for Phase 2:** none
> **Later research dependencies:** target proof selection is refined by the
> state-constructor, wide-arithmetic, public-declassification, and settlement
> prototypes
> **Authority:** Target-independent implementation analysis subordinate to the
> typed realization
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `compiler` package will analyze a validated target-independent
`realization::RealizationSpec` and produce one deterministic,
target-independent compilation analysis.

It owns:

- realization-scope validation;
- semantic relation DAG construction;
- canonical relation normalization;
- constant folding;
- proof-alternative planning;
- semantic disclosure analysis;
- target-safety disclosure requirements;
- fact-source requirements;
- witness-availability analysis;
- permissionless constructibility checks;
- representation lifecycle reachability;
- obligation-placement requirements;
- canonical transaction-layout requirements;
- target-capability requirements;
- relation coverage requirements;
- source provenance;
- deterministic compiler-analysis identities.

It does not emit target code.

The planned direction is:

```text
validated RealizationSpec
        +
typed compilation policy
        ↓
target-independent compiler analysis
        ├── relation graph
        ├── proof-alternative graph
        ├── disclosure plan
        ├── fact-source requirements
        ├── constructibility analysis
        ├── lifecycle analysis
        ├── placement requirements
        ├── layout requirements
        ├── target capability requirements
        └── relation coverage obligations
        ↓
target capability matching and backend lowering
```

The Phase-2 compiler initially analyzes the two realization pilots:

1. `compact-ash`;
2. `transfer-live-receipts`.

The package must establish that those operations are completely represented
before the first production backend operation is emitted.

---

## 2. Role in the package graph

The compiler sits below `realization` and above target/backend packages.

```text
architecture
    ↓
realization
    ↓
compiler
   ├─────────────────────────────┐
   │                             │
   ▼                             ▼
target capability matching   evidence planning
   │                             │
   ▼                             ▼
backend lowering             relation coverage skeleton
```

### 2.1 Upstream

The compiler consumes:

```text
realization
```

It may use architecture IDs re-exported through realization or depend directly
on `architecture` where that makes type ownership clearer.

It must not reconstruct architecture facts from generated publications.

### 2.2 Downstream

The compiler's typed outputs are consumed by:

- `target-elements`, through an abstract target-capability boundary or adapter;
- `tapscript`;
- future `simplicity`;
- `linker`;
- `transaction`, indirectly through backend-derived layout and ABI output;
- `vectors`;
- `release`, through identities and analysis/report bindings.

### 2.3 Independence from the model

The compiler must not depend on `model` as its source of semantics.

The model and compiler are compared through:

```text
RealizationSpec
```

and later through relation-indexed vectors.

A separate vector or conformance package may depend on both.

### 2.4 Independence from concrete targets

The compiler must not depend directly on:

- `target-elements`;
- `tapscript`;
- `simplicity`;
- Elements libraries;
- Bitcoin Script libraries;
- target transaction libraries.

The compiler may define or consume an abstract typed target-capability
description.

A target-specific package may implement or adapt to that abstract interface
without creating a reverse dependency from compiler to the concrete target.

---

## 3. Normative inputs

### 3.1 Validated realization

Primary semantic input:

```rust
&realization::RealizationSpec
```

The compiler requires the realization to be:

- internally valid;
- architecture-bound;
- scope-explicit;
- deterministic;
- type-correct;
- complete for every operation included in the compilation request.

A pilot realization is valid only for its declared pilot scope.

The compiler must not treat pilot scope as full release scope.

### 3.2 Compilation scope

The compilation request must identify which realization operations are in
scope.

Conceptually:

```rust
pub enum CompilationScope {
    Operations(BTreeSet<architecture::OperationId>),
    RealizationScope(realization::RealizationScopeId),
    FullArchitecture,
}
```

> Illustrative API; not frozen.

The compiler validates that the requested scope is a subset of complete
realization coverage.

A release compilation eventually requires:

```text
FullArchitecture
```

or another explicitly approved deployment profile whose excluded operations
have undergone normative review.

### 3.3 Analysis policy

Target-independent analysis may require a typed policy controlling choices
that do not alter protocol semantics.

Conceptually:

```rust
pub struct AnalysisPolicy {
    pub proof_selection: ProofSelectionPolicy,
    pub disclosure: DisclosurePolicy,
    pub placement: PlacementPolicy,
    pub layout: LayoutRequirementPolicy,
    pub coverage: CoveragePolicy,
}
```

> Illustrative API; not frozen.

The Phase-2 policy should be deliberately small and deterministic.

Policy must not contain handwritten operation semantics.

### 3.4 Optional abstract target capabilities

The package may support a later target-planning stage over an abstract
capability provider.

Conceptually:

```rust
pub trait TargetCapabilities {
    fn identity(&self) -> TargetCapabilityIdentity;

    fn supports(
        &self,
        capability: RequiredCapability,
    ) -> CapabilitySupport;
}
```

> Illustrative interface; not frozen.

The compiler core must remain able to construct target requirements before one
concrete target is selected.

Phase 2 may use:

- a target-independent analysis only;
- a deterministic mock capability set for tests;
- or a minimal generic capability adapter.

It must not require the real Elements target package before the compiler's
core analysis can run.

### 3.5 Typed deployment-independent parameters

The Phase-2 compiler should retain architecture bound references rather than
requiring calibrated values.

Later target planning may accept typed deployment parameters such as:

- calibrated finite bounds;
- target limits;
- policy-selected representation modes;
- backend configuration identity.

Those values are explicit typed inputs. They are not read from a deployment
profile file or environment implicitly.

---

## 4. Forbidden inputs

The compiler must not consume:

- architecture JSON or TOML;
- declassification JSON;
- model-label JSON;
- realization Markdown;
- Layer-0 LaTeX;
- files under `plans/`;
- model Rust source text;
- model test source text;
- target opcode Markdown;
- target source trees through runtime scraping;
- tapscript bytecode as semantic input;
- a linked bundle as the source of expected relations;
- command-line arguments directly;
- environment variables directly;
- wall-clock time;
- filesystem order.

The compiler must not depend on:

```text
model
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

A future compiler inspection/publication binary may live elsewhere or as a
narrow binary over the library and must follow ADR-010.

---

## 5. Typed outputs

The compiler should separate:

1. target-independent semantic analysis;
2. target-capability matching or target-selected planning.

This avoids putting target identity into the core semantic analysis.

### 5.1 Core analyzed program

Conceptually:

```rust
pub struct AnalyzedProgram {
    pub schema_version: CompilerIrSchemaVersion,

    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub scope: AnalyzedScope,

    pub relation_graph: RelationGraph,
    pub proof_alternatives: ProofAlternativeGraph,
    pub disclosure: DisclosureAnalysis,
    pub fact_sources: FactSourceRequirements,
    pub constructibility: ConstructibilityAnalysis,
    pub lifecycle: LifecycleAnalysis,
    pub placement: PlacementRequirements,
    pub layouts: LayoutRequirements,
    pub target_requirements: TargetRequirementSet,
    pub coverage: CoverageRequirements,

    pub diagnostics: AnalysisDiagnostics,
}
```

> Illustrative API; exact fields and names are not frozen until the Phase-2
> pilot analysis passes.

### 5.2 Target-selected plan

A later compiler stage may combine the analyzed program with typed target
capabilities and deployment/compiler policy.

Conceptually:

```rust
pub struct TargetCompilationPlan {
    pub analyzed_program: AnalyzedProgramIdentity,
    pub target: TargetIdentity,

    pub selected_proofs: SelectedProofPlan,
    pub selected_representations: SelectedRepresentationPlan,
    pub resolved_fact_sources: ResolvedFactSources,
    pub placement_requirements: PlacementRequirements,
    pub layout_requirements: LayoutRequirements,
    pub unresolved_backend_requirements: BackendRequirementSet,
}
```

> Illustrative API; not frozen.

The compiler may retain several backend-feasible proof candidates when final
selection requires backend resource formulas unavailable at the
target-independent stage.

### 5.3 Coverage skeleton

The compiler should emit typed requirements sufficient for the vector package
to know:

- every relation in scope;
- relation activation condition;
- required positive coverage;
- required negative coverage;
- expected mutation classes;
- target carrier requirement;
- representation variants requiring acceptance;
- representation variants requiring rejection;
- permissionless construction cases.

The compiler does not execute vectors.

### 5.4 Structured diagnostics

The compiler should return:

- fatal typed errors;
- deterministic nonfatal diagnostics;
- source provenance;
- relation/operation IDs;
- policy decision explanations.

Diagnostics must not become a separate untyped semantic channel.

---

## 6. Public API boundary

### 6.1 Core analysis API

The initial planned API is conceptually:

```rust
pub fn analyze(
    realization: &realization::RealizationSpec,
    scope: &CompilationScope,
    policy: &AnalysisPolicy,
) -> Result<AnalyzedProgram, CompileError>;
```

> Illustrative API; not frozen.

The function must be:

- pure;
- deterministic;
- independent of filesystem and environment;
- independent of concrete backend packages;
- fail-closed on incomplete realization scope;
- source-provenanced;
- stable under presentation-only source reordering where semantics are
  unchanged.

### 6.2 Target planning API

A possible later API is:

```rust
pub fn plan_for_target<T: TargetCapabilities>(
    analyzed: &AnalyzedProgram,
    target: &T,
    deployment: &DeploymentParameters,
    policy: &TargetPlanningPolicy,
) -> Result<TargetCompilationPlan, CompileError>;
```

> Illustrative API; not frozen.

This stage selects or prunes proof alternatives using abstract target
capabilities.

It does not emit target programs.

### 6.3 Validation API

Analyzed programs should be validated by construction where practical.

A public validation API may still be useful for:

- parsed external analysis publications;
- testing;
- cache verification;
- backend boundary checks.

Conceptually:

```rust
pub fn validate_analyzed_program(
    realization: &realization::RealizationSpec,
    program: &AnalyzedProgram,
) -> Result<(), Vec<CompileError>>;
```

### 6.4 Inspection API

Downstream backends and evidence tools need read-only typed access to:

- relation graph;
- proof alternatives;
- disclosure reasons;
- target requirements;
- placement requirements;
- layout requirements;
- source provenance;
- coverage requirements.

Do not expose mutable registries that allow downstream packages to silently
rewrite semantic analysis.

### 6.5 No execution API

The compiler does not execute:

- model operations;
- target scripts;
- transactions;
- indexer queries;
- deployment profiles.

Expression evaluation may be used for compile-time constants and analysis
tests. It is not a substitute for model or target execution.

---

## 7. Compiler identity ownership

### 7.1 Realization relation IDs are preserved

The compiler must preserve realization relation identity and provenance.

It may create compiler-owned nodes representing:

- normalized relation;
- proof alternative;
- selected proof;
- disclosure requirement;
- fact-source requirement;
- placement requirement;
- layout requirement;
- target capability requirement;
- coverage obligation.

These nodes must reference the originating realization relation.

### 7.2 Compiler-owned IDs

Likely compiler-owned IDs include:

```rust
pub struct AnalysisNodeId(...);
pub struct ProofPlanNodeId(...);
pub struct DisclosureRequirementId(...);
pub struct FactSourceRequirementId(...);
pub struct PlacementRequirementId(...);
pub struct LayoutRequirementId(...);
pub struct TargetRequirementId(...);
pub struct CoverageRequirementId(...);
```

> Illustrative types; not frozen.

### 7.3 Structural identity

Compiler IDs should derive from:

- node kind;
- typed operands;
- source realization IDs;
- policy-relevant parameters;
- canonical dependency ordering;
- schema/domain separator.

They must not derive from:

- source file path;
- source line;
- memory address;
- hash-map iteration order;
- target backend bytes;
- current time;
- execution duration.

### 7.4 Compiler configuration identity

The compiler configuration identity should eventually bind:

- compiler IR schema;
- analysis policy;
- proof-alternative library version;
- disclosure-analysis version;
- fact-source-analysis version;
- lifecycle-analysis version;
- placement-requirement policy;
- layout-requirement policy;
- coverage policy;
- deterministic tie-break rules.

This identity is separate from:

- architecture identity;
- realization identity;
- target identity;
- backend pattern-library identity;
- linked-bundle identity.

### 7.5 No published compiler hash before policy stabilizes

Phase 2 may use internal deterministic fingerprints.

Do not publish a stable compiler-analysis hash until:

- analyzed projection is documented;
- schema version is explicit;
- canonical ordering is defined;
- policy inclusion is defined;
- mutation tests exist;
- migration policy is written.

---

## 8. Relation graph construction

### 8.1 Preserve semantic relations

The compiler begins from the realization relation registry.

It must not replace a semantic relation with a compiler-local string or
backend pattern name.

### 8.2 Normalize into an analysis DAG

The compiler may normalize relations into an analysis DAG to support:

- dependency ordering;
- structural sharing;
- proof alternatives;
- disclosure propagation;
- fact-source planning;
- placement;
- target requirements;
- coverage.

The normalization must preserve source relation provenance.

### 8.3 Hash-consing

Structurally equal analysis nodes may be shared.

Examples:

- repeated amount-domain checks;
- repeated owner-authorization forms;
- repeated explicit closed-asset identity requirements;
- repeated sponsor-isolation relations;
- repeated cardinality forms.

Sharing occurs in compiler analysis.

A backend may later inline the shared node into multiple target programs under
the realization's translation discipline.

### 8.4 Sharing constraints

Do not merge nodes solely because their rendered text matches.

Nodes with distinct semantic provenance may need separate identities for:

- relation coverage;
- activation;
- diagnostics;
- operation ownership;
- target placement.

A compiler may share one structural implementation while retaining multiple
source-relation references.

### 8.5 Dependency graph

The compiler must produce a deterministic dependency graph.

It must reject or explicitly represent unsupported cycles.

Expected Phase-2 graph classes include:

```text
relation depends on expression/fact
proof alternative depends on relation
disclosure depends on expression/fact
fact source depends on fact and availability
constructibility depends on witness availability
lifecycle path depends on representation and operation
placement depends on relation and required facts
layout requirement depends on operation family/cardinality
coverage depends on relation and activation
```

### 8.6 Topological processing

Use deterministic topological ordering where the graph is acyclic.

Tie-break by stable structural ID or another documented canonical key.

A topological order is an analysis/rendering choice unless semantic evaluation
requires a specific order.

---

## 9. Constant folding and normalization

### 9.1 Purpose

Fold values known from:

- architecture declarations;
- realization constants;
- fixed amount limits;
- semantic identities;
- statically known cardinalities;
- operation activation facts known from scope.

Retain symbolic references for:

- transaction/runtime facts;
- deployment-calibrated bound values;
- target capabilities;
- concrete constructor references;
- target resource formulas.

### 9.2 Checked folding

Constant folding must use the realization's checked domain semantics.

It must not:

- wrap on overflow;
- silently clamp;
- use host-dependent integer width;
- convert amount to count without an explicit typed operation;
- simplify away a fail-closed domain check incorrectly.

### 9.3 Provenance retention

A folded constant should retain provenance to the original expression and
facts where needed for:

- diagnostics;
- relation coverage;
- disclosure analysis;
- mutation attribution;
- audit reports.

### 9.4 Conservative simplification

The initial compiler should implement only clearly semantics-preserving
rewrites, such as:

```text
constant arithmetic
boolean identity
duplicate identical conjunction member removal where set semantics apply
inactive conditional branch elimination when condition is statically known
```

Do not add speculative algebraic rewriting that changes checked-failure
behavior.

### 9.5 No optimizer-first work

The initial compiler does not attempt:

- global target-cost optimization;
- probabilistic check ordering;
- nondeterministic search;
- aggressive relation fusion;
- backend instruction scheduling;
- common target-code elimination.

Auditability and deterministic provenance take precedence.

---

## 10. Proof-alternative graph

### 10.1 Semantic relation versus proof

A semantic relation states what must be true.

A proof alternative states one class of target evidence that can establish it.

Examples:

```text
relation:
    receipt input value equals receipt output value

alternatives:
    explicit value equality
    commitment equality
```

```text
relation:
    aggregate receipt value is conserved

alternatives:
    explicit arithmetic
    confidential transaction conservation
```

```text
relation:
    redemption amount x is available for floor arithmetic

alternatives:
    explicit receipt value
    authenticated opening
    owner-authorized normalization before redemption
```

### 10.2 AND/OR graph

Proof alternatives naturally form an AND/OR graph.

Example:

```text
live transfer safe
    AND
        owner authorization
        live-class closure
        explicit U asset identity
        value conservation

value conservation
    OR
        explicit arithmetic proof
        CT conservation proof
```

The compiler should represent this structure explicitly rather than flattening
it into a string allowlist.

### 10.3 Proof-alternative source

Proof alternatives originate in the typed realization's semantic latitude and
the compiler's target-independent proof vocabulary.

The compiler must not invent a weaker alternative because one target lacks the
preferred proof.

### 10.4 Capability requirements

Every proof alternative produces one or more abstract target requirements.

For example:

```text
explicit arithmetic:
    input value inspection
    output value inspection
    checked amount arithmetic

CT conservation:
    confidential transaction conservation
    explicit closed-asset identity
    complete output-family closure

commitment equality:
    input commitment inspection
    output commitment inspection
    equality proof or direct commitment-byte equality
```

Exact target implementations remain backend-owned.

### 10.5 Selection policy

Target-independent analysis should preserve alternatives.

Target matching may prune alternatives lacking required capabilities.

Final selection may require backend resource information.

A deterministic selection policy may prioritize:

1. semantic validity;
2. capability support;
3. constructibility;
4. lifecycle reachability;
5. deployment policy;
6. resource feasibility;
7. disclosure minimality;
8. target cost;
9. canonical tie-break.

> Illustrative order; not frozen in Phase 2.

### 10.6 No unsupported default

If no proof alternative remains, target planning fails with a typed error.

It must not:

- force explicit values silently;
- add a signature;
- omit a relation;
- alter operation semantics;
- rely on undocumented off-chain verification.

---

## 11. Disclosure analysis

### 11.1 Inputs

Disclosure analysis consumes:

- realization declassification;
- fact dependency graph;
- public state assignments;
- public observables;
- permissionless constructibility;
- representation capabilities;
- target-safety requirements, at target-planning time;
- explicit deployment policy overrides.

### 11.2 Separate semantic and target disclosure

The compiler must distinguish:

1. **semantic disclosure**
   Required by state, interface, audit, or constructibility.

2. **target safety disclosure**
   Required because the initial target lacks a less-disclosing proof.

3. **deployment policy disclosure**
   Selected deliberately even though a supported lower-disclosure plan exists.

This distinction prevents target limitations from being mistaken for protocol
semantics.

### 11.3 Disclosure reasons

A typed reason vocabulary may include:

```rust
pub enum DisclosureReason {
    PublicStateDependency,
    PublicObservableDependency,
    PublicAuditDependency,
    PermissionlessConstructibility,
    TargetSafetyRequirement,
    DeploymentPolicyOverride,
}
```

> Illustrative API; not frozen.

Every disclosure in the selected plan must carry at least one reason.

### 11.4 Minimality check

Where the deployment claims minimal disclosure, the compiler must check that no
supported alternative:

- enforces the same semantic relations;
- satisfies constructibility;
- preserves lifecycle;
- fits target resources;
- reveals strictly less information.

This is a finite target/policy-relative check, not a universal cryptographic
minimum theorem.

### 11.5 Compiler output

The disclosure analysis should report:

- fact;
- operation;
- selected visibility;
- reasons;
- dependency provenance;
- alternative modes considered;
- rejection reason for lower-disclosure alternatives;
- target capability dependency where applicable.

### 11.6 Pilot analysis

#### `compact-ash`

Expected:

- ASH values must be publicly usable by arbitrary constructors;
- no owner-private witness may be introduced;
- no new owner information is disclosed;
- closed `U` identity remains a safety requirement in the initial Elements
  target plan.

#### `transfer-live-receipts`

Expected semantic result:

- no numerical receipt value must become public merely because of the lateral
  transfer relation;
- owner authorization and destination metadata remain semantically available;
- explicit `U` identity remains a target safety requirement under D005;
- explicit arithmetic and CT conservation remain proof alternatives.

---

## 12. Fact-source analysis

### 12.1 Purpose

Every proof or target predicate needs facts from an authenticated source.

The compiler must state which facts a backend must obtain and authenticate.

### 12.2 Target-independent source classes

Conceptual source classes include:

```rust
pub enum FactSourceRequirement {
    CompileTimeConstant,
    ArchitectureConstant,
    DeploymentConstant,
    TransactionInput,
    TransactionOutput,
    AuthenticatedInputMetadata,
    AuthenticatedOutputMetadata,
    PublicChainData,
    PublicOpening,
    CurrentOwnerWitness,
    OperatorWitness,
    SponsorLocalWitness,
    DerivedExpression,
}
```

> Illustrative vocabulary; not frozen.

Concrete sources such as `OP_INSPECTINPUTVALUE` remain backend-specific.

### 12.3 Authentication requirements

A fact-source requirement should identify:

- semantic fact;
- required authenticity;
- availability;
- owning operation;
- relation consumers;
- public/private status;
- candidate semantic family;
- target capabilities needed to obtain it.

### 12.4 No unauthenticated metadata

The compiler must not accept a fact source such as:

```text
caller supplies amount in metadata
```

without a relation authenticating it against the semantic consensus value.

### 12.5 Fact-source completeness

Every relation operand must have at least one candidate fact source.

A missing source is a compile error, not a backend concern to resolve
informally.

### 12.6 Backend refinement

The backend later maps a requirement to a concrete source, such as:

- transaction introspection;
- witness item;
- constructor commitment;
- target constant;
- signature message;
- public opening.

The selected concrete source must preserve the compiler's source requirement
and provenance.

---

## 13. Witness availability and constructibility

### 13.1 Availability classes

The compiler consumes realization availability and may refine it.

Conceptually:

```text
compile-time
architecture/deployment constant
public chain data
public opening
current owner secret
operator secret
sponsor local
```

### 13.2 Permissionless operations

For permissionless operations, every required witness must be available from:

- public data;
- public opening;
- deployment/bundle data;
- constructor-local sponsor data.

The compiler rejects any plan requiring:

- another owner's secret;
- operator secret;
- unpublished opening;
- retained blinding factor unavailable to arbitrary constructors.

### 13.3 Owner-authorized operations

Owner-authorized operations may require:

- current owner signatures;
- owner-known opening;
- owner-authorized normalization.

The compiler must still preserve:

- exact output commitment;
- multi-owner completeness;
- sponsor separation;
- lifecycle.

### 13.4 Constructibility versus target verification

A target capability saying “can verify commitment opening” is insufficient when
the authorized constructor cannot obtain the opening.

The compiler must check both:

```text
proof verifiable
witness available
```

### 13.5 Pilot analysis

#### `compact-ash`

Must be constructible without:

- owner secret;
- operator secret;
- private ASH opening.

#### `transfer-live-receipts`

May require:

- every current owner signature;
- owner-local confidential transaction witness material where selected;
- sponsor-local data for optional sponsor flow.

---

## 14. Lifecycle analysis

### 14.1 Purpose

A representation or proof plan is unsupported if it creates an object that
cannot reach required semantic exits.

### 14.2 Lifecycle graph

The compiler should construct a graph over:

```text
object semantic class
×
representation mode
×
authorized operation
```

Edges are supported transitions or normalization paths.

### 14.3 Required reachability

Examples:

```text
LiveReceipt
    → transfer
    → burn
    → redeem

TimeLockedReceipt
    → transfer
    → relabel after maturity

Ash
    → compact
    → clear

DepositRequest
    → cancel
    → admit

Entitlement
    → settle
```

Phase 2 need only analyze the pilot-relevant parts, but the data model must not
prevent later expansion.

### 14.4 Normalization paths

A normalization path is an explicit semantics-preserving transition between
representations.

It must identify:

- source representation;
- destination representation;
- operation;
- authorization;
- value preservation;
- asset preservation;
- public disclosure;
- witness availability;
- lifecycle consequences.

### 14.5 Pilot lifecycle

#### `compact-ash`

Public/openable ASH output must retain future:

```text
compact
clear
```

The compiler should record those requirements even before clear is compiled.

#### `transfer-live-receipts`

A private committed live receipt must retain paths to:

```text
transfer
burn
redeem
```

The exact burn/redeem declassification method is unresolved, so Phase 2 should
record those future target requirements rather than falsely declaring the mode
release-complete.

### 14.6 Scope-aware lifecycle verdicts

Pilot analysis may report:

```text
valid within pilot scope
future lifecycle obligation unresolved
```

It must not report full deployment support until every required exit has a
target-supported path.

---

## 15. Obligation-placement requirements

### 15.1 Compiler responsibility

The compiler identifies:

- relation locality;
- required facts;
- candidate semantic carriers;
- whether a global coordinator is needed;
- whether local duplication is required;
- whether placement may be target-selected.

The compiler does not assign final tapscript input indexes.

### 15.2 Local relations

Examples:

- current input has expected object family;
- input owner authorizes;
- current root matches predecessor;
- current receipt class is live.

These naturally require a carrier on each affected input family member.

### 15.3 Global relations

Examples:

- complete output-family closure;
- aggregate value conservation;
- sponsor boundary;
- family counts;
- event projection;
- state assignment over several inputs/outputs.

These require a coordinator or another target-specific global proof strategy.

### 15.4 Placement requirement output

Conceptually:

```rust
pub struct PlacementRequirement {
    pub id: PlacementRequirementId,
    pub relation: realization::RelationId,
    pub locality: RelationLocality,
    pub required_facts: BTreeSet<realization::FactId>,
    pub candidate_semantic_carriers: BTreeSet<SemanticCarrier>,
    pub duplication: DuplicationPolicy,
}
```

> Illustrative API; not frozen.

### 15.5 Unplaced relation failure

A target/backend may report that no concrete carrier can authenticate the
required facts.

That is a target planning/emission failure.

The compiler must never omit the relation because placement is difficult.

---

## 16. Layout requirements

### 16.1 Target-independent layout requirements

The compiler derives requirements such as:

- family must be countable;
- family has semantic minimum;
- family has architecture bound reference;
- family members need canonical enumeration;
- one global relation spans the complete family;
- output families must be complete and disjoint;
- sponsor region must not absorb protocol outputs;
- conditional family presence is tied to one activation expression;
- target ABI must expose witness availability.

These are requirements, not concrete positions.

### 16.2 No concrete index in compiler core

The core compiler must not say:

```text
STATE is input 0
ASH is output 1
```

unless the type is explicitly a target-selected layout artifact rather than the
target-independent analyzed program.

### 16.3 D006 handoff

The backend and transaction package later produce the concrete canonical ABI
under D006.

Compiler layout requirements retain provenance to:

- operation;
- family;
- cardinality;
- relation;
- constructibility;
- representation;
- coverage.

### 16.4 Mixed-branch requirements

The compiler should identify semantic operation families whose target programs
could be combined incorrectly.

It may emit a requirement such as:

```text
selected target layout must make Burn and TransferLive relations mutually
exclusive for one concrete transaction
```

The backend later performs target-specific satisfiability checks and vectors.

---

## 17. Target capability extraction

### 17.1 Capability set

The compiler derives one target requirement set from selected or candidate
proof alternatives.

Examples:

```text
input asset authentication
output asset authentication
input/output value access
owner signature with output commitment
confidential value conservation
commitment equality
public opening authentication
relative timelock
constructor verification
checked arithmetic
wide floor proof
transaction-family count introspection
```

### 17.2 Required versus alternative capabilities

A target requirement may be:

- unconditional;
- one member of an alternative set;
- conditionally active;
- required only for one representation mode;
- required only for one lifecycle path.

The compiler should preserve this structure.

### 17.3 Capability matching

Given an abstract target capability set, target planning should report:

- supported requirement;
- unsupported requirement;
- partially supported requirement;
- capability constraints;
- required backend implementation;
- proof alternatives pruned.

### 17.4 No semantic weakening

An unsupported capability cannot cause the compiler to:

- omit the relation;
- broaden an object family;
- add an unauthorized signer;
- change a formula;
- remove a lifecycle exit.

It produces an unsupported target plan.

---

## 18. Relation coverage requirements

### 18.1 Coverage output

For each relation, the compiler should produce:

- owning operation;
- activation condition;
- required positive case;
- required negative case;
- candidate mutation classes;
- placement/carrier requirement;
- representation cases;
- expected public projection checks;
- source provenance.

### 18.2 Pilot coverage

#### `compact-ash`

Require coverage for:

- minimum count;
- maximum bound reference;
- ASH recognition;
- permissionless authorization;
- exact ownerless value conservation;
- one ASH output;
- no burn projection;
- sponsor isolation;
- no root use;
- public constructibility.

#### `transfer-live-receipts`

Require coverage for:

- input/output cardinality;
- live class;
- every-owner authorization;
- exact value conservation;
- explicit `U` identity requirement for initial Elements plan;
- output-family closure;
- sponsor isolation;
- explicit-value proof alternative;
- confidential-value proof alternative;
- representation metamorphism;
- no root use.

### 18.3 Coverage policy identity

The coverage requirements are bound by compiler/evidence policy identity.

The vector package later materializes them.

The compiler does not mark coverage complete merely because requirements exist.

---

## 19. Pilot analysis requirements

### 19.1 `compact-ash`

The analyzed program must include at least:

#### Relations

- input family cardinality:
  ```text
  2 <= ash_input_count <= ASH_BATCH_MAX
  ```

- every protocol input is ASH;
- exactly one ASH output;
- exact semantic `U` conservation:
  ```text
  sum(ash input values) = ash output value
  ```
- permissionless operation;
- sponsor flow isolated;
- transition projection required;
- burn/clear/residue projections absent;
- no root effects.

#### Proof alternatives

- public amount arithmetic;
- closed asset identity authentication;
- constructor closure.

#### Constructibility

- ASH facts publicly available;
- no owner or operator secret;
- sponsor-local witness only for sponsor inputs.

#### Lifecycle

- output ASH remains compactable and clearable.

#### Placement requirements

- local ASH recognition on every ASH input;
- one global carrier for family count, output closure, conservation, and sponsor
  boundary.

#### Layout requirements

- bounded ASH family enumerable;
- exactly one ASH output;
- optional sponsor region distinguishable.

### 19.2 `transfer-live-receipts`

The analyzed program must include at least:

#### Relations

- nonempty bounded input family;
- nonempty bounded output family;
- every input is live receipt;
- every output is live receipt;
- every input owner authorizes;
- exact semantic `U` conservation;
- destination owner/value outputs are within the authorized transaction;
- sponsor flow isolated;
- transition projection required;
- no root effects.

#### Proof alternatives

Value conservation:

```text
explicit arithmetic
or
confidential transaction conservation
```

Exact local preservation is not required because split/merge is permitted;
aggregate conservation is required.

#### Representation

- explicit values supported;
- private committed values semantically permitted;
- closed `U` identity rigid in the initial Elements target plan;
- sponsor value may be confidential under selected policy.

#### Constructibility

- owner signatures available from all current owners;
- confidential proof material available to participating owners/transaction
  constructor;
- sponsor witness local to sponsor;
- no operator secret.

#### Lifecycle

- output receipts remain transferable;
- future burn/redeem exits remain required, though target support may remain
  unresolved in pilot scope.

#### Placement requirements

- local receipt recognition and owner authorization per input;
- global carrier for counts, output closure, aggregate conservation, and sponsor
  boundary.

#### Layout requirements

- bounded input/output families enumerable;
- every protocol-capable output classified;
- sponsor region separated.

---

## 20. Validation

### 20.1 Input validation

Reject:

- unsupported realization schema;
- invalid realization;
- incomplete requested scope;
- architecture/realization identity mismatch;
- duplicate relation identity;
- unresolved expression/fact;
- target-specific semantic leakage where detectable.

### 20.2 Analysis graph validation

Require:

- every node has deterministic ID;
- dependencies resolve;
- graph is acyclic or supported cycles are explicit;
- source provenance exists;
- all in-scope realization relations represented;
- no out-of-scope relation silently included as required;
- no semantic relation dropped.

### 20.3 Proof graph validation

Require:

- every target-enforced relation has at least one proof alternative;
- every alternative maps to target requirements;
- no alternative is semantically weaker;
- constructibility requirements attached;
- lifecycle implications attached;
- representation implications attached.

### 20.4 Disclosure validation

Require:

- every public fact has a reason;
- every semantic declassification preserved;
- no required public fact omitted;
- target/deployment disclosure distinguished;
- minimality result honest about unsupported alternatives.

### 20.5 Fact-source validation

Require:

- every proof operand has a source requirement;
- source is authenticatable;
- source availability is compatible with operation authorization;
- no unauthenticated metadata source;
- no circular source dependency without a defined constructor relation.

### 20.6 Placement validation

Require:

- every relation has at least one candidate semantic carrier or a target-global
  proof class;
- local relations cover all family members;
- global relations marked as requiring global carrier;
- duplication policy explicit;
- no optional-only carrier for unconditional relation.

### 20.7 Layout-requirement validation

Require:

- all repeated families bounded or explicitly target-unsupported;
- cardinality references resolve;
- required/optional families identified;
- family completeness requirements represented;
- sponsor policy represented;
- conditional presence tied to activation.

### 20.8 Coverage validation

Require:

- every relation has coverage requirements;
- activation represented;
- positive and negative cases requested;
- representation alternatives covered;
- permissionless construction case included.

### 20.9 Target-plan validation

When abstract capability matching is used:

- target identity present;
- all selected capabilities supported;
- no unresolved required capability;
- proof selection deterministic;
- representation selection valid;
- no semantic relation omitted;
- unsupported backend requirements explicit.

---

## 21. Error model

Compiler errors should be typed and deterministic.

Candidate classes include:

```rust
pub enum CompileError {
    UnsupportedRealizationSchema,
    InvalidRealization,
    IncompleteRealizationScope,
    ArchitectureBindingMismatch,

    DuplicateAnalysisNode(AnalysisNodeId),
    UnknownFact(realization::FactId),
    UnknownExpression(realization::ExprId),
    UnknownRelation(realization::RelationId),
    DependencyCycle,

    RelationDropped(realization::RelationId),
    MissingProofAlternative(realization::RelationId),
    WeakenedProofAlternative(realization::RelationId),

    MissingDisclosureReason(realization::FactId),
    MissingRequiredDisclosure(realization::FactId),
    UnsupportedMinimalityClaim(realization::FactId),

    MissingFactSource(realization::FactId),
    UnauthenticatedFactSource(realization::FactId),
    UnavailableWitness(realization::FactId),
    PermissionlessSecretDependency(realization::FactId),

    MissingLifecyclePath(architecture::ObjectId),
    UnreachableLifecycleExit(architecture::OperationId),

    MissingPlacement(realization::RelationId),
    InvalidPlacementCandidate(realization::RelationId),

    UnboundedTargetFamily(architecture::OperationId),
    MissingLayoutRequirement(realization::RelationId),

    MissingCoverageRequirement(realization::RelationId),

    UnsupportedTargetCapability(RequiredCapability),
    NoSupportedProofPlan(realization::RelationId),
    AmbiguousProofSelection,
}
```

> Illustrative vocabulary; not frozen.

Errors should include typed context rather than only strings.

Human display text may include operation/relation names and source provenance.

Compiler diagnostics must not contain secrets because compiler analysis should
not receive production secret values.

---

## 22. Determinism

### 22.1 Canonical node ordering

Suggested ordering:

```text
operation by architecture code
relation by realization RelationId
analysis node by structural ID
proof alternative by typed alternative ID
fact-source requirement by FactId
disclosure by FactId and reason
placement by RelationId
layout requirement by operation and family ID
coverage by RelationId and case kind
```

### 22.2 Deterministic policy

Every policy choice must have a canonical tie-break.

No policy may depend on:

- hash-map iteration;
- thread completion order;
- target probe timing;
- filesystem order;
- random search;
- environment variables not represented as explicit typed input.

### 22.3 Parallel analysis

The compiler may analyze operations in parallel later.

Parallel execution must produce the same typed result and canonical bytes as
serial execution.

Phase 2 does not need parallel implementation.

### 22.4 No ambient environment

The library must not read:

- `RUST_LOG`;
- process cwd;
- target node environment;
- source date;
- host CPU;
- locale.

CLI wrappers may configure diagnostics outside the pure analysis API.

### 22.5 Repeated analysis tests

Tests must require:

- same input and policy → equal analyzed program;
- operation declaration reordering → equal canonical analysis where order is
  not semantic;
- unrelated target availability not supplied → no effect;
- mock capability ordering → equal target plan;
- explicit policy mutation → changed affected plan identity.

---

## 23. Generated artifacts

### 23.1 Phase-2 default

No committed compiler artifact is required merely to implement analysis.

Prefer typed in-memory outputs and direct tests first.

### 23.2 Candidate derivative reports

Useful future publications may include:

```text
relation_graph.json
proof_alternatives.json
disclosure_plan.json
fact_source_requirements.json
lifecycle_paths.json
placement_requirements.json
layout_requirements.json
target_requirements.json
coverage_requirements.json
```

These names are illustrative.

Do not create one file per internal registry unless it improves independent
review or release evidence.

A single canonical analysis report may be more maintainable.

### 23.3 Artifact law

Any committed compiler artifact must have:

- typed source;
- schema version;
- canonical ordering;
- deterministic rendering;
- one generator;
- one non-writing checker;
- unknown-field rejection;
- identity binding;
- no reverse semantic dependency.

### 23.4 Release relevance

The release may eventually bind:

- compiler configuration identity;
- analyzed-program identity;
- selected target-plan identity;
- canonical analysis report hash.

Those identities do not replace bundle-specific evidence under D004.

---

## 24. Testing strategy

### 24.1 Unit tests

Cover:

- relation lowering;
- hash-consing;
- provenance preservation;
- topological ordering;
- constant folding;
- checked arithmetic folding;
- proof-alternative graph;
- disclosure propagation;
- fact-source resolution;
- witness availability;
- constructibility;
- lifecycle reachability;
- placement requirements;
- layout requirements;
- target capability extraction;
- coverage generation;
- deterministic IDs.

### 24.2 Realization mutation tests

Mutate the pilot realization and require errors for:

- relation omission;
- missing proof alternative;
- missing disclosure reason;
- secret dependency on permissionless compact-ash;
- missing ASH clear lifecycle exit;
- missing owner authorization;
- missing closed-asset classification;
- unbounded family;
- missing placement candidate;
- missing coverage case.

### 24.3 Mock target tests

Use one deterministic typed mock capability set to test:

- all pilot alternatives supported;
- CT conservation unsupported;
- explicit arithmetic unsupported;
- no proof alternative available;
- commitment equality advertised only when selected;
- target capability order independence;
- deterministic proof pruning.

A mock target is compiler unit-test infrastructure, not deployment evidence.

### 24.4 Pilot analysis tests

Require exact expected analysis for:

- `compact-ash`;
- `transfer-live-receipts`.

Avoid giant snapshot tests as the sole assertion.

Prefer typed assertions for:

- relation census;
- proof alternatives;
- disclosure;
- sources;
- constructibility;
- lifecycle;
- placement;
- layout;
- target requirements;
- coverage.

Canonical snapshot/publication tests may supplement typed assertions.

### 24.5 Public API tests

An external integration test should prove a downstream backend can:

- obtain an analyzed pilot program;
- enumerate relations;
- inspect proof alternatives;
- inspect target requirements;
- inspect placement/layout requirements;
- inspect coverage obligations;
- preserve realization provenance;
- do so without model or target internals.

### 24.6 Determinism tests

Repeated analysis and policy-equivalent permutations must yield equal results.

### 24.7 Compile-fail tests

Add only when needed to protect:

- mutable internal registry access;
- invalid ID construction;
- target dependency leakage.

Do not introduce a heavy compile-fail framework before a concrete API boundary
needs it.

---

## 25. Dependency and unsafe-code policy

The compiler inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace lints;
- `unsafe_code = "deny"`;
- Cargo `--locked`;
- deterministic output;
- permissive dependencies;
- no network access;
- no build-time target probing;
- no build script unless a strict typed-generation need is demonstrated.

Likely initial dependencies:

```text
realization
architecture, if direct ID use is clearer
thiserror
sha2, if structural identities are implemented here
serde, only when typed derivative publication is needed
```

Avoid initially:

- target libraries;
- SMT solvers;
- async runtimes;
- parser generators;
- general graph frameworks if simple deterministic code suffices;
- optimizer frameworks.

SMT experimentation belongs to research/evidence tooling rather than compiler
core unless later accepted.

---

## 26. Performance expectations

The Phase-2 graph is small.

Priorities are:

1. correctness;
2. deterministic identity;
3. provenance;
4. inspectability;
5. fail-closed validation;
6. stable diagnostics;
7. performance.

Use asymptotically sensible algorithms:

- ordered-map indexing;
- DAG traversal;
- topological sorting;
- finite fixed points;
- graph reachability;
- structural interning.

Do not optimize for millions of relations unless actual scale demonstrates the
need.

No performance benchmark is a Phase-2 gate unless analysis becomes
unexpectedly expensive.

---

## 27. Non-goals

Phase 2 does not implement:

- tapscript emission;
- Simplicity emission;
- concrete target opcodes;
- stack scheduling;
- target constructor bytes;
- target transaction indexes;
- taptree layout;
- linking;
- transaction assembly;
- signatures;
- rangeproofs;
- target execution;
- resource calibration;
- deployment profile generation;
- independent indexing;
- a general optimizer;
- a general programming language frontend;
- complete analysis of all thirteen operations before pilot validation.

The compiler is not:

- a replacement for realization;
- a replacement for the model;
- a target interpreter;
- a release gate by itself;
- evidence that emitted code is correct.

---

## 28. Phase-2 milestones

### C1.1 — Crate skeleton

Deliver:

- workspace package;
- crate documentation;
- realization dependency;
- analysis/error skeleton;
- no target dependency.

### C1.2 — Core analyzed-program schema

Deliver:

- scope binding;
- architecture/realization binding;
- analysis schema;
- internal IDs;
- deterministic registries.

### C1.3 — Relation DAG

Deliver:

- relation lowering;
- dependency graph;
- structural interning;
- provenance;
- topological validation.

### C1.4 — Constant folding

Deliver:

- typed folding;
- safe normalization;
- provenance retention;
- boundary tests.

### C1.5 — Proof alternatives

Deliver:

- AND/OR proof graph;
- capability requirements;
- deterministic pruning;
- unsupported-plan errors.

### C1.6 — Disclosure analysis

Deliver:

- semantic disclosure preservation;
- target/policy disclosure categories;
- minimality result structure;
- pilot outputs.

### C1.7 — Fact sources and constructibility

Deliver:

- source requirements;
- witness availability;
- permissionless secret rejection;
- pilot source plans.

### C1.8 — Lifecycle analysis

Deliver:

- pilot object/representation graph;
- required exits;
- unresolved future lifecycle reporting.

### C1.9 — Placement and layout requirements

Deliver:

- local/global classification;
- candidate semantic carriers;
- duplication policy;
- family/range requirements;
- sponsor isolation requirements.

### C1.10 — Coverage requirements

Deliver:

- positive/negative cases;
- activation;
- representation cases;
- permissionless construction cases.

### C1.11 — Pilot analyzed programs

Deliver complete typed analysis for both pilot operations.

### C1.12 — Public API and determinism

Deliver downstream API tests and repeatability tests.

### C1.13 — Phase gate

Run all workspace checks and record Phase-2 completion.

---

## 29. Phase-2 exit criteria

The compiler package is ready for target/backend work only when:

- [ ] `packages/compiler` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] the compiler depends on realization and no concrete target/backend;
- [ ] pilot realization scope is validated;
- [ ] all pilot realization relations appear in the relation DAG;
- [ ] no relation is silently dropped;
- [ ] relation provenance is preserved;
- [ ] structural IDs are deterministic;
- [ ] graph dependencies are validated;
- [ ] constant folding uses checked typed semantics;
- [ ] every target-enforced relation has at least one proof alternative;
- [ ] proof alternatives map to abstract target requirements;
- [ ] semantic, target-safety, and policy disclosure are distinguished;
- [ ] every disclosure has provenance;
- [ ] every required fact has an authenticatable source requirement;
- [ ] permissionless compact-ash has no private owner/operator witness;
- [ ] lifecycle requirements are represented;
- [ ] unresolved future lifecycle exits are reported honestly;
- [ ] every relation has placement requirements;
- [ ] every repeated family has layout requirements;
- [ ] no target input/output index appears in compiler core;
- [ ] every relation has coverage requirements;
- [ ] `compact-ash` analysis is complete;
- [ ] `transfer-live-receipts` analysis is complete;
- [ ] explicit and CT conservation alternatives are represented for live
      transfer;
- [ ] mock capability matching fails closed;
- [ ] public API tests pass;
- [ ] repeated analysis is equal;
- [ ] no generated file is consumed;
- [ ] no model source is scraped;
- [ ] no target code is emitted;
- [ ] debug and release workspace tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

Canonical verification includes:

```sh
scripts/ci.sh
```

plus the focused compiler/realization tests documented after implementation.

---

## 30. Open questions

### 30.1 Compiler core versus target planning split

Should target capability matching live:

- in the compiler package;
- in backend packages;
- in a small backend-neutral planning module?

Requirement:

> concrete target dependencies must not enter compiler core.

Initial preference:

- compiler owns the abstract capability vocabulary and generic matching;
- target/backend owns concrete capability implementation and resource-aware
  final selection.

### 30.2 Candidate proof plans versus one selected plan

Should compiler target planning return:

- one deterministic selected proof plan;
- all feasible plans;
- a Pareto frontier;
- one preferred plan plus alternatives?

Phase 2 should avoid complex optimization.

Initial preference:

> retain alternatives in core analysis; allow deterministic target pruning;
> defer resource-sensitive final selection to backend planning.

### 30.3 Analysis identity timing

Do not publish a stable analyzed-program hash before the Phase-2 schema and
policy inclusion are reviewed.

### 30.4 Shared structural relation nodes

How should one shared analysis node preserve several source relation
identities?

Candidates:

- one node with a source-relation set;
- template node plus operation instances;
- separate nodes sharing implementation fingerprint.

Relation coverage and diagnostics must remain operation-specific.

### 30.5 Disclosure minimality scope

Should Phase 2 prove only:

```text
no semantically required disclosure omitted
```

or also:

```text
no unnecessary disclosure under a mock target
```

Initial preference:

- semantic disclosure completeness in core;
- minimality evaluation only when target capabilities and policy are supplied.

### 30.6 Lifecycle incomplete verdict

How should a pilot representation report future required exits not yet target
planned?

The result must distinguish:

```text
pilot analysis valid
full deployment lifecycle unresolved
```

### 30.7 Placement complexity

Should compiler placement requirements allow arbitrary candidate carrier sets
or use a smaller local/global/coordinator classification?

Initial preference:

> start with a small explicit classification; avoid general set-cover
> optimization until target measurement proves need.

---

## 31. Risks

### 31.1 Compiler becoming the semantic source

If compiler lowering invents relations absent from realization, compiler and
semantics blur.

Mitigation:

- relation provenance;
- no dropped realization relation;
- target-only guards separately classified;
- package review;
- model/realization conformance.

### 31.2 Over-general IR

A generic compiler framework may outgrow protocol needs.

Mitigation:

- pilot operations;
- minimum vocabulary;
- no general parser;
- no optimizer-first work;
- additive typed nodes.

### 31.3 Proof-alternative ambiguity

Several valid alternatives may produce nondeterministic selection.

Mitigation:

- retain alternatives;
- explicit target/policy input;
- canonical tie-break;
- separate plan identity;
- no random search.

### 31.4 Hidden target leakage

Concrete target concerns may enter apparently generic structures.

Mitigation:

- no target dependency;
- target-neutral names;
- code review;
- future Simplicity audit;
- split concrete planning below compiler core.

### 31.5 Incomplete relation census

A realization or compiler relation may omit a semantic requirement.

Mitigation:

- architecture/realization/model conformance;
- relation coverage skeleton;
- normative review;
- mutation tests;
- independent backend and observer evidence.

### 31.6 Correlated analysis and evidence

The vector harness may trust compiler-generated coverage too strongly.

Mitigation:

- realization relation registry remains upstream;
- vectors compare compiler census to realization census;
- fixed critical vectors;
- independent report review;
- release checks relation equality.

### 31.7 Lifecycle deferred too easily

Pilot scope may allow private representations whose later exits are impossible.

Mitigation:

- unresolved lifecycle status explicit;
- no full deployment support claim;
- required future exits recorded from Phase 1;
- release requires complete reachability.

### 31.8 Minimality overclaim

A safe explicit plan may be labeled minimal merely because confidential
target support is not modeled.

Mitigation:

- target-relative wording;
- unsupported capability reported;
- separate policy override;
- accepting metamorphisms required for minimality claim.

### 31.9 Analysis artifact proliferation

Many derivative reports may create maintenance burden.

Mitigation:

- typed in-memory outputs first;
- one canonical analysis publication if needed;
- no file per internal map by default.

---

## 32. Definition of done

The compiler package plan is fulfilled for Phase 2 when the repository can take
the validated two-operation `RealizationSpec` and deterministically produce one
target-independent analyzed program whose relation graph is complete and
source-provenanced; whose checked constants are folded; whose proof,
disclosure, fact-source, constructibility, lifecycle, placement, layout,
target-capability, and coverage requirements are explicit; whose pilot
operations can be inspected by downstream backend code; and which contains no
concrete target opcode, target program, transaction index, generated-file
dependency, model-source dependency, or release overclaim.

---

## 33. One-line package contract

> `compiler` consumes a validated typed realization and deterministically
> derives a complete source-provenanced relation DAG plus proof alternatives,
> disclosure reasons, fact-source and witness-availability requirements,
> lifecycle paths, placement and canonical-layout requirements, target
> capability requirements, and relation coverage obligations—while emitting no
> target code and never reading generated publications or model source.
