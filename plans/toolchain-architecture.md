# Tripod Toolchain Architecture

> **Status:** ACTIVE
> **Scope:** Planned implementation architecture below the existing typed
> architecture and executable model
> **Authority:** Implementation plan only; normative protocol and realization
> artifacts take precedence
> **Current phase:** Phase 1 — typed realization foundation
> **Next substantial package:** `tripod-realization`

This document defines the planned cross-package architecture for the attestation realization/compiler toolchain.

It owns:

- the planned typed data flow;
- cross-package dependency direction;
- package responsibility boundaries;
- identity and hashing ownership;
- generated-artifact direction;
- assurance boundaries;
- deterministic-build requirements;
- representation and disclosure boundaries;
- release assembly rules.

It does not own:

- protocol economics;
- the architecture's enumerated sets and relations;
- executable model behavior;
- Elements opcode semantics;
- package-specific implementation detail;
- milestone sequencing;
- the current task queue.

Those concerns live in the normative source, package plans, research notes,
[`roadmap.md`](roadmap.md), and [`backlog.md`](backlog.md).

---

## 1. Normative context

The toolchain is subordinate to the repository's existing authority hierarchy.

### 1.1 Layer 0

The Layer-0 paper owns the abstract economic interface and specified
properties:

```text
papers/attestation/main.tex
papers/attestation/sections/
```

The toolchain must not reinterpret, extend, or weaken Layer 0.

### 1.2 Realization document

The realization document owns the prose conformance contract:

```text
docs/attestation/realization.md
```

This includes:

- the invariant interpretation;
- operation semantics not reducible to finite manifest structure;
- translation obligations;
- oracle obligations;
- trust assumptions and residuals;
- representation and disclosure rules;
- release pins.

A compiler implementation that conflicts with the realization document is
defective. If an implementation requirement exposes a genuine semantic
problem, the normative artifacts must be revised through their versioning
process before compiler behavior changes.

### 1.3 Typed architecture

The architecture crate owns every enumerated architecture set and relation:

```text
packages/architecture/
```

This includes:

- stable semantic identifiers and discriminants;
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
- publication and deployment-profile schemas;
- canonical architecture hashes.

The compiler must never maintain a second handwritten copy of these facts.

### 1.4 Executable model

The model crate owns the current executable reference behavior:

```text
packages/model/
```

It implements:

- operation constructors;
- abstract authorization;
- exact canonical and open-flow partitions;
- branch shape;
- transition-certificate derivation;
- root-history replay;
- invariant checking;
- attestation indexing;
- receipt-accounting audit;
- maintenance and quiescence;
- deterministic and generated test evidence.

The model is a transparent reference model, not a source language. The
compiler must not parse or scrape model Rust source to reconstruct semantics.

### 1.5 Planning status

This document is not normative protocol source and is not machine-consumed by
the toolchain.

A future implementation may promote stable, repository-wide choices from
[`decisions/`](decisions/README.md) into root ADRs. Once implemented, those
ADRs take precedence over this planning document.

---

## 2. Architectural objective

The toolchain must transform one typed, target-independent realization into a
deterministic, target-specific deployment bundle with evidence sufficient to
validate the translation.

The intended direction is:

```text
typed architecture
        ↓
typed target-independent realization
        ↓
typed relation / proof / disclosure / layout analysis
        ↓
typed target-specific relocatable programs
        ↓
typed linked deployment bundle
        ↓
typed canonical transaction and witness ABI
        ↓
relation-indexed and independent evidence
        ↓
validated deployment profile
        ↓
optional publication artifacts
```

The forbidden direction is:

```text
architecture.json
architecture.toml
declassification.json
model_labels.json
Markdown
LaTeX
model source text
        ↓ parse, deserialize, or scrape
compiler semantics or release policy
```

Generated publications may be compared, distributed, independently consumed
for interoperability, and attached to documentation. They are never trusted
semantic inputs to the first-party compiler, linker, transaction builder, or
release gate.

This direction is governed by
[Decision 001](decisions/001-typed-rust-is-normative.md).

---

## 3. Planned package map

The planned source-level package names are:

```text
packages/
├── architecture       existing typed architecture
├── realization        target-independent semantic declaration
├── model              existing executable reference model
│
├── compiler           target-independent analysis and Relation IR
├── target-elements    exact typed Elements capability model
├── tapscript          Elements tapscript backend
├── simplicity         future Simplicity backend
├── linker             constructor resolution and deployment linking
├── transaction        canonical transaction and witness ABI
├── vectors            translation-validation evidence
└── release            release/profile assembly and final gate
```

Cargo publication names retain the project namespace:

```text
tripod-architecture
tripod-realization
tripod-model
tripod-compiler
tripod-target-elements
tripod-tapscript
tripod-simplicity
tripod-linker
tripod-transaction
tripod-vectors
tripod-release
```

The short source-level names keep Rust imports readable:

```rust
use architecture::ARCHITECTURE;
use realization::RealizationSpec;
use compiler::AnalyzedProgram;
use linker::LinkedBundle;
use transaction::OperationTransaction;
```

---

## 4. Conceptual package responsibilities

### 4.1 `architecture`

`architecture` remains the source of every finite enumerated architecture fact.

It provides:

- stable architecture identifiers;
- the normative `ARCHITECTURE` declaration;
- draft and release validation;
- canonical semantic and behavioural hashes;
- deterministic publication DTOs;
- deployment-profile types and validation.

It must not depend on:

- realization;
- model;
- compiler;
- target packages;
- backend packages;
- linker;
- transaction construction;
- evidence tooling;
- release assembly.

### 4.2 `realization`

`realization` provides the target-independent semantic declaration required by
both model conformance and compiler analysis.

It owns typed declarations for:

- semantic facts;
- expressions and formulas;
- operation dependencies;
- semantic relations;
- state assignments;
- object recognition;
- authorization relations;
- recipient relations;
- conservation and closure;
- event projections;
- public observables;
- representation capabilities;
- constructibility requirements;
- lifecycle requirements;
- proof alternatives;
- derived declassification.

It depends on `architecture` and is subordinate to the realization document.

It contains no:

- Elements opcodes;
- tapscript stack positions;
- control blocks;
- Simplicity combinators;
- deployment network constants;
- generated-file parsers.

Its detailed contract is in
[`packages/realization.md`](packages/realization.md).

### 4.3 `model`

`model` remains the executable reference implementation.

During the realization migration, it will incrementally gain conformance tests
against `RealizationSpec`. It should not be rewritten in one step.

The intended transition is:

```text
current:
    architecture → model

planned:
    architecture → realization
    architecture + realization → model conformance
```

The model may continue to contain executable operation code and optimized
helpers. What matters is that its externally meaningful transition relation is
checked against the same typed realization consumed by compiler analysis.

The model must not become a dependency of the compiler merely so the compiler
can call model-specific internals or inspect operation source.

The vector harness may depend on both because it compares their independently
expressed roles.

### 4.4 `compiler`

`compiler` performs target-independent analysis.

It consumes:

- a validated `RealizationSpec`;
- typed deployment parameters;
- typed compilation policy;
- an abstract target-capability description.

It produces a deterministic analyzed program containing:

- relation graph;
- proof plan;
- disclosure plan;
- fact-source analysis;
- witness-availability analysis;
- lifecycle paths;
- obligation-placement plan;
- canonical layout requirements;
- target requirement set;
- architecture and realization identities.

It does not emit tapscript or Simplicity programs directly.

It does not parse:

- architecture JSON/TOML;
- declassification JSON;
- model source;
- Markdown;
- LaTeX.

Its detailed contract is in
[`packages/compiler.md`](packages/compiler.md).

### 4.5 `target-elements`

`target-elements` provides the exact typed capability and semantic pin for one
Elements deployment target.

It owns:

- reviewed upstream provenance (review metadata, not protocol identity;
  ADR-011);
- target network and genesis identity;
- tapscript activation status;
- leaf version;
- opcode assignments;
- semantics of opcodes used by the backend;
- explicit/confidential asset and value prefixes;
- sighash behavior;
- consensus and policy resource limits;
- target dependency claims requiring deployment evidence.

It does not own:

- protocol operations;
- architecture policy;
- operation layouts;
- proof planning;
- emitted scripts.

The Markdown survey under `plans/reference/` is not its input. The future target
crate must encode source-verified facts directly as typed Rust values.

Its detailed contract is in
[`packages/target-elements.md`](packages/target-elements.md).

### 4.6 `tapscript`

`tapscript` is the first target backend.

It consumes:

- an `AnalyzedProgram`;
- a typed Elements target;
- linker-owned relocatable-program interfaces;
- backend configuration.

It owns:

- typed script construction;
- instruction selection;
- stack-effect validation;
- target guard patterns;
- target proof patterns;
- narrow and wide arithmetic lowering;
- transaction introspection patterns;
- metadata/object-constructor patterns;
- canonical layout lowering;
- witness ABI derivation;
- resource formulas;
- relocatable tapscript program emission.

It emits typed relocatable programs. It does not perform final deployment
linking or release assembly.

Its detailed contract is in
[`packages/tapscript.md`](packages/tapscript.md).

### 4.7 `simplicity`

`simplicity` is a future backend behind the same target-independent compiler
boundary.

It remains parked until:

- the realization vocabulary is stable;
- the compiler IR has supported the first Elements backend;
- backend-neutral proof planning has demonstrated a real boundary;
- the first linked deployment path is understood.

The initial implementation must not contain tapscript assumptions in structures
that are claimed to be backend-independent.

Its detailed status is in
[`packages/simplicity.md`](packages/simplicity.md).

### 4.8 `linker`

`linker` consumes typed relocatable programs and typed deployment constants.

It owns:

- object-constructor symbol interfaces;
- reference-graph construction;
- strongly connected component analysis;
- relocation;
- taptree assembly;
- constructor resolution;
- deployment-constant substitution;
- deterministic bundle construction;
- bundle hashing;
- resource-calibration interfaces.

A linker symbol is generally an **object constructor**, not a single constant
scriptPubKey.

Examples include:

```text
STATE(state)
RECEIPT_L(owner)
RECEIPT_T(owner)
DEPOSIT_ENTITLEMENT(owner, target_cycle)
DISTRIBUTION_CONTROL(cycle, counters)
DISTRIBUTION_VAULT(cycle)
```

The linker does not define operation semantics. It resolves backend programs
whose obligations were already derived from the realization and compiler.

Its detailed contract is in
[`packages/linker.md`](packages/linker.md).

### 4.9 `transaction`

`transaction` owns the canonical transaction and witness ABI generated from a
linked bundle.

It owns:

- operation layout ABI;
- input and output-family ordering;
- coordinator placement;
- witness stack ABI;
- metadata encoding;
- leaf and control-block selection;
- operation transaction assembly;
- representation and blinding policy;
- worst-case transaction generation;
- measurement fixtures.

Its outputs are consumed by:

- clients and wallets;
- vector harnesses;
- resource calibration;
- deployment/genesis tooling.

It is a first-class deployment interface, not merely test plumbing.

Its detailed contract is in
[`packages/transaction.md`](packages/transaction.md).

### 4.10 `vectors`

`vectors` owns translation-validation evidence.

It consumes broadly:

- architecture and realization identities;
- model behavior;
- analyzed compiler programs;
- target capabilities;
- backend programs;
- linked bundles;
- transaction layouts;
- operation transactions.

It owns:

- relation-indexed accepting vectors;
- independent rejecting mutations;
- representation metamorphisms;
- mixed-branch vectors;
- constructor-continuity vectors;
- backend-versus-model comparisons;
- failure shrinking;
- resource reports;
- substrate tests;
- report rendering.

It does not replace independent deployment implementations. In particular, a
second invocation of the model's `ReferenceIndexer` is not the independently
implemented candidate indexer required by deployment evidence.

Its detailed contract is in
[`packages/vectors.md`](packages/vectors.md).

### 4.11 `release`

`release` owns final assembly and release validation.

It consumes typed:

- architecture identity;
- realization identity;
- compiler configuration identity;
- target identity;
- linked bundle;
- transaction/witness ABI identity;
- resource reports;
- vector reports;
- independent event/query/accounting reports;
- deployment profile.

It validates every release boundary and writes final publication artifacts only
after all checks pass.

It does not derive protocol semantics and does not repair incomplete evidence.

Its detailed contract is in
[`packages/release.md`](packages/release.md).

---

## 5. Planned dependency direction

The exact Cargo dependency graph may be refined while package APIs are drafted,
but the following direction is mandatory.

A conceptual data-flow view is:

```text
architecture
    │
    ▼
realization
   ├──────────────────▶ model conformance
   │
   ▼
compiler
   │
   ├─────────────── target-independent analyzed program
   │
   ▼
backend + typed target
   │
   ▼
relocatable programs
   │
   ▼
linker
   │
   ▼
linked bundle
   │
   ▼
transaction ABI and transaction builder
   │
   ▼
vectors and independent evidence
   │
   ▼
release
```

An illustrative acyclic Rust dependency structure is:

```text
realization
    depends on architecture

model
    depends on architecture
    incrementally depends on realization for conformance

compiler
    depends on realization

target-elements
    owns target facts
    must not depend on protocol model behavior

linker
    owns common relocatable and linked-bundle interfaces
    may depend on compiler identity/IR artifact types

tapscript
    depends on compiler
    depends on target-elements
    depends on linker interfaces

simplicity
    depends on compiler
    depends on linker interfaces
    does not depend on tapscript

transaction
    depends on linked-bundle/layout interfaces
    depends on target transaction types as required

vectors
    may depend broadly on model, realization, compiler, target,
    backends, linker, and transaction

release
    may depend broadly on typed identities, bundles, profiles,
    and evidence report types
```

The implementation must not introduce cycles merely to model a feedback loop.

---

## 6. Calibration without a Cargo dependency cycle

Resource calibration creates a conceptual loop:

```text
candidate bounds
    ↓
compile and link
    ↓
construct worst-case transactions
    ↓
measure
    ↓
choose new bounds
    ↺
```

That loop must not become a Rust crate cycle between `linker` and
`transaction`.

The planned separation is:

1. `linker` exposes deterministic candidate linking for typed bounds.
2. `transaction` consumes a candidate linked bundle and constructs worst-case
   transactions.
3. a higher-level calibration runner—initially under release or dedicated
   orchestration code—repeats candidate linking and measurement;
4. the runner passes measurements back as typed inputs;
5. the final linker invocation produces the bound bundle.

The linker may expose a pure search helper parameterized by a measurement
callback, but it must not depend directly on a transaction package that already
depends on linker output.

Calibration measures complete worst-case transactions, not isolated leaves.

Shared bounds are calibrated against the maximum across all affected operation
families.

---

## 7. Typed semantic pipeline

### 7.1 Architecture identity

The pipeline begins only with a validated architecture.

Architecture release remains responsible for:

- structural validation;
- final publication status;
- Layer-0 anchor-set pin;
- semantic-hash verification;
- behavioural-hash versioning gate;
- canonical publication artifacts.

The compiler does not recreate these checks from publication files. It calls
the typed architecture APIs.

### 7.2 Realization derivation

The realization layer produces one deterministic `RealizationSpec`.

The intended entry point is:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

> This API is illustrative and not frozen until the realization pilot
> milestone exits.

The derivation must validate that:

- every architecture operation has a realization;
- every object and quantity reference resolves;
- architecture and realization coverage are bidirectional;
- every expression is well typed;
- every semantic relation has typed operands;
- operation dependencies are complete;
- declassification derives from those dependencies;
- no target-specific primitive appears.

### 7.3 Compiler analysis

The compiler lowers the realization to an analyzed relation graph.

Each relation must have:

- deterministic identity;
- typed operands;
- source provenance;
- operation ownership;
- proof alternatives;
- disclosure dependencies;
- witness-availability requirements;
- lifecycle relevance;
- a placement requirement;
- target-capability requirements.

The analyzed program remains target-independent. It may describe required
capabilities such as:

```text
explicit-value introspection
confidential-value conservation
value-commitment equality
relative timelock
output scriptPubKey introspection
wide checked arithmetic
```

It does not name concrete opcode numbers.

### 7.4 Backend emission

A backend selects target-specific proof methods for the analyzed obligations and
emits relocatable programs.

The backend must fail closed when:

- no supported proof method discharges a relation;
- a required fact is unavailable;
- a permissionless path requires a private witness;
- an obligation has no carrying predicate;
- a representation has no lifecycle exit;
- a layout cannot enforce exact closure;
- resource formulas already exceed hard target limits.

The backend must not silently fall back to a weaker relation.

### 7.5 Linking

The linker resolves constructors and deployment constants.

Its output is one typed `LinkedBundle` containing:

- target identity;
- architecture identity;
- realization identity;
- compiler/backend configuration identity;
- linked object constructors;
- operation leaves/programs;
- taptree/control information;
- relocations resolved;
- resource formulas;
- canonical ordering;
- bundle hash.

### 7.6 Transaction ABI

The transaction package derives the canonical concrete ABI from the linked
bundle.

Its outputs include:

- operation layouts;
- input/output family ranges;
- coordinator rules;
- witness ordering;
- metadata encoding;
- leaf/control-block selection;
- representation requirements.

A wallet or test harness should be able to construct a transaction from this
typed ABI without reverse-engineering emitted script.

### 7.7 Evidence and release

The vector harness validates the emitted relation against the model and
realization.

The release package then checks:

- identity bindings;
- target pin;
- calibrated bounds;
- dependency evidence;
- artifact hashes;
- relation coverage;
- model/property reports;
- independent event-projection report;
- independent attestation-query report;
- independent receipt-accounting report;
- script-integration report;
- final deployment-profile status.

Only then may publication artifacts be written.

---

## 8. Identity and hashing ownership

Different hashes serve different assurance purposes. They must not be conflated
or reused across domains.

### 8.1 Architecture identities

Owned by `architecture`:

- architecture semantic hash;
- architecture behavioural hash;
- Layer-0 anchor-set hash;
- architecture schema;
- realization-version envelope field.

The compiler consumes these typed identities. It does not recompute them from
generated files.

### 8.2 Realization identity

The future `realization` package should define a domain-separated canonical
identity over the target-independent realization declaration.

That identity must bind at least:

- typed fact vocabulary;
- expression/formula graph;
- semantic relations;
- operation dependencies;
- public observables;
- constructibility requirements;
- lifecycle requirements;
- representation capabilities;
- proof alternatives;
- derived declassification.

The precise algorithm and schema must be decided before the first realization
identity is published. Until then, API sketches must not imply a frozen hash.

A realization identity is subordinate to the architecture identity and must
embed or bind it.

### 8.3 Compiler configuration identity

The compiler/backend configuration identity should bind choices that affect
emission but do not alter protocol semantics, including:

- compiler IR schema;
- proof-rule library version;
- deterministic proof-selection policy;
- layout policy;
- obligation-placement policy;
- backend pattern library;
- backend configuration;
- taptree weighting policy;
- resource model version.

Changing this identity may alter emitted bytes while leaving architecture and
realization identities fixed.

### 8.4 Target identity

Owned by the typed target package and deployment profile:

- typed compatibility-contract schema/version (upstream review provenance
  recorded beside it, outside the identity; ADR-011);
- network/genesis identity;
- activation set;
- leaf version;
- opcode semantics;
- sighash semantics;
- target resource/policy limits.

A target identity is not an architecture identity.

### 8.5 Bundle identity

Owned by `linker`:

- resolved constructors;
- emitted scripts/programs;
- taptrees;
- deployment constants;
- calibrated bounds;
- canonical ordering;
- target identity;
- relevant upstream identities.

The bundle hash is the deployment artifact commitment corresponding to the
deployment profile's emitted-script-bundle field.

### 8.6 Transaction ABI identity

Owned by `transaction`:

- operation layouts;
- witness ABI;
- metadata encoding;
- control-block selection rules;
- representation requirements.

The ABI may be included directly in the linked-bundle identity or separately
hashed and bound by it. The package plan must choose one canonical ownership
rule before publication.

### 8.7 Evidence identities

Each evidence class receives a separate report and hash.

At minimum:

- model unit-test report;
- property-test report;
- relation-coverage report;
- backend differential report;
- raw attestation-event projection report;
- canonical attestation-query report;
- receipt-accounting audit report;
- script-integration report;
- target dependency reports;
- resource calibration report.

Independent claims must not share one undifferentiated report hash.

### 8.8 Deployment-profile identity

Owned by the architecture deployment-profile API.

The deployment-profile hash is domain-separated from every architecture,
realization, compiler, bundle, ABI, and report hash.

---

## 9. Generated-artifact law

Every generated artifact must have:

1. one typed source;
2. one generator;
3. one non-writing checker;
4. canonical ordering;
5. a documented schema;
6. deterministic bytes;
7. a stale-artifact CI test;
8. no reverse semantic dependency.

The current `tripod-artifacts` crate establishes the initial pattern.

Future derivative artifacts may include:

```text
realization.json
declassification.json
operation_layouts.json
obligation_placement.json
object_constructors.json
witness_abi.json
resource_report.json
relation_coverage.json
target_identity.json
linked_bundle_manifest.json
```

The names above are illustrative, not committed.

The rule is not optional:

```text
typed Rust value
    ↓ canonical rendering
publication file
```

Never:

```text
publication file
    ↓ parsing
first-party semantic/compiler value
```

Tests may parse publications to verify interoperability and canonical
round-tripping. That test path must compare with independently derived typed
expected values and must not feed the parsed publication into compilation.

---

## 10. Assurance boundaries

The project has several distinct assurance layers. Passing one must never be
described as passing the next.

### 10.1 Architecture assurance

Proves:

- finite architecture declarations are internally valid;
- stable identifiers are complete;
- closure relations hold;
- publication is canonical;
- hashes verify;
- Layer 0 is pinned;
- final architecture release requirements hold.

Does not prove:

- target-independent formulas are completely declared;
- model behavior conforms to the future realization declaration;
- compiler analysis is correct;
- emitted script enforces the model;
- target semantics match assumptions;
- deployment bounds fit.

### 10.2 Realization assurance

Will prove:

- every architecture operation and object has a typed semantic realization;
- expressions and relations are well typed;
- operation dependencies are complete;
- declassification is derived;
- constructibility and lifecycle requirements are declared;
- model behavior conforms to the target-independent realization.

Does not prove:

- a target supports the required proofs;
- a compiler selected a correct target proof plan;
- emitted programs enforce the relations.

### 10.3 Compiler-analysis assurance

Will prove:

- every semantic relation is represented;
- proof alternatives are identified;
- disclosures have provenance;
- permissionless witness availability is satisfied;
- lifecycle paths are reachable;
- obligations are placed;
- layouts are complete;
- target requirements are explicit.

Does not prove:

- target patterns implement their contracts;
- stack scheduling is correct;
- linked constructors preserve continuity;
- concrete transactions execute as expected.

### 10.4 Backend and linker assurance

Will establish through tests and differential evidence:

- target patterns implement selected proof relations;
- every relation has a carrying predicate;
- object constructors and relocations are resolved;
- linked bytes are deterministic;
- target resource constraints are respected;
- constructor continuity holds.

This remains translation validation, not a claim that the compiler is formally
verified.

### 10.5 Transaction assurance

Will establish:

- canonical transaction layouts match linked predicates;
- witness ordering is correct;
- metadata encodings are canonical;
- representation and blinding choices satisfy target requirements;
- worst-case transactions are valid measurement witnesses.

### 10.6 Deployment assurance

Requires all preceding identities and evidence plus:

- exact target/network/genesis binding;
- calibrated finite bounds;
- verified substrate dependencies;
- emitted artifact hashes;
- canonical wire vectors;
- independent event/query/accounting reports;
- final deployment-profile status.

Architecture finality and a green model are necessary and insufficient.

---

## 11. Translation-validation model

The initial compiler is not claimed to be fully verified.

Instead, every released bundle is validated against the target-independent
realization and executable model.

Each semantic relation must have:

- one stable relation identifier;
- at least one accepting vector;
- at least one mutation that breaks that relation independently;
- an identified carrying predicate or leaf;
- expected and actual target verdicts;
- source provenance;
- evidence status in the coverage report.

The computed coverage relation is conceptually:

```text
relation
× vector
× carrying predicate
× expected result
× actual result
```

A broad successful operation test cannot substitute for relation coverage.

The evidence story is governed by
[Decision 004](decisions/004-translation-validation-over-compiler-trust.md).

---

## 12. Model and compiler separation

The compiler and model have related but different roles.

### Model

The model answers:

> Given an abstract world and a declared operation, is the transition accepted,
> and what semantic successor and projections result?

### Compiler

The compiler answers:

> Which target-independent relations define that acceptance and successor, what
> facts do they depend on, which target proof alternatives can discharge them,
> and where must those obligations be enforced?

### Vector harness

The vector harness answers:

> Does this concrete linked deployment accept and reject consistently with the
> target-independent relation and model over the tested relation coverage?

The compiler must not call model internals as a substitute for representing
relations. The model must not ingest compiler artifacts to decide abstract
behavior.

Shared formulas may eventually move into the realization package or a
target-independent typed expression evaluator, but migration must preserve
independent checking rather than collapse the model and compiler into one
implementation.

---

## 13. Target and backend boundary

The target-independent compiler must not assume:

- a stack machine;
- tapscript witness order;
- Elements opcode numbers;
- control blocks;
- a particular taptree shape;
- Simplicity jets;
- target-specific cost units.

It may require abstract capabilities, for example:

```rust
pub enum RequiredCapability {
    InputAssetInspection,
    InputValueInspection,
    OutputAssetInspection,
    OutputValueInspection,
    OutputProgramInspection,

    ExplicitValueArithmetic,
    ConfidentialValueConservation,
    ValueCommitmentEquality,
    AuthenticatedValueOpening,

    RelativeTimelock,
    OutputCommittingSignature,
    CheckedWideArithmetic,
    HashConstruction,
    ConstructorVerification,
}
```

> This vocabulary is illustrative and not frozen.

A target/backend adapter maps abstract capabilities to concrete proof methods.

The first implementation targets Elements tapscript. Simplicity remains a
second backend behind the same semantic boundary, as recorded by
[Decision 003](decisions/003-multiple-backends-tapscript-first.md).

---

## 14. Representation boundary

The abstract model has semantic values. A target may represent those values in
different supported ways.

The initial representation classes are conceptually:

```text
PrivateCommitted
PublicCommitted
Explicit
```

These are target representation modes, not distinct abstract values.

The initial Elements policy is:

> value-representation-parametric and closed-asset-identity-rigid.

This means:

- lateral value-preserving movement may use confidential commitments where
  target proof methods establish the same semantic relation;
- public state, public workflow, public event, and permissionless-construction
  seams may require explicit values or authenticated public openings;
- closed protocol asset identity remains explicitly classified at every
  protocol seam until a separately verified target proof supports confidential
  asset classification.

Closed protocol assets include:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

A confidential or unclassified output must not silently carry one of these
assets outside the recognized object closure.

This policy is recorded by
[Decision 005](decisions/005-value-parametric-asset-rigid.md).

---

## 15. Safety and disclosure minimality

Safety and disclosure minimality are separate claims.

### 15.1 Safety

A concrete transaction is safe only if target acceptance implies an authorized
semantic transition and invariant-preserving successor.

Safety evidence is reject-dominant:

- wrong asset;
- wrong value;
- missing root;
- missing relation;
- wrong recipient;
- mixed operation branches;
- malformed commitment or opening;
- closed-asset exfiltration;
- unclassified protocol output;
- incorrect state successor;
- invalid event projection.

### 15.2 Disclosure minimality

A backend is minimally demanding only if it does not require disclosure that
the semantic relation does not need when a supported lower-disclosure proof
exists.

Minimality evidence is accept-dominant:

- confidential value-preserving transfer;
- amount-blind commitment-preserving relabel;
- confidential sponsor value;
- representation metamorphisms preserving public semantic projection.

A blinded-accept vector is not a safety proof. A rejection vector is not a
minimality proof.

The compiler must derive disclosure dependencies from semantic relation
dependencies. It must not ingest a handwritten disclosure publication as
policy.

---

## 16. Permissionless constructibility

Permissionless does not merely mean that a script can verify a transaction
without a signature.

It means an arbitrary constructor can build a valid transaction from:

- public chain state;
- public openings or proof capsules;
- the constructor's own sponsor funds.

A permissionless operation must not require:

- another owner's private value opening;
- another owner's blinding factor;
- an operator secret;
- private metadata available only to an object creator;
- an unpublished witness;
- an unavailable off-chain database.

The realization and compiler must represent witness availability explicitly.

A conceptual vocabulary is:

```rust
pub enum WitnessAvailability {
    CompileTime,
    DeploymentConstant,
    PublicChainData,
    PublicOpening,
    CurrentOwnerSecret,
    OperatorSecret,
    SponsorLocal,
}
```

> This vocabulary is illustrative and not frozen.

For a permissionless operation, required witnesses cannot have
`CurrentOwnerSecret` or `OperatorSecret` availability.

Representation support is incomplete if it makes an object's required
lifecycle exit impossible for an authorized constructor.

---

## 17. Canonical layouts and obligation placement

Elements tapscript has no general loop construct. Bounded collection relations
therefore require deterministic concrete layouts.

Each operation receives a canonical layout defining:

- input-family order;
- output-family order;
- fixed and ranged slots;
- cardinality witnesses;
- coordinator input;
- sponsor region;
- data-output region;
- optional families;
- carrying predicates for transaction-global obligations.

Counts alone do not authenticate ranges.

The carrying coordinator must verify:

- transaction input/output counts;
- range bounds;
- family constructors in each slot;
- range disjointness;
- protocol-family completeness;
- sponsor/open suffix policy.

Every semantic obligation must be assigned to at least one carrying predicate.

No obligation may exist only in compiler metadata while being enforced by no
script.

The canonical-layout policy is recorded by
[Decision 006](decisions/006-canonical-transaction-layout-abi.md).

---

## 18. Object constructors and linking

Protocol outputs are metadata-parameterized.

Therefore the linker resolves constructor families, not merely fixed byte
strings.

A constructor may depend on:

- object kind;
- internal key;
- static operation subtree;
- dynamic owner;
- class;
- cycle;
- state fields;
- control counters;
- target schema;
- deployment constants.

Reference resolution may use:

1. **identity introspection**
   The successor program is byte-identical to a consumed program.

2. **link-time constant/hash resolution**
   An acyclic reference can be resolved before deployment.

3. **in-script constructor reconstruction**
   A metadata-dependent successor is reconstructed from authenticated fields.

4. **authenticated witnessed-root continuity**
   A self-referential static subtree is supplied once, authenticated against
   the predecessor, and reused for successor construction.

The exact STATE constructor remains prototype-dependent. It is tracked in
[`research/state-object-constructor.md`](research/state-object-constructor.md).

The package interfaces must not freeze a constructor strategy until that
prototype satisfies its acceptance, rejection, and resource criteria.

---

## 19. Deterministic-build law

Identical typed inputs must produce byte-identical outputs.

This applies to:

- realization derivation;
- relation IDs;
- expression IDs;
- proof plans;
- disclosure plans;
- fact-source plans;
- obligation placement;
- operation layouts;
- target requirements;
- backend scripts;
- relocatable programs;
- taptrees;
- linked bundles;
- witness ABIs;
- resource reports;
- vector reports;
- publication artifacts.

Required implementation practices include:

- ordered collections for canonical output;
- no dependency on hash-map iteration order;
- canonical structural hashing;
- explicit domain separation;
- deterministic tie-breaks;
- no wall-clock timestamps;
- no host-path leakage;
- no environment-dependent output unless the environment value is an explicit
  typed input;
- pinned target and dependency resolution;
- deterministic temporary/output handling;
- clean-tree verification after checks and generators.

Optimization must not introduce nondeterministic search.

If two proof plans are equally acceptable, a documented canonical tie-break
chooses one.

---

## 20. Error and failure policy

Every package must fail closed.

Examples:

- unresolved architecture reference → error;
- incomplete realization coverage → error;
- ill-typed expression → error;
- missing proof method → error;
- unsupported target capability → error;
- private witness required by permissionless path → error;
- unplaced obligation → error;
- unresolved constructor cycle → error;
- resource-limit violation → error;
- ambiguous ABI → error;
- missing independent evidence → release failure;
- stale generated artifact → check failure.

A package must not:

- silently drop a relation;
- replace an unavailable proof with a weaker one;
- omit an output family;
- default an unresolved deployment constant;
- choose the first ambiguous profile entry;
- accept an unknown schema;
- manufacture missing evidence;
- downgrade a release check to a warning.

First-party command-line wrappers for future packages are governed by ADR-010.

---

## 21. Unsafe code and dependency policy

All future packages inherit ADR-011:

- Rust edition 2024;
- declared MSRV;
- `Cargo.lock` committed and authoritative;
- `--locked` in CI, build, and release paths;
- dependencies declared through workspace dependencies;
- permissive dependency licenses;
- advisory review;
- unsafe code denied workspace-wide unless explicitly scoped and justified.

A future FFI package requiring unsafe code must:

- scope the allowance to the smallest module;
- document the safety invariant;
- provide boundary tests;
- keep the rest of the workspace unsafe-free.

No target backend receives a blanket unsafe exception merely because it handles
cryptographic or binary data.

---

## 22. Release-writing ownership

Compilation, linking, checking, and evidence generation should operate in
memory or write only to caller-supplied output paths.

Final release publication is a distinct side effect.

The planned rule is:

- package libraries return typed values;
- generators render deterministic bytes;
- checkers compare without writing;
- release validates identities and evidence;
- only an explicit release/generation command writes final artifacts.

No test may repair or rewrite tracked source artifacts.

No compiler command may silently update a deployment profile or generated
manifest.

---

## 23. Non-goals

The initial toolchain is not intended to provide:

- a general-purpose programming language;
- a parser for architecture JSON/TOML as first-party semantic input;
- source-code scraping of the model;
- a fully verified compiler;
- a general optimizer;
- nondeterministic global proof search;
- confidential closed protocol asset identity;
- a complete Simplicity backend before tapscript;
- a general Elements script semantics formalization;
- an on-chain attestation accumulator;
- unbounded on-chain folds;
- arbitrary foreign sponsor sidecars;
- protocol redesign hidden inside backend choices;
- deployment release based only on a green model.

These may be revisited only through explicit decisions and, where applicable,
normative versioning.

---

## 24. Architecture invariants for every future package

Every package plan and implementation must answer:

1. What typed values are normative inputs?
2. What inputs are forbidden?
3. What typed values are returned?
4. Which package owns each identifier?
5. Which hashes or identities does the package produce?
6. What ordering makes its output deterministic?
7. What evidence validates its output?
8. What generated artifacts does it own?
9. Which assurance boundary does it establish?
10. Which assurance boundary does it explicitly not establish?
11. What are its public API and trust boundaries?
12. How does it fail closed?
13. Which research results block interface freeze?
14. What phase gate declares it ready?

A package that cannot answer these questions is not ready to enter the
workspace.

---

## 25. Definition of architectural readiness

This cross-package architecture is ready to support substantial implementation
when:

- [ ] the Phase-0 baseline gate is green;
- [ ] the planning rewrite is complete;
- [ ] Decisions 001–006 are reviewed and accepted;
- [ ] the realization package plan is approved;
- [ ] architecture and realization ownership are unambiguous;
- [ ] compiler and model roles remain distinct;
- [ ] the target/backend boundary contains no tapscript assumptions above it;
- [ ] linker and transaction dependencies are acyclic;
- [ ] calibration orchestration has no crate cycle;
- [ ] generated artifacts remain one-way derivatives;
- [ ] identity/hash domains are separately owned;
- [ ] permissionless witness availability is represented;
- [ ] safety and minimality evidence remain separate;
- [ ] independent indexer/auditor evidence remains outside the compiler trust
      path;
- [ ] every package has a focused plan with exit criteria.

---

## 26. One-line architecture

> The typed architecture declares the finite system; the typed realization
> declares its target-independent semantic relations; the compiler derives
> deterministic proof, disclosure, lifecycle, placement, and layout plans;
> target backends emit relocatable enforcement programs; the linker resolves
> constructors and deployment constants; the transaction package publishes the
> canonical concrete ABI; vectors validate the translation relation by
> relation; and release accepts a deployment only when all typed identities,
> calibrated bounds, substrate claims, and independent evidence reports are
> present and valid.
