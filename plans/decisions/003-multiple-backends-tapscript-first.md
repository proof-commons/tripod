# D003: Design for Multiple Backends and Implement Elements Tapscript First

> **Status:** ACCEPTED
> **Scope:** Compiler/backend boundary and target implementation order
> **Decision class:** Package architecture
> **Applies to:** `realization`, `compiler`, `target-elements`, `tapscript`,
> future `simplicity`, `linker`, `transaction`, `vectors`, and `release`
> **Depends on:** [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md);
> [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md)
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** behavioural rather than byte-identical
> conformance; target-independent translation obligations; model-to-script
> evidence boundary; deterministic release and deployment-profile requirements
> in `docs/attestation/realization.md`
> **Related research:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/wide-arithmetic.md`](../research/wide-arithmetic.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> [`../research/settlement-layout.md`](../research/settlement-layout.md)
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The realization document defines conformance behaviourally rather than by
requiring one exact script bundle.

A conforming implementation must preserve:

- the same architecture identity;
- the same target-independent semantic transition relation;
- the same invariant clauses;
- the same public observables;
- the same authorization and no-capture properties;
- the same representation and disclosure obligations;
- the same oracle and deployment-evidence boundaries.

It need not use one unique:

- script language;
- opcode sequence;
- tapleaf arrangement;
- witness layout;
- object-constructor strategy;
- arithmetic implementation;
- target proof primitive.

This creates a natural compiler architecture:

```text
target-independent semantics
        ↓
target-independent analysis
        ↓
target-specific proof selection and emission
```

The repository currently plans two relevant backend families.

### 1.1 Elements tapscript

Elements tapscript is the first practical target.

The available target family includes capabilities relevant to the realization,
including:

- transaction introspection;
- input and output asset/value inspection;
- scriptPubKey inspection;
- relative timelocks;
- signed 64-bit arithmetic;
- hashing and byte concatenation;
- signature verification;
- target-specific commitment and tweak verification.

The exact production target must still be pinned as typed Rust in the future
`target-elements` package. The current Markdown opcode survey is reference
material, not the target identity.

Tapscript is attractive for the first backend because:

- it is close to the current realization document's translation model;
- the relevant covenant-style introspection facilities are available on the
  intended substrate family;
- regtest execution can validate real transactions and script behavior;
- its limitations expose the concrete transaction-layout and resource problems
  that the compiler must solve;
- it provides the shortest path to a deployable target.

Its principal implementation costs are:

- explicit stack scheduling;
- bounded unrolling because no general loop facility is assumed;
- manual wide arithmetic;
- metadata-dependent object-constructor verification;
- target-specific resource accounting;
- transaction and witness ABI design.

### 1.2 Simplicity

Simplicity is a plausible future backend because it offers a typed,
combinator-based execution model and may eventually reduce some of the
hand-audited arithmetic and stack-machine surface.

It is not selected as the first implementation target.

The project does not yet have:

- a stable target-independent compiler IR proven against one backend;
- a complete Simplicity capability mapping;
- a production Simplicity backend implementation;
- resource and jet planning for every operation;
- a complete transaction/linking/ABI path;
- evidence that implementing Simplicity first would shorten the route to a
  deployment.

Making Simplicity a prerequisite would delay the first deployment and could
force the target-independent semantic boundary to be designed around an
unimplemented backend.

### 1.3 Risk of a tapscript-only compiler

A compiler designed directly around tapscript may accidentally embed:

- stack positions;
- opcode sequences;
- witness ordering;
- input indexes;
- taptree structure;
- control-block assumptions;
- Elements commitment prefixes;
- tapscript cost units

into types described as semantic or target-independent.

That would make a second backend a rewrite rather than an implementation of the
same semantic analysis.

It would also weaken the architecture's behavioural conformance claim by
allowing one target encoding to become the implicit definition of semantics.

### 1.4 Risk of implementing two backends immediately

Implementing tapscript and Simplicity simultaneously would multiply:

- target research;
- package APIs;
- constructor designs;
- resource models;
- transaction builders;
- vector runners;
- release artifacts;
- debugging surfaces

before the target-independent relation vocabulary is known to be adequate.

The first backend must test and refine that vocabulary. A second backend should
consume an already demonstrated boundary rather than co-design every layer at
once.

The required balance is therefore:

> preserve a real multi-backend boundary now, implement and validate one
> backend first, and defer the second backend until that boundary is proven.

---

## 2. Decision

Design the target-independent realization and compiler for multiple target
backends.

Implement **Elements tapscript first**.

Keep **Simplicity parked** until the target-independent realization, compiler
analysis, linked-bundle interfaces, transaction ABI, and relation-indexed
evidence have been demonstrated by the tapscript backend.

The planned structure is:

```text
architecture
    ↓
realization
    ↓
compiler analysis
    ├───────────────────────────────┐
    │                               │
    ▼                               ▼
typed Elements target          future Simplicity target
    │                               │
    ▼                               ▼
tapscript backend              Simplicity backend
    │                               │
    └──────────────┬────────────────┘
                   ▼
        backend-neutral linked-bundle roles
                   ▼
       target-specific transaction construction
                   ▼
             shared evidence model
```

This diagram is conceptual. Exact Rust crate dependencies must remain acyclic
and may place shared artifact interfaces in the package best suited to own
them.

The central decisions are:

1. **Semantics remain backend-independent.**
   `RealizationSpec` must not contain tapscript or Simplicity implementation
   details.

2. **Compiler analysis remains backend-independent above explicit capability
   selection.**
   Semantic relation identity, dependency analysis, constructibility,
   lifecycle, disclosure provenance, and source provenance are not defined by
   one target.

3. **Target capabilities are typed inputs.**
   A target package states which proof mechanisms and resource limits exist.

4. **Backends discharge analyzed obligations.**
   Each backend maps selected proof obligations to target programs,
   constructors, witness conventions, and resource formulas.

5. **Unsupported target relations fail closed.**
   A backend that cannot discharge one required relation reports an error. It
   does not weaken the relation.

6. **Tapscript is the only active production backend in the initial roadmap.**
   It receives the first complete implementation, transaction builder,
   calibration path, and deployment evidence.

7. **Simplicity remains a planned but parked backend.**
   Its package plan records the required neutrality boundary and reactivation
   conditions. An empty production crate need not be added merely to claim
   multi-backend design.

8. **Backend conformance is behavioural, not byte-identical.**
   Different target bundles conform only when each independently satisfies the
   same target-independent realization and its target-specific deployment
   obligations.

9. **Evidence is reusable at the semantic level but target-specific at the
   execution level.**
   Relation and mutation definitions may be shared. Each backend must produce
   its own execution and resource reports.

This decision does not require the first compiler implementation to solve every
possible backend abstraction in advance. It requires target-specific
assumptions to remain below a deliberate boundary.

---

## 3. Required consequences

### 3.1 Target-neutral realization types

The `realization` package must not contain:

- Elements opcode numbers or names;
- tapscript leaf versions;
- stack indexes;
- witness stack order;
- taptree depth;
- control-block representation;
- Elements explicit/confidential prefix bytes;
- Simplicity combinators;
- Simplicity jets;
- target-specific cost units;
- target-specific transaction serialization.

It may declare abstract semantic facts and proof alternatives such as:

```text
value is explicit and arithmetically inspectable
value conservation is proven by a target commitment system
two value commitments are equal
an authenticated value opening is available
relative age satisfies a bound
a signature commits the required transaction projection
a successor constructor is authenticated
```

The exact Rust vocabulary remains provisional until the pilot operations and
backend prototypes establish it.

### 3.2 Target-neutral compiler relation identity

A semantic relation must keep the same identity regardless of backend.

For example:

```text
transfer-live-receipts:
    sum(input U values) = sum(output U values)
```

is one target-independent relation.

It may be discharged by:

```text
tapscript:
    explicit arithmetic
or
    Elements confidential transaction conservation

future Simplicity:
    a Simplicity expression or jet-backed proof
```

The relation does not become a different semantic fact merely because one
backend chooses a different proof.

### 3.3 Explicit target-capability interface

The compiler must express required capabilities abstractly.

An illustrative capability vocabulary might include:

```rust
pub enum RequiredCapability {
    InputAssetInspection,
    InputValueInspection,
    InputProgramInspection,

    OutputAssetInspection,
    OutputValueInspection,
    OutputProgramInspection,

    ExplicitValueArithmetic,
    ConfidentialValueConservation,
    ValueCommitmentEquality,
    AuthenticatedValueOpening,

    RelativeTimelock,
    OutputCommittingSignature,
    HashConstruction,
    ConstructorVerification,
    CheckedWideArithmetic,
}
```

> Illustrative API; not frozen by this decision.

The typed target package reports support for concrete proof mechanisms and
their constraints.

The compiler or backend must reject a plan whose required capability is absent.

### 3.4 Separation of semantic requirements and target proof methods

The compiler pipeline should distinguish at least:

1. **semantic relation**
   What must be true.

2. **proof alternative**
   Which classes of target proof could establish it.

3. **selected proof plan**
   Which target-supported alternative is chosen under policy.

4. **emitted target pattern**
   Which concrete program enforces the selected proof.

5. **linked target artifact**
   Which deployed constructor/program bytes and constants result.

This separation is necessary for multiple backends and for translation
validation.

### 3.5 Backend-specific configuration identity

Target/backend choices must be bound outside realization identity.

A backend configuration identity should eventually bind choices such as:

- target identity;
- backend pattern-library version;
- proof-selection policy;
- stack scheduling policy;
- operation-layout lowering;
- metadata-constructor strategy;
- taptree weighting;
- resource-formula version;
- witness ABI version.

Changing these values may change emitted bytes without changing the semantic
realization.

### 3.6 Target-specific analyzed plans may differ

The target-independent relation graph remains fixed, but a selected proof plan
may differ by target.

For example:

```text
relation identity:
    value-preserving receipt transfer

Elements proof plan:
    explicit U asset identity
    + CT value conservation
    + owner signatures
    + output constructor closure

future Simplicity proof plan:
    target-specific typed program establishing the same relation
```

Therefore the project should distinguish:

- target-independent relation identity;
- target-selected proof-plan identity;
- emitted bundle identity.

A target-selected analyzed artifact must bind the target-capability identity it
used.

### 3.7 Backend-neutral artifact roles

Backends should produce typed artifacts with common conceptual roles:

- object constructor;
- operation program/leaf;
- symbol definition;
- symbol reference;
- relocation;
- witness requirement;
- resource formula;
- target requirement;
- source relation provenance.

The exact concrete program representation is backend-specific.

For example:

```rust
pub enum BackendProgram {
    Tapscript(TapscriptProgram),
    Simplicity(SimplicityProgram),
}
```

is not necessarily the preferred architecture because it can centralize target
dependencies incorrectly.

A generic trait or associated-type interface may be more appropriate:

```rust
pub trait Backend {
    type Target;
    type RelocatableProgram;
    type Constructor;
    type WitnessAbi;
    type ResourceFormula;

    fn emit(
        &self,
        analyzed: &AnalyzedProgram,
        target: &Self::Target,
    ) -> Result<Self::RelocatableProgram, BackendError>;
}
```

> Illustrative API; not frozen by this decision.

The required property is a stable conceptual boundary, not one exact trait.

### 3.8 No lowest-common-denominator semantics

Supporting multiple backends does not require weakening the semantic
realization to features common to every possible target.

The realization declares what the protocol requires.

A target either:

- supports proof alternatives sufficient for the relation;
- supports the relation after an explicit normalization/lifecycle path;
- or rejects compilation as unsupported.

The compiler must not erase a protocol relation merely because one backend
cannot implement it.

### 3.9 Tapscript-first package work

The active backend path is:

```text
target-elements
tapscript
linker
transaction
vectors
release
```

The first production target milestones are:

1. target pin and foundational patterns;
2. end-to-end `compact-ash`;
3. live receipt transfer;
4. STATE constructor integration through maturity announcement;
5. burn/ASH/clear;
6. arithmetic and redemption;
7. admission;
8. settlement;
9. cycle;
10. complete deployment evidence.

The ordered roadmap is authoritative on sequencing.

### 3.10 Simplicity remains parked, not absent from design review

The project should maintain:

- a package plan under `plans/packages/simplicity.md`;
- backend-neutral semantic types;
- backend-neutral relation identities;
- backend-neutral source provenance;
- target-independent vector definitions where possible;
- no tapscript stack assumption above the backend boundary.

It need not initially maintain:

- a workspace Simplicity crate;
- empty placeholder modules;
- unused dependencies;
- unsupported release commands;
- fake capability declarations;
- speculative emitted artifacts.

### 3.11 Evidence separates shared relation coverage from target execution

The vector system should distinguish:

#### Shared semantic vector definition

Contains:

- relation identity;
- abstract/model fixture;
- semantic mutation;
- expected acceptance or rejection;
- expected semantic projection.

#### Backend-specific realization

Contains:

- concrete target transaction;
- witness;
- selected target proof;
- executed target verdict;
- target resource result;
- emitted carrying predicate.

The same semantic mutation may feed more than one backend, but each backend
produces its own result.

### 3.12 Backend comparison is additional evidence

Once two backends exist, backend-versus-backend comparison may expose:

- under-specified realization relations;
- inconsistent public projection;
- different target assumptions;
- proof-selection ambiguity;
- encoding-specific mistakes.

However, agreement between two backends does not replace:

- comparison to the model/realization;
- target substrate tests;
- independent indexer/auditor reports;
- release validation.

Two backends can share one misunderstanding.

---

## 4. Prohibited consequences

### 4.1 No tapscript concepts in `RealizationSpec`

The following are prohibited above the backend boundary:

```text
OP_* opcode
stack depth
witness item index
input position
output position
control block
tapleaf
taptree
x-only key byte encoding
Elements confidential prefix
sigops budget
```

Target-independent operation families and semantic references remain allowed.

For example:

```text
the operation consumes a bounded ASH family
```

is target-independent.

```text
the first ASH input is the coordinator leaf
```

is a target layout decision and belongs below the compiler semantic layer.

### 4.2 No Simplicity-shaped semantic IR

The realization/compiler must not encode semantics as:

- combinator trees;
- jet names;
- CMR/IMR/AMR fields;
- Simplicity-specific type structures

merely to preserve a future backend option.

Simplicity lowers from the same semantic relation rather than defining it.

### 4.3 No simultaneous production requirement

Accepting multiple-backend design does not require the first release to contain:

- tapscript and Simplicity bundles;
- cross-backend byte equality;
- cross-backend cost equality;
- a final Simplicity ABI;
- a Simplicity deployment profile.

The initial release may be tapscript-only if every required semantic and
deployment obligation is satisfied.

### 4.4 No speculative package dependencies

Do not add Simplicity libraries to the workspace before they are required by an
active phase.

Do not create a dependency from target-independent compiler code onto a
Simplicity implementation crate “for future compatibility.”

The neutral boundary should be demonstrated through clean types and dependency
direction, not unused dependencies.

### 4.5 No target-specific fallback hidden as semantic default

If tapscript cannot support a proof alternative, it must report an unsupported
relation or require an explicitly declared normalization path.

It must not silently:

- reveal every value;
- drop a lifecycle requirement;
- add an owner signature to a permissionless path;
- omit a relation;
- weaken output closure;
- choose a different formula;
- change an observable.

A more revealing target representation may be acceptable only if it remains
semantically correct and complies with the accepted minimality policy for that
deployment.

### 4.6 No backend-specific architecture IDs

Tapscript and Simplicity must not introduce competing semantic operation IDs.

Target-specific leaf/program IDs are allowed, but they must carry provenance to
the shared architecture operation and realization relation.

### 4.7 No common linker assumption that only fits tapscript

Shared linker interfaces must not assume that every backend artifact is:

- a tapscript leaf;
- a Bitcoin Script byte vector;
- a taptree;
- a control block;
- an x-only key.

The first concrete linker implementation may contain tapscript-specific
modules, but shared artifact roles must not be falsely labeled backend-neutral
if they expose tapscript details.

### 4.8 No claim of second-backend validation before implementation

Documentation must not claim:

- backend diversity has been demonstrated;
- Simplicity conformance exists;
- backend-versus-backend evidence exists;
- the backend trait has been proven adequate

until a second backend actually exercises it.

Designing for multiple backends is an architectural constraint, not evidence
that multiple backends are complete.

---

## 5. Backend boundary

### 5.1 Above the boundary

The following belong above target emission:

- architecture identity;
- realization identity;
- semantic facts;
- typed expressions;
- semantic relations;
- operation dependencies;
- public observables;
- constructibility requirements;
- witness-availability classes;
- lifecycle requirements;
- representation capabilities;
- proof alternatives;
- disclosure provenance;
- source provenance;
- target capability requirements;
- relation-level accepting and rejecting vector definitions.

### 5.2 At capability selection

The compiler may combine:

- target-independent relation;
- typed target capabilities;
- typed compilation policy;
- typed deployment policy

to choose a supported proof plan.

The selection must be:

- deterministic;
- explainable;
- source-provenanced;
- target-identity-bound;
- fail-closed;
- separately identifiable from the semantic relation.

### 5.3 Below the boundary

The following are backend/target concerns:

- concrete opcodes or combinators;
- instruction selection;
- stack or combinator scheduling;
- object-constructor encoding;
- target commitment construction;
- concrete transaction introspection;
- target signature pattern;
- witness serialization;
- concrete layout lowering;
- concrete resource formulas;
- target error details;
- relocatable program representation.

### 5.4 Linker and transaction consequences

Some target details continue after emission:

- taptree assembly is tapscript-specific;
- Simplicity commitment/program assembly will have different concrete rules;
- transaction witnesses may differ;
- control data differs;
- resource accounting differs.

Therefore the linker and transaction packages must distinguish:

- common conceptual artifact roles;
- target-specific concrete representations.

A single package may contain target-specific implementations without
pretending those concrete types are universal.

---

## 6. Tapscript-first rationale

### 6.1 Existing target relevance

The realization document's translation rules are already naturally expressible
in terms of:

- transaction predicates;
- input/output inspection;
- explicit amount arithmetic;
- relative timelocks;
- signature predicates;
- object constructor commitments;
- bounded transaction layouts.

Tapscript directly tests whether those rules are deployable on the intended
Elements/Liquid substrate family.

### 6.2 Strongest immediate feedback

Implementing tapscript first exposes real constraints early:

- no general loops;
- stack-element limits;
- initial stack limits;
- transaction weight;
- witness cost;
- crypto budget;
- script policy;
- confidential transaction construction;
- exact object-constructor continuity;
- transaction-layout feasibility.

A higher-level backend implemented first could hide these target-shape problems
until later.

### 6.3 Target execution environment

A pinned Elements regtest environment can provide:

- real transaction validation;
- real tapscript execution;
- actual introspection semantics;
- real confidential transaction behavior;
- measured resource use;
- substrate dependency tests.

This supports the project's translation-validation strategy.

### 6.4 Incremental operation ladder

The roadmap allows the tapscript backend to grow through operations of
increasing semantic complexity rather than requiring a complete compiler
before any target feedback.

The first end-to-end operation, `compact-ash`, avoids:

- root state;
- metadata-dependent STATE reconstruction;
- owner authorization;
- wide floor arithmetic;
- settlement mapping.

That makes tapscript an appropriate first backend for validating the pipeline
itself.

---

## 7. Simplicity parking and reactivation

### 7.1 Parked status

Simplicity remains:

```text
Status: PARKED
```

Parking means:

- it is not on the current critical path;
- it does not block the tapscript backend;
- no production release is promised;
- package interfaces above the backend must avoid tapscript leakage;
- the project retains the right to activate the backend later.

### 7.2 Reactivation conditions

Simplicity planning becomes active after at least:

1. `RealizationSpec` supports all operations required by the first deployment;
2. compiler relation and proof-plan identities are stable enough to consume
   from a second backend;
3. the tapscript backend has completed at least the STATE constructor and wide
   arithmetic milestones;
4. linked-bundle and transaction-ABI boundaries are understood;
5. relation-indexed vector definitions are target-independent in practice;
6. one exact Simplicity target compatibility contract and deployment
   environment can be defined (with recorded review provenance);
7. a concrete project need justifies implementation cost.

A narrower experimental backend may begin earlier if it is explicitly a
prototype and does not block the active roadmap.

### 7.3 Initial Simplicity milestone

When reactivated, the first milestone should not be the complete protocol.

It should compile one already-supported pilot, likely:

```text
compact-ash
```

or another operation chosen because it tests the backend boundary without
introducing unrelated semantic research.

The milestone should compare:

- same realization relations;
- same semantic acceptance and rejection vectors;
- backend-specific execution;
- backend-specific resource report;
- identical public semantic projection;
- distinct emitted-bundle identity.

### 7.4 No false backend neutrality

If implementing Simplicity reveals that a supposedly target-independent type is
actually tapscript-specific, the project should:

1. identify the leaking assumption;
2. move it below the backend boundary;
3. migrate identities deliberately;
4. add a regression test;
5. update this decision or supersede it if the architecture itself was wrong.

The second backend is partly a test of the abstraction boundary.

---

## 8. Alternatives considered

### 8.1 Build only a tapscript compiler

#### Proposal

Design all IR, layouts, constructors, linker types, and evidence around
Elements tapscript. Revisit abstraction only if a second backend becomes
necessary.

#### Advantages

- smallest immediate implementation;
- fewer generic types;
- easier code generation;
- direct access to target assumptions;
- no speculative abstraction.

#### Rejection

Tapscript-specific assumptions would likely leak upward into:

- semantic relation types;
- expression lowering;
- object constructors;
- layout identities;
- witness availability;
- resource modeling;
- bundle interfaces.

Later extracting a backend-neutral layer would require a rewrite precisely when
the compiler and deployment formats were most entrenched.

The project already has a backend-independent conformance thesis. Preserving
that boundary now is cheaper than recovering it later.

The decision does not require abstracting every detail prematurely; it
requires keeping clearly target-specific concerns below a deliberate boundary.

### 8.2 Implement Simplicity first

#### Proposal

Use Simplicity's typed execution model and available formal foundations as the
first backend.

#### Advantages

- potentially cleaner arithmetic;
- potentially smaller hand-audited stack surface;
- typed target semantics;
- long-term formal verification potential;
- may align naturally with semantic DAGs.

#### Rejection for the first backend

The current implementation and deployment path is less mature within this
repository.

Starting there would require simultaneous work on:

- semantic IR;
- target capability model;
- Simplicity emission;
- target transaction construction;
- resource analysis;
- linking;
- deployment tooling;
- evidence harness.

Tapscript provides a more direct route to testing the current realization's
transaction and introspection assumptions on the intended substrate family.

Simplicity remains a future backend rather than being rejected permanently.

### 8.3 Implement both backends simultaneously

#### Proposal

Require every semantic and compiler milestone to emit both tapscript and
Simplicity from the start.

#### Advantages

- immediate proof that abstractions are backend-neutral;
- early backend comparison;
- reduced risk of tapscript leakage;
- shared vectors from the beginning.

#### Rejection

This doubles the target implementation surface before the semantic boundary is
validated.

Failures would be difficult to attribute among:

- realization vocabulary;
- compiler analysis;
- tapscript backend;
- Simplicity backend;
- target assumptions;
- transaction builders;
- linker interfaces.

The staged approach gets target feedback early while preserving future backend
neutrality.

### 8.4 Compile to one target-shaped common bytecode

#### Proposal

Create a universal low-level IR close to a stack machine, then translate that
IR to tapscript and Simplicity.

#### Advantages

- one lowering implementation;
- shared instruction selection;
- apparently simple backend adapters.

#### Rejection

A stack-shaped common IR would privilege tapscript and force Simplicity through
an unnatural execution model.

Conversely, a combinator-shaped IR would privilege Simplicity.

The shared layer should represent semantic relations and proof plans, not a
lowest-common-denominator virtual machine.

### 8.5 Use conditional compilation inside one backend crate

#### Proposal

Place tapscript and Simplicity emission behind Cargo features in one package.

#### Advantages

- shared internal helpers;
- fewer crates;
- one backend trait implementation surface.

#### Rejection as the planned architecture

Target dependencies, program representations, resource models, and test
environments differ materially.

Separate backend packages provide:

- clearer dependency boundaries;
- clearer target identities;
- independent evidence reports;
- easier feature and release isolation;
- less accidental cross-target coupling.

Shared helper code may later be extracted only where its target neutrality is
demonstrated.

### 8.6 Delay all backend abstraction until after tapscript works

#### Proposal

Implement tapscript directly, then refactor based on observed needs.

#### Advantages

- avoids speculative traits;
- fastest route to executable code;
- APIs emerge from experience.

#### Partial acceptance and constraint

The project agrees that exact backend traits and associated types should not be
overdesigned before the first backend.

However, it rejects allowing target concepts into realization and
target-independent compiler types.

The compromise is:

- freeze the semantic/backend boundary;
- keep exact Rust backend interfaces provisional;
- implement tapscript first;
- refactor below the boundary as experience accumulates.

---

## 9. Assurance and evidence consequences

### 9.1 One semantic relation set

The realization and compiler produce one relation set for an architecture.

Each backend must demonstrate that its emitted bundle enforces that same set.

A backend may add target guards required for safe implementation, including:

- canonical encoding checks;
- explicit/confidential representation checks;
- target domain checks;
- target transaction-shape constraints;
- constructor-continuity checks.

Such guards must carry provenance to the semantic or target safety obligation
that requires them.

### 9.2 Per-backend relation coverage

Each backend report must show, for every required relation:

- carrying target predicate;
- accepting vector;
- independently failing mutation;
- expected verdict;
- actual verdict;
- target identity;
- emitted bundle identity.

One backend's green report does not prove another backend.

### 9.3 Target-specific substrate evidence

Tapscript release requires evidence for the exact Elements target, including
relevant claims such as:

- asset and value introspection;
- confidential transaction conservation;
- commitment equality where used;
- sighash profile;
- relative timelock behavior;
- issuance behavior;
- program/constructor introspection;
- package relay where required;
- unspendable output handling;
- resource limits.

A future Simplicity backend requires its own target and dependency evidence.

### 9.4 Shared semantic vectors

Vector definitions should be reusable where they describe semantic mutations.

For example:

```text
mutate output receipt class from live to time-locked
```

is backend-independent.

The concrete transaction realizing that mutation is backend/ABI-specific.

### 9.5 Cross-backend comparison

When a second backend exists, compare:

- semantic acceptance/rejection;
- public state successor;
- event projections;
- public observables;
- declassification result;
- lifecycle behavior.

Do not compare:

- byte encoding;
- program tree shape;
- cost units;
- witness serialization

unless the comparison is explicitly about a shared ABI.

### 9.6 No verified-compiler overclaim

Multiple backend support does not imply the compiler is verified.

The initial assurance model remains translation validation under D004.

A formally stronger Simplicity path may improve one backend's evidence without
automatically proving:

- compiler lowering;
- proof-plan selection;
- linker relocation;
- transaction construction;
- target deployment assumptions.

### 9.7 Independent deployment reports remain separate

Backend diversity does not replace the independently implemented:

- attestation event projector;
- attestation query implementation;
- receipt-accounting auditor.

Those observe public deployment behavior and remain separate assurance
boundaries.

---

## 10. Determinism and identity consequences

### 10.1 Realization identity is backend-invariant

The same target-independent realization must retain one identity across:

- tapscript;
- Simplicity;
- test-only target evaluators;
- future backends.

A backend-specific field must not become a realization hash input.

### 10.2 Relation identity is backend-invariant

Semantic relation IDs remain stable across targets.

Target proof-plan nodes receive separate IDs bound to:

- relation ID;
- target capability identity;
- proof alternative;
- compiler/backend configuration.

### 10.3 Target identity is explicit

Every emitted program binds the exact target identity used during proof
selection and emission.

The target identity includes all target semantics that could affect
acceptance or resources.

A bundle compiled for one target must not be silently relabeled for another.

### 10.4 Backend configuration identity is separate

Changes such as:

- pattern-library update;
- stack scheduler update;
- taptree weighting;
- constructor encoding;
- layout lowering;
- witness ABI;
- optimization policy

must move a compiler/backend configuration or bundle identity as appropriate.

They need not move architecture or realization identity when semantics remain
unchanged.

### 10.5 Bundle identities are target-specific

A tapscript bundle and a Simplicity bundle implementing the same realization
must have distinct bundle identities.

Each bundle identity binds:

- architecture identity;
- realization identity;
- target identity;
- compiler/backend configuration;
- deployment parameters;
- calibrated bounds;
- emitted program bytes or target commitment;
- ABI identity.

### 10.6 Deterministic backend selection

If a target supports multiple valid proof alternatives, selection must use a
documented deterministic policy.

A possible priority order is:

1. semantic validity;
2. target capability support;
3. permissionless constructibility;
4. deployment policy;
5. resource feasibility;
6. disclosure minimality;
7. target cost;
8. canonical tie-break.

> Illustrative policy; exact selection order is not frozen here.

Nondeterministic search must not change bundle bytes between identical builds.

---

## 11. Implementation and migration

### 11.1 Phase 1: preserve target neutrality

During realization implementation:

- do not add a backend trait merely to satisfy this decision;
- do not create a Simplicity crate;
- ensure target-specific crates are absent from realization dependencies;
- express proof alternatives semantically;
- test the pilots without target encoding.

### 11.2 Phase 2: define target requirements

During compiler analysis:

- derive target capability requirements;
- keep relation IDs target-independent;
- distinguish proof alternatives from selected proof plans;
- add a simple test or mock capability provider if useful;
- avoid hardcoding tapscript pattern names in compiler-core relations.

A mock target may be used to test unsupported-capability failure. It is not a
production backend.

### 11.3 Phase 3: add typed Elements target

Create:

```text
packages/target-elements/
```

as a typed compatibility contract and capability model (upstream review
provenance recorded, not identity-bearing; ADR-011).

Replace assumptions in plans/reference prose with typed target facts and
deployment tests.

### 11.4 Phase 3 onward: implement tapscript backend

Create:

```text
packages/tapscript/
```

and implement the roadmap incrementally.

Backend interfaces may evolve before the first linked operation. Changes must
not leak into realization semantics.

### 11.5 Linker interface design

The linker plan must distinguish target-neutral concepts from target-specific
representations.

Likely common concepts include:

```text
symbol
constructor
reference
relocation
resource formula
witness requirement
program provenance
```

Concrete tapscript concepts remain target-specific.

Do not require the future Simplicity backend to emit fake tapscript-shaped
objects merely to satisfy one interface.

### 11.6 Transaction package design

The transaction package may have target-specific modules:

```text
transaction::elements
transaction::simplicity
```

or target-specific associated types.

The package owns the public operation-layout and witness ABI for each linked
target bundle.

One target's ABI need not equal another's. Both must map to the same semantic
operation and observables.

### 11.7 Vectors and report schema

The vector package should separate:

```text
semantic vector definition
target transaction materialization
target execution result
target resource result
```

Report schemas should bind target and bundle identities explicitly.

### 11.8 Simplicity reactivation

When reactivation conditions are met:

1. review `plans/packages/simplicity.md`;
2. source-pin one exact target;
3. create a proposed implementation decision if new architecture is required;
4. implement one pilot;
5. run shared semantic vectors;
6. produce a separate backend report;
7. use findings to audit the backend boundary.

Do not retroactively describe the original tapscript implementation as
backend-neutral without this exercise.

---

## 12. Risks and limitations

### 12.1 Premature abstraction

A speculative backend trait can create awkward generic types and slow the first
backend.

Mitigation:

- freeze responsibility boundaries, not exact trait signatures;
- implement minimal associated interfaces required by tapscript;
- keep target-specific representations concrete;
- refactor shared roles only when demonstrated.

### 12.2 Hidden tapscript leakage

Types may be target-specific even if their names appear generic.

Examples:

- `Program` that is always a byte script;
- `Layout` that assumes per-input stack witnesses;
- `Constructor` that assumes taproot tweaking;
- `Cost` that assumes witness weight and sigops.

Mitigation:

- package dependency review;
- explicit target-specific names;
- second-backend audit when activated;
- avoid placing concrete target representations in realization/compiler core.

### 12.3 Lowest-common-denominator proof planning

Trying to support every target may discourage use of strong target-specific
proof mechanisms.

Mitigation:

- semantic relations remain strong;
- proof alternatives are target-specific below the relation;
- unsupported targets fail;
- no requirement that all targets accept every representation mode.

### 12.4 Parked backend decay

Simplicity assumptions in planning may become stale.

Mitigation:

- keep the package plan short;
- avoid detailed unverified capability claims;
- require a fresh substrate review with recorded provenance at
  reactivation;
- treat the current reference as non-authoritative.

### 12.5 Duplicate target infrastructure

Separate backends may duplicate:

- identity mapping;
- relation provenance;
- report rendering;
- resource orchestration;
- transaction fixture logic.

Mitigation:

- share only code proven target-neutral;
- keep semantic vector definitions common;
- avoid premature helper extraction that introduces cross-target coupling.

### 12.6 Different target acceptance sets

Two backends may accidentally accept different semantic transitions due to:

- encoding ambiguity;
- missing guard;
- target capability mismatch;
- layout difference;
- constructor behavior;
- representation support.

Mitigation:

- same realization relation IDs;
- relation-indexed vectors;
- public projection comparison;
- backend-specific rejecting mutations;
- later cross-backend differential checks.

### 12.7 Resource-model incomparability

Tapscript and Simplicity use different resource units.

The project must not compare raw costs as though they were the same.

Each target calibrates against its own limits and publishes target-specific
resource evidence.

### 12.8 Scope inflation

“Design for multiple backends” may be misread as requiring general compiler
infrastructure before any operation works.

Mitigation:

- tapscript remains the active implementation path;
- Simplicity is parked;
- the first end-to-end milestone is one tapscript operation;
- backend-neutrality review focuses on dependency and type boundaries.

---

## 13. Supersession conditions

This decision may be superseded if:

1. the project deliberately commits to one permanent backend;
2. evidence shows preserving a multi-backend boundary creates unacceptable
   complexity with no credible second target;
3. normative realization constraints become target-specific;
4. one formally verified target language becomes the sole deployment
   substrate;
5. a new compiler architecture provides a better semantic/backend separation.

Supersession requires:

- explicit replacement decision;
- package and dependency migration;
- impact analysis for realization and relation identities;
- generated artifact and ABI migration;
- evidence/release migration;
- review of whether the change affects protocol or only implementation
  latitude;
- root ADR once implemented.

This decision is not superseded merely because:

- only tapscript is implemented initially;
- Simplicity remains parked for several phases;
- the first compiler has only one production backend;
- exact backend trait APIs change;
- some proof alternatives are target-specific;
- target transaction ABIs differ.

---

## 14. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/architecture/src/deployment.rs`](../../packages/architecture/src/deployment.rs)
- [`../../packages/model/src/manifest.rs`](../../packages/model/src/manifest.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/ledger.rs`](../../packages/model/src/ledger.rs)

### Related decisions

- [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
- [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md)
- [D004: Translation Validation Over Compiler Trust](004-translation-validation-over-compiler-trust.md)
- [D005: Value-Parametric, Closed-Asset-Identity-Rigid](005-value-parametric-asset-rigid.md)
- [D006: Canonical Transaction-Layout ABI](006-canonical-transaction-layout-abi.md)

### Package plans

- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/target-elements.md`](../packages/target-elements.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/simplicity.md`](../packages/simplicity.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)

### Technical reference

- [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)

---

## 15. Decision summary

> Keep architecture, realization relations, compiler analysis, disclosure,
> constructibility, lifecycle, and semantic vector definitions independent of
> one target. Implement and validate Elements tapscript as the first production
> backend. Keep Simplicity parked until the shared boundary has been proven by
> the first backend, then use a second implementation to test that boundary
> rather than allowing either target's execution model to define semantics.
