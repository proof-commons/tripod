# D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler

> **Status:** ACCEPTED
> **Scope:** Assurance model for compiler analysis, backend emission, linking,
> transaction construction, evidence generation, and deployment release
> **Decision class:** Assurance
> **Applies to:** `realization`, `model`, `compiler`, `target-elements`,
> `tapscript`, future `simplicity`, `linker`, `transaction`, `vectors`, and
> `release`
> **Depends on:** [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md);
> [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md);
> [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md)
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** the conformance thesis, oracle obligations,
> model-to-script boundary, relation-indexed translation rules, deployment
> evidence profile, and release distinction in
> `docs/attestation/realization.md`
> **Related research:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/wide-arithmetic.md`](../research/wide-arithmetic.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> [`../research/settlement-layout.md`](../research/settlement-layout.md)
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The planned toolchain translates a target-independent semantic realization
into deployable target programs.

The translation path contains several independently fallible stages:

```text
typed architecture
        ↓
typed realization
        ↓
compiler relation analysis
        ↓
proof and disclosure planning
        ↓
operation layout and obligation placement
        ↓
target-specific instruction selection
        ↓
stack or target-program scheduling
        ↓
object-constructor emission
        ↓
relocation and linking
        ↓
transaction and witness assembly
        ↓
deployment calibration
        ↓
release publication
```

A defect at any stage can produce a concrete bundle that differs from the
intended abstract transition relation.

Examples include:

- omitting one root-succession check;
- checking the wrong output slot;
- accepting an undeclared output family;
- using a lower-bound-only quotient check;
- allowing closed protocol value to escape to an unclassified output;
- placing an obligation on no executable leaf;
- authenticating a predecessor constructor but not the successor;
- selecting a proof that requires an unavailable private witness;
- compiling a permissionless path with a hidden signature gate;
- mispatching a deployment constant;
- constructing a witness in the wrong order;
- calibrating an isolated script while the full transaction exceeds a limit;
- emitting nondeterministic bundle bytes from identical inputs.

Source review, compiler unit tests, and a green executable model reduce these
risks. They do not establish that one released target bundle enforces the
intended relation.

### 1.1 A verified compiler is not the initial project

A fully verified compiler would require a formal semantics and refinement proof
for at least:

- the typed realization language;
- the compiler IR;
- proof-plan selection;
- disclosure analysis;
- target capability mapping;
- target program semantics;
- instruction selection;
- stack scheduling;
- constructor emission;
- relocation;
- linking;
- transaction assembly;
- target consensus and policy assumptions.

That is a substantial research and implementation program.

The initial project instead needs a credible path to a deployable bundle while
being explicit about what remains trusted and what is checked.

### 1.2 The model is necessary but insufficient

The executable model provides strong evidence for the abstract transition
system:

- operation semantics;
- exact canonical and open-flow partitions;
- root succession;
- invariants;
- event projections;
- canonical indexing;
- accounting;
- mutation and property tests.

A green model does not prove that:

- compiler relations are complete;
- target proof selection is sound;
- target programs implement those proofs;
- linked constructors preserve code continuity;
- transaction witnesses match emitted programs;
- target opcodes have the assumed semantics;
- deployed bounds fit target limits.

Model evidence must remain distinct from backend and deployment evidence.

### 1.3 Translation validation fits the release model

The deployment profile already separates architecture identity from deployment
evidence and binds:

- implementation artifacts;
- calibrated bounds;
- target dependency evidence;
- model and property reports;
- independent event-projection reports;
- independent query reports;
- independent accounting reports;
- script-integration reports.

This is naturally compatible with **per-artifact translation validation**:

> For each candidate release bundle, validate the emitted artifact and its
> concrete transaction relation against the typed realization and executable
> reference behavior, then bind the resulting reports into the deployment
> profile.

The compiler implementation remains important, but the released bundle is
accepted because the bundle-specific evidence passes—not merely because the
compiler source was reviewed.

### 1.4 Relation-level coverage is required

Operation-level smoke tests are too coarse.

A `burn` test can succeed even if the emitted branch omits:

- no-ASH-input enforcement;
- one output-family closure check;
- record ordinal continuity;
- one owner authorization;
- sponsor isolation;
- event-type provenance.

The realization and compiler therefore need stable semantic relation
identities. Evidence can then be indexed by relation rather than only by
operation.

For each required relation, the release process should know:

- where the relation originated;
- which target predicate carries it;
- which valid vector exercises it;
- which focused mutation breaks it;
- the expected target verdict;
- the observed target verdict.

This does not mathematically prove all possible target executions. It makes
coverage omissions explicit and mechanically reviewable.

### 1.5 Independent observations remain separate

Even complete relation-indexed backend vectors do not replace independent
deployment observation.

The repository distinguishes at least:

1. abstract model behavior;
2. compiler and backend translation;
3. target substrate behavior;
4. attestation event recognition;
5. attestation query computation;
6. receipt-accounting audit.

A separately implemented indexer or auditor can detect failures outside the
compiler's own execution path. Those reports remain separate release
requirements.

---

## 2. Decision

The initial attestation compiler is **not** accepted as a formally verified
compiler.

Instead, every release candidate is accepted through **per-bundle translation
validation**.

The release assurance direction is:

```text
typed architecture and realization
        ↓
candidate analyzed program
        ↓
candidate emitted and linked bundle
        ↓
candidate canonical transactions and witnesses
        ↓
relation-indexed execution evidence
        +
target substrate evidence
        +
independent event/query/accounting evidence
        ↓
validated final deployment profile
```

The central decisions are:

1. **The released artifact is the validation subject.**
   Compiler source review and unit tests are necessary but insufficient. Each
   linked bundle must produce evidence bound to that exact bundle identity.

2. **Semantic relations receive stable identities.**
   Evidence is organized by target-independent realization relation and its
   source provenance.

3. **Every required relation has a carrying target predicate.**
   An obligation that exists in compiler metadata but is enforced by no
   executable target program is a compilation failure.

4. **Every required relation has positive and negative evidence.**
   At minimum:
   - one valid vector exercises it;
   - one focused mutation independently breaks it and is rejected.

5. **Operation tests do not substitute for relation coverage.**
   Broad end-to-end tests supplement, rather than replace, focused relation
   evidence.

6. **Model, compiler, backend, target, transaction, and independent-observer
   evidence remain distinct.**
   A green report at one boundary never silently upgrades another boundary.

7. **Reports are deterministic and identity-bound.**
   Each report binds the exact architecture, realization, target, compiler
   configuration, linked bundle, ABI, and vector-set identities relevant to
   its claim.

8. **Missing or stale evidence is a release failure.**
   Release tooling does not infer, repair, synthesize, or waive missing reports.

9. **The project makes no stronger claim than the evidence supports.**
   Translation validation is finite evidence over declared relations and
   vectors. It is not a proof over every possible concrete transaction unless
   a separately stated formal method establishes that result.

10. **Formal verification may be added incrementally.**
    A formally verified arithmetic gadget, backend pattern, or future backend
    strengthens the relevant evidence boundary without automatically proving
    the entire compiler.

This decision is implemented through the future `vectors` and `release`
packages and the deployment-profile evidence model.

---

## 3. Required consequences

### 3.1 Stable semantic relation registry

The target-independent realization must assign a deterministic identity to
every semantic relation that requires enforcement or evidence.

A relation record should conceptually carry:

```rust
pub struct Relation {
    pub id: RelationId,
    pub operation: architecture::OperationId,
    pub kind: RelationKind,
    pub operands: Vec<TypedOperand>,
    pub dependencies: Vec<RelationId>,
    pub provenance: RelationProvenance,
}
```

> Illustrative API; exact types are not frozen by this decision.

Relation identities must not depend on:

- target backend;
- target opcode;
- target stack layout;
- vector implementation;
- source line number;
- nondeterministic declaration order.

### 3.2 Relation source provenance

Every relation must trace to its semantic source, for example:

- architecture input/output family;
- root-use declaration;
- bound reference;
- authorization class;
- value-flow class;
- target-independent formula;
- state assignment;
- invariant-supporting relation;
- public event projection;
- constructibility requirement;
- lifecycle requirement;
- representation capability.

The compiler must not emit unnamed target checks that cannot be traced to a
semantic or target-safety requirement.

Target-only guards, such as canonical encoding checks, must carry separate
target provenance.

### 3.3 Obligation placement and carrying predicates

Every relation selected for target enforcement must be assigned to at least one
concrete carrying predicate.

A placement record should identify:

- relation ID;
- target program or constructor;
- operation/leaf role;
- input family or coordinator where relevant;
- whether enforcement is primary or deliberately duplicated;
- target proof method;
- source relation provenance.

An unplaced obligation is a compilation error.

A relation checked only by an off-chain vector or transaction builder is not
on-chain enforcement unless the normative contract explicitly assigns it
off-chain.

### 3.4 Positive evidence

Every target-enforced relation requires at least one valid vector that:

- reaches the carrying predicate;
- supplies a nontrivial valid witness;
- exercises the relation rather than bypassing it through an inactive branch;
- succeeds on the target;
- yields the expected semantic successor or public projection.

A positive vector should not be considered coverage merely because the target
program executed. The harness should record whether the relation's activation
condition was true.

### 3.5 Independent negative mutation

Every target-enforced relation requires at least one focused mutation that:

1. begins from an otherwise valid concrete transaction;
2. modifies the smallest practical fact set;
3. violates the selected relation;
4. preserves enough unrelated structure to reach the carrying predicate;
5. is rejected by target execution.

Examples:

- wrong output owner for recipient pin;
- wrong successor root;
- omitted canonical source;
- duplicated output reference;
- wrong floor quotient;
- wrong event projection;
- confidential closed-asset escape;
- hidden operator gate on a permissionless path;
- successor metadata under a different code subtree.

The mutation does not need to preserve every other relation perfectly if the
target cannot isolate them, but the report must state which other relations may
also be affected.

Claims of independent condition coverage must be honest.

### 3.6 Relation coverage matrix

The vector harness should produce a deterministic coverage matrix with a
conceptual shape:

```text
relation
× operation
× vector
× mutation
× carrying predicate
× proof method
× expected result
× actual result
```

The matrix must distinguish:

- relation activated;
- relation not activated;
- positive coverage;
- negative coverage;
- expected rejection;
- actual rejection;
- unsupported relation;
- missing vector;
- missing carrier.

Release requires no missing required cell under the selected coverage policy.

### 3.7 Coverage policy is versioned

The definition of sufficient relation coverage is itself a versioned compiler
or evidence configuration.

It should bind:

- required positive coverage;
- required negative coverage;
- activation criteria;
- accepted mutation independence;
- treatment of duplicated enforcement;
- required target execution environments;
- report schema.

Changing coverage policy may move report and compiler configuration identities
without moving architecture or realization identity.

### 3.8 Model-to-realization conformance evidence

Before target emission evidence is meaningful, the model must be checked
against the typed realization.

For each operation in scope:

- accepted model transitions satisfy realization relations;
- expected projections agree;
- relevant rejected model transitions violate named relations or guards;
- model property tests preserve invariants;
- relation dependencies agree with semantic observables.

This report is distinct from backend target execution.

### 3.9 Compiler-analysis evidence

Compiler tests must establish:

- every realization relation lowers;
- no relation is silently dropped;
- target requirements are complete;
- proof alternatives are valid for their relation classes;
- disclosures have provenance;
- permissionless witnesses are available;
- lifecycle paths exist;
- placement is complete;
- layout requirements are complete;
- analysis is deterministic.

These tests validate compiler analysis. They do not prove emitted programs.

### 3.10 Backend pattern evidence

Every reusable target pattern must have its own contract and tests.

A target pattern should identify:

- semantic/proof relation implemented;
- stack or target-program precondition;
- stack or target-program postcondition;
- failure behavior;
- resource formula;
- target capabilities required;
- canonical encoding;
- positive vectors;
- negative vectors.

Examples include:

- explicit asset check;
- explicit value check;
- 64-bit addition;
- exact quotient/remainder verifier;
- owner signature;
- relative timelock;
- output constructor equality;
- root successor authentication;
- commitment equality;
- confidential conservation bridge.

Pattern-level evidence supplements bundle-level translation validation.

### 3.11 Linker evidence

The linker must validate and report:

- every symbol/reference resolved;
- every relocation applied exactly once where required;
- no unresolved relocation remains;
- constructor reference graph;
- strongly connected components;
- selected reference strategy;
- deterministic taptree/program assembly;
- deployment constants bound;
- linked relation provenance preserved;
- bundle hash recomputable;
- no obligation lost during dead-code elimination or assembly;
- resource formulas associated with the final program.

Linking success alone is not semantic evidence. It is one necessary boundary.

### 3.12 Transaction and witness evidence

The transaction package must demonstrate:

- concrete transaction layout matches linked predicates;
- input/output family ranges are complete and disjoint;
- coordinator assumptions are satisfied;
- witness items use the declared ABI;
- metadata encoding is canonical;
- target representation requirements are met;
- selected leaf/control data are correct;
- transaction construction is deterministic where semantic randomness is not
  explicit;
- worst-case measurement fixtures are valid transactions.

A target program may be correct while the transaction builder supplies the
wrong witness. Both paths require evidence.

### 3.13 Target substrate evidence

Every target dependency required by the deployment profile must have a
separately named report containing:

- target identity;
- source/tool version;
- test name;
- nonzero report hash;
- verification status;
- tested claim;
- expected behavior;
- observed behavior.

Examples include:

- native asset conservation;
- issuance semantics and introspection;
- explicit-value introspection;
- confidential value conservation;
- value-commitment equality;
- authenticated opening if used;
- sighash behavior;
- relative timelocks;
- package relay;
- unspendable-output exclusion;
- constructor/weld enforcement;
- L-BTC settlement assumptions;
- script-emission fidelity.

A backend vector running on a mock interpreter does not replace a substrate
test against the exact pinned target.

### 3.14 Independent event/query/accounting evidence

Deployment release requires three independently committed observer reports:

1. **event projection**
   - recognized burn and clear events;
   - event identity and order;
   - block context;
   - ASH values;
   - record sequence;
   - clear values.

2. **attestation query**
   - context validation;
   - clear assignment;
   - grouping;
   - exact rational terms;
   - canonical bytes.

3. **receipt-accounting audit**
   - residue event sequence;
   - historical class totals;
   - current accounting projection.

The candidate implementations must be separately implemented from the model
path according to the deployment evidence policy.

One report cannot stand in for another.

### 3.15 Report identity binding

Every bundle-specific report must identify, where applicable:

- architecture semantic hash;
- architecture behavioural hash;
- realization identity;
- compiler configuration identity;
- target identity;
- linked-bundle hash;
- transaction ABI identity;
- vector-set identity;
- report schema and tool version.

A report produced for one bundle must not be attached to another merely because
the source architecture matches.

### 3.16 Deterministic report rendering

Given the same:

- typed inputs;
- target environment;
- vector set;
- bundle;
- tool version;
- explicit random seed where relevant,

reports must render byte-identically.

Reports must not contain uncontrolled:

- wall-clock timestamps;
- temporary paths;
- hostnames;
- process IDs;
- nondeterministic map order;
- unpinned environment data.

If execution duration is recorded, it must be excluded from canonical report
identity or normalized under an explicit policy.

### 3.17 Release fail-closed policy

The release package must reject:

- missing required report;
- zero report hash;
- report identity mismatch;
- stale report;
- incomplete relation coverage;
- unsupported required relation;
- expected/actual verdict mismatch;
- unresolved target dependency;
- calibration mismatch;
- bundle hash mismatch;
- unknown report schema;
- non-final deployment profile.

Release does not run a generator to repair evidence automatically.

---

## 4. Prohibited consequences

### 4.1 No claim of a verified compiler

Documentation, release records, and user-facing reports must not state that the
compiler is formally verified unless a separately scoped formal proof exists.

Permitted claims include:

```text
the released bundle passed relation-indexed translation validation
the arithmetic gadget passed the stated equivalence check
the backend pattern passed its target vectors
the deployment profile validated
```

Prohibited overclaims include:

```text
the compiler is proven correct
all possible target transactions are proven safe
the emitted script is mathematically equivalent to the model
```

unless the exact formal scope is established.

### 4.2 No compiler-source trust as release evidence

Compiler source review, code ownership, or a compiler binary hash does not
replace bundle-specific evidence.

The compiler binary/configuration hash is useful provenance. It is not proof of
correct translation.

### 4.3 No operation-level-only coverage

A release cannot claim complete translation evidence solely from one
successful and one failing test per operation.

Relations within the operation need explicit coverage accounting.

### 4.4 No mutation-count vanity metric

The number of vectors or mutations is not itself evidence of completeness.

Coverage must be tied to:

- relation identities;
- activation;
- carrying predicates;
- target verdicts.

Ten thousand random vectors may miss one absent root check.

### 4.5 No test-double independence claim

The following do not constitute separately implemented deployment evidence by
themselves:

- cloning a `ReferenceIndexer`;
- rebuilding it from its own checkpoint;
- wrapping it in another process;
- changing one result after calling the reference implementation;
- invoking the same model library from another crate;
- generating both expected and actual reports from one algorithm.

Such tests remain valuable for validating the differential harness. They are
not independent deployment implementations.

### 4.6 No silent waiver

A failed or missing relation may not be converted to:

- a warning;
- an `ignored` test;
- an empty report;
- a zero hash;
- a “known limitation” inside a final profile;
- an undocumented allowlist entry.

An explicit release waiver policy, if ever introduced, requires a separate
decision and deployment-profile representation. No such policy is accepted by
this decision.

### 4.7 No report conflation

Do not combine event, query, accounting, substrate, resource, and
script-integration evidence into one opaque “all tests passed” hash.

The claims have different failure modes and independence requirements.

An aggregate release manifest may reference them, but their individual reports
and hashes remain available.

### 4.8 No generated expected output from the candidate under test

The target backend being validated must not generate both:

- candidate output;
- expected semantic answer

through the same code path.

Expected semantic results should derive from the model/realization side.
Candidate target behavior derives from the emitted bundle and target executor.

Shared typed fixtures are allowed. Shared result algorithms must be identified
and not mislabeled as independent comparison.

### 4.9 No evidence reuse across incompatible identities

Reports must not be reused across:

- changed bundle bytes;
- changed relied-upon target substrate facts;
- changed transaction ABI;
- changed compiler proof policy;
- changed relation set;
- changed calibrated bounds;
- changed vector-set identity

unless the report schema explicitly proves the change is outside its claim and
the release policy permits such reuse.

Default behavior is to regenerate evidence.

### 4.10 No release from mocks alone

Mock targets and interpreters are useful for unit tests.

A deployment release requires execution and dependency evidence against the
exact pinned target environment or an explicitly accepted equivalent
environment whose equivalence is separately established.

### 4.11 No claim that finite vectors prove complete semantics

Translation-validation reports must state their finite scope.

Relation coverage improves reviewability and catches omissions. It does not
automatically prove:

- every possible witness;
- every transaction shape;
- every target-consensus edge case;
- every cryptographic assumption;
- every implementation-defined environment behavior.

Where a relation requires stronger assurance, add:

- formal equivalence proof;
- exhaustive bounded proof;
- property testing;
- target source audit;
- independent implementation;
- deployment monitoring

as a separately named evidence class.

---

## 5. Evidence architecture

### 5.1 Evidence layers

The project uses a layered evidence model.

| Layer | Subject | Typical evidence |
|---|---|---|
| Architecture | finite declarations and identities | validation, mutations, hash/version gates, publication equality |
| Realization | typed semantic relation | type checks, coverage, dependency derivation, model conformance |
| Model | abstract execution | unit tests, property tests, corruption fixtures, invariant checks |
| Compiler | analysis completeness | deterministic lowering, proof/disclosure/lifecycle/placement checks |
| Backend pattern | target proof primitive | contract tests, boundary vectors, optional formal equivalence |
| Linked bundle | exact release candidate | relation-indexed target vectors, constructor and relocation checks |
| Transaction ABI | concrete construction | layout/witness tests, valid and malformed transaction vectors |
| Target substrate | consensus/policy assumptions | integration tests against a supported node, with recorded test provenance |
| Independent observers | public deployment projection | separate event, query, and accounting implementations |
| Deployment release | complete evidence binding | final profile validation and domain-separated hash |

Passing one row never silently satisfies another.

### 5.2 Semantic vector definition

A backend-independent vector definition should conceptually include:

```rust
pub struct SemanticVector {
    pub id: VectorId,
    pub operation: architecture::OperationId,
    pub relation: RelationId,
    pub fixture: SemanticFixtureId,
    pub mutation: Option<MutationId>,
    pub expected: ExpectedSemanticResult,
}
```

> Illustrative API; not frozen.

The semantic vector identifies what is being tested, not how one backend
serializes it.

### 5.3 Concrete target vector

A target-specific materialization should conceptually include:

```rust
pub struct TargetVector<T> {
    pub semantic_vector: VectorId,
    pub target: TargetIdentity,
    pub bundle: BundleIdentity,
    pub transaction: T::Transaction,
    pub witness: T::Witness,
    pub expected_verdict: bool,
}
```

> Illustrative API; not frozen.

The vector harness executes the concrete target artifact and compares the
actual verdict and public projection.

### 5.4 Mutation provenance

Every mutation should record:

- source valid vector;
- intended violated relation;
- changed facts;
- expected target rejection;
- other relations potentially affected;
- shrink history if generated;
- deterministic mutation seed where relevant.

This prevents a mutation from being labeled independent merely because it has
a narrow name.

### 5.5 Target executor independence

The target executor should be as close as practical to the deployed semantics.

For the initial tapscript backend, preferred evidence includes:

- execution against a supported Elements regtest node (version recorded
  as test provenance);
- target-native transaction validation;
- upstream interpreter tests at the reviewed revision;
- optional secondary local interpreter for diagnostic comparison.

A local reimplementation can improve diagnostics but does not replace target
execution.

### 5.6 Differential result classes

Reports should distinguish at least:

```text
expected accept / actual accept
expected reject / actual reject
expected accept / actual reject
expected reject / actual accept
target execution failure
transaction construction failure
unsupported proof plan
resource failure
infrastructure failure
```

Only the first two are successful verdict matches.

A construction failure does not count as target rejection unless the test's
claim is specifically constructor-level validation.

### 5.7 Public projection comparison

For accepted vectors, verdict equality is insufficient.

The harness should compare target-derived public semantics where applicable:

- successor state fields;
- root identities;
- canonical deltas;
- event projections;
- output recipients/classes;
- chain fee;
- public disclosure;
- canonical query bytes;
- accounting projection.

A target transaction that is valid but produces the wrong semantic successor
is a translation failure.

### 5.8 Rejected-vector atomicity

For model transitions, rejection is structurally non-mutating.

For target transactions, consensus rejection means no state transition is
committed.

The harness must ensure rejected candidate transactions are not accidentally
followed by a state mutation in the test environment or fixture model.

### 5.9 Shrinking

When generated traces or transactions expose a mismatch, the harness should
attempt deterministic minimization.

Potential methods include:

- property-test shrinking on semantic actions;
- transaction-aware delta debugging;
- family-count reduction;
- witness simplification;
- metadata simplification;
- amount reduction while preserving the failing relation.

A shrunk vector supplements the original failure record. It must preserve
identity and mutation provenance.

---

## 6. Coverage policy

### 6.1 Minimum relation coverage

The initial minimum policy is:

For every required target-enforced relation:

- at least one activated accepting vector;
- at least one rejecting mutation;
- at least one carrying predicate;
- matching target verdicts;
- relation/source provenance;
- no unresolved target requirement.

For every representation alternative claimed as supported:

- at least one accepting vector using that representation;
- public semantic projection equal to the reference representation;
- negative vectors for prohibited representations or missing proof.

For every permissionless operation:

- at least one construction using public data plus sponsor-local data only;
- no owner/operator secret requirement;
- explicit failure if such a requirement appears.

### 6.2 Duplicate enforcement

A relation may be deliberately enforced in more than one target predicate.

The coverage report must identify:

- primary carrier;
- duplicated carriers;
- reason for duplication;
- positive reachability;
- negative behavior where practical.

Duplicated enforcement must not hide the absence of enforcement on a branch
where it is actually needed.

### 6.3 Conditional relations

A conditionally active relation requires vectors for:

- inactive valid branch;
- active valid branch;
- active invalid mutation.

Examples include:

- cycle issuance only when `Q > 0`;
- control/vault creation only for nonempty cycle principal;
- distribution residue only at terminal settlement;
- ASH residual only when clear leaves remainder;
- maturity conversion only at the selected cycle.

### 6.4 Cardinality boundaries

Every bounded family should include:

- minimum valid count;
- maximum calibrated valid count where feasible;
- zero/below-minimum invalid count;
- above-maximum invalid count;
- duplicate member;
- wrong-family member.

Full maximum vectors also feed resource calibration.

### 6.5 Arithmetic boundaries

Every arithmetic relation should include:

- zero where allowed;
- one;
- maximum safe domain values;
- exact division;
- nonzero remainder;
- quotient one below;
- quotient one above;
- divisor zero;
- overflow attempts;
- active-backing-cap boundaries where relevant.

### 6.6 Authorization boundaries

Every authorization relation should include:

- correct signer set;
- empty signer set;
- wrong signer;
- one missing signer in a multi-owner set;
- extra irrelevant signer where valid;
- permissionless path without signatures;
- hidden operator/owner gate mutation;
- cadence-band boundary cases.

### 6.7 Closure boundaries

Every object/asset closure relation should include:

- correct object family;
- wrong class;
- wrong explicit asset;
- malformed metadata;
- unclassified output;
- duplicate output family;
- omitted family;
- closed-asset exfiltration attempt;
- foreign/open decoy.

---

## 7. Alternatives considered

### 7.1 Trust compiler implementation and source review

#### Proposal

Accept the emitted bundle when:

- compiler source has been reviewed;
- compiler unit tests pass;
- architecture and model tests pass;
- bundle hash is recorded.

#### Advantages

- much less evidence machinery;
- faster implementation;
- conventional compiler trust model;
- fewer vector/report artifacts.

#### Rejection

The compiler is a new, security-critical translator across several semantic
boundaries. Source review and unit tests do not prove one released artifact is
complete or correctly linked.

This approach also weakens the deployment profile's explicit evidence model.

Bundle-specific translation validation provides stronger, more reviewable
assurance without requiring a fully verified compiler.

### 7.2 Build a formally verified compiler before deployment

#### Proposal

Formalize the entire language and target stack, prove compiler correctness, and
release only afterward.

#### Advantages

- strongest possible central compiler assurance;
- potentially smaller recurring vector burden;
- clear mathematical theorem.

#### Rejection for the initial roadmap

The work is too large and includes target and deployment semantics outside the
compiler itself.

It would delay target feedback and deployment indefinitely.

Formal methods remain welcome for bounded, load-bearing components and may
eventually supersede parts of the translation-validation burden.

### 7.3 Differential-test only at operation level

#### Proposal

For each operation, generate a collection of valid and invalid transactions
and compare model and target verdicts.

#### Advantages

- straightforward;
- catches many practical bugs;
- smaller report format;
- easy to understand.

#### Rejection as the complete policy

Operation-level tests do not reveal which relation lacks coverage.

A broad successful operation vector can bypass inactive or omitted relations.
Relation-indexed coverage is required for reviewable completeness.

Operation-level scenario tests remain useful in addition to relation coverage.

### 7.4 Random fuzzing as the main assurance model

#### Proposal

Generate large numbers of random model and target transactions and compare
results.

#### Advantages

- explores unexpected interactions;
- often finds bugs not anticipated by reviewers;
- integrates naturally with property testing;
- can shrink failures.

#### Rejection as the sole assurance model

Random generation gives no deterministic guarantee that each relation was
exercised independently.

The accepted approach combines:

- deterministic relation coverage;
- focused mutations;
- generated property traces;
- fuzzing where useful.

### 7.5 Prove only reusable backend patterns

#### Proposal

Verify each arithmetic, signature, introspection, and constructor pattern
independently, then trust composition.

#### Advantages

- finite reusable proof surface;
- avoids per-bundle vector explosion;
- strong local reasoning.

#### Rejection as the complete policy

Composition can fail through:

- wrong relation-to-pattern mapping;
- omitted pattern;
- wrong layout;
- wrong placement;
- relocation error;
- stack interaction;
- transaction ABI mismatch;
- target configuration mismatch.

Pattern evidence is required, but final bundle evidence remains necessary.

### 7.6 Compare emitted bytes with a golden bundle

#### Proposal

Store an expected script bundle and require compiler output to match it exactly.

#### Advantages

- deterministic;
- simple CI;
- catches any emitted-byte change;
- easy release pin.

#### Rejection as semantic assurance

Byte equality proves only equality with one golden artifact. It does not prove
the golden artifact correctly realizes the model.

It also prevents legitimate backend improvements without explaining semantic
equivalence.

Golden bundle hashes remain useful release identities after semantic evidence
validates them.

### 7.7 Use two compilers and compare outputs

#### Proposal

Implement two independent compilers and compare their target output or
behavior.

#### Advantages

- catches correlated implementation errors;
- strong diversity;
- resembles independent indexer evidence.

#### Rejection for the initial roadmap

Two compilers are substantially more work and may still share one semantic
misunderstanding.

A future independent compiler would be valuable additional evidence. It does
not remove the need for realization/model and target validation.

### 7.8 Allow release waivers

#### Proposal

Permit explicitly documented known failures in a final deployment profile.

#### Advantages

- practical release flexibility;
- allows partial evidence;
- prevents one noncritical test from blocking progress.

#### Rejection under this decision

The current deployment profile and conformance claims do not define waiver
semantics.

Introducing waivers would require:

- typed waiver records;
- scope and expiration;
- severity classification;
- consumer visibility;
- hash binding;
- normative compatibility;
- separate decision.

Until then, missing required evidence blocks release.

---

## 8. Assurance limitations

### 8.1 Finite evidence is not universal proof

Relation-indexed vectors are finite.

They reduce the chance of omitted or incorrectly implemented relations and make
coverage inspectable. They do not universally quantify over every:

- value;
- witness;
- transaction order;
- confidential blinding;
- target consensus state;
- reorganization;
- external environment.

Property testing and formal checks can strengthen specific relations.

### 8.2 Relation vocabulary completeness is trusted

Translation validation assumes the typed realization relation set adequately
captures the intended semantic contract.

If the realization omits a requirement, perfect relation coverage cannot test
it.

Mitigations include:

- normative review;
- architecture/realization conformance;
- model comparison;
- invariant and mutation suites;
- independent public observers;
- second-backend review;
- target incident analysis.

### 8.3 Vector materialization can be wrong

The semantic vector may be correct while target transaction construction
materializes the wrong mutation.

Mitigations include:

- typed transaction ABI;
- transaction constructor tests;
- mutation provenance;
- independent target inspection;
- public projection comparison;
- focused manual vectors for critical relations.

### 8.4 Target executor is trusted

Executing against the exact target reduces interpreter divergence but still
depends on:

- target software;
- target configuration;
- recorded test provenance;
- build correctness;
- test harness interaction;
- consensus activation.

These dependencies must be named and evidenced.

### 8.5 Cryptographic assumptions remain assumptions

Tests cannot prove cryptographic hardness or target commitment binding.

Deployment evidence names the cryptographic and consensus assumptions relied
upon.

### 8.6 Independent implementations can still correlate

Independent indexers or auditors may share:

- documentation;
- test fixtures;
- libraries;
- conceptual misunderstandings.

The deployment evidence should document independence boundaries rather than
using the word “independent” without qualification.

### 8.7 Evidence maintenance cost

Per-bundle evidence increases:

- CI runtime;
- artifact count;
- schema maintenance;
- release complexity;
- storage requirements;
- review burden.

That cost is intentional for a security-critical compiler. Reports should
remain structured, deterministic, and focused to keep the burden manageable.

---

## 9. Determinism and identity consequences

### 9.1 Vector-set identity

The vector set must have a deterministic identity over:

- semantic vector definitions;
- mutation definitions;
- coverage policy;
- fixture identities;
- canonical ordering;
- representation cases;
- expected semantic projections.

Changing the vector set moves its identity even if all bundles still pass.

### 9.2 Report identity

A report identity should bind:

- report schema;
- tool version;
- vector-set identity;
- relevant semantic identities;
- target identity;
- bundle identity;
- ABI identity;
- canonical results.

It should exclude nonsemantic presentation details and uncontrolled timing.

### 9.3 Compiler binary identity

A release may record:

- compiler source revision;
- compiler binary hash;
- compiler configuration identity.

These provide provenance and reproducibility. They do not substitute for
translation evidence.

### 9.4 Seeded generation

Property and fuzz evidence must record explicit deterministic seeds when
reproducibility is required.

Novel random exploration may run outside canonical release reports. Any failure
promoted into release regression evidence receives a stable fixture and
identity.

### 9.5 Result ordering

Reports use canonical ordering by stable IDs:

```text
operation
relation
vector
mutation
target
carrier
```

Do not order by execution completion time or filesystem enumeration.

### 9.6 Environment identity

Target execution reports should bind relevant environment facts:

- target software revision;
- target network mode;
- activation state;
- build configuration;
- test runner version.

Host-specific irrelevant details should not change canonical report bytes.

---

## 10. Implementation and migration

### 10.1 Phase 1: relation-ready realization

The realization package must create stable relation and provenance structures
for the pilot operations.

Model-conformance tests establish the first expected semantic relation set.

No backend report exists yet.

### 10.2 Phase 2: compiler coverage skeleton

The compiler should produce, for each pilot relation:

- lowered relation ID;
- proof alternatives;
- disclosure dependencies;
- target requirements;
- placement requirement;
- vector obligations.

At this phase, the coverage matrix may show target vectors as unmaterialized.
It must not omit relations.

### 10.3 Phase 3: pattern-level target tests

The target and tapscript packages add:

- target capability evidence;
- pattern contracts;
- positive/negative pattern vectors;
- resource formulas;
- constructor and arithmetic prototypes.

Prototype reports remain research evidence until accepted and integrated.

### 10.4 Phase 4: first complete bundle report

`compact-ash` becomes the first complete per-bundle translation-validation
subject.

The report should cover:

- every realization relation;
- target carriers;
- positive vectors;
- rejecting mutations;
- transaction layout;
- permissionless construction;
- resource use;
- exact target identity;
- linked-bundle identity.

This report is the template for later operations.

### 10.5 Incremental operation coverage

Each roadmap operation phase adds:

1. realization/model conformance;
2. compiler-analysis coverage;
3. backend pattern coverage;
4. bundle relation coverage;
5. transaction ABI vectors;
6. target dependency evidence;
7. resource measurements;
8. release report updates.

Do not defer all evidence integration to Phase 12.

### 10.6 Report schemas

Report schemas should be added only when their typed producer and consumer
roles are clear.

Each report schema must have:

- typed source;
- schema version;
- deterministic serialization;
- unknown-field rejection;
- canonical ordering;
- generator;
- non-writing checker where committed;
- identity binding;
- migration policy.

### 10.7 Release package

The release package should accept typed report values or validated typed report
envelopes.

It must check cross-report identity consistency before producing a final
profile.

### 10.8 Root ADR promotion

Once the vector/release architecture is implemented and governs releases,
promote this decision to an ADR.

The ADR should state the exact implemented evidence classes and commands rather
than retaining provisional API sketches.

---

## 11. Risks

### 11.1 False confidence from green matrices

A complete matrix can appear stronger than it is if:

- relation vocabulary is incomplete;
- vectors do not reach carriers;
- mutations break unrelated earlier checks;
- target execution uses a mock;
- expected projections are generated by the candidate.

Mitigation:

- activation tracking;
- mutation provenance;
- independent manual review;
- target-native execution;
- source provenance;
- public projection comparison;
- explicit report limitations.

### 11.2 Evidence generated by the same bug

The compiler may emit an incorrect relation and generate vectors matching its
incorrect interpretation.

Mitigation:

- vectors originate from realization/model relation IDs;
- expected semantic results derive from model/realization;
- target transaction materialization is separated;
- critical vectors are reviewed and retained as fixed fixtures;
- independent observers remain required.

### 11.3 Excessive test cost

Full maximum-bound transactions and target execution can be expensive.

Mitigation:

- tiered test lanes;
- deterministic caching by exact identity;
- focused per-operation development tests;
- complete release lane;
- retain full relation coverage even if not every vector runs in every local
  edit loop.

### 11.4 Brittle mutation independence

One transaction mutation may violate several relations.

Mitigation:

- report affected fact set;
- distinguish intended relation from collateral relations;
- design canonical layouts for local mutation where possible;
- use multiple mutations for especially important relations;
- do not overclaim strict MC/DC where the target shape prevents isolation.

### 11.5 Report schema proliferation

Many evidence classes can produce too many schemas and tools.

Mitigation:

- shared typed envelope conventions;
- one report per genuinely distinct claim;
- avoid separate files for trivial internal details;
- aggregate indexes may reference individual claim reports;
- maintain separate hashes where independence matters.

### 11.6 Release bottleneck

A strict release gate can expose failures late if evidence is added only at the
end.

Mitigation:

- evidence developed operation by operation;
- Phase gates require relation coverage;
- release package APIs drafted before final phase;
- independent implementation work proceeds in parallel.

### 11.7 Formal-method scope creep

A successful gadget proof may lead to attempts to formalize the entire target
before deployment.

Mitigation:

- formalize bounded load-bearing components where cost-effective;
- preserve the active translation-validation roadmap;
- require a roadmap decision before formal work blocks delivery.

---

## 12. Supersession conditions

This decision may be superseded if the project obtains a formally verified
translation chain whose proof covers:

- realization semantics;
- compiler lowering;
- proof-plan selection;
- backend emission;
- linking;
- transaction assembly;
- relevant target semantics.

A replacement decision must state which translation-validation obligations are:

- removed because they are formally subsumed;
- retained as regression tests;
- retained because they cover deployment assumptions outside the proof;
- retained as independent public-observer evidence.

This decision may also be refined if:

- an independently implemented second compiler becomes release-required;
- proof-carrying code replaces some relation vectors;
- exhaustive bounded target verification becomes practical;
- a future Simplicity backend provides stronger formal target semantics;
- deployment monitoring adds post-release evidence.

It is not superseded merely because:

- some backend patterns are formally verified;
- arithmetic gadgets gain SMT or proof-assistant evidence;
- two backends agree;
- compiler source is audited;
- the vector suite becomes very large;
- one release passes all reports.

---

## 13. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/deployment.rs`](../../packages/architecture/src/deployment.rs)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/model/src/lib.rs`](../../packages/model/src/lib.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/ledger.rs`](../../packages/model/src/ledger.rs)
- [`../../packages/model/src/audit.rs`](../../packages/model/src/audit.rs)
- [`../../packages/model/src/tests/indexer_conformance_tests.rs`](../../packages/model/src/tests/indexer_conformance_tests.rs)
- [`../../packages/model/src/tests/residue_noninterference_tests.rs`](../../packages/model/src/tests/residue_noninterference_tests.rs)

### Related decisions

- [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
- [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md)
- [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md)
- [D005: Value-Parametric, Closed-Asset-Identity-Rigid](005-value-parametric-asset-rigid.md)
- [D006: Canonical Transaction-Layout ABI](006-canonical-transaction-layout-abi.md)

### Package plans

- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)
- [`../backlog.md`](../backlog.md)

---

## 14. Decision summary

> Do not initially claim or rely on a formally verified compiler. For every
> candidate release, validate the exact linked bundle against the typed
> realization and executable model relation by relation; bind positive vectors,
> focused rejecting mutations, carrying predicates, concrete target execution,
> transaction ABI checks, resource measurements, substrate reports, and
> separately implemented event/query/accounting reports into the final
> deployment profile; and fail release whenever required evidence is missing,
> mismatched, stale, or incomplete.
