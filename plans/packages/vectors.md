# Translation-Validation Vectors Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/vectors`
> **Planned Cargo package:** `tripod-vectors`
> **Planned Rust library name:** `vectors`
> **Implementation phases:** Phase 4 onward — first complete
> `compact-ash` backend evidence; expanded operation by operation through
> Phase 12
> **Depends on packages:** `architecture`, `realization`, `model`, `compiler`,
> `target-elements`, `tapscript`, `linker`, and `transaction`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/wide-arithmetic.md`](../research/wide-arithmetic.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> and [`../research/settlement-layout.md`](../research/settlement-layout.md)
> determine later vector families
> **Authority:** Evidence generation and comparison only; no authority over
> architecture, realization semantics, target semantics, or release policy
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `vectors` package will provide translation-validation evidence for exact
linked attestation-contract backend bundles.

It compares:

```text
typed target-independent semantic expectation
```

with:

```text
concrete target transaction behavior under one exact linked bundle
```

It owns:

- semantic vector definitions;
- deterministic fixture identities;
- reference-model execution adapters;
- realization-relation evaluation;
- ABI-driven target transaction materialization;
- valid and invalid target transaction vectors;
- focused semantic mutations;
- relation activation tracking;
- relation-to-carrier coverage matrices;
- representation metamorphisms;
- closed-asset safety vectors;
- mixed-program/branch vectors;
- object-constructor continuity vectors;
- target-native execution orchestration;
- target verdict comparison;
- public semantic projection comparison;
- target resource measurement;
- failure classification;
- deterministic failure shrinking;
- substrate dependency test reports;
- backend integration reports;
- canonical vector and evidence report schemas;
- report identity and reproducibility.

It does not own:

- protocol semantics;
- realization relation definitions;
- model operation semantics;
- compiler proof-plan selection;
- backend program emission;
- transaction ABI definition;
- target opcode semantics;
- linked-bundle construction;
- independent event/query/accounting implementation;
- deployment-profile release acceptance.

The planned evidence direction is:

```text
architecture + realization
        ↓
semantic relation and coverage requirements
        +
model reference execution
        ↓
semantic fixtures and expected results
        +
exact linked bundle and transaction ABI
        ↓
concrete target transaction materialization
        ↓
target-native execution
        ↓
verdict and public-projection comparison
        ↓
relation coverage and resource reports
        ↓
release evidence binding
```

The first complete evidence subject is:

```text
compact-ash
```

The second is:

```text
transfer-live-receipts
```

Evidence expands in the operation order defined by
[`../roadmap.md`](../roadmap.md).

---

## 2. Assurance boundary

The vector harness establishes finite, bundle-specific evidence that:

- the exact linked bundle exposes a carrier for every required in-scope
  relation;
- valid semantic fixtures can be materialized through the exact transaction
  ABI;
- target-native execution accepts required positive vectors;
- focused mutations are rejected where required;
- accepted target transactions produce the expected semantic successor and
  public projection;
- representation alternatives preserve semantic results where claimed;
- prohibited representations and closed-asset escapes reject;
- permissionless construction uses no unavailable private witness;
- mixed target programs cannot combine into an unintended semantic operation;
- constructor continuity rejects wrong metadata or wrong static code;
- resource measurements correspond to exact bundle and ABI identities;
- evidence reports are deterministic and identity-bound.

The vector harness does **not** prove universally that:

- the realization relation set is complete;
- every possible target transaction has been tested;
- the compiler is formally correct;
- target cryptographic assumptions are true;
- one target executor implementation is flawless;
- independent public observers are correct;
- the deployment will relay or confirm;
- the deployment profile is final.

The report must state this finite scope explicitly.

---

## 3. Evidence classes

The package must preserve distinct evidence classes.

### 3.1 Realization/model conformance

Subject:

```text
typed realization relation
versus
executable model behavior
```

Evidence:

- accepted model transitions satisfy active realization relations;
- model certificate/projection agrees with typed observables;
- focused abstract mutations violate expected relations;
- model invariants remain valid after accepted transitions.

This is not target-program evidence.

### 3.2 Compiler-analysis conformance

Subject:

```text
typed realization
versus
AnalyzedProgram and target-plan requirements
```

Evidence:

- complete relation census;
- no relation dropped;
- proof alternatives present;
- disclosure provenance present;
- fact sources present;
- constructibility and lifecycle requirements preserved;
- placement/layout/coverage requirements complete.

This is not emitted-program evidence.

### 3.3 Backend pattern evidence

Subject:

```text
one reusable target proof pattern
```

Evidence:

- typed pattern contract;
- target capabilities;
- stack/program behavior;
- positive and negative vectors;
- resource formula;
- target-native execution where required.

This does not prove the pattern was placed correctly in every bundle.

### 3.4 Linked-bundle translation evidence

Subject:

```text
one exact linked bundle
```

Evidence:

- relation carriers;
- exact linked target programs;
- ABI-driven transactions;
- target-native verdicts;
- semantic successor comparison;
- constructor and relocation continuity;
- bundle-specific resource measurements.

This is the primary translation-validation report.

### 3.5 Target substrate evidence

Subject:

```text
one exact typed target contract and deployment environment
```

Evidence:

- opcode semantics;
- asset/value introspection;
- confidential transaction behavior;
- issuance/reissuance;
- sighash;
- timelocks;
- resource/policy behavior;
- package relay;
- unspendable output behavior;
- other required target claims.

These reports are separate from protocol-operation vectors.

### 3.6 Independent observer evidence

Subject:

1. raw attestation event projection;
2. canonical attestation query;
3. receipt-accounting audit.

These require separately implemented candidate tools.

The `vectors` package may:

- define canonical input fixtures;
- define expected report schemas;
- invoke external candidate tools through explicit adapters;
- validate and compare candidate reports.

It must not present another invocation or wrapper of the model's
`ReferenceIndexer` as independent deployment evidence.

### 3.7 Release evidence

The release package consumes and cross-validates reports.

The vectors package does not decide that the deployment profile is final.

---

## 4. Dependency direction

### 4.1 Broad evidence dependencies are intentional

Unlike semantic packages, the vector harness may depend broadly because its
role is to compare package boundaries.

Expected dependencies include:

```text
architecture
realization
model
compiler
target-elements
tapscript
linker
transaction
```

This broad access is evidence infrastructure, not semantic authority.

### 4.2 Prohibited dependencies

The package should not depend on:

```text
release
```

if release already depends on vector report types, because that would create a
cycle.

Preferred direction:

```text
release → vectors report types
```

The package must not depend on a future independent indexer implementation as
a normal semantic library if that dependency destroys independence.

External candidate tools should use:

- process execution;
- canonical files;
- typed adapter interfaces in a separate integration layer;
- or another clearly documented independence boundary.

### 4.3 Artifacts dependency

If the existing `artifacts` crate owns committed vector/report publications, it
may depend on `vectors`.

Prefer:

```text
artifacts → vectors
```

rather than vector semantics depending on generated artifact files.

If this creates excessive dependency breadth in `artifacts`, a future evidence
artifact package may be justified. Do not create one before needed.

### 4.4 No release-policy ownership

The vector package may define report schemas and whether one report is
internally complete.

The release package decides which reports are mandatory for one deployment
profile.

---

## 5. Normative typed inputs

### 5.1 Architecture and realization identities

Every vector set binds:

- architecture semantic hash;
- architecture behavioural hash;
- realization identity or validated internal realization schema/fingerprint;
- operation scope;
- semantic relation registry.

The package obtains these as typed values.

### 5.2 Executable model

The model provides:

- reference worlds;
- accepted transitions;
- rejected transitions;
- transition certificates;
- invariant results;
- public observables;
- attestation queries;
- accounting audit projections;
- property trace fixtures.

The vector harness uses model APIs and explicit conformance adapters.

It does not parse model source.

### 5.3 Analyzed compiler program

The compiler provides:

- complete relation census;
- source provenance;
- proof alternatives;
- selected target requirements;
- disclosure plan;
- constructibility/lifecycle plan;
- placement requirements;
- layout requirements;
- coverage requirements.

The harness compares compiler relation scope with realization scope before
target execution.

### 5.4 Exact target

The target package provides:

- exact target identity;
- exact deployment-instance identity;
- target-native semantics/provenance;
- encoding rules;
- resource and policy limits;
- evidence requirement registry.

### 5.5 Exact backend and linked bundle

The harness consumes:

- backend configuration identity;
- relocatable bundle identity where pattern/prelink evidence is produced;
- final or candidate linked-bundle identity;
- linked programs;
- relation carriers;
- constructors;
- layouts;
- witness requirements;
- resource formulas;
- calibration status.

Release-quality bundle reports require a final linked bundle unless the report
is explicitly a candidate/prototype report.

### 5.6 Exact transaction ABI

The transaction package provides:

- target and bundle binding;
- operation layouts;
- typed operation requests;
- constructor recipes;
- metadata schemas;
- witness schemas;
- calibrated or candidate bounds;
- canonical transaction construction;
- worst-case fixtures.

### 5.7 Explicit seeds and fixture inputs

Generated evidence must receive explicit:

- property/random seeds;
- mutation seeds;
- deterministic test randomness;
- test keys;
- checkpoint/chain fixture identity;
- target execution configuration.

No canonical report depends on ambient randomness.

---

## 6. Forbidden inputs and behavior

The vector package must not consume:

- architecture JSON/TOML as first-party semantics;
- declassification JSON as compiler policy;
- realization Markdown as parsed semantics;
- plans;
- model source text;
- target opcode Markdown;
- backend disassembly as relation source;
- current time;
- filesystem order;
- environment variables inside pure vector generation;
- production private keys or wallet data;
- unredacted RPC credentials;
- arbitrary candidate reports without schema and identity validation.

The package must not:

- define a missing semantic relation;
- generate expected target behavior from the candidate backend under test;
- claim operation coverage without relation activation;
- count target transaction construction failure as target script rejection
  unless the test explicitly targets construction;
- claim strict mutation independence without recording collateral relations;
- treat mock target execution as production target evidence;
- treat a model wrapper as an independent indexer;
- combine distinct evidence claims into one opaque pass/fail hash;
- repair stale artifacts during checks;
- attach one report to a different target or bundle;
- log secret keys, blinding factors, openings not declared public, or
  credentials.

---

## 7. Typed outputs

The package should produce typed reports before serialization.

### 7.1 Semantic vector registry

Conceptually:

```rust
pub struct SemanticVectorRegistry {
    pub schema_version: SemanticVectorSchemaVersion,
    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub scope: VectorScope,
    pub fixtures: BTreeMap<SemanticFixtureId, SemanticFixture>,
    pub vectors: BTreeMap<VectorId, SemanticVector>,
    pub mutations: BTreeMap<MutationId, SemanticMutation>,
    pub identity: SemanticVectorSetIdentity,
}
```

> Illustrative API; not frozen.

### 7.2 Target vector registry

Conceptually:

```rust
pub struct TargetVectorRegistry {
    pub semantic_vectors: SemanticVectorSetBinding,
    pub target: TargetBinding,
    pub bundle: BundleBinding,
    pub abi: TransactionAbiBinding,
    pub vectors: BTreeMap<TargetVectorId, TargetVector>,
    pub identity: TargetVectorSetIdentity,
}
```

### 7.3 Execution report

Conceptually:

```rust
pub struct TargetExecutionReport {
    pub schema_version: ExecutionReportSchemaVersion,

    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub analyzed_program: AnalyzedProgramBinding,
    pub target: TargetBinding,
    pub bundle: BundleBinding,
    pub abi: TransactionAbiBinding,
    pub vector_set: TargetVectorSetBinding,

    pub results: Vec<TargetVectorResult>,
    pub environment: TargetExecutionEnvironment,
    pub current: bool,
    pub identity: ExecutionReportIdentity,
}
```

### 7.4 Relation coverage report

Conceptually:

```rust
pub struct RelationCoverageReport {
    pub relations: Vec<RelationCoverageEntry>,
    pub missing_positive: Vec<realization::RelationId>,
    pub missing_negative: Vec<realization::RelationId>,
    pub missing_carriers: Vec<realization::RelationId>,
    pub verdict_mismatches: Vec<TargetVectorId>,
    pub complete: bool,
}
```

### 7.5 Representation report

The representation report records separately:

- safety vectors;
- minimality/metamorphic vectors;
- selected representation modes;
- expected public projections;
- lifecycle/constructibility result;
- target proof methods;
- unsupported modes and reasons.

### 7.6 Resource report

The resource report records:

- exact target/bundle/ABI;
- operation and activation branch;
- family counts;
- representation mode;
- predicted resources;
- observed resources;
- target limit verdict;
- policy verdict;
- worst-case objective;
- measurement tool identity.

### 7.7 Substrate report

One target dependency claim per report or clearly separated entry.

The report records:

- requirement ID;
- target;
- tool/source version;
- test name;
- expected result;
- actual result;
- status;
- evidence hash/identity.

### 7.8 External observer comparison report

Separate types for:

- event projection;
- query;
- accounting audit.

Do not combine the reports merely because one tool produces all three.

---

## 8. Public API boundary

### 8.1 Semantic vector derivation

Conceptually:

```rust
pub fn derive_semantic_vectors(
    realization: &realization::RealizationSpec,
    model_fixtures: &ModelFixtureRegistry,
    compiler: &compiler::AnalyzedProgram,
    policy: &VectorPolicy,
) -> Result<SemanticVectorRegistry, VectorError>;
```

> Illustrative API; not frozen.

This function derives or validates vector obligations from typed relation and
coverage registries.

Critical fixed vectors may be declared as typed Rust fixtures in the vectors
package.

### 8.2 Target materialization

Conceptually:

```rust
pub fn materialize_target_vectors(
    semantic: &SemanticVectorRegistry,
    bundle: &linker::LinkedBundle,
    abi: &transaction::TransactionAbi,
    target: &target_elements::ElementsTarget,
    fixtures: &TargetFixtureInputs,
) -> Result<TargetVectorRegistry, VectorError>;
```

### 8.3 Execution

Target execution should be behind an explicit interface.

Conceptually:

```rust
pub trait TargetExecutor {
    type Transaction;
    type Result;

    fn identity(&self) -> TargetExecutorIdentity;

    fn execute(
        &mut self,
        vector: &TargetVector<Self::Transaction>,
    ) -> Result<Self::Result, ExecutionError>;
}
```

> Illustrative interface; not frozen.

The release target executor should drive the exact pinned target environment.

### 8.4 Comparison

Conceptually:

```rust
pub fn compare_execution(
    expected: &SemanticVectorRegistry,
    target_vectors: &TargetVectorRegistry,
    observed: &[ObservedTargetResult],
) -> Result<TargetExecutionReport, VectorError>;
```

### 8.5 Coverage

Conceptually:

```rust
pub fn relation_coverage(
    realization: &realization::RealizationSpec,
    compiler: &compiler::AnalyzedProgram,
    bundle: &linker::LinkedBundle,
    vectors: &TargetVectorRegistry,
    execution: &TargetExecutionReport,
    policy: &CoveragePolicy,
) -> Result<RelationCoverageReport, VectorError>;
```

### 8.6 External candidate tools

A separate adapter may invoke independently implemented observer tools using
canonical files or process execution.

The vectors library should validate reports but must not hide independence
behind one generic in-process trait implementation.

### 8.7 No process execution in pure derivation APIs

Semantic vector derivation and report comparison are pure.

Node/process execution belongs in explicit runner modules or binaries governed
by ADR-010.

---

## 9. Proposed module structure

A likely initial structure is:

```text
vectors/src/
├── lib.rs
├── error.rs
├── identity.rs
├── policy.rs
├── report.rs
│
├── semantic/
│   ├── mod.rs
│   ├── fixture.rs
│   ├── vector.rs
│   ├── mutation.rs
│   ├── expected.rs
│   └── projection.rs
│
├── model_adapter/
│   ├── mod.rs
│   ├── world.rs
│   ├── operation.rs
│   ├── facts.rs
│   ├── certificate.rs
│   └── queries.rs
│
├── materialize/
│   ├── mod.rs
│   ├── request.rs
│   ├── valid.rs
│   ├── invalid.rs
│   └── representation.rs
│
├── execute/
│   ├── mod.rs
│   ├── target.rs
│   ├── regtest.rs
│   └── environment.rs
│
├── coverage/
│   ├── mod.rs
│   ├── relation.rs
│   ├── activation.rs
│   └── matrix.rs
│
├── mutations/
│   ├── mod.rs
│   ├── layout.rs
│   ├── authorization.rs
│   ├── value.rs
│   ├── asset.rs
│   ├── constructor.rs
│   ├── witness.rs
│   └── projection.rs
│
├── representation/
│   ├── mod.rs
│   ├── metamorphism.rs
│   ├── safety.rs
│   └── minimality.rs
│
├── shrink/
│   ├── mod.rs
│   ├── semantic.rs
│   ├── transaction.rs
│   └── trace.rs
│
├── resources/
│   ├── mod.rs
│   ├── predicted.rs
│   ├── observed.rs
│   └── worst_case.rs
│
├── substrate/
│   ├── mod.rs
│   ├── requirement.rs
│   ├── runner.rs
│   └── report.rs
│
└── observers/
    ├── mod.rs
    ├── event.rs
    ├── query.rs
    └── accounting.rs
```

> Illustrative module structure; create only modules justified by active
> evidence work.

Operation-specific vector fixtures may live under:

```text
vectors/src/operations/
```

as operations enter the roadmap.

---

## 10. Semantic fixture model

### 10.1 Purpose

A semantic fixture describes the abstract/model context from which one or more
vectors are derived.

It should be target-independent.

### 10.2 Fixture contents

A fixture may contain or derive:

- initial valid model world;
- operation request;
- canonical semantic order;
- expected accepted/rejected model result;
- expected successor semantic projection;
- active relation set;
- public observables;
- constructibility assumptions;
- representation capability assumptions;
- source fixture provenance.

### 10.3 Fixture identity

Fixture identity binds:

- architecture;
- realization;
- operation;
- initial semantic state;
- operation request;
- order/checkpoint context;
- explicit fixture parameters.

It does not bind:

- target transaction bytes;
- target witness;
- bundle;
- backend proof pattern.

### 10.4 Valid fixture requirement

A positive semantic fixture must:

- begin from an invariant-valid model state;
- use the public/normative transition surface;
- produce an accepted transition;
- satisfy active realization relations;
- identify expected projections.

A corruption fixture must be clearly labeled and must not be used as valid
state evidence.

### 10.5 Reuse

One valid fixture may support several focused mutations.

The registry records the source fixture for every mutation.

---

## 11. Model reference adapter

### 11.1 Role

The adapter converts model inputs/results into realization fact assignments and
expected semantic projections.

It is evidence code.

It must not become the source of realization relation definitions.

### 11.2 Accepted model path

For valid evidence, construct worlds through:

```text
genesis
+
successful execute calls
```

or another explicitly valid model path documented by the model trust boundary.

Directly corrupted worlds are used only for fault evidence.

### 11.3 Fact projection

The adapter projects:

- input family members;
- output family members;
- assets;
- values;
- owners/classes;
- metadata;
- state before/after;
- root before/after;
- canonical deltas;
- open flows;
- event projections;
- public observables.

Every projected fact traces to a realization `FactId`.

### 11.4 Relation activation

Evaluate activation conditions using the realization evaluator and projected
facts.

The report must distinguish:

- relation active and satisfied;
- relation inactive;
- relation active and violated;
- fact projection/evaluation failure.

### 11.5 Expected result

Expected semantic result should include:

```text
accepted/rejected
active relation set
successor semantic projection
public observable projection
event projection
disclosure projection
```

For rejected model operations, expected relation failure may be partial because
the model can reject before every semantic fact is available.

### 11.6 Shared-code disclosure

If the model adapter and realization evaluator share code, the report or package
documentation should state that fact.

Do not call shared-code agreement independent evidence.

---

## 12. Semantic vectors

### 12.1 Vector classes

Semantic vectors include:

1. valid acceptance;
2. semantic rejection;
3. focused relation mutation;
4. representation metamorphism;
5. lifecycle/constructibility case;
6. mixed-operation case;
7. boundary arithmetic case;
8. public-observable case;
9. resource-objective fixture.

### 12.2 Semantic vector record

Conceptually:

```rust
pub struct SemanticVector {
    pub id: VectorId,
    pub operation: architecture::OperationId,
    pub fixture: SemanticFixtureId,
    pub case: SemanticCase,
    pub primary_relations: BTreeSet<realization::RelationId>,
    pub activated_relations: BTreeSet<realization::RelationId>,
    pub expected: ExpectedSemanticResult,
    pub provenance: VectorProvenance,
}
```

> Illustrative API; not frozen.

### 12.3 Coverage intent

A vector may cover several relations.

The report must distinguish:

- primary relation under test;
- other activated relations;
- positive coverage;
- negative coverage;
- incidental activation.

Do not assign negative independence merely because the vector has one primary
label.

### 12.4 Canonical fixed vectors

Security-critical relations should have fixed named vectors even if property
tests also cover them.

Fixed vectors support:

- review;
- reproducibility;
- external implementations;
- regression diagnosis;
- release identity.

### 12.5 Generated semantic vectors

Property strategies may generate additional fixtures.

Generated release regressions should record:

- strategy version;
- seed;
- shrunk case;
- canonical typed fixture;
- expected result.

The release vector set should contain stable concrete cases, not only a request
to rerun a random search.

---

## 13. Mutation model

### 13.1 Mutation purpose

A mutation starts from an otherwise valid fixture or concrete target
transaction and changes one focused fact or structural relation.

The goal is to demonstrate that the target rejects a transaction that violates
a required relation.

### 13.2 Typed mutation record

Conceptually:

```rust
pub struct SemanticMutation {
    pub id: MutationId,
    pub source: SemanticFixtureId,
    pub intended_relation: realization::RelationId,
    pub changed_facts: BTreeSet<realization::FactId>,
    pub collateral_relations: BTreeSet<realization::RelationId>,
    pub operation: MutationOperation,
    pub expected: ExpectedMutationResult,
}
```

> Illustrative API; not frozen.

### 13.3 Mutation layers

Mutations may occur at different layers:

#### Semantic mutation

Changes a typed operation fact before target materialization.

Example:

```text
change one receipt output class from live to time-locked
```

#### ABI/layout mutation

Changes concrete family placement or witness order.

Example:

```text
move sponsor change into the protocol receipt range
```

#### Target transaction mutation

Changes serialized target fields.

Example:

```text
replace one explicit U asset ID
```

#### Witness mutation

Changes signature, opening, arithmetic witness, selected leaf, or control path.

#### Bundle/constructor mutation

Changes linked static root, constructor key, or operation program reference.

The report must identify the mutation layer.

### 13.4 Mutation independence

Strict one-relation independence is desirable but not always possible.

A mutation is strictly independent only when evidence shows:

- the intended relation changes;
- all other active relation facts remain unchanged;
- the transaction still reaches the intended carrier.

Otherwise, label it:

```text
focused but not strictly independent
```

and list collateral relations.

Do not overstate MC/DC-style independence.

### 13.5 Reachability

A mutation that fails during transaction construction does not demonstrate the
target carrier rejects it.

When the evidence claim is target enforcement, the harness must materialize a
concrete transaction that reaches target execution.

Constructor-level rejection remains useful but is a distinct evidence result.

### 13.6 Mutation preservation

Some target mutations require recomputing:

- transaction IDs;
- signatures;
- confidential proofs;
- witness commitments;
- control data.

The harness must distinguish:

1. mutate before signing/proof generation and recompute valid surrounding
   evidence;
2. mutate after signing to test signature commitment;
3. deliberately provide malformed proof/signature.

The mutation intent determines which is correct.

---

## 14. Target vector materialization

### 14.1 ABI-driven construction

Valid target vectors are built through the `transaction` package using:

- exact bundle;
- exact ABI;
- typed operation request;
- target fixture input view;
- test-only signer adapters;
- deterministic test randomness.

Do not hand-code one valid target transaction shape independently.

### 14.2 Invalid target transactions

Some invalid vectors cannot be produced by the safe transaction API because the
API correctly rejects them.

The vectors package may use a lower-level **mutation API** over a valid
transaction to create malformed cases.

That API belongs in the vectors/evidence layer and must:

- preserve source vector identity;
- record exact changes;
- avoid becoming a production construction path;
- validate that the mutated bytes differ as intended;
- keep secret handling safe.

### 14.3 Construction failure classification

If semantic input cannot be materialized because:

- target capability absent;
- representation unsupported;
- witness unavailable;
- ABI incomplete;
- transaction library rejects invalid request;

record a construction/planning result.

Do not count it as target execution coverage unless the coverage requirement is
specifically about constructor rejection.

### 14.4 Signing and proof fixtures

Use test-only:

- owner keys;
- operator keys;
- sponsor keys;
- blinding factors;
- rangeproof randomness;
- public-opening fixtures.

They must be deterministic for canonical release vectors and clearly marked
non-production.

### 14.5 Target vector identity

Bind:

- semantic vector;
- mutation;
- target;
- bundle;
- ABI;
- concrete transaction template;
- concrete transaction/witness bytes;
- test fixture key/randomness identity;
- expected result.

Do not include secret values in a public report unless they are explicitly
test fixture material intended for publication.

---

## 15. Target-native execution

### 15.1 Preferred release executor

The preferred release executor is the exact pinned Elements node running the
selected regtest/deployment-compatible target.

It validates:

- transaction consensus;
- target script execution;
- CT proofs;
- sighash;
- timelocks;
- target resource behavior;
- policy where the test submits through a policy path.

### 15.2 Execution modes

Distinguish:

1. **consensus execution**
   Does the target accept the transaction under consensus rules?

2. **mempool/policy execution**
   Does the selected node policy accept it?

3. **package execution**
   Does the selected package/relay behavior accept the intended group?

4. **script-only diagnostic execution**
   Does a local interpreter evaluate one program under a synthetic context?

Reports must name the mode.

### 15.3 Hermetic environment

The target runner should:

- use a temporary data directory;
- use exact binary/source identity;
- use isolated network ports;
- use deterministic chain setup where practical;
- verify target network/genesis;
- verify activation state;
- avoid ambient user config;
- redact credentials;
- collect machine-readable logs;
- clean up on failure.

### 15.4 Execution environment identity

Record:

- target definition identity;
- deployment-instance identity;
- node binary/source revision;
- relevant build configuration;
- network flavor;
- genesis ID;
- activation state;
- executor tool version;
- policy mode.

Exclude:

- RPC secret;
- cookie contents;
- temporary paths;
- hostname;
- process ID;
- wall-clock timing from canonical identity.

### 15.5 Local interpreter

A local interpreter may improve:

- pattern unit tests;
- stack diagnostics;
- shrinking speed;
- failure attribution.

It does not replace exact target-native release execution unless equivalence is
separately established.

### 15.6 Node/process diagnostics

All first-party runner binaries follow ADR-010:

- JSON diagnostics;
- no raw child argv;
- secret redaction;
- correct exit classes;
- no silent stream loss.

---

## 16. Verdict comparison

### 16.1 Result classes

Use explicit classes:

```rust
pub enum VectorOutcome {
    ExpectedAcceptActualAccept,
    ExpectedRejectActualReject,

    ExpectedAcceptActualReject,
    ExpectedRejectActualAccept,

    PlanningUnsupported,
    ConstructionRejected,
    SigningFailed,
    ProofConstructionFailed,
    TargetInfrastructureFailed,
    TargetExecutionFailed,
    ResourceLimitFailed,
    ProjectionMismatch,
}
```

> Illustrative vocabulary; not frozen.

### 16.2 Successful evidence

Only these are semantic verdict matches:

```text
expected accept / actual accept
expected reject / actual reject
```

Even then:

- accepted vectors require semantic projection comparison;
- rejected vectors require reaching the intended evidence layer;
- infrastructure failure is not rejection evidence.

### 16.3 Accepted projection comparison

For accepted vectors, derive target public semantics and compare with expected:

- created object families;
- asset IDs;
- public values/openings;
- owners/classes;
- STATE successor;
- root successor/termination;
- canonical deltas;
- data outputs;
- event projection;
- chain fee;
- disclosure projection;
- target-independent protocol observable.

The target projection adapter is evidence code and must be tested separately.

### 16.4 Rejected atomicity

A rejected transaction must not become part of the chain state used by later
vectors.

The runner should isolate vectors or restore deterministic snapshots.

### 16.5 Unexpected target rejection

An expected-accept/actual-reject result must preserve:

- target error code;
- target mode;
- relevant logs;
- concrete vector identity;
- construction report;
- predicted resources.

Diagnostics are not part of semantic identity unless explicitly normalized.

### 16.6 Unexpected target acceptance

An expected-reject/actual-accept result is a high-severity translation failure.

The harness must:

- preserve the exact target transaction and witness;
- preserve target environment identity;
- attempt deterministic shrinking;
- prevent release;
- avoid automatically rewriting expected results.

---

## 17. Relation activation and coverage

### 17.1 Relation census

Before execution, require equality among:

```text
realization in-scope relation IDs
compiler in-scope relation IDs
linked relation carrier IDs
coverage requirement relation IDs
```

Differences are evidence failures.

### 17.2 Carrier reachability

A carrier counts only if:

- linked program is reachable through the ABI;
- vector executes the relevant program/branch;
- activation condition is true;
- target reaches the carrier rather than failing earlier in construction.

### 17.3 Positive coverage

Positive coverage requires:

- relation active;
- expected transaction accepted;
- semantic projection matches;
- carrier executed or target proof otherwise demonstrated according to the
  target pattern.

### 17.4 Negative coverage

Negative coverage requires:

- intended relation violated;
- concrete target transaction reaches target execution;
- expected rejection observed;
- intended carrier is reachable under the mutated case or the report explains
  which earlier guard rejects;
- mutation provenance recorded.

### 17.5 Conditional relation coverage

For each conditionally active relation, require:

- inactive valid vector;
- active valid vector;
- active invalid mutation.

### 17.6 Duplicated carriers

When one relation is enforced in several programs:

- report every carrier;
- identify primary carrier;
- show every branch that requires the relation has one reachable carrier;
- avoid counting one carrier as coverage for an unrelated branch.

### 17.7 Coverage completeness

A report is complete only when:

- no required relation lacks positive coverage;
- no required relation lacks negative coverage under policy;
- no required relation lacks carrier;
- no verdict mismatch exists;
- no unsupported representation is claimed supported;
- no permissionless construction case depends on private witness.

### 17.8 Coverage is inspired by MC/DC, not automatically MC/DC

The relation coverage model has similarities to modified
condition/decision coverage.

Do not claim formal MC/DC unless the independence and decision structure
requirements are actually met.

Use precise terms:

```text
relation-indexed positive/negative coverage
strictly independent mutation
focused mutation with collateral relations
```

---

## 18. Representation evidence

### 18.1 Separate safety and minimality

Produce separate report sections or report types.

#### Safety

Prove rejection of:

- wrong asset;
- confidential/unclassified closed asset;
- value imbalance;
- wrong class;
- wrong recipient;
- malformed commitment/proof;
- missing authorization;
- output-family escape;
- unavailable lifecycle path.

#### Minimality

Prove acceptance of supported lower-disclosure representations:

- confidential live receipt value transfer;
- confidential sponsor value;
- commitment-preserving relabel where eventually supported;
- public-committed synchronization where supported.

### 18.2 Representation metamorphism

A metamorphism maps one accepted concrete representation into another while
preserving semantic facts.

Conceptually:

```rust
pub struct RepresentationMetamorphism {
    pub id: MetamorphismId,
    pub operation: architecture::OperationId,
    pub source_mode: RepresentationMode,
    pub target_mode: RepresentationMode,
    pub preserved_facts: BTreeSet<realization::FactId>,
    pub expected_public_projection: ProjectionEqualityPolicy,
}
```

> Illustrative API; not frozen.

### 18.3 Metamorphism requirements

A valid metamorphism must preserve:

- semantic input amount;
- closed asset identity;
- authorization;
- recipient relation;
- state successor;
- event projection;
- public observables;
- lifecycle support.

Only declared disclosure differences may change.

### 18.4 Blinded-accept evidence

A valid confidential target transaction demonstrates:

- the target/backend does not unnecessarily require explicit value;
- selected proof plan is constructible;
- public semantic projection matches.

It does not demonstrate all safety relations. Separate rejecting vectors remain
required.

### 18.5 Public opening vectors

When supported, include:

- valid public commitment/opening;
- wrong amount;
- wrong blinding;
- wrong commitment;
- noncanonical opening;
- unavailable opening in permissionless construction;
- residual blinding imbalance;
- successor unable to reuse opening.

### 18.6 Closed asset vectors

For every closed asset seam, include:

- explicit correct asset;
- explicit wrong asset;
- confidential asset commitment;
- unclassified output;
- sponsor output attempting to conceal closed asset;
- issuance destination with wrong asset class.

---

## 19. Mixed-program and branch vectors

### 19.1 Purpose

One object constructor may expose several target programs/leaves.

A malicious transaction may combine programs from incompatible semantic
operations across inputs.

### 19.2 Candidate combinations

Examples include:

- burn program on one receipt and transfer program on another;
- clear relation mixed with another STATE-spending operation;
- relabel local receipt program with wrong coordinator;
- settlement entitlement programs paired with another control;
- sponsor program used inside protocol range;
- operator cadence leaf combined with delayed permissionless assumptions.

### 19.3 Analysis method

Initial method:

- enumerate relevant pairs/tuples from linked constructor program sets;
- derive combined layout/constraint requirements;
- materialize concrete transactions where satisfiable;
- require rejection unless the combination denotes one valid declared
  operation.

A future constraint solver may improve this process.

### 19.4 Report

Record:

- selected program combination;
- semantic operations represented;
- concrete layout;
- satisfiability/materialization result;
- target verdict;
- relation conflicts;
- expected result.

A construction impossibility due to the canonical ABI is valid evidence for ABI
mutual exclusivity, but it is distinct from target script rejection.

---

## 20. Constructor-continuity vectors

After constructor research is accepted, every metadata-dependent constructor
needs vectors for:

- valid predecessor;
- valid successor;
- wrong metadata;
- correct metadata under wrong static code subtree;
- different predecessor/successor witnessed static roots;
- wrong internal key;
- wrong parity;
- wrong schema;
- field-order mutation;
- alternate spendable metadata leaf;
- omitted metadata leaf;
- sibling constructor substitution;
- stale constructor recipe from another bundle;
- dynamic metadata outside domain.

These vectors should exist at:

1. pattern/constructor unit level;
2. linked bundle level;
3. complete operation transaction level where applicable.

---

## 21. Arithmetic vectors

### 21.1 Narrow arithmetic

Cover:

- zero where allowed;
- one;
- maximum safe amount;
- exact ratio;
- nonzero remainder;
- multiplication overflow attempt;
- division by zero;
- malformed width;
- negative fixed-width value where prohibited;
- success flag omitted or false.

### 21.2 Wide floor arithmetic

After research acceptance, cover:

- exact division;
- remainder one;
- remainder `d - 1`;
- quotient zero;
- quotient one;
- maximum operands;
- carry at each limb boundary;
- borrow at each comparison limb;
- quotient one below;
- quotient one above;
- wrong remainder;
- remainder equal to divisor;
- zero divisor;
- malformed limb;
- target signed-overflow boundary.

### 21.3 Differential reference

Expected arithmetic derives from a clear Rust big-integer or checked wide
reference implementation independent of emitted target patterns.

If the realization evaluator uses the same helper, add separate fixed
mathematical fixtures and document the shared implementation.

### 21.4 Formal evidence

If SMT or proof-assistant evidence is added, bind:

- exact pattern bytes/program identity;
- target semantics model version;
- theorem/query identity;
- solver/proof checker version;
- result.

Formal gadget evidence supplements target-native vectors.

---

## 22. Operation vector families

### 22.1 `compact-ash`

Positive:

- minimum two inputs;
- larger batch;
- maximum calibrated batch;
- sponsorless;
- sponsored;
- canonical input permutation normalization.

Negative:

- zero/one ASH input;
- above maximum;
- duplicate input;
- wrong asset;
- wrong object constructor;
- wrong ASH output value;
- zero ASH output;
- two ASH outputs;
- undeclared `U` output;
- confidential closed asset;
- wrong coordinator;
- malformed range count;
- sponsor overlap;
- hidden owner/operator signature;
- wrong program/control path;
- no transition carrier.

Resource:

- maximum ASH batch;
- maximum sponsor batch;
- deepest control path;
- largest witness form.

### 22.2 `transfer-live-receipts`

Positive:

- one-to-one transfer;
- split;
- merge;
- multi-owner transfer;
- maximum input/output counts;
- explicit value;
- confidential value;
- confidential sponsor value where supported.

Negative:

- empty input/output;
- above bounds;
- missing owner;
- wrong owner;
- one missing signer in multi-owner set;
- time-locked input/output;
- ASH output;
- wrong explicit asset;
- confidential asset output;
- value imbalance;
- unclassified `U`;
- sponsor overlap;
- output mutation after signing;
- wrong receipt constructor;
- mixed transfer/burn programs.

### 22.3 Later operation families

Add vectors in roadmap order for:

- announce maturity;
- burn;
- compact/clear;
- redemption;
- cancellation/request;
- admission;
- settlement;
- cycle.

Each package/roadmap phase defines the minimum vector set. The compiler coverage
registry provides the relation census.

---

## 23. Resource measurement

### 23.1 Predicted versus observed

For every measured vector, record:

- backend/linker predicted resources;
- transaction-package predicted resources;
- target-observed resources;
- difference;
- limit verdict.

### 23.2 Complete transaction dimensions

Measure at least:

- transaction weight;
- serialized witness bytes;
- control-path bytes;
- per-input initial stack items;
- peak stack/altstack where observable or computed;
- maximum stack element;
- crypto operations and budget;
- executed target operation/project cost;
- policy acceptance;
- package behavior where relevant.

### 23.3 Worst-case objectives

A report identifies which resource dimension one fixture maximizes.

Do not claim one transaction is universally worst-case without proof.

### 23.4 Measurement identity

Bind:

- target;
- bundle;
- ABI;
- operation;
- candidate/final bounds;
- representation;
- fixture;
- executor/tool version.

### 23.5 Calibration handoff

The calibration orchestrator consumes typed resource reports.

The vector package does not choose final bounds unless that responsibility is
explicitly assigned by a later package/decision.

### 23.6 Formula mismatch

A predicted/observed mismatch is a failure.

Do not silently replace the predicted value in the report and continue.

---

## 24. Substrate evidence

### 24.1 Requirement source

Consume target evidence requirements from `target-elements` and deployment
dependency requirements from architecture.

Map them explicitly.

### 24.2 Report per claim

Each claim report should state:

- architecture dependency ID where applicable;
- target evidence requirement ID;
- exact target;
- source/tool version;
- test name;
- fixture/vector;
- expected behavior;
- observed behavior;
- status;
- evidence identity/hash.

### 24.3 Initial claim families

Expected claims include:

- native asset conservation;
- issuance introspection;
- explicit value introspection;
- confidential value conservation;
- commitment equality;
- authenticated opening if selected;
- output-committing sighash;
- relative timelock;
- target program/constructor verification;
- unspendable output exclusion;
- package relay;
- script emission fidelity;
- policy/resource enforcement.

### 24.4 Deployment-specific claims

Some claims cannot be fully demonstrated on generic regtest, such as production
network activation or functionary settlement.

Mark the evidence boundary honestly and require production/deployment-specific
reports where needed.

### 24.5 No static declaration as evidence

The target package declaring a capability does not satisfy the evidence
requirement.

The report must execute or source-check the required claim under the defined
policy.

---

## 25. Independent observer integration

### 25.1 Required reports

The release requires separately committed:

1. event projection report;
2. attestation query report;
3. receipt-accounting report.

### 25.2 Independence declaration

Every candidate report should include an independence declaration describing:

- implementation repository/package;
- programming language;
- shared libraries;
- shared schemas;
- shared fixtures;
- whether model code is linked;
- whether reference indexer code is reused;
- tool version.

The declaration is review evidence, not an automatic proof of independence.

### 25.3 Event comparison

Compare exact canonical event sequences:

- event kind;
- identity;
- order;
- block hash/context;
- ASH value;
- burn-record sequence;
- clear values;
- residue event sequence in the accounting report.

Do not compare only grouped totals.

### 25.4 Query comparison

Compare canonical query bytes after both queries pass semantic validation.

Include zero-result addresses and every address in the union of observed event
records for finite fixtures.

### 25.5 Accounting comparison

Compare:

- exact residue event sequence;
- historical class totals;
- current state terms;
- current circulating values;
- ASH;
- distribution class remainders.

Aggregate equality does not replace event-sequence equality.

### 25.6 Process adapter

External tool execution must:

- use explicit input/output paths;
- follow ADR-010 for first-party wrappers;
- capture machine-readable reports;
- reject plain text or malformed JSON where canonical reports are expected;
- verify report identities;
- preserve stderr diagnostics separately;
- redact credentials.

### 25.7 Test doubles

Model wrappers and deliberately wrong candidates remain useful for validating
comparison behavior.

Label them as:

```text
differential harness test double
```

not:

```text
independent deployment implementation
```

---

## 26. Failure shrinking

### 26.1 Purpose

When generated traces or large transactions expose a mismatch, reduce it to a
smaller reproducible fixture without losing the failure.

### 26.2 Semantic shrinking

Use property-test strategies to shrink:

- action sequences;
- operation family counts;
- amounts;
- owner sets;
- record sets;
- representation choices;
- lifecycle paths.

### 26.3 Transaction-aware delta debugging

For concrete target failures, attempt deterministic reductions such as:

- remove optional sponsor inputs;
- reduce family count;
- remove nonessential outputs;
- reduce records;
- reduce amount while preserving branch;
- simplify representation;
- simplify witness;
- shorten trace prefix.

### 26.4 Validity preservation

A shrink step must state which invariants it preserves.

For expected-accept failures, preserve semantic validity.

For expected-reject failures, preserve the intended mutation and enough
structure to reach the relevant carrier.

### 26.5 Deterministic shrink order

Use one stable shrink ordering and fixed budget.

Record:

- original vector;
- shrink policy;
- intermediate accepted/rejected states where useful;
- final minimized vector;
- whether minimization is proven or best-effort.

### 26.6 Report retention

Preserve both:

- original release failure;
- minimized regression case.

The minimized case may be promoted to the canonical vector set.

---

## 27. Report schemas and identities

### 27.1 Separate report schemas

Use separate typed report schemas for:

- realization/model conformance;
- compiler analysis;
- backend pattern tests;
- linked-bundle execution;
- relation coverage;
- representation safety;
- representation minimality;
- resource measurement;
- substrate dependencies;
- event projection;
- attestation query;
- receipt accounting;
- script integration.

A release index may reference all reports.

### 27.2 Report envelope

A shared evidence envelope may contain:

```rust
pub struct EvidenceEnvelope<T> {
    pub schema: EvidenceEnvelopeSchema,
    pub kind: EvidenceKind,
    pub tool: ToolIdentity,
    pub architecture: ArchitectureBinding,
    pub realization: Option<RealizationBinding>,
    pub target: Option<TargetBinding>,
    pub bundle: Option<BundleBinding>,
    pub abi: Option<TransactionAbiBinding>,
    pub vector_set: Option<VectorSetBinding>,
    pub report: T,
}
```

> Illustrative API; not frozen.

Optional bindings are permitted only when the report claim genuinely does not
depend on that identity.

### 27.3 Canonical identity

Report identity binds:

- report schema;
- evidence kind;
- tool version;
- all applicable upstream identities;
- canonical result entries;
- explicit test seed/configuration;
- environment identity where relevant.

### 27.4 Identity exclusions

Exclude:

- execution duration;
- wall-clock timestamp;
- temporary path;
- hostname;
- process ID;
- RPC credential;
- private key;
- secret opening;
- nondeterministic log ordering.

A noncanonical diagnostic attachment may contain timing/path data but must be
excluded from the report identity and redacted.

### 27.5 Tool identity

Bind:

- package/tool version;
- source commit where available;
- binary hash if release policy requires it;
- configuration identity.

Tool identity is provenance, not proof by itself.

### 27.6 Unknown-field rejection

Canonical report readers reject unknown fields unless the schema explicitly
defines an extension mechanism.

### 27.7 Report status

Use typed status:

```text
passed
failed
incomplete
unsupported
infrastructure_error
```

Only `passed` satisfies a required release report.

---

## 28. Determinism and reproducibility

### 28.1 Canonical ordering

Order by typed identity:

- fixtures;
- semantic vectors;
- mutations;
- target vectors;
- relations;
- carriers;
- results;
- resource entries;
- evidence requirements.

Do not order by execution completion.

### 28.2 Explicit randomness

Canonical vector generation uses explicit seeds and test randomness.

All seeds appear in nonsecret report fields where appropriate.

### 28.3 Parallel execution

The runner may execute vectors in parallel.

Canonical report ordering remains stable.

Inter-vector state must be isolated or reset deterministically.

### 28.4 No ambient environment in canonical results

Environment facts are explicit typed input.

The runner must not let locale, current directory, host time, or unrelated
environment variables change canonical results.

### 28.5 Repeated report test

Given the same exact inputs and target environment, two runs produce
byte-identical canonical reports.

Target software that produces nondeterministic diagnostics must be normalized
or excluded from canonical report identity.

---

## 29. Error model

Errors should be typed by evidence stage.

Candidate classes include:

```rust
pub enum VectorError {
    UnsupportedVectorSchema,
    UnsupportedReportSchema,
    ArchitectureIdentityMismatch,
    RealizationIdentityMismatch,
    AnalyzedProgramIdentityMismatch,
    TargetIdentityMismatch,
    BundleIdentityMismatch,
    AbiIdentityMismatch,
    VectorSetIdentityMismatch,

    MissingRelation(realization::RelationId),
    RelationCensusMismatch,
    MissingCarrier(realization::RelationId),
    UnreachableCarrier(realization::RelationId),
    MissingPositiveCoverage(realization::RelationId),
    MissingNegativeCoverage(realization::RelationId),

    InvalidSemanticFixture(SemanticFixtureId),
    ModelExecutionFailed(SemanticFixtureId),
    ModelInvariantFailure(SemanticFixtureId),
    FactProjectionFailed(SemanticFixtureId),
    RelationEvaluationFailed(realization::RelationId),
    ExpectedProjectionUnavailable(VectorId),

    InvalidMutation(MutationId),
    MutationDidNotChangeIntendedRelation(MutationId),
    MutationNotMaterializable(MutationId),
    MutationCollateralSetMismatch(MutationId),

    TargetMaterializationFailed(TargetVectorId),
    SigningFixtureFailed(TargetVectorId),
    ProofFixtureFailed(TargetVectorId),
    TargetSerializationFailed(TargetVectorId),

    TargetExecutorUnavailable,
    TargetEnvironmentMismatch,
    TargetInfrastructureFailure(TargetVectorId),
    TargetExecutionFailure(TargetVectorId),

    VerdictMismatch(TargetVectorId),
    ProjectionMismatch(TargetVectorId),
    ConstructionFailureMisclassified(TargetVectorId),

    RepresentationSafetyFailure(TargetVectorId),
    RepresentationMinimalityFailure(TargetVectorId),
    ConfidentialClosedAssetAccepted(TargetVectorId),
    PermissionlessPrivateWitness(TargetVectorId),

    ResourcePredictionMissing(TargetVectorId),
    ResourceMeasurementFailed(TargetVectorId),
    ResourceMismatch(TargetVectorId),
    TargetLimitExceeded(TargetVectorId),

    ExternalReportMalformed(EvidenceKind),
    ExternalReportIdentityMismatch(EvidenceKind),
    IndependentImplementationRequirementNotMet(EvidenceKind),

    ShrinkFailed(TargetVectorId),
    NonDeterministicVectorGeneration,
    NonDeterministicReport,
}
```

> Illustrative vocabulary; not frozen.

Errors and reports must not leak secret fixture material unless that material is
explicitly public test data.

---

## 30. Testing strategy

### 30.1 Unit tests

Cover:

- fixture identities;
- semantic vector identities;
- mutation identities;
- relation activation;
- coverage matrix;
- result classification;
- canonical ordering;
- report serialization;
- unknown-field rejection;
- identity mismatch;
- deterministic shrinking;
- redaction/secret-safe report projection.

### 30.2 Model adapter tests

For known model fixtures:

- projected facts match expected;
- active relation set correct;
- semantic successor correct;
- event projection correct;
- corrupted world clearly labeled;
- model rejection not confused with target rejection.

### 30.3 Materialization tests

- valid request produces ABI-valid target transaction;
- semantic mutation maps to intended target field;
- safe transaction API rejects invalid production request;
- low-level mutation path produces intended malformed target transaction;
- signing/proofs recomputed when mutation requires;
- target vector identity stable.

### 30.4 Executor tests

- exact target environment validated;
- expected accept;
- expected reject;
- infrastructure failure classification;
- policy versus consensus mode distinction;
- isolated state reset;
- secret redaction.

### 30.5 Coverage tests

- relation missing from compiler;
- relation missing carrier;
- relation inactive in all positive vectors;
- negative mutation missing;
- duplicate carrier;
- conditional branch missing inactive case;
- focused versus strict independence labeling.

### 30.6 Representation tests

- explicit and confidential positive equivalence;
- wrong commitment;
- confidential closed asset;
- missing public opening;
- unavailable permissionless witness;
- unsupported minimality claim;
- public projection mismatch.

### 30.7 Report tests

- deterministic bytes;
- changed bundle invalidates report binding;
- changed target invalidates report binding;
- unknown field rejected;
- zero/missing report identity rejected;
- diagnostic attachment excluded from canonical identity.

### 30.8 External observer test doubles

Use deliberately:

- over-recognizing event candidate;
- under-recognizing event candidate;
- offsetting-event candidate;
- wrong-clear-assignment query candidate;
- wrong-accounting candidate.

These prove the comparison harness detects differences.

They are not independent deployment evidence.

---

## 31. Initial `compact-ash` evidence gate

The first complete bundle evidence must include:

### Semantic/model

- architecture/realization relation census;
- valid model fixture;
- invariant-preserving successor;
- exact ownerless `U` conservation;
- no specialized event.

### Compiler

- every relation lowered;
- proof alternatives selected;
- disclosure plan;
- public constructibility;
- placement/layout/coverage requirements.

### Backend/linker

- every relation has carrier;
- linked ASH constructor;
- linked operation programs;
- no unresolved relocation;
- deterministic bundle.

### Transaction

- ABI-valid request;
- canonical transaction;
- no private protocol witness;
- valid sponsorless and sponsored cases;
- worst-case candidate fixtures.

### Target positive

- minimum valid batch accepted;
- larger valid batch accepted;
- selected maximum candidate accepted;
- expected semantic ASH output observed.

### Target negative

- too few inputs;
- above bound;
- wrong asset;
- wrong object;
- wrong output value;
- wrong output count;
- unclassified `U`;
- confidential closed asset;
- wrong coordinator;
- malformed range;
- sponsor overlap;
- hidden signature requirement mutation;
- wrong program/control path.

### Resource

- predicted/observed report;
- target limits;
- candidate-bound verdict.

### Coverage

- no missing positive;
- no missing negative;
- no missing carrier;
- no verdict/projection mismatch.

Only after this gate may the operation be described as translation-validated for
the exact tested bundle/target.

---

## 32. Live-transfer evidence gate

Add:

### Owner authorization

- one owner;
- several owners;
- missing one owner;
- wrong owner;
- output mutation after signing.

### Closure

- live-only output;
- time-locked substitution;
- ASH substitution;
- unclassified `U`;
- wrong explicit asset;
- confidential asset escape.

### Value

- explicit split;
- explicit merge;
- explicit imbalance rejection;
- confidential split/merge acceptance;
- confidential imbalance rejection;
- same public semantic projection.

### Sponsor

- explicit sponsor value;
- confidential sponsor value where supported;
- sponsor overlap;
- sponsor value cannot alter receipt conservation.

### Mixed programs

- transfer/burn leaf combination;
- wrong coordinator;
- local owner leaf under another global operation.

### Minimality

- blinded-accept vector for receipt values;
- report distinguishing minimality from safety.

---

## 33. Generated artifacts

### 33.1 Candidate artifact set

Potential committed outputs include:

```text
vectors/
├── semantic-vectors.json
├── target-vectors.json
├── canonical-transactions/
├── relation-coverage.json
├── representation-safety.json
├── representation-minimality.json
├── resource-report.json
├── substrate-reports/
└── integration-report.json
```

Names and structure are illustrative.

### 33.2 Publication discipline

Each committed artifact has:

- one typed source;
- one explicit generator;
- one non-writing checker;
- deterministic path and byte ordering;
- schema;
- unknown-field rejection;
- identity binding;
- no production secret;
- no reverse semantic dependency.

### 33.3 Binary transactions

Canonical binary transaction files may be committed when useful.

A manifest must bind:

- file path;
- vector ID;
- target;
- bundle;
- ABI;
- expected result;
- byte hash.

### 33.4 Size policy

Large vector sets may require:

- compact canonical formats;
- generated-on-demand release artifacts;
- external release storage;
- LFS, only through a separate repository policy decision.

Do not introduce LFS silently.

### 33.5 Test artifacts versus release reports

Development fuzz output is not automatically a canonical release artifact.

Promote selected failures into stable regression fixtures deliberately.

---

## 34. Binaries and ADR-010

Likely first-party binaries include:

```text
generate-vectors
check-vectors
run-target-vectors
compare-observer-reports
measure-resources
```

Names are illustrative.

All binaries follow ADR-010:

- JSON diagnostics on stderr;
- JSON result object/NDJSON on stdout when applicable;
- stdout TTY refusal for result data;
- explicit output paths for files;
- no plain text;
- no raw secret-bearing argv;
- exit 0/1/2 classes;
- no test/check mutation of tracked files;
- panic payload hidden without debug.

Asset-generating commands write only to caller-selected output directories.

Check commands are non-writing.

---

## 35. Dependency and unsafe-code policy

The package inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace lints;
- first-party `unsafe_code = "deny"`;
- Cargo `--locked`;
- permissive dependencies;
- deterministic canonical output;
- no hidden network access in library code;
- target process execution only in explicit integration runners.

Potential dependencies include:

- `proptest` for generated semantic cases;
- `tempfile` for hermetic target environments;
- `serde`/`serde_json` for reports;
- exact target transaction/client libraries;
- subprocess/process control support;
- hashing libraries;
- big-integer reference arithmetic.

Every dependency must be workspace-owned and reviewed.

An SMT solver or proof tool should initially be an optional external evidence
tool, not a load-bearing dependency of the core vector library.

---

## 36. Performance and test-lane policy

### 36.1 Lanes

Separate:

1. fast pure unit tests;
2. model/realization conformance;
3. compiler/backend pattern tests;
4. target-native pilot vectors;
5. full operation target vectors;
6. generated/property exploration;
7. worst-case resource measurement;
8. external observer comparison;
9. release evidence lane.

### 36.2 Local versus release

Developers should be able to run focused operation vectors.

Release runs the complete required matrix.

No lane may be silently skipped in release because it is slow. A missing
external tool is a recorded incomplete/failure according to release policy.

### 36.3 Caching

Cache only by exact identity:

- target;
- bundle;
- ABI;
- vector set;
- executor/tool;
- environment;
- explicit seed.

A cached report must be revalidated before release attachment.

### 36.4 Parallelism

Use parallel target execution only when environment isolation is guaranteed.

Canonical report order remains independent of execution order.

---

## 37. Milestones

### V1 — Package and report foundation

Deliver:

- workspace package;
- typed vector/report identities;
- result classes;
- deterministic serialization;
- no target execution yet.

### V2 — Model/realization adapter

Deliver:

- pilot semantic fixtures;
- fact projection;
- relation activation;
- expected semantic results.

### V3 — Compiler coverage skeleton

Deliver:

- relation census comparison;
- carrier requirements;
- positive/negative requirement registry.

### V4 — ABI target materialization

Deliver:

- valid target transaction generation through transaction package;
- typed mutation layer;
- target vector identity.

### V5 — Target-native executor

Deliver:

- hermetic Elements regtest runner;
- environment validation;
- target verdict reports;
- ADR-010 binaries.

### V6 — `compact-ash` complete report

Deliver all Phase-4 evidence described above.

### V7 — Resource measurement and calibration handoff

Deliver:

- worst-case reports;
- predicted/observed comparison;
- typed calibration input.

### V8 — Live-transfer safety and minimality

Deliver:

- explicit/confidential vectors;
- owner authorization;
- closed-asset safety;
- mixed-program vectors.

### V9 — Constructor continuity

Deliver after STATE constructor research.

### V10 — Arithmetic evidence

Deliver after wide-arithmetic research.

### V11 — Later operation evidence

Add in roadmap order.

### V12 — Independent observer integration

Deliver validated event/query/accounting reports from separate implementations.

### V13 — Final release evidence set

Deliver complete deterministic reports for Phase 12.

---

## 38. Phase-4 vector exit criteria

The vector package is ready to provide complete `compact-ash` evidence when:

- [ ] `packages/vectors` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] dependency direction is acyclic;
- [ ] semantic fixture and vector schemas are typed;
- [ ] model/realization adapter validates the pilot fixture;
- [ ] realization/compiler/linker relation censuses agree;
- [ ] every relation has coverage requirements;
- [ ] target vectors derive from the exact candidate/final ABI;
- [ ] invalid vectors use a clearly separate mutation path;
- [ ] target environment identity is validated;
- [ ] target-native runner distinguishes consensus/policy/infrastructure
      results;
- [ ] accepted vectors compare semantic projections;
- [ ] rejected vectors reach the intended evidence layer;
- [ ] relation activation is recorded;
- [ ] focused mutations record collateral relations;
- [ ] no relation lacks a carrier;
- [ ] no positive/negative coverage is missing;
- [ ] predicted and observed resource reports exist;
- [ ] worst-case fixtures are valid;
- [ ] reports bind target/bundle/ABI/vector identities;
- [ ] reports are deterministic;
- [ ] no production secret appears;
- [ ] ADR-010 runner behavior is tested;
- [ ] debug/release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

---

## 39. Final evidence exit criteria

The vectors/evidence set is release-ready only when:

- [ ] every approved deployment operation has complete relation coverage;
- [ ] realization, compiler, linker, and coverage relation censuses agree;
- [ ] every required relation has reachable carrier;
- [ ] every required relation has activated positive coverage;
- [ ] every required relation has negative coverage under the selected policy;
- [ ] conditional relations have inactive/active/invalid cases;
- [ ] representation safety report passes;
- [ ] every claimed minimality mode has an accepting metamorphism;
- [ ] closed-asset confidential/unclassified escape vectors reject;
- [ ] permissionless construction vectors use no hidden private witness;
- [ ] mixed-program analysis is complete for the selected policy;
- [ ] constructor continuity vectors pass;
- [ ] arithmetic boundary vectors pass;
- [ ] target-native execution uses the exact pinned target;
- [ ] consensus and policy results are distinguished;
- [ ] predicted/observed resources agree;
- [ ] every calibrated bound has complete affected-operation measurements;
- [ ] substrate reports satisfy every required dependency claim;
- [ ] independent event report is present and passes;
- [ ] independent query report is present and passes;
- [ ] independent accounting report is present and passes;
- [ ] observer independence declarations are present;
- [ ] script-integration report is present and passes;
- [ ] every report identity matches the final target, bundle, ABI, and vector
      set;
- [ ] reports are canonical and reproducible;
- [ ] no required result is incomplete, unsupported, or infrastructure-error;
- [ ] non-writing checkers report current artifacts;
- [ ] the checkout remains clean.

The release package still performs final cross-report and deployment-profile
validation.

---

## 40. Non-goals

The vector package does not:

- define protocol semantics;
- define realization relations;
- choose compiler proof plans;
- emit target scripts;
- link bundles;
- define transaction layouts;
- hold production keys;
- provide a production wallet;
- broadcast production transactions;
- guarantee confirmation;
- prove cryptographic hardness;
- prove the whole compiler correct;
- replace independent observer implementations;
- decide release waivers;
- mutate tracked artifacts during tests;
- count random-test volume as relation completeness;
- claim finite relation vectors are universal proof.

---

## 41. Open questions

### 41.1 Report-envelope ownership

Should shared evidence envelope types live:

- in `vectors`;
- in `release`;
- in a small future evidence-schema package?

Initial preference:

> define them in `vectors` while vectors is the producer and release the
> consumer; extract only if independent producers need a stable lower-level
> package.

### 41.2 Target runner implementation

Choose the exact node/process control library and RPC adapter while preserving:

- recorded test provenance;
- hermetic setup;
- ADR-010;
- secret redaction;
- deterministic reports.

### 41.3 Model fixture ownership

Should reusable valid scenario fixtures remain in model tests, move to a public
model evidence module, or be recreated through public APIs in vectors?

Requirement:

> avoid depending on `#[cfg(test)]` internals from another crate.

Initial preference:

- create public, explicitly evidence-oriented fixture builders in vectors using
  model public APIs;
- extract model helper APIs only when justified.

### 41.4 Target mutation API

Define a low-level but test-only mutation interface without making it a
production transaction bypass.

### 41.5 Coverage-policy schema

Set exact requirements for:

- duplicated carriers;
- mutation independence;
- conditional relations;
- target-only guards;
- representation alternatives.

### 41.6 Resource observation

Determine which target metrics are directly observable and which must be
computed from typed program/transaction structures.

### 41.7 External observer transport

Choose canonical report formats and process invocation boundary for independent
tools.

### 41.8 SMT/proof-tool integration

Decide after the wide-arithmetic spike whether formal gadget reports enter the
required release evidence.

### 41.9 Vector artifact scale

Decide whether all concrete binary transactions are committed or generated
during release.

### 41.10 Production-equivalent target evidence

Define how regtest bundle vectors relate to production network activation and
policy evidence.

---

## 42. Risks

### 42.1 Expected and actual share one bug

The harness may derive both through shared compiler/backend helpers.

Mitigation:

- model/realization expected path;
- ABI-only materialization;
- target-native execution;
- fixed manually reviewed critical vectors;
- independent observers;
- second backend later.

### 42.2 Coverage matrix gives false completeness

Relations may be omitted upstream or vectors may not activate carriers.

Mitigation:

- census equality;
- activation tracking;
- carrier reachability;
- normative review;
- model conformance;
- mutation provenance.

### 42.3 Invalid transaction mistaken for target rejection

Construction, signing, or proof generation may fail before target execution.

Mitigation:

- explicit result classes;
- layer-specific evidence;
- require target execution for target-reject coverage.

### 42.4 Mutation breaks too much

A focused mutation may trigger an earlier unrelated guard.

Mitigation:

- collateral relation report;
- mutate before/after signing intentionally;
- canonical layouts;
- multiple variants;
- honest non-independence labels.

### 42.5 Target environment drift

Node revision or activation may differ from report identity.

Mitigation:

- environment validation;
- exact target binding;
- release cross-check;
- source/tool version report.

### 42.6 Secret leakage

Canonical vectors may include test secrets or runners may log RPC credentials.

Mitigation:

- test-only fixture policy;
- secret wrapper/redacted `Debug`;
- report projection;
- ADR-010;
- secret-leak regression tests.

### 42.7 Resource measurement incompleteness

One fixture may miss a larger branch.

Mitigation:

- objective-specific worst cases;
- branch/relation census;
- complete affected-operation set;
- final remeasurement;
- manual review.

### 42.8 External independence is overstated

Separate process/language may still reuse model logic.

Mitigation:

- independence declaration;
- dependency/source review;
- no model linking where independence is required;
- separate report identities.

### 42.9 Evidence cost delays implementation

Comprehensive vectors are expensive.

Mitigation:

- build evidence operation by operation;
- tiered lanes;
- deterministic caching;
- fixed pilot scope;
- do not defer all work to release.

### 42.10 Report schema proliferation

Many reports become difficult to maintain.

Mitigation:

- shared envelope;
- separate only genuinely distinct claims;
- canonical release index;
- no artifact for every internal map.

---

## 43. Definition of done

The vectors package plan is fulfilled for the initial deployment when the
repository can derive one deterministic semantic vector registry from the exact
realization/model/compiler relation scope; materialize valid and focused-invalid
target transactions through the exact linked bundle and ABI; execute them
against the exact pinned target; compare verdicts and accepted public semantic
projections relation by relation; produce separate safety, minimality,
resource, substrate, and integration reports; validate separately implemented
event, query, and accounting reports; shrink and preserve failures
deterministically; bind every report to exact identities; and do so without
defining semantics, confusing construction failure with target rejection,
overstating mutation independence, leaking secrets, using generated
publications as first-party semantic input, or describing finite evidence as a
verified compiler proof.

---

## 44. One-line package contract

> `vectors` binds the typed realization, executable model, compiler relation
> census, exact target, linked bundle, and transaction ABI into deterministic
> semantic fixtures, concrete target vectors, focused mutations,
> representation metamorphisms, target-native executions, projection
> comparisons, resource measurements, substrate tests, and independent
> observer-report checks—producing relation-indexed bundle-specific evidence
> without owning semantics, target programs, transaction ABI, or final release
> acceptance.
