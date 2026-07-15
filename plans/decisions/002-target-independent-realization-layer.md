# D002: Introduce a Target-Independent Realization Layer

> **Status:** ACCEPTED
> **Scope:** Semantic package boundary between the typed architecture,
> executable model, and future compiler
> **Decision class:** Package architecture
> **Applies to:** `architecture`, future `realization`, `model`, `compiler`,
> target packages, backends, vectors, and release
> **Depends on:** [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** the register-authority rule, conformance
> thesis, translation discipline, representation rules, oracle obligations, and
> code-generation checklist in
> `docs/attestation/realization.md`
> **Related research:** none for the package boundary; exact relation and
> expression vocabularies are settled incrementally by the Phase-1 pilot
> operations
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The repository currently contains two mature typed technical layers relevant to
future compilation.

### 1.1 Typed architecture

The architecture crate declares the finite architecture:

```text
packages/architecture/
```

It owns:

- stable identifiers and discriminants;
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
- authorization-evidence classes;
- structural validation;
- publication and deployment-profile schemas;
- canonical architecture identities.

The architecture is intentionally declarative and finite. It describes which
entities and relations exist.

It does not attempt to encode every formula, branch postcondition, dependency
edge, representation alternative, constructibility requirement, or lifecycle
proof required by a compiler.

### 1.2 Executable model

The model crate implements the abstract state machine:

```text
packages/model/
```

It owns executable behavior for:

- operation constructors;
- authorization checks;
- exact canonical and open-flow partitions;
- checked arithmetic;
- branch shape;
- root succession;
- transition-certificate derivation;
- state and history invariants;
- attestation indexing;
- accounting audit;
- maintenance and quiescence;
- property and mutation evidence.

The model is intentionally transparent and executable. Its public state can be
inspected and deliberately corrupted by auditors and fault harnesses.

It is not a compiler language. Its operation semantics are distributed across:

- operation constructors;
- kernel helpers;
- branch-shape validation;
- manifest-conformance helpers;
- arithmetic helpers;
- certificate derivation;
- invariant checks;
- history replay;
- test fixtures.

A compiler cannot safely recover a complete target-independent relation by
scraping or interpreting those implementation details.

### 1.3 The semantic gap

The future compiler needs more than the architecture arrays and less than a
Rust implementation of the full model.

For each operation, the compiler needs a typed declaration of facts such as:

- which semantic values and metadata fields are read;
- which expressions are evaluated;
- which preconditions define acceptance;
- which state fields are assigned;
- which inputs authorize participation;
- which operation-level authorization applies;
- which recipient and conservation relations must hold;
- which root-successor relations are required;
- which events and public observables are derived;
- which facts may remain representation-private;
- which facts must become public;
- which proof alternatives may establish one semantic relation;
- which witnesses are available to an owner, operator, sponsor, or arbitrary
  permissionless constructor;
- which lifecycle exits every supported representation must retain;
- which dependencies determine declassification.

The architecture manifest does not contain all of this detail, and should not
be expanded into a target compiler IR.

The executable model implements much of it, but its source is not an
appropriate typed declaration for compiler consumption.

Without an intermediate semantic layer, the project would have to choose one
of several undesirable paths:

1. duplicate formulas and relations directly in the compiler;
2. scrape the executable model;
3. enlarge `architecture` until it contains target-compilation concerns;
4. make the compiler call model implementation internals;
5. use the realization Markdown as a source language;
6. compile directly from generated publications.

Each path creates ambiguity, target leakage, or correlated failures.

### 1.4 Required shared boundary

The project therefore needs one typed target-independent realization
declaration that can be:

- validated against the finite architecture;
- checked against executable model behavior;
- consumed by compiler analysis;
- evaluated independently in tests;
- used to derive declassification;
- published canonically if useful;
- kept free of target opcodes and concrete transaction layout.

This layer is distinct from the prose realization document.

The document remains the normative conformance contract. The new Rust package
is the typed implementation declaration corresponding to that contract.

---

## 2. Decision

Introduce a new workspace package:

```text
directory: packages/realization
Cargo package: tripod-realization
Rust library name: realization
```

The `realization` package sits between `architecture` and `compiler`.

The planned semantic direction is:

```text
architecture
    ↓
realization
   ├──────────────▶ model conformance
   └──────────────▶ compiler analysis
```

The package provides one validated, deterministic, target-independent
`RealizationSpec` containing the semantic declarations required by model
conformance and compiler analysis.

The intended entry point is:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

> Illustrative API; the exact signature is not frozen by this decision.

The central decisions are:

1. **Architecture remains the finite enumerated source.**
   `realization` does not redefine architecture IDs, operation families, roots,
   objects, quantities, bounds, or tags.

2. **Realization owns target-independent semantic relation detail.**
   It declares the formulas, dependencies, state assignments, authorization
   relations, closure relations, observables, constructibility requirements,
   lifecycle requirements, representation capabilities, and proof alternatives
   needed below architecture.

3. **The model and compiler share this layer without becoming one
   implementation.**
   The model is checked against the realization; the compiler consumes the
   realization.

4. **The compiler does not infer semantics from model source.**
   Model source scraping, syntactic read-set inference, and test-based semantic
   recovery are prohibited by D001.

5. **The realization is target-independent.**
   It contains no Elements opcode, tapscript stack, Simplicity combinator,
   control block, witness byte offset, concrete taptree, or deployment network
   assumption.

6. **Declassification derives from typed semantic dependencies.**
   The realization provides the dependency graph from which disclosure is
   derived. A generated declassification artifact remains derivative.

7. **The package is introduced incrementally.**
   Its vocabulary is proven first on `compact-ash` and
   `transfer-live-receipts`. The model is not rewritten wholesale before the
   pilots establish that the semantic declaration is adequate.

This package boundary is governed by
[`../packages/realization.md`](../packages/realization.md) and Phase 1 of
[`../roadmap.md`](../roadmap.md).

---

## 3. Required consequences

### 3.1 Package dependency direction

The initial direct dependency is:

```text
realization → architecture
```

The realization package must not depend on:

- `model`;
- `compiler`;
- `target-elements`;
- `tapscript`;
- `simplicity`;
- `linker`;
- `transaction`;
- `vectors`;
- `release`.

The model may later depend on realization for conformance checks or shared
target-independent evaluation:

```text
model → architecture
model → realization
```

The compiler depends on realization:

```text
compiler → realization
```

The compiler should not depend on model merely to obtain semantic relation
definitions.

The vector harness may depend on both:

```text
vectors → realization
vectors → model
vectors → compiler
```

because its role is comparison across assurance boundaries.

### 3.2 Architecture remains authoritative on enumerated facts

The realization must use architecture identifiers directly.

Examples:

```rust
architecture::OperationId
architecture::ObjectId
architecture::AssetId
architecture::RootId
architecture::QuantityId
architecture::BoundId
architecture::TagId
```

It must not define parallel variants such as:

```rust
enum RealizationOperation {
    Burn,
    Clear,
    // ...
}
```

unless such a type is a transparent, exhaustively validated wrapper whose
purpose cannot be served by the architecture ID itself.

Any mapping between architecture IDs and realization-owned structures must be:

- exhaustive;
- deterministic;
- bidirectionally validated;
- protected against transposition;
- covered by mutation tests.

### 3.3 Realization owns semantic facts and expressions

The package introduces typed semantic identities, expected to include concepts
such as:

```text
FactId
ExprId
RelationId
ObservableId
LifecycleRequirementId
ProofAlternativeId
```

Exact names are provisional.

Semantic facts may describe:

- input asset;
- input value;
- input owner;
- input class;
- input metadata field;
- output asset;
- output value;
- output owner;
- output class;
- output metadata field;
- state field;
- root identity;
- family cardinality;
- architecture constant;
- deployment-calibrated value reference;
- canonical transaction-semantic fact that is target-independent.

Facts must not encode:

- stack positions;
- opcode numbers;
- witness indexes;
- transaction byte offsets;
- control-block bytes;
- target-specific prefixes.

### 3.4 Realization owns typed formulas

Economy-determining formulas must be represented in a typed target-independent
expression form.

Required formula classes eventually include:

- checked amount addition and subtraction;
- bounded multiplication;
- exact floor multiplication/division;
- ratio floors;
- minimum and maximum;
- equality and ordering;
- conditional activation;
- bounded sums over operation families;
- state assignments.

The expression layer must distinguish semantic domains such as:

```text
amount
count
cycle
block age
ratio
boolean
asset
owner/address
object/root identity
```

An untyped generic integer expression is insufficient.

Formula semantics must be:

- total over declared domains or explicitly fail-closed;
- evaluable by a reference evaluator;
- structurally identifiable;
- traversable for dependency analysis;
- independent of target arithmetic implementation.

### 3.5 Realization owns semantic relation declarations

The relation vocabulary must eventually express:

- domain checks;
- cardinality;
- object recognition;
- object/class closure;
- input authorization;
- operation authorization;
- value conservation;
- exact equality;
- ordered comparison;
- checked arithmetic;
- floor relations;
- recipient pinning;
- issuance;
- destruction;
- open-flow isolation;
- root succession or termination;
- state assignments;
- event projection;
- public observables;
- constructibility;
- lifecycle requirements.

The exact enum organization is not frozen by this decision.

The pilot operations determine the minimum useful vocabulary before the package
expands to every operation.

### 3.6 Realization owns constructibility declarations

The realization must distinguish script verifiability from transaction
constructibility.

For each required witness or fact, it should represent availability categories
such as:

```text
compile-time
deployment constant
public chain data
public opening
current owner secret
operator secret
sponsor-local data
```

> Illustrative categories; exact type names are not frozen.

For permissionless operations, the realization must make it possible to reject
a representation or proof plan that requires:

- another owner's secret;
- an operator secret;
- a private opening unavailable to arbitrary constructors;
- unpublished metadata;
- an unavailable off-chain database.

This requirement is semantic enough to belong above target emission.

### 3.7 Realization owns representation capabilities

The package describes semantic latitude, not concrete encoding.

It may declare that a relation can be established by alternatives such as:

```text
explicit value arithmetic
confidential value conservation
commitment equality
authenticated opening
normalization before use
```

The realization must not select a target opcode sequence.

It should describe:

- which facts are semantically public;
- which facts may remain private;
- which public facts may be represented by authenticated public commitments;
- which lifecycle exits are required;
- which proof alternatives are semantically acceptable.

The compiler and target backend later decide which alternatives are supported
and selected.

### 3.8 Realization owns public observables

The package should declare typed public observables needed for:

- state commitment;
- transition certificates;
- burn event projection;
- clear event projection;
- distribution residue projection;
- attestation query inputs;
- accounting audit inputs;
- other realization-defined public outputs.

It must preserve the distinction between:

- abstract state;
- transition evidence;
- public interface output;
- audit-only output.

The existing quantity-reader firewall remains architecture-owned and must be
reflected, not redefined.

### 3.9 Declassification is derived here

The realization's typed expression and relation graph is the source for
declassification analysis.

For each operation, the derivation should identify facts required because they:

1. affect public state;
2. are needed for permissionless construction;
3. affect public event/interface/audit output.

The result must carry provenance.

The direction is:

```text
RealizationSpec
    ↓ dependency analysis
typed Declassification
    ↓ optional canonical rendering
declassification.json
```

The compiler consumes the typed result.

### 3.10 Architecture/realization validation is bidirectional

Validation must prove both:

```text
every architecture operation/object in scope has the required realization
```

and:

```text
every realization operation/object reference maps to a declared architecture
entity
```

It must also validate:

- every relation operand resolves;
- every expression is well typed;
- no duplicate realization identity exists;
- every bound reference is architecture-declared;
- every authorization refers to the declared operation/family;
- every public observable has a permitted reader role;
- every operation dependency is finite and deterministic;
- every representation capability has required lifecycle conditions;
- no target-specific primitive appears.

### 3.11 Model integration is conformance, not substitution

The model remains an executable implementation.

Phase-1 model integration should:

1. derive the realization;
2. construct model transitions through public operation APIs;
3. project model inputs, outputs, state, and certificates into realization
   facts;
4. evaluate realization relations;
5. compare declared projections with model projections;
6. mutate one relation or fact and require conformance failure.

The realization does not call model operation code to define the relation.

The model does not read compiler output to determine acceptance.

### 3.12 Compiler analysis begins from realization

The compiler must receive a validated typed realization, not a set of model
callbacks.

A conceptual request is:

```rust
pub struct CompilationRequest<'a, T> {
    pub realization: &'a RealizationSpec,
    pub target: &'a T,
    pub deployment: &'a DeploymentParameters,
    pub policy: &'a CompilationPolicy,
}
```

> Illustrative API; not frozen.

The compiler may use a target-independent evaluator for constant folding and
test generation. It must not use the model as the hidden implementation of
semantic analysis.

---

## 4. Prohibited consequences

### 4.1 The realization package is not a new architecture manifest

It must not duplicate or rename the finite architecture registry.

It must not become authoritative on:

- operation enumeration;
- object enumeration;
- architecture discriminants;
- dependency enumeration;
- architecture bounds;
- invariant-clause IDs;
- architecture publication status.

Those remain architecture-owned.

### 4.2 The architecture crate must not absorb compiler semantics

Accepting a realization layer does not authorize expanding `architecture` to
include:

- target proof plans;
- tapscript layouts;
- stack positions;
- witness ABIs;
- target constructor strategies;
- concrete disclosure choices;
- target resource formulas.

The architecture remains a finite target-independent declaration.

### 4.3 The realization document is not parsed

The Rust realization package is not generated by parsing
`realization.md`.

The prose document remains normative on conformance meaning. Consistency is
maintained by:

- review;
- typed identifiers;
- label welds;
- conformance tests;
- generated appendices;
- versioning gates;
- package documentation.

Markdown parsing is not the semantic bridge.

### 4.4 The model is not an AST

The compiler must not treat:

- `World`;
- `Transition`;
- operation constructor structs;
- `Guard`;
- `TransitionCertificate`;
- Rust function calls;
- test fixtures

as a compiler syntax tree.

Those types are executable evidence and may inform test adapters, but the
compiler's semantic input is `RealizationSpec`.

### 4.5 Realization must not contain target details

The package must not name:

- `OP_INSPECTINPUTVALUE`;
- `OP_TWEAKVERIFY`;
- tapscript leaf versions;
- Elements confidential prefixes;
- stack shuffles;
- control-block depth;
- Simplicity jets;
- target sigops budgets;
- transaction input/output positions;
- byte-order encoding.

It may require abstract capabilities such as value introspection or constructor
verification.

### 4.6 Realization must not freeze premature backend choices

A semantic relation may admit several proof alternatives.

The realization must not state, for example:

```text
transfer value conservation is always explicit arithmetic
```

if confidential transaction conservation is an accepted semantic proof
alternative.

Likewise, it must not require public wire encoding when the semantic
requirement is only publicly authenticated availability.

### 4.7 Shared code must not erase independent evidence

Moving formulas into a typed expression evaluator can reduce duplication, but
the project must not make model, compiler, backend, and vector expectations all
call one function and then describe their agreement as independent evidence.

At minimum:

- model behavior remains executable at the abstract transition level;
- realization relations remain inspectable data;
- compiler lowering remains separately testable;
- backend execution remains target-specific;
- independent indexer/auditor reports remain separately implemented.

### 4.8 Phase 1 must not declare the complete future vocabulary prematurely

This decision accepts the package boundary, not one final giant semantic enum.

The initial vocabulary should be the smallest typed set sufficient for:

- `compact-ash`;
- `transfer-live-receipts`;
- deterministic identity;
- declassification derivation;
- model-conformance checking.

Later operations may require additive vocabulary under explicit schema and
identity review.

---

## 5. Alternatives considered

### 5.1 Compile directly from `architecture`

#### Proposal

Make the compiler consume `architecture::ARCHITECTURE` directly and add
compiler-local logic for formulas and semantic postconditions.

#### Advantages

- fewer packages;
- immediate access to stable identifiers;
- compiler implementation can start quickly;
- no additional identity or validation layer.

#### Rejection

The architecture does not encode all semantic detail the compiler requires.

Compiler-local additions would create a second semantic source for:

- formulas;
- state deltas;
- constructibility;
- lifecycle;
- representation alternatives;
- disclosure dependencies;
- event semantics.

The compiler would then be both the relation declaration and its target
translation, weakening independent validation.

Architecture should remain finite and target-independent rather than becoming
a compiler IR.

### 5.2 Compile directly from the executable model

#### Proposal

Call model operation constructors or inspect model outputs to generate target
predicates.

#### Advantages

- executable behavior already exists;
- formulas are already implemented;
- avoids declaring relations separately;
- generated vectors are easy to obtain.

#### Rejection

An executable transition function answers whether one abstract operation
succeeds and what successor results. It does not directly provide:

- a finite relation graph;
- source provenance per obligation;
- proof alternatives;
- disclosure dependencies;
- witness availability;
- constructibility;
- lifecycle paths;
- obligation placement;
- target capability requirements.

Using the model as compiler input would also correlate the compiler and
expected behavior too strongly. A model bug could be compiled faithfully and
then “validated” against itself.

The model remains an essential reference implementation and vector source, not
the compiler's semantic declaration.

### 5.3 Scrape model source

#### Proposal

Analyze Rust source or expanded syntax to infer operation dependencies and
formulas.

#### Advantages

- avoids adding a new semantic package;
- appears to preserve one implementation;
- may automate read-set extraction.

#### Rejection

Source-level reads are not semantic dependencies:

- helpers obscure data flow;
- refactoring changes syntax without changing meaning;
- dead reads may not be semantic;
- public constructibility is not inferable from a read set;
- proof alternatives are not syntactic;
- lifecycle requirements are not local reads;
- target-independent observables are not reliably recovered;
- macro expansion and generics complicate analysis.

This would turn model Rust into an undocumented source language and violate
D001.

### 5.4 Put semantic declarations inside `model`

#### Proposal

Add typed relation declarations to the model crate and let compiler depend on
model.

#### Advantages

- fewer crates;
- declarations and implementation remain adjacent;
- easy access to model domain types;
- incremental migration appears simple.

#### Rejection

The model would become both:

- executable reference behavior;
- compiler semantic source.

That weakens package direction and makes target-independent semantics depend on
a transparent, test-oriented state implementation.

A separate realization package allows:

- architecture-only dependency;
- model conformance;
- compiler consumption;
- no compiler dependency on model internals;
- clearer assurance boundaries.

The model may share generic evaluator/domain code only if ownership remains
clear and dependency direction stays acyclic.

### 5.5 Put semantic declarations inside `compiler`

#### Proposal

Let compiler IR be the first typed semantic relation representation.

#### Advantages

- no extra lowering stage;
- fewer IDs and schemas;
- direct implementation path.

#### Rejection

The compiler IR will need compiler-specific concerns such as:

- proof planning;
- disclosure planning;
- target requirements;
- placement;
- layout;
- lowering identities.

Those concerns should not become the semantic source used to validate the
model.

Separating realization from compiler analysis preserves:

```text
semantic relation
    ↓
compiler plan
```

rather than conflating them.

### 5.6 Use the realization document as a DSL

#### Proposal

Parse structured labels, tables, listings, and prose from
`realization.md`.

#### Advantages

- document and implementation appear unified;
- minimizes an additional source;
- human-readable.

#### Rejection

The document is not a formally typed syntax:

- prose carries rationale and exceptions;
- tables are presentation;
- labels are references, not AST nodes;
- listings include goal-level pseudocode;
- the appendix is generated architecture publication;
- parsing Markdown would create fragile semantic coupling.

The document remains the conformance contract. The Rust realization is its
typed implementation declaration, checked rather than parsed.

### 5.7 Generate model and compiler from a new external DSL

#### Proposal

Create a language-neutral semantic DSL and generate both model and compiler
structures.

#### Advantages

- one explicit semantic source;
- potential multi-language implementations;
- strong synchronization.

#### Rejection for the initial work

This requires:

- a language design;
- parser and diagnostics;
- schema/version policy;
- code generators;
- bootstrap trust;
- migration of mature model code;
- new source-of-truth governance.

The current missing need is a typed semantic layer, which Rust can provide
without creating a new language.

A future DSL could supersede this decision only through an explicit migration
that preserves semantic identities and assurance boundaries.

---

## 6. Assurance and evidence consequences

### 6.1 New architecture-to-realization conformance boundary

The new package introduces a check:

```text
typed architecture
        ↔
typed realization coverage
```

Evidence must include:

- complete operation coverage;
- complete object and quantity reference coverage;
- bound and tag resolution;
- authorization agreement;
- projection agreement;
- no undeclared semantic entity;
- deterministic IDs;
- duplicate and transposition rejection.

### 6.2 New realization-to-model conformance boundary

The model must demonstrate:

```text
successful model transition
    ⇒ realization relations hold
```

and, for focused faults:

```text
mutated semantic fact/relation
    ⇒ intended relation fails
```

The initial evidence is pilot-specific. It expands operation by operation.

This is model-level conformance evidence. It is not emitted-script evidence.

### 6.3 Compiler provenance improves

Each compiler relation can point back to a realization relation instead of
appearing as an unnamed backend predicate.

This enables:

- relation-indexed vector coverage;
- independent mutation per relation;
- explicit proof alternatives;
- disclosure provenance;
- obligation-placement completeness;
- target requirement extraction.

### 6.4 Declassification gains a typed source

The declassification map becomes a derivation from the same semantic graph used
for compiler analysis and model conformance.

This prevents three independent tables:

```text
model reads
compiler disclosure policy
declassification publication
```

### 6.5 Backend evidence remains distinct

A realization relation stating:

```text
values are conserved
```

does not prove that:

- a tapscript pattern checks conservation;
- CT consensus semantics are as assumed;
- an object constructor binds metadata;
- a transaction layout covers every output;
- target resources fit.

Those remain backend, linker, target, transaction, and deployment evidence.

### 6.6 Independent evidence remains independent

The realization package may define public observables and expected query
semantics, but an independently implemented deployment indexer must not depend
on the same executable model implementation merely to manufacture agreement.

Separate event, query, and accounting reports remain required.

---

## 7. Determinism and identity consequences

### 7.1 Realization identity

The package will eventually require a domain-separated canonical identity over
its target-independent semantic declaration.

The identity should bind at least:

- fact vocabulary and types;
- expression graph;
- relation graph;
- operation ownership;
- state assignments;
- public observables;
- constructibility requirements;
- lifecycle requirements;
- representation capabilities;
- proof alternatives;
- derived declassification;
- architecture identity.

The exact algorithm and schema are **not** frozen by this decision.

No realization hash should be published until:

1. the pilot vocabulary is stable;
2. inclusion/exclusion rules are documented;
3. canonical ordering is defined;
4. mutation tests exist;
5. the algorithm identifier is fixed;
6. migration policy is documented.

### 7.2 Structural IDs

Fact, expression, and relation IDs should not depend on:

- source declaration order where order is not semantic;
- source paths;
- line numbers;
- debug formatting;
- memory addresses;
- hash-map iteration;
- wall-clock time;
- target backend selection.

IDs may derive from:

- explicit semantic names;
- stable architecture IDs;
- typed structural hashing;
- canonical operand ordering where semantically valid;
- explicit domain separators.

### 7.3 Target-independent identity stability

Changing only:

- tapscript instruction selection;
- stack schedule;
- taptree layout;
- control blocks;
- target resource model;
- reviewed target substrate facts;
- backend configuration

must not move the realization identity.

Those changes belong to compiler configuration, target, backend, linker, or
bundle identities.

Changing:

- a formula;
- accepted semantic relation;
- state assignment;
- public observable;
- constructibility requirement;
- lifecycle requirement;
- semantic proof alternative where alternatives define conformance latitude

may move the realization identity and requires versioning review.

### 7.4 Architecture binding

A `RealizationSpec` must bind to the architecture identity it realizes.

It must not be possible to apply one realization silently to an architecture
with:

- different operation IDs;
- different object registry;
- different bounds;
- different amount limits;
- different authorization;
- different projection declarations.

### 7.5 Deterministic derivation

For identical typed architecture and realization declarations:

```text
derive()
```

must return equal typed values and, if serialized, byte-identical canonical
publications.

No derivation step may read:

- environment variables;
- current time;
- filesystem order;
- target configuration;
- deployment network;
- host paths.

---

## 8. Implementation and migration

### 8.1 Phase-1 crate creation

Add:

```text
packages/realization/
```

and register it in the workspace.

Initial dependency set should be minimal:

```text
architecture
```

Additional generic dependencies may be added deliberately through workspace
policy when required for typed serialization, hashing, or errors.

### 8.2 Initial modules

A likely initial module structure is:

```text
realization/src/
├── lib.rs
├── derive.rs
├── validate.rs
├── identity.rs
├── fact.rs
├── expression.rs
├── relation.rs
├── operation.rs
├── authorization.rs
├── observable.rs
├── representation.rs
├── constructibility.rs
├── lifecycle.rs
└── declassification.rs
```

> Illustrative module layout; not frozen by this decision.

Begin with the minimum modules required by the pilots. Do not create empty
architecture for every future concern merely to match the plan.

### 8.3 Pilot A: `compact-ash`

The first declaration tests:

- architecture operation binding;
- bounded family cardinality;
- ownerless `U` conservation;
- one output-family relation;
- permissionless constructibility;
- sponsor isolation;
- no root use;
- no public value disclosure beyond public ASH facts;
- transition projection.

It intentionally avoids:

- STATE;
- owner signatures;
- wide division;
- issuance;
- confidential-to-public synchronization;
- variable recipient routing.

### 8.4 Pilot B: `transfer-live-receipts`

The second declaration tests:

- owner authorization;
- same-class closure;
- exact semantic value conservation;
- multiple inputs/outputs;
- destination-owner freedom;
- sponsor isolation;
- representation alternatives;
- owner-secret witness availability;
- no root use.

It introduces the first meaningful proof alternative:

```text
explicit arithmetic
or
confidential value conservation
```

at the target-independent semantic level.

### 8.5 Incremental model conformance

For each pilot:

1. map architecture operation to realization declaration;
2. construct valid model transitions;
3. project semantic facts;
4. evaluate realization expressions and relations;
5. compare model certificate/projection with declared expectations;
6. create focused mutations;
7. verify deterministic derivation.

Do not migrate all model formulas at once.

### 8.6 Declassification migration

Once pilot dependencies exist:

1. derive pilot declassification in `realization`;
2. update artifact generation to consume the typed realization result;
3. preserve exact publication formatting or deliberately version it;
4. compare committed artifact bytes through the existing checker;
5. remove provisional model-side derivation after full operation coverage is
   available.

During migration, dual derivations may coexist only as a temporary
cross-check. One must not silently become a permanent second source.

### 8.7 Compiler onboarding

Only after the Phase-1 exit gate:

1. create the compiler crate;
2. consume `RealizationSpec`;
3. lower pilot relations;
4. preserve realization relation IDs/provenance;
5. add target-independent proof and disclosure plans;
6. keep target emission out of compiler core.

### 8.8 Publication timing

Do not publish a stable `realization.json` or realization hash merely because
the crate exists.

Publish only when:

- a consumer needs it;
- schema ownership is clear;
- the canonical projection is defined;
- stale-artifact checking exists;
- identity migration policy exists;
- the publication remains one-way derivative.

---

## 9. Risks and limitations

### 9.1 Semantic duplication risk

The realization declaration and executable model both express operation
meaning in different forms.

That duplication is deliberate enough to provide a conformance boundary, but it
can drift.

Mitigations:

- architecture IDs shared directly;
- bidirectional coverage validation;
- model-conformance tests;
- relation-indexed mutations;
- deterministic derived declassification;
- gradual operation-by-operation migration;
- no third compiler-local semantic table.

### 9.2 Premature universal IR risk

Designing one exhaustive semantic vocabulary before implementing pilots may
produce an abstract, overgeneralized system that fits no target or model well.

Mitigation:

- accept the package boundary now;
- keep exact enums provisional;
- implement the minimum vocabulary for two pilots;
- add vocabulary only when later operations demonstrate need;
- version published identities deliberately.

### 9.3 Shared evaluator correlated-failure risk

If model checks and realization evaluation share every arithmetic helper, one
bug can appear as agreement.

Mitigations:

- compare against independent mathematical boundary vectors;
- retain model operation tests;
- use relation mutations;
- later compare backend execution;
- use independent deployment implementations where required;
- avoid describing shared-code agreement as independent proof.

### 9.4 Layer-boundary leakage risk

Target concerns may enter realization because they are convenient to express
early.

Examples:

- explicit-value prefix;
- tapscript input index;
- coordinator leaf;
- control-block depth;
- particular commitment encoding.

Mitigation:

- crate dependency rules;
- package review;
- target-neutral type names;
- compiler capability abstraction;
- tests or lints forbidding target dependencies.

### 9.5 Architecture bloat risk

Once realization exists, contributors may move finite architecture facts into
it or semantic formulas back into architecture without clear ownership.

Mitigation:

- architecture owns enumerated finite sets and relations;
- realization owns target-independent semantic detail;
- compiler owns analysis/planning;
- target/backend owns proof mechanisms;
- decision updates required for boundary changes.

### 9.6 Identity migration risk

Publishing realization identities too early could freeze an immature
projection.

Mitigation:

- no stable realization hash before pilot completion;
- document included fields;
- use explicit algorithm/schema identifiers;
- retain migration records rather than silently redefining algorithms.

### 9.7 Model API pressure

Compiler-era tests may pressure the model to expose kernel internals publicly.

Mitigation:

- use explicit test/conformance adapters;
- preserve crate-private low-level construction;
- treat model state transparency separately from kernel construction access;
- add only APIs justified by typed conformance.

---

## 10. Supersession conditions

This decision may be superseded if the project adopts another single typed
semantic source that preserves the same role, such as:

- a language-neutral typed realization DSL;
- generated Rust realization types from a normative schema;
- a formally verified semantic language shared by model and compiler.

Supersession requires:

1. one authoritative typed semantic source;
2. complete architecture binding;
3. model-conformance path;
4. compiler-consumption path;
5. declassification derivation;
6. target independence;
7. deterministic identity and migration policy;
8. no publication-file reverse dependency;
9. no model source scraping;
10. explicit replacement decision and implemented ADR.

This decision is not superseded merely because:

- the realization is serialized for external use;
- model and realization share arithmetic domain types;
- compiler IR structurally resembles realization relations;
- a backend supports only one proof alternative initially;
- Simplicity is deferred;
- operation declarations are introduced incrementally.

---

## 11. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/architecture/src/ids.rs`](../../packages/architecture/src/ids.rs)
- [`../../packages/architecture/src/validate.rs`](../../packages/architecture/src/validate.rs)
- [`../../packages/model/src/lib.rs`](../../packages/model/src/lib.rs)
- [`../../packages/model/src/manifest.rs`](../../packages/model/src/manifest.rs)
- [`../../packages/model/src/transition.rs`](../../packages/model/src/transition.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)

### Related decisions

- [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
- [D003: Multiple Backends, Tapscript First](003-multiple-backends-tapscript-first.md)
- [D004: Translation Validation Over Compiler Trust](004-translation-validation-over-compiler-trust.md)
- [D005: Value-Parametric, Closed-Asset-Identity-Rigid](005-value-parametric-asset-rigid.md)

### Planning architecture

- [`../README.md`](../README.md)
- [`../toolchain-architecture.md`](../toolchain-architecture.md)
- [`../roadmap.md`](../roadmap.md)
- [`../backlog.md`](../backlog.md)
- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)

---

## 12. Decision summary

> Add one typed target-independent `realization` package between architecture
> and compiler. It preserves architecture as the owner of finite enumerated
> facts, declares the semantic formulas and relations required for compilation,
> derives declassification and constructibility requirements, is checked
> against executable model behavior, and contains no target opcode or concrete
> transaction-layout policy.
