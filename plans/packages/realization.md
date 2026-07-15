# Typed Realization Package Plan

> **Status:** ACTIVE — NEXT SUBSTANTIAL PACKAGE
> **Planned source directory:** `packages/realization`
> **Planned Cargo package:** `tripod-realization`
> **Planned Rust library name:** `realization`
> **Current implementation phase:** Phase 1 — typed realization foundation
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md)
> **Open research dependencies for Phase 1:** none
> **Later research dependencies:** state-object constructors, wide arithmetic,
> public declassification, and settlement layout affect later operations but do
> not block the two Phase-1 pilots
> **Authority:** Typed implementation declaration subordinate to the Attestation
> paper, realization conformance document, and typed architecture
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `realization` package will provide the typed, target-independent semantic
declaration shared by:

- executable-model conformance tests;
- compiler analysis;
- disclosure/declassification derivation;
- relation-indexed evidence planning;
- future canonical realization publication.

It bridges the gap between:

```text
finite typed architecture
```

and:

```text
target-independent semantic relations required by a compiler
```

The architecture crate declares which assets, roots, objects, operations,
quantities, witnesses, clauses, dependencies, decisions, bounds, limits, tags,
and authorization classes exist.

The realization package declares the target-independent semantic detail needed
to implement those entities, including:

- operation facts and dependencies;
- typed arithmetic expressions;
- operation preconditions;
- object-recognition relations;
- authorization relations;
- exact value relations;
- recipient relations;
- state assignments;
- root succession or termination;
- canonical event projections;
- public observables;
- constructibility requirements;
- witness availability;
- lifecycle requirements;
- representation capabilities;
- target-independent proof alternatives;
- derived declassification.

The package does not execute target programs and does not emit scripts.

The planned direction is:

```text
architecture::Architecture
        +
typed realization declarations
        ↓
validated RealizationSpec
   ├─────────────────────▶ model conformance
   └─────────────────────▶ compiler analysis
```

The first two complete operation declarations are:

1. `compact-ash`;
2. `transfer-live-receipts`.

They are the design tests for the package vocabulary.

---

## 2. Why this package is required

Neither the architecture nor the executable model alone is a sufficient
compiler input.

### 2.1 Architecture is deliberately finite and structural

The architecture owns:

- operation and object registries;
- stable IDs;
- cardinality declarations;
- root use;
- input authorization modes;
- operation authorization classes;
- value-flow classes;
- canonical-delta families;
- data-output families;
- projections;
- quantity readers and writers;
- finite bounds.

It does not fully encode:

- economy-determining formulas;
- expression dependencies;
- exact state assignments;
- recipient formulas;
- constructibility;
- witness availability;
- representation alternatives;
- lifecycle normalization paths;
- target-independent proof alternatives;
- derived disclosure provenance.

Adding those concerns directly to architecture would turn the architecture
manifest into a compiler semantic IR and would blur its authority over finite
enumerated facts.

### 2.2 The model is executable behavior, not a semantic AST

The model distributes behavior across:

- operation constructors;
- checked arithmetic helpers;
- kernel flow declarations;
- shape validation;
- certificate derivation;
- manifest-conformance checks;
- invariant checking;
- history replay;
- indexer and audit code;
- test fixtures.

A compiler cannot safely infer the complete semantic relation by:

- scraping operation source;
- observing one successful transition;
- parsing model labels;
- inspecting test fixtures;
- calling model internals as a hidden semantic engine.

The model remains essential evidence. It will be checked against the typed
realization rather than becoming the compiler's source language.

### 2.3 Compiler analysis needs inspectable relations

The compiler must be able to enumerate and analyze every required relation.

It needs typed access to questions such as:

- Which operation fact does this formula read?
- Which state assignment depends on this amount?
- Which relation requires public disclosure?
- Which witness is available to an arbitrary permissionless constructor?
- Which proof alternatives can establish conservation?
- Which lifecycle exit must remain reachable?
- Which public event is derived?
- Which semantic relation must receive a target carrier?
- Which mutation should independently break that relation?

A target-independent typed realization supplies those answers without
introducing target opcodes or duplicating architecture registries.

---

## 3. Normative inputs

The package consumes typed Rust values only.

### 3.1 Typed architecture

Primary input:

```rust
&architecture::Architecture
```

The architecture provides:

- architecture document identity;
- assets;
- roots;
- objects;
- operations;
- quantities;
- witnesses;
- invariant clauses;
- dependencies;
- decisions;
- bounds;
- amount limits;
- tags;
- authorization evidence.

The realization must use architecture IDs directly where they already identify
the semantic entity.

### 3.2 Typed realization declarations

Not every semantic relation can be mechanically inferred from architecture
arrays.

The package will contain target-independent typed Rust declarations for
semantic content such as:

- formulas;
- state fields and assignments;
- recipient rules;
- constructibility;
- lifecycle;
- public observables;
- semantic proof alternatives.

These declarations are implementation-level normative inputs for the
first-party toolchain, subordinate to the realization document and validated
against architecture.

They must not be authored in:

- JSON;
- TOML;
- Markdown;
- LaTeX;
- source comments;
- model tests.

### 3.3 No deployment instance in Phase 1

The Phase-1 realization derivation should not require concrete deployment
values.

Architecture-bound finite limits remain references such as:

```text
BoundId::AshBatchMax
BoundId::TransferInputMax
BoundId::TransferOutputMax
BoundId::FeeSponsorInputMax
```

The target-independent realization records the bound reference and semantic
minimum. A later compiler/deployment layer resolves the bound to one calibrated
runtime value.

---

## 4. Forbidden inputs

The package must not consume:

- `packages/model/generated/architecture.json`;
- `packages/model/generated/architecture.toml`;
- `packages/model/generated/declassification.json`;
- `packages/model/generated/model_labels.json`;
- `docs/attestation/realization.md` through parsing;
- Attestation LaTeX through parsing;
- any file under `plans/`;
- model Rust source text;
- model test source text;
- target opcode Markdown;
- Elements source code directly;
- tapscript bytecode;
- Simplicity programs;
- environment variables;
- process arguments;
- current time;
- filesystem directory order.

The package may cite normative documents in Rust documentation. It must not
parse them to derive semantics.

The package must not depend on:

```text
model
compiler
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

A future canonical artifact generator may depend on `realization`; the
realization package must not depend on the generator.

---

## 5. Typed outputs

The primary output is a validated target-independent realization value.

Conceptually:

```rust
pub struct RealizationSpec {
    pub architecture: ArchitectureBinding,

    pub objects: BTreeMap<architecture::ObjectId, ObjectRealization>,
    pub operations: BTreeMap<architecture::OperationId, OperationRealization>,

    pub facts: FactRegistry,
    pub expressions: ExpressionArena,
    pub relations: RelationRegistry,

    pub observables: ObservableRegistry,
    pub representation: RepresentationRegistry,
    pub constructibility: ConstructibilityRegistry,
    pub lifecycle: LifecycleRegistry,
    pub proof_alternatives: ProofAlternativeRegistry,

    pub declassification: DeclassificationRegistry,
}
```

> Illustrative API; exact fields and names are not frozen until the Phase-1
> pilot declarations and validation suite pass.

The package should also return structured validation errors and may eventually
return a realization identity.

No stable realization hash should be published in Phase 1 unless its complete
canonical projection, algorithm identifier, and migration policy are reviewed
separately.

---

## 6. Public API boundary

### 6.1 Planned derivation API

The intended entry point is:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

> Illustrative API; not frozen.

The function must be:

- pure;
- deterministic;
- total over a valid supported architecture or fail with structured errors;
- independent of the filesystem;
- independent of target selection;
- independent of deployment constants;
- independent of model implementation.

### 6.2 Planned validation API

The package may expose:

```rust
pub fn validate(
    architecture: &architecture::Architecture,
    realization: &RealizationSpec,
) -> Result<(), Vec<RealizationError>>;
```

or make validated construction the only public constructor.

Preferred direction:

- invalid realization values should be difficult to construct externally;
- internal typed declarations are assembled through one derivation path;
- externally parsed derivative publications, if supported later, must undergo
  complete validation before use.

### 6.3 Planned evaluator API

A target-independent expression/relation evaluator is useful for:

- realization unit tests;
- model-conformance adapters;
- compiler constant-folding tests;
- vector expected-result construction.

Conceptually:

```rust
pub fn evaluate_expression(
    spec: &RealizationSpec,
    expression: ExprId,
    facts: &FactAssignment,
) -> Result<Value, EvaluationError>;
```

```rust
pub fn evaluate_relation(
    spec: &RealizationSpec,
    relation: RelationId,
    facts: &FactAssignment,
) -> Result<bool, EvaluationError>;
```

> Illustrative APIs; not frozen.

The evaluator must not call model operation code or target execution.

### 6.4 No CLI in the realization crate

Phase 1 needs a library, not a standalone command.

If a future inspection or publication binary is added, it should live in an
appropriate tooling crate or a narrowly scoped binary and follow ADR-010.

---

## 7. Identity ownership

### 7.1 Architecture-owned IDs

The realization reuses architecture IDs for architecture entities:

```text
AssetId
RootId
ObjectId
OperationId
QuantityId
WitnessId
InvariantClauseId
DependencyId
DecisionId
BoundId
AmountLimitId
TagId
```

It must not create independent replacements for them.

### 7.2 Realization-owned IDs

The package introduces typed IDs for semantic declarations not represented by
architecture.

Likely examples:

```rust
pub struct FactId(...);
pub struct ExprId(...);
pub struct RelationId(...);
pub struct ObservableId(...);
pub struct ConstructibilityRequirementId(...);
pub struct LifecycleRequirementId(...);
pub struct ProofAlternativeId(...);
```

> Illustrative types; not frozen.

### 7.3 Deterministic local identity

Every operation-local declaration needs a deterministic identity.

A possible structure is:

```text
operation ID
+
typed local semantic key
+
structural fingerprint
```

The exact choice remains to be implemented.

Requirements:

- no source line numbers;
- no memory addresses;
- no filesystem paths;
- no hash-map iteration order;
- no target identity;
- no deployment value;
- no current time;
- no debug-format dependency.

### 7.4 Name and structure

Human-readable semantic names are useful for:

- diagnostics;
- relation coverage reports;
- documentation;
- mutation attribution.

Structural identity is useful for:

- deterministic equality;
- changed-relation detection;
- canonical publication;
- compiler provenance.

The package may carry both:

```text
stable typed/local name
structural fingerprint
```

A display string must not be the sole collision boundary for semantic
identity.

### 7.5 Collision policy

If structural hashes are used:

- use explicit domain separation;
- include type/schema version;
- compare full canonical structure on collision where practical;
- fail closed on duplicate IDs with unequal structure;
- never silently merge unequal relations.

---

## 8. Typed semantic domains

The expression and relation layer must distinguish semantic domains.

The initial domain vocabulary should include only what the pilots require, with
a clear additive path for later operations.

### 8.1 Required Phase-1 domains

At minimum:

```text
Bool
Count
Amount
Asset
Object
Owner
InputReference
OutputReference
```

Later operations will require:

```text
Cycle
BlockAge
Ratio
StateField
Root
Address
Tag
CanonicalOrder
TransactionIdentifier
```

### 8.2 Amount domain

The semantic amount domain corresponds to the realization's checked protocol
amount, currently modeled by `Sat`.

The realization package should not depend on model's `Sat` if doing so would
invert package direction.

Options include:

1. define a target-independent amount type in `realization`;
2. move a generic amount domain to a smaller shared crate later;
3. use a realization-owned domain and explicit model adapters.

For Phase 1, avoid broad package extraction unless the pilots demonstrate a
real shared-type need.

The amount semantics must state:

- nonnegative;
- bounded by the realization arithmetic domain;
- checked addition/subtraction;
- explicit overflow/underflow failure;
- exact equality and ordering.

### 8.3 Count domain

Counts are not monetary amounts.

The count domain must support:

- minimum;
- exact maximum;
- architecture bound reference;
- zero/nonzero;
- bounded family sum where needed.

A count cannot be substituted for an amount merely because both are integers.

### 8.4 Owner and address distinction

Owner keys and attestation addresses are distinct semantic domains.

The realization must not use one generic 32-byte identity type for both without
a typed wrapper preserving the distinction.

### 8.5 Input/output references

References identify semantic family members or transaction-produced values at
the target-independent level.

They must not be concrete tapscript indexes in Phase 1.

The later compiler/backend maps semantic references to canonical target layout
positions.

---

## 9. Fact model

Facts are typed semantic inputs to expressions and relations.

### 9.1 Fact scope

A fact should carry enough scope to identify:

- owning operation;
- semantic source;
- family or state object;
- field;
- value domain;
- availability;
- public/private semantic status;
- source provenance.

### 9.2 Candidate fact families

The eventual vocabulary may include:

```rust
pub enum FactKind {
    InputAsset(InputFamilyId),
    InputValue(InputFamilyId),
    InputOwner(InputFamilyId),
    InputClass(InputFamilyId),
    InputMetadata(InputFamilyId, MetadataFieldId),

    OutputAsset(OutputFamilyId),
    OutputValue(OutputFamilyId),
    OutputOwner(OutputFamilyId),
    OutputClass(OutputFamilyId),
    OutputMetadata(OutputFamilyId, MetadataFieldId),

    InputCount(InputFamilyId),
    OutputCount(OutputFamilyId),

    StateBefore(StateFieldId),
    StateAfter(StateFieldId),

    RootBefore(architecture::RootId),
    RootAfter(architecture::RootId),

    ArchitectureConstant(ArchitectureConstantId),
    BoundReference(architecture::BoundId),
}
```

> Illustrative vocabulary; not frozen.

The pilot operations should determine the first subset.

### 9.3 Family facts

A repeated input/output family requires typed aggregate access, such as:

```text
for each family member:
    asset
    value
    owner
    class
    metadata

over family:
    count
    sum(value)
    set(owner)
```

The realization expresses bounded semantic aggregation. It does not prescribe
target looping or unrolling.

### 9.4 Fact availability

Facts should be associated with availability requirements such as:

```text
public chain data
public opening
current owner secret
operator secret
sponsor local
deployment constant
derived from other facts
```

Availability is needed for constructibility and proof planning.

### 9.5 Fact provenance

Every fact must trace to:

- architecture family;
- realization state field;
- public observable;
- architecture constant;
- deployment-bound placeholder;
- semantic derivation.

Compiler-created target facts should not be inserted into `RealizationSpec` as
though they were abstract semantic facts.

---

## 10. Expression model

### 10.1 Arena/DAG structure

Expressions should be stored in a typed arena or hash-consed DAG rather than
deeply owned recursive trees.

Benefits:

- deterministic IDs;
- structural sharing;
- dependency traversal;
- canonical serialization;
- efficient evaluation;
- explicit type checking;
- compiler provenance.

### 10.2 Candidate expression vocabulary

The eventual vocabulary may include:

```rust
pub enum ExprKind {
    Fact(FactId),
    Constant(ConstantValue),

    Add(ExprId, ExprId),
    Sub(ExprId, ExprId),
    Mul(ExprId, ExprId),

    FloorMulDiv {
        lhs: ExprId,
        rhs: ExprId,
        divisor: ExprId,
    },

    FloorRatio {
        value: ExprId,
        numerator: ExprId,
        denominator: ExprId,
    },

    Min(ExprId, ExprId),
    Max(ExprId, ExprId),

    Eq(ExprId, ExprId),
    Lt(ExprId, ExprId),
    Le(ExprId, ExprId),

    Not(ExprId),
    And(Vec<ExprId>),
    Or(Vec<ExprId>),

    Sum {
        family: FamilyId,
        term: ExprId,
    },

    Conditional {
        condition: ExprId,
        when_true: ExprId,
        when_false: ExprId,
    },
}
```

> Illustrative vocabulary; Phase 1 should implement only what the pilots
> require.

### 10.3 Checked arithmetic

Arithmetic expressions must preserve fail-closed semantics.

The expression language must distinguish:

```text
mathematical result exists
```

from:

```text
operation is rejected because a checked domain operation failed
```

An evaluator must not use wrapping arithmetic.

### 10.4 Floor semantics

Future floor expressions must define exact mathematical flooring, independent
of one target witness or quotient gadget.

The target backend later selects how to prove the relation.

### 10.5 Canonical operands

For commutative expressions, canonical operand ordering may improve stable
identity.

Do not reorder when:

- evaluation order carries fail-closed meaning;
- operands have distinct semantic roles;
- diagnostics rely on left/right provenance;
- subtraction/division/order is noncommutative.

The canonicalization policy must be explicit and tested.

### 10.6 Dependency traversal

Every expression must expose its transitive fact dependencies.

This powers:

- declassification;
- witness availability;
- model fixture projection;
- compiler fact-source planning;
- relation coverage;
- deterministic diagnostics.

---

## 11. Relation model

Relations state what must be true semantically.

### 11.1 Phase-1 relation vocabulary

The two pilots require at least:

- family cardinality;
- object/asset recognition;
- class closure;
- owner authorization;
- permissionless authorization;
- exact value conservation;
- output-family closure;
- sponsor/open-flow isolation;
- projection requirement;
- constructibility requirement;
- representation capability.

The exact type structure may be:

```rust
pub enum RelationKind {
    Cardinality(CardinalityRelation),
    ObjectRecognition(ObjectRecognitionRelation),
    Authorization(AuthorizationRelation),
    Equality(EqualityRelation),
    Conservation(ConservationRelation),
    Closure(ClosureRelation),
    OpenFlow(OpenFlowRelation),
    Projection(ProjectionRelation),
    Constructibility(ConstructibilityRelation),
    Representation(RepresentationRelation),
}
```

> Illustrative vocabulary; not frozen.

### 11.2 Later relation additions

Later operations will require:

- checked comparison;
- floor arithmetic;
- state assignment;
- root succession;
- root termination;
- issuance;
- destruction;
- recipient pin;
- event payload;
- maturity transition;
- cadence band;
- lifecycle normalization;
- reader/firewall relation.

These should be added when the corresponding operation enters the realization,
not preemptively in Phase 1.

### 11.3 Activation conditions

A relation may be unconditional or conditionally active.

Examples:

- ASH residual only when positive;
- issuance only when amount is positive;
- control closure only at terminal settlement;
- maturity conversion only at the announced cycle.

Activation conditions are semantic expressions and must receive their own
dependencies and tests.

### 11.4 Relation dependencies

Relations may depend on other relations.

For example:

```text
closed value conservation
    depends on
        complete family closure
        correct asset identity
        exact issuance/destruction declarations
```

Dependency cycles must be detected and either rejected or represented through a
well-defined mutually supporting semantic group.

Phase 1 should prefer an acyclic relation dependency graph.

### 11.5 Relation provenance

Each relation must identify:

- owning architecture operation;
- source architecture declarations;
- realization semantic key;
- related formula;
- related projection or lifecycle requirement;
- relevant realization-document label where useful.

Document labels are provenance and review aids, not the sole relation identity.

---

## 12. Operation model

An operation realization should collect every target-independent semantic
declaration for one architecture operation.

Conceptually:

```rust
pub struct OperationRealization {
    pub operation: architecture::OperationId,

    pub inputs: Vec<InputFamily>,
    pub outputs: Vec<OutputFamily>,
    pub data_outputs: Vec<DataOutputFamily>,

    pub facts: Vec<FactId>,
    pub expressions: Vec<ExprId>,
    pub relations: Vec<RelationId>,

    pub authorization: OperationAuthorization,
    pub state_assignments: Vec<StateAssignment>,
    pub root_effects: Vec<RootEffect>,
    pub projections: Vec<ObservableId>,

    pub constructibility: Vec<ConstructibilityRequirementId>,
    pub lifecycle_effects: Vec<LifecycleRequirementId>,
    pub representation: Vec<RepresentationRequirement>,
    pub proof_alternatives: Vec<ProofAlternativeId>,
}
```

> Illustrative API; not frozen.

### 12.1 Architecture-derived fields

The operation realization must derive or validate against architecture:

- operation ID;
- input/output families;
- static cardinality minima;
- maximum kind and bound ID;
- authorization modes;
- root use;
- canonical-delta families;
- data-output families;
- open-flow roles;
- quantity reads/writes;
- witness families;
- value-flow classes;
- projection policy.

### 12.2 Realization-declared fields

The package adds target-independent semantic detail such as:

- exact conservation equations;
- state formulas;
- recipient expressions;
- constructibility;
- lifecycle;
- representation latitude;
- proof alternatives;
- public observable contents.

### 12.3 No concrete layout

An operation realization must not contain:

- input index 0;
- output index 2;
- coordinator input;
- sponsor suffix;
- witness stack order;
- tapleaf identity;
- control-block path.

Those belong to compiler/backend/transaction ABI under D006.

---

## 13. Object realization

An object realization describes semantic facts needed below the finite
architecture declaration.

It may include:

- metadata fields and domains;
- semantic owner versus routing destination;
- public observability;
- representation capabilities;
- construction requirements;
- lifecycle exits;
- recognition dependencies;
- constructor semantic parameters;
- consensus-value authority;
- applicable operation roles.

Phase 1 needs object details only for:

```text
ASH
RECEIPT_L
PLAIN_LBTC sponsor role
```

Do not fully redesign every object before the pilot operation vocabulary is
proven.

---

## 14. Authorization model

The realization must preserve the distinction between:

1. input participation authorization;
2. operation authorization.

### 14.1 Input authorization

Architecture-owned modes include:

```text
covenant-companion
input-owner
refund-key
sponsor-owner
permissionless
```

The realization may add semantic requirements such as:

- which fact identifies the owner;
- which output set the authorization commits;
- which operation relation the authorization guards;
- whether authorization applies per family member;
- availability of the required secret.

### 14.2 Operation authorization

Architecture-owned classes include:

```text
client-authorized
refund-key
permissionless
cadence-band
receipt-owners
operator
```

The realization declares the target-independent authorization relation.

It does not declare:

- `CHECKSIG`;
- a sighash byte;
- a target pubkey encoding;
- a specific cadence tapleaf.

### 14.3 Permissionless authorization

Permissionless means:

- no owner secret;
- no operator secret;
- public construction inputs;
- safety through fixed/derived state effects and recipient closure.

The realization must make hidden secret requirements detectable.

---

## 15. Constructibility model

### 15.1 Purpose

Constructibility asks:

> Can an authorized actor obtain all facts and witnesses required to build an
> accepted transition?

This is distinct from:

> Can a target program verify the supplied witnesses?

### 15.2 Candidate availability vocabulary

```rust
pub enum WitnessAvailability {
    CompileTime,
    ArchitectureConstant,
    DeploymentConstant,
    PublicChainData,
    PublicOpening,
    CurrentOwnerSecret,
    OperatorSecret,
    SponsorLocal,
}
```

> Illustrative vocabulary; not frozen.

### 15.3 Constructibility requirements

An operation may declare requirements such as:

```text
all ASH amounts publicly available
all receipt owners available to their own signing clients
all sponsor facts local to sponsor
no private witness required for compact-ash
```

Later:

```text
settlement entitlement principal publicly available
relabel successor proof publicly constructible
clear ASH opening publicly available
```

### 15.4 Compiler consequence

The compiler rejects a proof/representation plan when a required witness:

- is unavailable to the operation's authorized constructor;
- has a stronger secrecy requirement than the operation permits;
- depends on an undeclared off-chain service;
- breaks a required permissionless exit.

---

## 16. Lifecycle model

Lifecycle declarations ensure a supported representation does not strand an
object.

### 16.1 Phase-1 lifecycle requirements

`ASH` must retain:

```text
compact
clear
```

without owner/operator secret dependence.

`RECEIPT_L` must retain:

```text
transfer
burn
redeem
```

under the appropriate authorization and representation paths.

Phase 1 declares the semantic requirement; later target research resolves
concrete declassification/normalization.

### 16.2 Lifecycle graph

The realization may represent a graph over:

```text
object semantic class
×
representation capability
×
authorized transition
```

The compiler later checks target-supported reachability.

No target-specific script node belongs in the realization lifecycle graph.

---

## 17. Representation and proof alternatives

### 17.1 Semantic representation modes

Under D005, the realization may permit:

```text
PrivateCommitted
PublicCommitted
Explicit
```

as semantic representation capabilities.

These are not exact Elements encodings.

### 17.2 Closed asset identity

For the initial deployment policy, closed asset identity is rigid.

The realization should declare the semantic requirement that object identity is
authenticated. The target/compiler policy later requires explicit closed asset
identity for Elements.

Do not encode Elements prefix bytes in realization.

### 17.3 Proof alternatives

Phase-1 alternatives include:

#### `compact-ash`

The values are intended to be publicly usable.

Accepted semantic proof class:

```text
publicly authenticated amount arithmetic
```

The exact explicit/public-commitment representation remains target-selected.

#### `transfer-live-receipts`

Accepted semantic proof alternatives:

```text
explicit value arithmetic
confidential value conservation
```

Both require:

- exact `U` asset classification;
- live receipt class closure;
- owner authorization;
- output-family closure.

### 17.4 Proof alternative identity

A proof alternative is target-independent.

A selected target proof plan later binds:

```text
relation
+
proof alternative
+
target capability
+
backend pattern
```

---

## 18. Observable model

Observables describe public semantic outputs.

Phase 1 needs:

### `compact-ash`

- transition-certificate projection;
- no burn projection;
- no clear projection;
- no residue projection;
- resulting ASH output/value as a semantic successor object.

### `transfer-live-receipts`

- transition-certificate projection;
- no specialized event;
- destination receipt owner/class/value semantic outputs;
- no public value disclosure requirement merely from lateral transfer.

Later observables include:

- STATE successor;
- burn event;
- clear event;
- residue event;
- attestation terms;
- accounting audit.

Observables must preserve reader-role distinctions from architecture.

---

## 19. Declassification derivation

### 19.1 Source

Declassification derives from the typed realization dependency graph.

It is not independently authored.

### 19.2 Demand classes

The initial derivation should support reasons such as:

```text
PublicStateDependency
PermissionlessConstructibility
PublicAuditDependency
```

Target safety and deployment-policy disclosure may be added at compiler
planning time rather than changing abstract declassification.

### 19.3 Algorithm

For each operation:

1. seed facts feeding public state assignments;
2. seed facts feeding public observables/events;
3. seed facts required for permissionless construction;
4. traverse expression dependencies backward;
5. deduplicate and canonically order facts;
6. record reason and provenance per fact;
7. reject unresolved or cyclic dependencies that make the result ambiguous;
8. return a typed result.

### 19.4 Pilot expectation

Expected high-level result:

```text
compact-ash:
    no newly private owner value;
    ASH facts are already required as publicly usable ownerless workflow facts

transfer-live-receipts:
    no economy-determining numerical value disclosure required by the
    target-independent lateral transfer relation
```

The exact publication vocabulary may differ from the existing
`declassification.json` during migration.

Any publication change requires an explicit artifact-schema review.

### 19.5 Compiler consumption

The compiler consumes:

```rust
&DeclassificationRegistry
```

or equivalent typed data.

It never reads `declassification.json`.

---

## 20. Derivation pipeline

The `derive` implementation should follow a deterministic staged pipeline.

### Stage 1 — architecture validation

Require the input architecture to pass the appropriate typed validation.

Phase 1 may accept a valid draft or require the current supported architecture
identity. The policy must be explicit.

Recommended initial policy:

- `derive` accepts a structurally valid supported architecture;
- publication finality is a release concern;
- unsupported schema/realization identity is rejected.

### Stage 2 — architecture indexing

Build deterministic indexes over:

- operations;
- objects;
- assets;
- roots;
- quantities;
- bounds;
- tags;
- witnesses;
- clauses.

Do not rely on source declaration order where semantics are set-like.

### Stage 3 — typed declaration assembly

Load the realization-owned typed declarations for supported objects and
operations.

For Phase 1, complete support is required for the two pilots. Other operations
may be represented as explicitly unsupported/incomplete only if the Phase-1
API and validation report that incompleteness clearly.

Do not return a value falsely claiming complete realization coverage.

Possible approaches:

1. `derive_pilots` during the first milestone;
2. one `RealizationSpec` with explicit supported-operation scope;
3. complete skeleton entries with no release-valid status.

The package plan should choose before implementation.

Preferred approach:

> `RealizationSpec` carries an explicit scope/profile, and validation can require
> complete coverage for that scope. Full release validation later requires all
> architecture operations.

### Stage 4 — fact construction

Create operation/object facts with deterministic IDs and typed domains.

### Stage 5 — expression construction

Create expression DAG, type-check every node, and compute fact dependencies.

### Stage 6 — relation construction

Create relation registry and validate operands, activation, and dependencies.

### Stage 7 — semantic adjuncts

Construct:

- authorization requirements;
- observables;
- constructibility;
- lifecycle;
- representation;
- proof alternatives.

### Stage 8 — declassification derivation

Compute typed declassification from dependencies.

### Stage 9 — cross-validation

Check architecture/realization agreement and internal closure.

### Stage 10 — canonical identity/fingerprint

Compute internal deterministic fingerprints needed for tests and diagnostics.

Do not publish a stable realization hash until the identity policy is accepted.

---

## 21. Validation

### 21.1 Architecture binding

Validate:

- architecture identity/schema supported;
- operation IDs resolve;
- object IDs resolve;
- asset IDs resolve;
- bound IDs resolve;
- tag IDs resolve;
- authorization agrees;
- root use agrees;
- projection declarations agree;
- input/output/data-output families agree;
- quantity readers/writers are not contradicted.

### 21.2 Fact validation

Validate:

- unique FactId;
- declared type;
- valid scope;
- valid source;
- no target-specific source;
- availability declared where needed;
- no owner/address conflation.

### 21.3 Expression validation

Validate:

- unique ExprId;
- all references resolve;
- operand types match;
- result type is known;
- no wrapping arithmetic;
- aggregate family valid;
- no illegal dependency cycle;
- canonical identity stable.

### 21.4 Relation validation

Validate:

- unique RelationId;
- all operands resolve;
- owning operation correct;
- activation expression boolean;
- dependency graph valid;
- architecture provenance exists;
- no target proof masquerades as semantic relation.

### 21.5 Operation validation

Validate:

- operation scope complete;
- every family matches architecture;
- required semantic relation present;
- every relation belongs to the operation or a declared shared relation;
- authorization complete;
- projections complete;
- constructibility complete;
- lifecycle effects complete for supported representations;
- declassification derivable.

### 21.6 Pilot-specific validation

#### `compact-ash`

Require:

- ASH input minimum equals architecture minimum;
- maximum references `ASH_BATCH_MAX`;
- exactly one ASH output;
- no STATE/RESV/root family;
- permissionless authorization;
- ownerless `U` conservation;
- sponsor flow references `FEE_SPONSOR_INPUT_MAX`;
- no specialized burn/clear/residue event;
- transition projection required;
- publicly constructible workflow facts.

#### `transfer-live-receipts`

Require:

- live receipt input/output families;
- nonempty minima;
- maximum references transfer bounds;
- receipt-owner authorization;
- same-class closure;
- `U` conservation;
- sponsor flow references sponsor bound;
- no root family;
- transition projection required;
- explicit and confidential value proof alternatives represented;
- closed asset identity requirement represented;
- owner-secret availability represented.

### 21.7 Complete-scope validation

The package should distinguish:

```text
pilot-valid
full-architecture-valid
release-valid
```

or equivalent validation levels.

A pilot realization must not be mistaken for a complete release realization.

---

## 22. Error model

Errors should be typed, deterministic, and useful for CI.

Candidate classes include:

```rust
pub enum RealizationError {
    UnsupportedArchitectureSchema,
    UnsupportedArchitectureIdentity,
    ArchitectureValidationFailed,

    MissingObjectRealization(architecture::ObjectId),
    MissingOperationRealization(architecture::OperationId),
    UnexpectedObjectRealization(architecture::ObjectId),
    UnexpectedOperationRealization(architecture::OperationId),

    DuplicateFact(FactId),
    DuplicateExpression(ExprId),
    DuplicateRelation(RelationId),

    UnknownFact(FactId),
    UnknownExpression(ExprId),
    UnknownRelation(RelationId),

    TypeMismatch,
    InvalidAggregateFamily,
    ArithmeticDomainMismatch,
    DependencyCycle,

    ArchitectureFamilyMismatch {
        operation: architecture::OperationId,
    },

    AuthorizationMismatch {
        operation: architecture::OperationId,
    },

    ProjectionMismatch {
        operation: architecture::OperationId,
    },

    UnavailablePermissionlessWitness,
    MissingLifecycleExit,
    UndeclaredDisclosure,
    UnresolvedProofAlternative,
}
```

> Illustrative vocabulary; not frozen.

Errors should avoid embedding:

- nondeterministic debug maps;
- host paths;
- source line numbers as stable identity;
- secret values.

A display implementation may include human-readable context while preserving
typed error identity.

---

## 23. Model-conformance boundary

### 23.1 Dependency direction

The realization crate does not depend on model.

Model unit/integration tests may import realization.

A model-conformance adapter may live in:

- model tests;
- a future dedicated conformance module;
- vectors later.

### 23.2 Fact projection

For each model transition, adapters construct a typed `FactAssignment` from:

- predecessor world;
- operation request;
- successor world;
- transition certificate;
- public projections.

The adapter must be reviewed as evidence machinery.

It must not become the source of realization facts.

### 23.3 Positive conformance

For valid pilot transitions:

- every required fact is assignable;
- every expression evaluates;
- every active relation holds;
- expected observable projection matches;
- declassification/public status agrees.

### 23.4 Negative conformance

Use focused model mutations/fault fixtures to show:

- wrong class breaks class closure;
- missing owner breaks authorization;
- wrong output total breaks conservation;
- wrong ASH cardinality breaks cardinality;
- wrong specialized event breaks projection policy;
- private/unavailable fact breaks permissionless constructibility where
  represented.

Model guards and realization relation failures need not have one-to-one names.
The mapping must be documented where evidence depends on it.

### 23.5 Avoid false independence

If the model and realization evaluator share one arithmetic function, tests
must state that shared implementation.

Critical formulas later require independent boundary vectors or formal target
evidence.

---

## 24. Tests

### 24.1 Unit tests

Add tests for:

- deterministic IDs;
- fact typing;
- expression typing;
- checked arithmetic;
- dependency traversal;
- relation validation;
- duplicate rejection;
- scope validation;
- canonical ordering;
- declassification derivation;
- constructibility;
- lifecycle reachability.

### 24.2 Architecture mutation tests

Mutate architecture input or realization declaration and require failure for:

- missing pilot operation;
- wrong object family;
- wrong bound reference;
- wrong authorization;
- wrong projection;
- wrong open-flow role;
- wrong value-flow class;
- transposed operation mapping;
- duplicate relation.

### 24.3 Determinism tests

Require:

- repeated derivation equality;
- declaration-order permutation does not alter set-like semantic IDs;
- target environment does not alter realization;
- filesystem working directory does not alter output;
- no environment variable changes the spec.

Avoid tests that mutate global environment concurrently without serialization.

### 24.4 Model-conformance tests

Add model-side tests for both pilots as described above.

### 24.5 Public API tests

An integration test should prove the downstream compiler-facing public API can:

- derive the pilot realization;
- enumerate pilot operations;
- inspect facts/relations;
- obtain typed declassification;
- evaluate pilot relations from fact assignments;
- do so without target or model internals.

### 24.6 Compile-fail/API-boundary tests

Consider compile-fail tests only if needed to enforce:

- target types cannot enter realization APIs;
- invalid IDs cannot be freely forged;
- internal builders remain private.

Do not add a heavy compile-test framework without a concrete boundary need.

---

## 25. Determinism and identity

### 25.1 Canonical ordering

Use deterministic order based on stable typed IDs.

Likely:

- architecture operations by architecture code;
- objects by architecture code;
- facts by scoped semantic key;
- expressions by structural identity;
- relations by operation then semantic key;
- observables by typed ID;
- declassification facts by typed fact ID.

### 25.2 No declaration-order semantics by accident

If relation order is not semantic, changing source declaration order must not
move realization identity or derivative bytes.

If relation order is semantic for evaluation or evidence, state and test that
explicitly.

### 25.3 Internal schema version

Even before a published realization hash, the in-memory/canonical structural
format should carry an internal schema version for tests and future migration.

Do not reuse architecture schema 17 as the realization schema.

### 25.4 Hash publication gate

Before publishing a realization hash, write a focused decision or implemented
ADR defining:

- algorithm identifier;
- domain separator;
- body projection;
- envelope fields;
- included/excluded declarations;
- canonical ordering;
- error behavior;
- mutation expectations;
- migration process;
- versioning relation to architecture identity.

### 25.5 Reproducible diagnostic output

Canonical diagnostics/reports must not depend on map iteration.

Human error text may evolve without changing semantic identity.

---

## 26. Generated artifacts

### 26.1 Phase-1 default

No new committed realization artifact is required merely to create the crate.

Prefer typed in-memory values and tests first.

### 26.2 Declassification migration

The existing `declassification.json` may eventually become a publication of the
realization's typed derivation.

Before switching ownership:

- cover all operations or clearly version a pilot artifact;
- compare old/new semantic intent;
- define schema;
- update the artifacts crate;
- preserve generator/check separation;
- update realization document claims if wording changes;
- ensure compiler consumes the typed result.

### 26.3 Future realization publication

A future `realization.json` may be useful for:

- independent implementations;
- review;
- report provenance;
- external interoperability.

It is not required for first-party compilation and must remain derivative.

### 26.4 Artifact ownership

If new artifacts are added, assign one generator owner. Avoid adding ad hoc
writers inside the realization library.

---

## 27. Dependency and unsafe-code policy

The package inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace dependency versions;
- `--locked`;
- workspace lints;
- `unsafe_code = "deny"`;
- no target FFI;
- permissive dependencies only;
- deterministic generation;
- no build script unless strictly necessary.

Likely initial dependencies:

```text
architecture
```

Possible later dependencies, added deliberately:

```text
serde
sha2
thiserror
```

Do not add:

- parser generators;
- target libraries;
- async runtimes;
- proc-macro frameworks

without a demonstrated need.

---

## 28. Performance expectations

The Phase-1 architecture is small.

Prefer:

- clarity;
- stable structure;
- explicit validation;
- deterministic maps;
- inspectable errors

over premature optimization.

Hash-consing and arenas may be appropriate because they support identity and
dependency analysis, not because performance demands them.

No benchmark suite is required until expression/relation volume or compiler use
demonstrates a performance concern.

---

## 29. Non-goals

Phase 1 does not implement:

- tapscript;
- Simplicity;
- target opcodes;
- transaction layouts;
- witness ABI;
- object constructor bytes;
- linking;
- calibration;
- deployment profiles;
- full compiler proof planning;
- target resource formulas;
- confidential transaction assembly;
- rangeproof generation;
- all thirteen operations;
- a new external DSL;
- a stable realization publication hash;
- a broad model rewrite.

The package is not:

- a replacement for architecture;
- a replacement for the model;
- a parser for the realization document;
- a compiler backend;
- an independent indexer;
- release evidence by itself.

---

## 30. Phase-1 milestones

### R1.1 — Crate skeleton

Deliver:

- workspace member;
- package metadata;
- crate documentation;
- architecture dependency;
- error skeleton;
- no target dependencies.

### R1.2 — Identity and domains

Deliver:

- fact/expression/relation ID design;
- typed semantic domains;
- deterministic identity tests;
- collision/duplicate policy.

### R1.3 — Expression arena

Deliver:

- minimum pilot expression vocabulary;
- type checking;
- checked arithmetic;
- dependency traversal;
- evaluator;
- deterministic DAG identity.

### R1.4 — Relation vocabulary

Deliver the minimum relation set for both pilots.

### R1.5 — Operation adjuncts

Deliver:

- authorization;
- constructibility;
- lifecycle;
- representation;
- proof alternatives;
- observables.

### R1.6 — Derivation and validation

Deliver:

- pure `derive`;
- architecture indexing;
- scope/profile handling;
- bidirectional validation;
- structured errors.

### R1.7 — `compact-ash`

Deliver complete declaration and tests.

### R1.8 — `transfer-live-receipts`

Deliver complete declaration and tests.

### R1.9 — Declassification

Deliver typed pilot derivation and provenance.

### R1.10 — Model conformance

Deliver model adapters and positive/negative tests.

### R1.11 — Public API and determinism

Deliver downstream API test and repeated derivation checks.

### R1.12 — Phase gate

Run all workspace checks and record Phase-1 completion.

---

## 31. Phase-1 exit criteria

The package is ready for compiler analysis only when:

- [ ] `packages/realization` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] the crate depends on architecture and no target/backend package;
- [ ] the public derivation is pure and deterministic;
- [ ] architecture IDs are reused rather than duplicated;
- [ ] realization-owned IDs are typed and deterministic;
- [ ] the minimum semantic domains are distinct;
- [ ] the expression arena is typed and checked;
- [ ] dependency traversal is complete;
- [ ] the minimum pilot relation vocabulary is sufficient;
- [ ] `compact-ash` is completely declared;
- [ ] `transfer-live-receipts` is completely declared;
- [ ] both operations are bidirectionally validated against architecture;
- [ ] authorization and constructibility are explicit;
- [ ] representation alternatives are target-independent;
- [ ] lifecycle requirements are represented;
- [ ] public observables are represented;
- [ ] declassification is derived from typed dependencies;
- [ ] model behavior is checked against both pilot declarations;
- [ ] focused semantic mutations fail;
- [ ] pilot scope cannot be mistaken for full release coverage;
- [ ] no generated file is consumed;
- [ ] no source file is scraped;
- [ ] no target opcode or concrete transaction index appears;
- [ ] repeated derivation returns equal values;
- [ ] any derivative publication is deterministic and check-only;
- [ ] debug and release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

Canonical verification includes:

```sh
scripts/ci.sh
```

plus focused public API and conformance tests documented when the package is
implemented.

---

## 32. Open questions

These questions must be resolved during Phase 1.

### 32.1 Scope representation

How should a pilot realization explicitly state incomplete architecture
coverage?

Candidates:

- `RealizationScope`;
- operation-profile parameter;
- separate `derive_pilots`;
- validation mode.

Requirement:

> A pilot value must not be accepted as a complete release realization.

### 32.2 Amount type ownership

Should the target-independent amount type live in:

- realization;
- a new lower-level shared semantic crate;
- architecture;
- another existing crate?

Initial preference:

> keep it local to realization until a real cross-package extraction is
> justified.

### 32.3 Explicit IDs versus structural hashes

Should fact/expression/relation IDs be:

- explicit typed local keys;
- canonical structural digests;
- both?

Requirements:

- deterministic;
- readable diagnostics;
- mutation sensitivity;
- no accidental declaration-order dependence;
- future publication compatibility.

### 32.4 Relation sharing

Should relations common to several operations be:

- duplicated with separate operation-scoped IDs;
- structurally shared;
- represented as reusable templates instantiated per operation?

Pilot evidence should guide this. Operation provenance must remain explicit.

### 32.5 Evaluator ownership

Should the evaluator live in:

- `realization`;
- a later shared semantic evaluator module;
- model conformance tooling?

Initial preference:

> a small realization-owned evaluator for typed expressions and relations,
> independent of model behavior.

### 32.6 Realization identity timing

Do not publish a stable realization hash in Phase 1 unless an actual consumer
requires it and the projection is reviewed.

---

## 33. Risks

### 33.1 Over-generalization

The package may become a general theorem language rather than a focused
protocol realization.

Mitigation:

- two pilots first;
- additive vocabulary;
- explicit non-goals;
- package review for every abstraction;
- no speculative target constructs.

### 33.2 Under-specification

A vocabulary sufficient for the pilots may omit concepts required later.

Mitigation:

- treat Phase-1 schema as provisional;
- add operations incrementally;
- avoid publishing stable identity too early;
- maintain explicit scope.

### 33.3 Drift from the realization document

Typed declarations may differ from prose.

Mitigation:

- source citations in documentation;
- architecture welds;
- model conformance;
- document review;
- future generated semantic tables where useful;
- versioning review.

### 33.4 Correlated model/realization implementation

Shared helpers may hide one mistake.

Mitigation:

- relations remain inspectable data;
- model transition code remains distinct;
- independent boundary vectors;
- later backend differential tests;
- explicit shared-code disclosures.

### 33.5 Target leakage

Compiler convenience may pressure target fields into realization.

Mitigation:

- dependency policy;
- code review;
- target-neutral names;
- no backend crate dependency;
- future second-backend audit.

### 33.6 Premature identity freeze

A stable hash could make later vocabulary correction expensive.

Mitigation:

- delay publication;
- use internal schema;
- explicit migration decisions;
- pilot gate first.

### 33.7 Excessive validation complexity

Bidirectional checks can become repetitive.

Mitigation:

- generic typed validation helpers;
- deterministic registries;
- focused error types;
- avoid stringly reflection.

---

## 34. Definition of done

The realization package plan is fulfilled for Phase 1 when the repository has
one target-independent typed realization value, derived from the supported
architecture scope, that completely describes `compact-ash` and
`transfer-live-receipts`; derives pilot declassification; can be evaluated and
inspected by downstream code; is checked bidirectionally against architecture
and behaviorally against the executable model; contains no target or layout
detail; and passes the full deterministic clean-tree verification gate.

---

## 35. One-line package contract

> `realization` turns the typed architecture plus target-independent Rust
> semantic declarations into one validated deterministic relation graph,
> expression graph, constructibility/lifecycle/representation contract,
> observable registry, and derived declassification map shared by model
> conformance and compiler analysis—without reading generated files, scraping
> model source, or naming any target opcode or concrete transaction layout.
