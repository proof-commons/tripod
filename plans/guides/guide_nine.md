# Guide 9 — Target-Native Primitive Conformance and Typed Tapscript Instruction Core

## Mission

Complete the second half of the Phase-3 target foundation:

```text
reviewed static Elements target contract
    ↓
typed tapscript instruction encoding
    ↓
typed success/failure stack analysis
    ↓
secretless target-native execution
    ↓
typed primitive-conformance reports
```

Guide 9 must establish, for every primitive admitted by the reviewed Elements
target contract:

1. the target byte is correct;
2. the primitive executes in the declared domain;
3. every valid success form has the declared stack effect;
4. every declared failure class has the declared target effect;
5. operand and result encodings agree with the target;
6. resource accounting agrees with the target;
7. required deployment evidence is produced or remains explicitly unresolved;
8. no attestation-contract operation semantics are emitted yet.

Guide 9 also establishes the first typed tapscript instruction representation,
serializer, parser for the supported subset, and abstract stack validator.

The guide must not emit `compact-ash`, link a bundle, define an attestation-contract
transaction ABI, calibrate architecture bounds, or claim production target
activation.

At completion, the project should possess an honest typed answer to:

> Do the target primitives and encodings described by the static Elements
> contract behave that way under an actual Elements interpreter, and can the
> backend represent and reason about their exact instruction and stack effects
> without weakening or guessing?

---

# 1. Executive rulings

Guide 9 adopts the following rulings before implementation begins.

## 1.1 Repair the target contract before consuming it operationally

The typed instruction core and target-native harness must not be built on a
contract whose successful stack effects are incomplete, whose redundant
subcontracts can contradict one another, or whose reviewed trust state can be
asserted by an arbitrary caller.

The Guide-9 preflight therefore closes these target-foundation defects first:

1. every valid primitive success form becomes representable;
2. opcode, authorization, encoding, capability, confidential-value, issuance,
   resource, and evidence subcontracts are bidirectionally welded;
3. target capability status is closed transitively over prerequisites;
4. encoding interpretation is independent of whether a byte order happened to
   be supplied;
5. a generic validated target is distinct from the first-party reviewed
   Elements contract;
6. static target assessment is distinct from deployment-aware assessment;
7. the compiler’s external-evidence-role census survives the tapscript adapter;
8. architecture identities require validated architecture values;
9. compare-if-changed publication repairs mode-only corruption;
10. digest-policy exceptions or migrations are explicit.

Guide 9 must not hide one of these repairs inside the instruction-core work.
Each repair receives focused mutation coverage before the next layer consumes
it.

## 1.2 “Validated” and “reviewed” are different trust states

A target assembled by a caller may be internally well-formed without being the
reviewed Elements target.

Use distinct states:

```text
TargetDefinition
    unvalidated declaration

ValidatedTargetDefinition
    internally coherent declaration

ReviewedElementsTapscriptDefinition
    exact first-party reviewed Elements contract
```

Only the reviewed state may satisfy a boundary whose claim is:

```text
this is the reviewed Elements tapscript contract
```

A custom validated target may remain useful for mutation tests and future
alternative targets. It must not silently acquire reviewed provenance.

## 1.3 Static target contract, deployment declaration, and evidence remain separate

These are three different values:

```text
static target contract
    what semantics the project has reviewed

development deployment binding
    what environment the caller intends to exercise

target-native evidence report
    what an actual executor observed
```

Passing one does not pass another.

In particular:

```text
Reviewed capability
    ≠ deployment active

Activation declared
    ≠ activation observed

Native report passed
    ≠ production support

Static assessment passed
    ≠ resource-feasible backend operation
```

## 1.4 Primitive success is a relation, not one output vector

Several primitives have more than one valid successful stack result.

Examples include:

```text
input issuance:
    issuance present
    issuance absent

value inspection:
    explicit value
    confidential value

nonce inspection:
    explicit nonce
    confidential nonce
    null nonce

program inspection:
    witness program
    non-witness script digest

relative timelock:
    operand retained on success
```

The target contract must represent these as typed alternatives or retained-stack
relations. A comment describing an alternative the type cannot represent is not
a contract.

## 1.5 Failure behavior is part of semantics

The target does not fail uniformly.

Guide 9 preserves at least:

```text
abort evaluation

consume operands and push false

retain operands and push false
```

The typed instruction validator must not normalize these to one “failure”
state. Stack depth and surviving values differ exactly on the path a backend
must handle safely.

## 1.6 Native evidence must execute the target implementation

Target-native evidence means execution through the selected Elements
interpreter or node implementation.

It does not mean:

- running the first-party abstract stack validator;
- invoking the same Rust function twice;
- comparing one target-contract DTO with itself;
- a mocked executor;
- a local reimplementation of the opcode;
- parsing the upstream source and treating the parse as execution.

Mocks are allowed for protocol and failure-path tests of the harness. They
cannot satisfy the target-native evidence gate.

## 1.7 The external executor boundary remains secretless

The first-party native-conformance package accepts public test data and an
explicit executable capability.

It does not accept:

- RPC usernames;
- RPC passwords;
- bearer tokens;
- cookie paths;
- private keys;
- signing nonces;
- production blinding factors;
- private openings;
- wallet paths;
- production endpoints.

If the selected external executor communicates with a node, it owns that
communication outside the first-party interface. It runs in an externally
established secretless development environment.

The repository does not authenticate or sandbox the executor.

## 1.8 No target, report, or instruction digest is minted

Guide 9 introduces no:

```text
TargetDefinitionHash
TapscriptProgramHash
InstructionSetHash
NativeReportHash
FixtureSetHash
ExecutorHash
DeploymentHash
```

Direct typed comparison is sufficient for the new in-process contracts.

Native reports may be explicit build assets, but no persistent report identity
is admitted until a real cross-process release consumer exists and ADR-016’s
identity admission conditions are satisfied.

## 1.9 No attestation-contract operation emission

Primitive fixture scripts may contain reviewed target instructions.

They must not encode:

- `compact-ash`;
- receipt transfer;
- STATE succession;
- burn;
- clear;
- redemption;
- settlement;
- cycle;
- request admission;
- any other attestation-contract operation.

Guide 9 proves the instruction and primitive substrate. Operation patterns begin
only after this guide passes.

---

# 2. Entry conditions

Guide 9 begins only after the Guide-8 target foundation has passed and the
second target-boundary review findings have been accepted into the active
queue.

Expected entry state:

```text
Phase 3:
    active

target-elements:
    typed static contract implemented
    reviewed primitive registry implemented
    development binding implemented
    target-native evidence absent

tapscript:
    compiler capability adapter implemented
    instruction core absent
    backend patterns absent

compiler:
    abstract target-requirement boundary public
    target-independent
    complete analyzed-program internals private
```

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean.

The implementation branch must identify the exact Guide-8 gate commit and the
exact second-review finding register it is closing.

If any high-severity preflight finding remains open, target-native evidence may
be prototyped experimentally but must not be recorded as satisfying the final
Guide-9 gate.

---

# 3. Required reading and authority

Read these repository-policy owners first:

```text
AGENTS.md

adr/010-command-line-output-contract.md
adr/011-toolchain-and-dependency-policy.md
adr/014-meson-lint-census-and-stamps.md
adr/015-public-data-and-execution-trust.md
adr/016-semantic-identities-and-evidence-binding.md
adr/017-path-scope-and-host-filesystem-trust.md
```

Read these accepted implementation decisions:

```text
plans/decisions/001-typed-rust-source.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/008-exact-certified-mathematics.md
```

Read these active package and phase contracts:

```text
plans/packages/compiler.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/phases/03-target-foundation.md
plans/reference/elements-tapscript.md
```

Read the implemented target and adapter source:

```text
packages/compiler/src/target.rs

packages/target-elements/src/authorization.rs
packages/target-elements/src/capability.rs
packages/target-elements/src/confidential.rs
packages/target-elements/src/definition.rs
packages/target-elements/src/deployment.rs
packages/target-elements/src/encoding.rs
packages/target-elements/src/evidence.rs
packages/target-elements/src/evidence_registry.rs
packages/target-elements/src/opcode.rs
packages/target-elements/src/resource.rs

packages/tapscript/src/capability.rs
packages/tapscript/src/error.rs
packages/tapscript/src/lib.rs
```

The human target reference is review support only.

No first-party package may parse planning or reference Markdown as target
semantics.

---

# 4. Scope

## 4.1 In scope

Guide 9 implements:

- second-review target-contract repairs;
- reviewed-target trust-state separation;
- complete primitive success-stack contracts;
- exact cross-subcontract weld validation;
- transitive target-capability status validation;
- noncircular encoding semantics;
- explicit static versus deployment assessment boundaries;
- external-evidence-role assessment;
- validated architecture identity input;
- mode-aware publication repair;
- typed tapscript instruction values;
- exact instruction serialization;
- parsing of the supported instruction subset;
- typed stack items;
- abstract stack-state execution over success and failure alternatives;
- explicit work limits for stack-state expansion;
- generic target-native primitive fixtures;
- a secretless executor protocol;
- a first-party native-conformance harness package;
- target-native execution of every reviewed opcode;
- target-native encoding tests;
- signature and sighash mutation matrices;
- relative-timelock boundary tests;
- confidential-value and issuance target tests;
- primitive resource observations;
- typed native-conformance reports;
- exact report census validation;
- target and adapter documentation updates;
- the Guide-9 gate record.

## 4.2 Out of scope

Guide 9 must not implement:

- an attestation-contract target program;
- backend proof patterns;
- relation placement;
- stack scheduling for attestation-contract operations;
- STATE constructors;
- wide floor arithmetic;
- public-opening proofs;
- relocatable bundles;
- linking;
- taptrees;
- control blocks for protocol operations;
- transaction ABI;
- protocol transaction construction;
- relation-indexed operation vectors;
- resource calibration of architecture bounds;
- production target binding;
- production node access;
- release evidence identity;
- release publication;
- production signing or wallet support.

---

# 5. Package boundaries

## 5.1 `tripod-target-elements`

The static target package remains the owner of:

- execution domain;
- leaf version;
- opcode IDs and bytes;
- operand, result, success, and failure contracts;
- encoding classes;
- authorization and timelock contracts;
- confidential-value and issuance contracts;
- resource contracts;
- capability prerequisites and statuses;
- evidence requirements;
- generic target-definition validation;
- reviewed Elements target construction;
- development binding declarations.

It remains dependency-free.

Guide 9 must not add regular or development dependencies to
`tripod-target-elements`.

The package’s tests may use standard-library-only fixtures. External execution
belongs elsewhere.

## 5.2 `tripod-tapscript`

The tapscript package owns:

- typed instruction values;
- typed stack items;
- instruction serialization and parsing;
- target-contract-driven opcode lookup;
- abstract instruction-sequence stack validation;
- instruction-level resource projection;
- compiler-to-static-target assessment;
- deployment-aware assessment only if explicitly implemented as a distinct API.

Expected direct first-party dependencies remain:

```text
compiler
target-elements
```

No new third-party dependency is expected for the instruction core.

It must not depend on:

```text
architecture
realization
model
linker
transaction
vectors
release
artifacts
```

## 5.3 `tripod-target-elements-conformance`

Create a separate public-data conformance package:

```text
packages/target-elements-conformance/
    Cargo.toml
    README.md
    meson.build
    src/lib.rs
    src/protocol.rs
    src/fixture.rs
    src/report.rs
    src/validate.rs
    src/executor.rs
    src/bin/check-target-elements-native.rs
    src/tests/...
    tests/subprocess_contract.rs
```

Cargo identities:

```toml
[package]
name = "tripod-target-elements-conformance"

[lib]
name = "target_elements_conformance"
```

Allowed direct first-party dependencies:

```text
target-elements
tapscript
cli-common
```

Expected generic dependencies:

```text
clap
serde
serde_json
thiserror
tempfile for focused tests
```

Add no hashing dependency. No report digest is produced.

Forbidden dependencies:

```text
architecture
realization
model
compiler
linker
transaction
vectors
release
artifacts
labels
```

The conformance package owns execution protocol and evidence mechanics, not
target semantics.

## 5.4 Workspace and Meson

Update:

```text
Cargo.toml
packages/meson.build
meson.build
```

Add every new source file to the nearest explicit `meson.build` census in the
same commit.

Add the package README to the DOC census.

Integration-test source remains an explicit same-typed census exclusion if it
does not participate in the label graph.

The ordinary repository build must not require an external native executor.
The Guide-9 native lane is explicit and separately configured.

---

# 6. Preflight target-contract repair

## 6.1 Separate generic validation from reviewed Elements

Introduce:

```rust
pub struct ValidatedTargetDefinition {
    definition: TargetDefinition,
}

pub struct ReviewedElementsTapscriptDefinition {
    definition: ValidatedTargetDefinition,
}
```

The generic validator proves internal coherence only:

```rust
pub fn validate_target_definition(
    definition: TargetDefinition,
) -> Result<ValidatedTargetDefinition, Vec<TargetError>>;
```

The reviewed constructor proves exact first-party contract identity by
construction or exact typed comparison:

```rust
pub fn reviewed_elements_tapscript(
) -> Result<ReviewedElementsTapscriptDefinition, Vec<TargetError>>;
```

If a generic validated definition can be promoted, promotion requires exact
equality with the independently derived reviewed value:

```rust
pub fn validate_as_reviewed_elements(
    offered: ValidatedTargetDefinition,
) -> Result<ReviewedElementsTapscriptDefinition, TargetError>;
```

A generic validated definition must not satisfy an API requiring the reviewed
Elements type.

The reviewed wrapper is a type-level trust state, not a digest.

## 6.2 Represent complete successful execution

Replace the single successful result vector with a typed success contract.

A suitable conceptual form is:

```rust
pub enum SuccessContract {
    Fixed {
        consumed_operands: usize,
        results: Vec<StackValueType>,
    },

    RetainsOperands {
        results: Vec<StackValueType>,
    },

    Alternatives {
        cases: Vec<SuccessCase>,
    },
}

pub struct SuccessCase {
    condition: SuccessCondition,
    stack_effect: SuccessStackEffect,
}
```

Possible conditions include:

```rust
pub enum SuccessCondition {
    Always,
    ExplicitEncoding,
    ConfidentialEncoding,
    NullEncoding,
    WitnessProgram,
    NonWitnessProgram,
    IssuancePresent,
    IssuanceAbsent,
}
```

Equivalent typed factoring is acceptable.

Every successful form must state:

- operands consumed or retained;
- results pushed;
- exact result types;
- resulting stack depth;
- condition selecting the form.

## 6.3 Required success alternatives

At minimum, represent these exact alternatives.

### Input value inspection

```text
explicit:
    explicit payload
    explicit prefix

confidential:
    confidential payload
    confidential prefix
```

### Output value inspection

Same two alternatives.

### Input/output asset inspection

Represent the explicit and confidential asset payload/prefix forms distinctly.

### Output nonce inspection

```text
explicit nonce
confidential nonce
null nonce
```

### Input program inspection

```text
witness program + version
non-witness script digest + negative marker
```

### Output program inspection

Same alternatives.

### Input issuance inspection

```text
issuance present:
    asset amount payload/prefix
    inflation-keys amount payload/prefix
    entropy
    blinding nonce

issuance absent:
    one null marker
```

### Relative timelock

Success retains the inspected operand and pushes nothing.

## 6.4 Cross-subcontract welds

Add explicit validators for every repeated semantic claim.

### Signature weld

Require agreement among:

- `CheckSig`;
- `CheckSigVerify`;
- stack-message signature variants;
- `SignaturePrimitiveContract`;
- signature/public-key encodings;
- per-check validation budget;
- evidence requirements.

Check separately:

```text
empty signature
invalid nonempty signature
invalid key
unknown key type
budget exhaustion
```

### Timelock weld

Require agreement among:

- `CheckSequenceVerify`;
- `RelativeTimelockContract`;
- sequence encoding;
- transaction-version prerequisite;
- block/time modes;
- unsatisfied behavior;
- evidence requirements.

### Issuance weld

Require agreement among:

- `IssuanceField::ALL`;
- `IssuanceContract::fields`;
- `InspectInputIssuance`;
- outpoint issuance flags;
- null issuance marker;
- entropy and blinding-nonce encodings;
- issuance capabilities;
- reissuance capability;
- evidence requirements.

### Confidential-value weld

Require agreement among:

- confidential-value claim states;
- target capability statuses;
- value encoding classes;
- value introspection primitives;
- CT conservation evidence;
- commitment-equality evidence;
- authenticated-opening status.

### Resource weld

Require agreement among:

- opcode resource costs;
- signature/curve per-check budget;
- validation-budget offset;
- consensus resource dimensions;
- policy resource dimensions;
- resource capability statuses;
- resource evidence requirements.

### Evidence weld

Every target subcontract carrying an evidence set must:

- name only declared evidence requirements;
- carry every required evidence role;
- reject an empty set where the claim requires deployment evidence.

## 6.5 Capability prerequisite status closure

Define one total status-composition rule.

Required minimum rule:

\[\operatorname{status}(c)=\mathrm{Reviewed}\Longrightarrow\forall p\in\operatorname{prereq}^{+}(c),\operatorname{status}(p)=\mathrm{Reviewed}\]

Recommended total ordering:

```text
Unsupported < Incomplete < Reviewed
```

A capability cannot have a stronger status than its weakest transitive
prerequisite.

Validation reports:

- reviewed capability with incomplete prerequisite;
- reviewed capability with unsupported prerequisite;
- incomplete capability with unsupported prerequisite where policy requires
  propagation;
- prerequisite cycle;
- unknown prerequisite.

The tapscript adapter rechecks the complete prerequisite closure defensively.

## 6.6 Independent encoding interpretation

Do not derive numericity from byte-order presence.

Use:

```rust
pub enum PayloadInterpretation {
    Opaque,
    SignedInteger,
    UnsignedInteger,
}
```

or derive interpretation through an exhaustive match over `EncodingClass`.

Validation then enforces:

```text
numeric class:
    byte order required

opaque class:
    byte order forbidden
```

For reviewed V1, also enforce class-specific:

- domain;
- width;
- prefix set;
- interpretation;
- byte order;
- canonicality;
- unknown-prefix rule.

A generic target may vary these only under a different explicitly supported
contract version or typed target family.

## 6.7 Validated architecture identity

Introduce a validated architecture identity boundary before any new target work
records architecture identities.

Use separate checked wrappers where useful:

```text
ValidatedDraftArchitecture
ValidatedReleaseArchitecture
```

Public architecture semantic hashing and publication construction should accept
the checked value.

Unchecked canonical projection stays crate-private for mutation tests.

## 6.8 Publication mode as part of freshness

Compare-if-changed publication equality means:

```text
bytes equal
and
required mode equal
```

Repair:

- public reports to `0644`;
- generated public assets to `0644`;
- synced PDF/flattened outputs to `0644`;
- synced helper binaries to `0755`.

Mode-only repair must not alter bytes.

A second run after repair must be a no-op.

## 6.9 Digest-policy resolution

Do not silently change an existing digest recipe.

For architecture semantic and anchor-set hashes, choose one:

1. record a reviewed grandfathered exception to ADR-016’s domain-separated
   semantic identity form; or
2. introduce a new recipe identifier and explicit migration.

Guide 9 should not move existing hashes unless the migration is separately
reviewed and every consumer is updated in one implementation series.

---

# 7. Static and deployment target assessments

## 7.1 Static assessment

The current compiler capability adapter fundamentally answers a static
question:

> Given the reviewed target contract, what primitive, structural, pattern, or
> evidence obligations remain?

Make that API explicit:

```rust
pub fn assess_static_capability(
    target: &ReviewedElementsTapscriptDefinition,
    required: RequiredCapability,
) -> StaticCapabilityAssessment;
```

The static assessment does not read network, activation, or policy overrides.

Its name and input type must make that boundary obvious.

## 7.2 Deployment-aware assessment

Add a deployment-aware assessment only if Guide 9 has a concrete consumer for
it.

Conceptual form:

```rust
pub fn assess_development_capability(
    target: &ElementsTarget,
    required: RequiredCapability,
    evidence: &TargetEvidenceReport,
) -> DevelopmentCapabilityAssessment;
```

It may distinguish:

```text
static primitive unsupported
static primitive incomplete
execution domain not declared active
activation not evidenced
development policy blocks
required target evidence missing
required target evidence failed
statically available, backend pattern still absent
```

If no concrete consumer exists in Guide 9, defer this function and narrow the
existing API to static assessment instead of accepting and ignoring an
`ElementsTarget`.

## 7.3 Compiler external-evidence roles

Preserve exact compiler evidence roles through the adapter.

Use an assessment set such as:

```rust
pub struct TargetAssessmentSet {
    capabilities:
        BTreeMap<RequiredCapability, StaticCapabilityAssessment>,

    external_evidence:
        BTreeMap<ExternalEvidenceRole, ExternalEvidenceAssessment>,
}
```

The adapter handles `ExternalEvidenceRole::ALL` exhaustively.

For the current role:

```text
SubstrateConservation
    → target whole-transaction conservation evidence
```

Require exact census equality in both directions:

```text
compiler required capabilities
=
adapter capability assessments

compiler external evidence roles
=
adapter external-evidence assessments
```

A new compiler capability or evidence role must force adapter work.

---

# 8. Typed tapscript instruction core

## 8.1 Instruction values

Introduce a typed instruction representation:

```rust
pub enum TapscriptInstruction {
    Opcode(target_elements::OpcodeId),
    Push(StackItem),
}
```

The exact names may differ.

Do not expose a general:

```rust
RawOpcode(u8)
RawInstruction(Vec<u8>)
RawProgram(Vec<u8>)
```

through the safe construction API.

Raw bytes may enter a parser as untrusted input and must either produce a typed
instruction or fail.

## 8.2 Stack items

Define:

```rust
pub struct StackItem {
    bytes: Vec<u8>,
}
```

with checked construction against the target’s stack-element limit.

Provide typed constructors where the target contract fixes encoding:

```rust
StackItem::script_number(...)
StackItem::signed_le64(...)
StackItem::unsigned_le32(...)
StackItem::unsigned_le64(...)
StackItem::encoded(...)
```

Typed constructors:

- use canonical encoding;
- reject out-of-domain values;
- reject nonminimal script numbers;
- preserve exact bytes;
- carry no protocol meaning.

Do not add:

```text
ReceiptValue
StateOmega
AshAmount
```

to the target instruction package.

## 8.3 Push encoding

Review and type the target’s canonical data-push rules needed by fixture
scripts.

Add a target-owned push contract covering:

- empty push;
- direct short push;
- extended push forms used within accepted limits;
- minimal encoding rule;
- maximum literal size;
- malformed or nonminimal behavior;
- evidence requirement.

The tapscript serializer selects the unique minimal form.

Unknown or nonminimal push encoding fails parsing.

## 8.4 Program representation

A primitive fixture program is:

```rust
pub struct TapscriptProgram {
    instructions: Vec<TapscriptInstruction>,
}
```

Fields remain private.

Construction validates:

- instruction census;
- stack-item sizes;
- target execution domain;
- target contract version;
- serialization availability;
- explicit work limits.

Guide 9 does not assign attestation-contract relation IDs or carriers to programs.

## 8.5 Exact serialization

The serializer resolves every opcode byte from the reviewed target contract.

It must not duplicate raw opcode numbers in tapscript.

Required property:

\[\operatorname{decode}(\operatorname{encode}(p))=p\]

for every supported typed program.

The encoded byte vector is deterministic and independent of:

- declaration order;
- map iteration;
- source path;
- host;
- thread count;
- environment.

## 8.6 Parsing

Implement parsing for the exact supported subset:

- reviewed opcode bytes;
- reviewed push forms;
- no unknown opcode;
- no nonminimal push;
- no trailing partial instruction;
- no oversized push;
- no execution-domain substitution.

The parser is a validation boundary for target-native result correlation and
tests. It is not a general Elements script parser.

## 8.7 No instruction identity

Instructions are compared by typed value.

Programs are compared by typed instruction sequence and exact bytes.

No instruction or program digest is minted.

---

# 9. Abstract stack validation

## 9.1 Purpose

The abstract stack validator answers:

> Given a typed initial stack and the reviewed primitive contracts, what
> successful, non-aborting failure, and aborting states can this instruction
> sequence produce?

It does not claim target execution.

## 9.2 Stack state

Use a typed state such as:

```rust
pub struct AbstractStackState {
    main: Vec<StackValueType>,
    alternate: Vec<StackValueType>,
}
```

The initial Guide-9 primitive set may leave the alternate stack unchanged, but
the type should represent it so future instructions cannot add an untracked
axis.

## 9.3 Execution result

Use explicit outcomes:

```rust
pub struct AbstractInstructionResult {
    success: BTreeSet<AbstractStackState>,
    nonaborting_failure: BTreeSet<AbstractStackState>,
    aborts: BTreeSet<FailureCause>,
}
```

Equivalent factoring is acceptable.

Do not collapse:

```text
success
branchable false
abort
```

into one Boolean.

## 9.4 Success alternatives

For every `SuccessCase`, the validator:

1. checks operand shape;
2. applies the case’s consumed/retained rule;
3. pushes case-specific results;
4. checks stack and element limits;
5. records the resulting state.

If a discriminant cannot be decided from abstract types alone, retain every
compatible alternative.

Do not select the first alternative.

## 9.5 Failure paths

For each declared `FailureEffect`:

- `AbortEvaluation` records an abort;
- `ConsumeOperandsPushFalse` consumes exactly the declared operands and pushes
  canonical false;
- `RetainOperandsPushFalse` leaves operands unchanged and pushes canonical
  false.

The validator must reject a failure contract that cannot be applied to its
declared operands.

## 9.6 State-set limits

Alternative success and failure states may grow combinatorially.

Define explicit limits:

```text
maximum abstract states
maximum stack depth
maximum altstack depth
maximum instructions
maximum result alternatives
```

Exhaustion returns a typed complexity error and no partial validated program.

Diagnostic counters saturate; semantic budget counters fail before overflow.

## 9.7 Program validation

A program validates only when:

- every instruction is target-supported;
- every incoming state has sufficient operands;
- every type relation is compatible;
- every state stays within target limits;
- every non-aborting failure state is represented;
- no hidden raw instruction exists;
- the complete result set is nonempty unless the program is intentionally
  always-aborting and typed as such.

Guide 9 does not yet require branch-join normalization for protocol programs.

---

# 10. Generic native fixture language

## 10.1 Purpose

Primitive fixtures describe generic target transactions and stack contexts.

They contain no attestation-contract semantic object or operation.

A fixture binds:

- reviewed target contract version;
- development binding;
- execution domain;
- leaf version;
- exact script bytes;
- exact initial stack;
- generic transaction context where introspection requires one;
- expected primitive success or failure case;
- expected final stack or abort class;
- expected resource observation.

## 10.2 Fixture identity

Use a complete typed key, not a digest:

```rust
pub struct NativeCaseId {
    group: NativeCaseGroup,
    opcode: Option<OpcodeId>,
    ordinal: u32,
}
```

The ordinal is stable only within the declared fixture set and is not a public
semantic identity.

A better implementation may use a typed case enum with no ordinal.

## 10.3 Fixture groups

```rust
pub enum NativeCaseGroup {
    ExecutionDomain,
    LeafVersion,
    InstructionEncoding,
    PushEncoding,
    InputIntrospection,
    OutputIntrospection,
    TransactionIntrospection,
    Arithmetic,
    Comparison,
    Conversion,
    StreamingHash,
    EllipticCurve,
    Signature,
    Sighash,
    RelativeTimelock,
    ConfidentialValue,
    Issuance,
    Resource,
}
```

## 10.4 Generic transaction context

Provide only target-owned fields:

```text
transaction version
locktime
current input index
input count
output count

per input:
    outpoint
    spent asset field
    spent value field
    spent program
    sequence
    issuance fields
    initial witness stack

per output:
    asset field
    value field
    nonce field
    program

script path:
    leaf version
    exact script bytes
    control data where the executor requires it
```

No protocol family positions, roots, or owners appear.

## 10.5 Explicit and confidential fixtures

Use deterministic public test values.

Where a valid confidential transaction requires randomness, use an explicit
test-only deterministic seed in the external executor fixture.

The report must label it test-only.

Production randomness and secret blinding data are out of scope.

## 10.6 Expected results

Expected results derive from the typed reviewed target contract and independent
explicit vectors.

They must not derive from the executor’s returned result.

For byte encodings, expected bytes are handwritten reviewed vectors or produced
by an independent minimal reference implementation.

---

# 11. Secretless external executor protocol

## 11.1 Process boundary

The conformance command receives an explicit executable:

```text
--executor PROGRAM
```

Selecting it grants execution authority.

The first-party harness does not authenticate or sandbox it.

The executor runs in a secretless external environment under ADR-015.

## 11.2 No raw child diagnostics

The harness must not log:

- raw executor path;
- raw child argv;
- raw child stderr;
- environment values.

On external failure it reports:

- fixed diagnostic text;
- typed protocol phase;
- child status;
- safe case ID;
- argument count where useful.

Raw child stderr is omitted, not heuristically redacted.

## 11.3 Protocol transport

Use length-delimited JSON or NDJSON over child stdin/stdout.

A suitable protocol is:

```text
startup:
    one handshake request
    one handshake response

execution:
    one request object per case
    one response object per case
```

The child’s stdout is protocol data only.

Unknown fields, unsupported schemas, duplicate case IDs, out-of-order results,
missing results, and trailing protocol data fail closed.

## 11.4 Handshake

The handshake reports public provenance:

```rust
pub struct ExecutorHandshake {
    protocol_schema: u32,
    implementation_name: String,
    implementation_version: String,
    upstream_revision: Option<String>,
    supported_domains: BTreeSet<ExecutionDomain>,
    supported_leaf_versions: BTreeSet<LeafVersion>,
    capabilities: BTreeSet<ExecutorCapability>,
}
```

These fields are report provenance, not target identity and not authenticity.

The first-party gate explicitly selects the reviewed executor implementation.
A fake executor can lie in the handshake; the report does not claim otherwise.

## 11.5 Execution request

Conceptually:

```rust
pub struct NativeExecutionRequest {
    schema: u32,
    case: NativeCaseId,
    fixture: NativeFixture,
}
```

## 11.6 Execution response

Conceptually:

```rust
pub struct NativeExecutionResponse {
    schema: u32,
    case: NativeCaseId,
    verdict: NativeVerdict,
    final_stack: Option<Vec<Vec<u8>>>,
    final_altstack: Option<Vec<Vec<u8>>>,
    observed_failure: Option<ObservedFailureClass>,
    resources: NativeResourceObservation,
}
```

Verdicts remain distinct:

```rust
pub enum NativeVerdict {
    Accepted,
    Rejected,
    InfrastructureError,
}
```

Infrastructure error is never expected rejection.

## 11.7 Timeout and process failure

Timeouts are explicit typed configuration.

A timeout produces:

```text
infrastructure_error
```

It must not count as:

```text
target rejected malformed fixture
```

The harness terminates the external executor and publishes no success stamp.

## 11.8 Mock executor

A mock executor is permitted only for:

- protocol parsing tests;
- missing response;
- duplicate response;
- malformed JSON;
- wrong schema;
- timeout;
- child failure;
- reordered result;
- extra result;
- report/stamp behavior.

A mock result can never satisfy the target-native gate.

The native gate requires an explicit nonmock runner selection and records that
selection as ordinary provenance.

---

# 12. Typed native-conformance reports

## 12.1 Report purpose

The report states what one executor observed for one exact reviewed target
contract and development binding.

It does not change the target definition.

## 12.2 Report shape

Conceptually:

```rust
pub struct NativeConformanceReport {
    schema: u32,
    target_contract_version: TargetContractVersion,
    environment: DeploymentEnvironment,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    activation: ActivationDeclaration,
    executor: ExecutorProvenance,
    cases: Vec<NativeCaseResult>,
    evidence: Vec<EvidenceRequirementResult>,
    summary: NativeReportSummary,
}
```

No report digest is present.

## 12.3 Per-case result

```rust
pub struct NativeCaseResult {
    case: NativeCaseId,
    expected: ExpectedNativeOutcome,
    observed: ObservedNativeOutcome,
    status: CaseStatus,
}
```

Statuses:

```text
passed
failed
infrastructure_error
```

No required case may be `skipped` in a passing Guide-9 report.

Unsupported contract capabilities are not execution cases that mysteriously
pass. They remain explicit unsupported or unresolved contract facts.

## 12.4 Evidence requirement result

```rust
pub enum EvidenceDisposition {
    Passed,
    Failed,
    UnresolvedByDesign,
    InfrastructureError,
}
```

`UnresolvedByDesign` is not success.

The report summary distinguishes:

```text
complete for the Guide-9 required evidence plan
partial because optional or future evidence remains unresolved
failed
```

Do not call a partial report green.

## 12.5 Evidence plan

Before execution, derive a typed evidence plan dividing
`TargetEvidenceRequirementId::ALL` into:

1. required in Guide 9;
2. unresolved because the corresponding capability is currently incomplete;
3. unsupported by the reviewed static contract;
4. deferred to complete protocol transaction evidence.

The plan is first-party typed policy, not executor output.

## 12.6 Expected Guide-9 evidence status

The default Guide-9 target is:

| Evidence requirement | Guide-9 expectation |
|---|---|
| tapscript execution domain | pass |
| leaf-version activation | pass |
| opcode semantics | pass for every reviewed opcode |
| encoding semantics | pass for every backend-used encoding |
| input introspection | pass |
| output introspection | pass |
| transaction introspection | pass |
| arithmetic semantics | pass |
| comparison semantics | pass |
| conversion semantics | pass |
| streaming hash semantics | pass |
| signature semantics | pass |
| relative-timelock semantics | pass |
| elliptic-curve semantics | pass |
| issuance introspection | pass |
| consensus resource limits | pass or remain explicitly external if the executor cannot establish them directly |
| policy resource limits | pass for the bound development environment |
| sighash semantics | pass for every dimension promoted to reviewed; otherwise remain unresolved |
| confidential-value conservation | pass as target/consensus evidence where the executor supports complete transactions |
| commitment equality | remain unsupported unless a reviewed target mechanism is added |
| authenticated opening | remain unsupported; no current evidence identity exists for it |

A required evidence row cannot be omitted merely because the executor lacks a
feature. The Guide-9 gate remains incomplete until every required row passes or
the evidence plan is deliberately revised with owning-policy review.

## 12.7 Report publication

`check-target-elements-native` follows ADR-010 and ADR-014:

```text
direct mode:
    one JSON result on stdout
    terminal stdout refused

build mode:
    --report FILE
    --stamp FILE
    report published first
    empty stamp touched second
```

Failure:

- writes no stdout result;
- does not freshly date the stamp;
- emits JSON diagnostics;
- leaves an existing nonempty stamp untouched.

Report and stamp destinations must be lexically distinct.

---

# 13. Primitive execution matrix

Every member of `OpcodeId::ALL` receives target-native positive and negative
coverage.

The expected opcode bytes must come from an independently reviewed table in the
test oracle, not by asking the production registry for both expected and
actual.

## 13.1 Execution domain and leaf version

Positive:

- reviewed leaf version executes each reviewed extension primitive;
- standard signature and timelock primitives execute in the reviewed domain.

Negative:

- unreviewed leaf version rejected before execution;
- reviewed extension primitive outside the required execution domain fails as
  declared;
- reviewed extension bytes are not treated as success opcodes;
- wrong control/leaf context fails.

## 13.2 Streaming SHA-256

For initialize, update, and finalize:

Positive:

- empty message;
- one chunk;
- several chunks;
- maximum accepted chunk;
- state serialization round trip;
- exact known digest vectors.

Negative:

- stack underflow;
- malformed serialized context;
- oversized or invalid context;
- context write failure where constructible;
- wrong update/finalize ordering;
- execution-domain failure.

Compare final digest with an independent standard SHA-256 implementation or
published test vectors. The external executor remains the actual target result.

## 13.3 Input introspection

Cover:

```text
input outpoint
input asset
input value
input program
input sequence
input issuance
current input index
```

Positive forms:

- first and last valid index;
- explicit asset;
- confidential asset;
- explicit value;
- confidential value;
- witness program;
- non-witness script digest;
- minimum and maximum sequence;
- issuance present;
- issuance absent;
- current index for several inputs.

Negative:

- negative script-number index;
- out-of-range index;
- malformed script number;
- missing introspection context;
- wrong operand width;
- execution-domain failure.

Check exact final stack shape and bytes for every successful alternative.

## 13.4 Output introspection

Cover:

```text
output asset
output value
output nonce
output program
```

Positive forms:

- explicit and confidential asset;
- explicit and confidential value;
- explicit, confidential, and null nonce;
- witness and non-witness output program;
- first and last valid output index.

Negative:

- negative index;
- out-of-range index;
- malformed index;
- missing context;
- wrong operand width;
- execution-domain failure.

## 13.5 Transaction introspection

Cover:

```text
version
locktime
input count
output count
transaction weight
```

Positive:

- minimum and representative values;
- count boundaries;
- exact byte order;
- observed weight against independently calculated fixture weight.

Negative:

- missing introspection context where the primitive declares it;
- target-domain failure;
- any unsupported context mode discovered during source review.

If a current static contract claims no context failure for version or locktime,
the native vectors must confirm that claim or force a contract correction.

## 13.6 Signed fixed-width arithmetic

For addition, subtraction, multiplication, division, and negation:

Positive:

- zero;
- one;
- negative one;
- minimum and maximum signed values where valid;
- representative mixed signs;
- exact nonoverflowing boundary;
- Euclidean division sign cases;
- exact remainder and quotient order.

Negative/non-aborting:

- positive overflow;
- negative overflow;
- negation of minimum signed value;
- division by zero;
- minimum divided by negative one where overflow applies;
- wrong operand width;
- stack underflow.

For every overflow/division failure, verify:

```text
operands retained
false pushed above them
```

The abstract validator and native executor must agree on the complete resulting
stack.

## 13.7 Signed comparisons

Cover every comparison opcode:

```text
less than
less than or equal
greater than
greater than or equal
```

Positive semantic cases:

- less;
- equal;
- greater;
- negative values;
- signed boundaries.

Failure cases:

- stack underflow;
- wrong operand width;
- execution-domain failure.

Verify that false comparison is successful output, not failure.

## 13.8 Numeric conversions

### Script number to fixed width

- zero;
- positive;
- negative;
- maximum valid script number;
- minimum valid script number;
- nonminimal encoding;
- oversized encoding;
- stack underflow.

### Fixed width to script number

- zero;
- positive;
- negative;
- exact range boundary;
- one above and below admissible script-number range;
- wrong width;
- stack underflow.

### Unsigned 32 to signed 64

- zero;
- maximum unsigned 32-bit value;
- high bit set;
- wrong width;
- stack underflow.

Expected bytes come from an independent small canonical reference.

## 13.9 Elliptic-curve and tweak verification

Positive:

- valid scalar multiplication relation;
- valid tweak relation;
- both accepted public-key parities;
- exact scalar and tweak boundaries admitted by the target.

Negative:

- wrong scalar;
- wrong point;
- wrong tweak;
- malformed compressed key;
- malformed x-only key;
- malformed scalar;
- stack underflow;
- validation-budget exhaustion;
- execution-domain failure.

The fixture uses public deterministic test values only.

No production key or secret scalar enters.

## 13.10 Signature primitives

Cover:

```text
transaction-sighash check
transaction-sighash verify
stack-message check
stack-message verify
```

Positive:

- valid signature;
- valid public-key encoding;
- valid stack message;
- validation-budget accounting.

Failure behavior:

- empty signature;
- nonempty invalid signature;
- malformed public key;
- unknown public-key type;
- stack underflow;
- budget exhaustion.

Verify the distinction:

```text
check + empty signature:
    consumes operands
    pushes false

check + invalid nonempty signature:
    aborts

verify form + empty signature:
    aborts
```

If native execution disagrees, the static target contract is corrected before
the report can pass.

## 13.11 Sighash mutation matrix

For each reviewed or candidate sighash mode, begin from one valid signed
transaction and mutate independently:

- every output;
- output count;
- signing input;
- another input;
- issuance fields;
- transaction version;
- locktime;
- tapleaf hash;
- internal key;
- spent outputs;
- input-set extension.

Record whether the signature remains valid.

Use this matrix to classify every `SighashDimension` as:

```text
reviewed committed
reviewed not committed
unreviewed
unsupported
```

Do not mark `OutputCommittingSighash` or `InputCommitmentControl` reviewed
unless exact native mutations establish the required behavior.

A signature primitive existing is not sighash evidence.

## 13.12 Relative timelock

Cover:

- transaction version below minimum;
- exact minimum version;
- sequence disable flag;
- block-height mode;
- time-interval mode;
- one below required age;
- exact required age;
- one above required age;
- negative operand;
- malformed script number;
- mode mismatch;
- sequence-mask boundary;
- stack underflow.

Verify:

- success retains the operand;
- unsatisfied lock aborts;
- disabled or low-version behavior matches the typed contract exactly.

No attestation-contract cadence band is emitted.

## 13.13 Confidential-value conservation

Target-native complete-transaction fixtures cover:

- explicit balanced transaction;
- confidential balanced transaction;
- mixed explicit/confidential balanced transaction;
- one-unit imbalance;
- wrong commitment balance;
- malformed rangeproof;
- malformed surjection proof where applicable;
- issuance interaction where supported.

This evidence proves only target-wide conservation.

It does not prove:

- object recognition;
- recipient closure;
- sponsor isolation;
- protocol flow partition;
- attestation-contract operation correctness.

## 13.14 Commitment equality and authenticated opening

The current reviewed contract marks these unsupported.

Guide 9 must preserve that status unless a complete source review and native
mechanism are added.

Low-level curve and hash primitives are not sufficient.

Required tests:

- adapter reports the capability unsupported;
- no instruction-core helper claims equality/opening;
- no native report marks the evidence requirement passed;
- no backend pattern identity is introduced.

## 13.15 Issuance and reissuance

Positive:

- issuance-present introspection;
- issuance-absent result;
- asset amount;
- inflation-keys amount;
- entropy;
- zero and nonzero blinding nonce;
- issuance/reissuance distinction;
- outpoint issuance flag.

Negative:

- malformed issuance encoding;
- wrong absent marker;
- wrong entropy width;
- wrong nonce width;
- out-of-range input;
- missing context.

Verify the complete result alternative and stack order.

Do not map target issuance to `U`, `ENT`, or `DIST_CTL`.

## 13.16 Resource observations

For each opcode, record:

- script bytes;
- initial witness bytes;
- initial stack items;
- peak stack items;
- peak altstack items;
- maximum element bytes;
- validation-budget use;
- target operation cost where applicable;
- transaction weight where a complete fixture exists.

Compare observed values with target-contract resource projections.

Do not derive architecture batch bounds from these measurements.

---

# 14. Encoding conformance

## 14.1 Independent expected vectors

Maintain explicit expected vectors for every `EncodingClass`.

Do not create expected bytes by serializing the same `EncodingSpec` the parser
under test uses.

## 14.2 Required encoding vectors

Cover:

- explicit/confidential asset prefixes;
- explicit/confidential value prefixes;
- explicit/confidential/null nonce;
- outpoint txid/index/flags;
- sequence;
- issuance entropy and nonce;
- script number;
- signed little-endian 64;
- unsigned little-endian 32 and 64;
- SHA-256 context and digest;
- x-only/compressed public keys;
- signatures;
- EC scalar;
- target tweak;
- witness program;
- non-witness script digest.

## 14.3 Malformed vectors

- unknown prefix;
- prefix from the wrong encoding domain;
- truncated payload;
- oversized payload;
- incorrect fixed width;
- wrong byte order;
- nonminimal script number;
- alternate zero encoding;
- trailing bytes;
- null form with payload;
- explicit form without payload;
- confidential form with explicit width;
- explicit form with confidential width.

Every unknown form fails closed.

## 14.4 Asset/value independence

Include all four combinations where the target permits them:

```text
explicit asset + explicit value
explicit asset + confidential value
confidential asset + explicit value
confidential asset + confidential value
```

The typed target must not infer one axis from the other.

These are target encoding facts, not attestation-contract representation policy.

---

# 15. Instruction-core conformance

## 15.1 Byte census

For every `OpcodeId::ALL` member:

- serializer emits the independently expected byte;
- parser returns the exact typed opcode;
- no two IDs emit the same byte;
- no supported byte parses to two IDs;
- unknown byte fails;
- extension bytes are domain-gated correctly.

## 15.2 Push census

For every supported push length:

- serializer chooses the minimal form;
- parser accepts the minimal form;
- parser rejects longer equivalent forms where minimality is required;
- empty push is canonical;
- maximum permitted item passes;
- one above maximum fails.

## 15.3 Stack-contract static oracle

Maintain an independent test oracle containing exact:

- operand sequence;
- success alternatives;
- consumed/retained behavior;
- non-aborting failure stack;
- aborting causes;
- resource cost.

The oracle must not call the production target-contract construction helper.

## 15.4 Native/static equality

For every native fixture:

1. encode the program through the instruction core;
2. compute expected abstract outcomes through the static contract;
3. execute exact bytes through the native executor;
4. compare native verdict, final stacks, and resources;
5. fail on any mismatch.

The native result remains the target observation. The abstract validator is the
typed expectation.

## 15.5 Parser mutation tests

Mutate:

- opcode byte;
- push length;
- push payload;
- truncation;
- unknown extension byte;
- duplicate bytes;
- trailing partial instruction.

Require focused parse errors.

---

# 16. Capability and evidence closure

## 16.1 Target capability census

Validate:

```text
ElementsCapability::ALL
=
target capability registry keys
```

exactly and duplicate-sensitively.

## 16.2 Prerequisite closure

For each capability:

- every prerequisite exists;
- the graph is acyclic;
- every reviewed capability has only reviewed transitive prerequisites;
- adapter assessment includes every missing/unsupported transitive
  prerequisite;
- insertion order does not affect closure.

## 16.3 Primitive coverage closure

For every reviewed capability requiring opcodes:

```text
required opcode set
⊆ natively passed opcode set
```

This does not make the capability pattern-complete. It establishes only that
its primitive prerequisites behaved as declared.

## 16.4 Evidence closure

For every reviewed target claim used by the adapter:

```text
claim
→ evidence requirement
→ Guide-9 evidence-plan disposition
→ native report result or explicit unresolved status
```

No claim is silently evidence-free.

## 16.5 Adapter non-weakening

For every compiler capability:

1. assess against the complete reviewed target;
2. downgrade one required target primitive;
3. reassess;
4. require the assessment to degrade;
5. preserve the compiler capability in the result;
6. restore the primitive;
7. require the original assessment.

Repeat for transitive prerequisites.

---

# 17. Security and execution trust

## 17.1 Public-data interface

The conformance package accepts:

- public target contract;
- public development network/genesis identifiers;
- public generic transaction fixtures;
- public script and witness bytes;
- public deterministic test keys;
- public deterministic test commitments;
- explicit external executor selection;
- explicit report/stamp destinations.

It does not accept production secret material.

## 17.2 Test keys and signatures

Target signature tests use deterministic, clearly synthetic test keys.

They are public fixture material and must be labeled as such.

Do not reuse them outside tests.

No interface accepts a production private key.

## 17.3 External executor

The executor is trusted code selected by the caller.

It may access filesystem and network with the host process’s authority.

The first-party harness does not:

- authenticate it;
- sandbox it;
- sanitize its deliberately returned protocol bytes;
- protect against malicious executor behavior;
- prove implementation independence.

Run it in a secretless isolated environment.

## 17.4 Crash artifacts

Core dumps and executor crash bundles are not ordinary diagnostics.

Repository-controlled CI must not publish them automatically.

## 17.5 Paths

Every executor path, fixture path, report path, and stamp path is supplied
explicitly.

External paths are caller-granted host capabilities.

The conformance package makes no TOCTOU or hostile-filesystem containment
claim.

## 17.6 No credential transport

Do not add:

```text
--rpc-user
--rpc-password
--cookie
--token
--wallet
--private-key
```

If an external executor needs node authentication, that boundary is established
outside the first-party process and reviewed separately before production use.

---

# 18. Determinism

## 18.1 Canonical fixture census

Fixtures sort by typed case ID.

Duplicate case IDs fail.

Input declaration order must not affect:

- fixture set;
- program bytes;
- expected stack states;
- report case order;
- summary counts.

## 18.2 Report determinism

Given identical:

- reviewed target definition;
- development binding;
- executor protocol result;
- fixture set;
- explicit work limits;

the report bytes are identical.

Exclude:

- wall-clock time;
- elapsed time from canonical result;
- hostname;
- username;
- process ID;
- temporary path;
- raw executor path;
- environment;
- nondeterministic map order.

Timing may appear only in explicitly noncanonical diagnostics, never in the
report compared for evidence.

## 18.3 Native executor ordering

The executor may process cases internally in any order, but the first-party
harness:

- requires exactly one response per case;
- correlates by typed case ID;
- canonicalizes output order;
- rejects unknown or duplicate responses.

## 18.4 Thread count

Target-native execution may use target-internal parallelism only if result
bytes and verdicts are schedule-independent.

Thread count is provenance, not identity.

---

# 19. Error vocabulary

## 19.1 Target errors

Add only errors reached by real target-validation branches.

Likely additions:

```rust
pub enum TargetError {
    ReviewedDefinitionMismatch,

    IncompleteSuccessContract(OpcodeId),
    DuplicateSuccessCase {
        opcode: OpcodeId,
        case: SuccessCondition,
    },
    ContradictorySuccessCase(OpcodeId),
    InvalidRetainedOperandContract(OpcodeId),

    SignatureContractMismatch,
    TimelockContractMismatch,
    ConfidentialContractMismatch,
    IssuanceContractMismatch,
    ResourceContractMismatch,
    EvidenceContractMismatch,

    CapabilityStatusExceedsPrerequisite {
        capability: ElementsCapability,
        prerequisite: ElementsCapability,
    },

    EncodingInterpretationMismatch(EncodingClass),
    EncodingDomainMismatch(EncodingClass),
    EncodingWidthMismatch(EncodingClass),
    EncodingByteOrderMismatch(EncodingClass),
    EncodingCanonicalityMismatch(EncodingClass),
}
```

Exact variants follow actual validation paths.

## 19.2 Tapscript instruction errors

```rust
pub enum TapscriptError {
    UnsupportedTargetContractVersion,
    UnreviewedTargetDefinition,

    UnsupportedInstruction(OpcodeId),
    UnknownOpcodeByte(u8),
    DuplicateOpcodeByte(u8),

    NonMinimalPush,
    OversizedStackItem,
    TruncatedInstruction,
    TrailingInstructionBytes,

    StackUnderflow {
        instruction: usize,
    },
    StackTypeMismatch {
        instruction: usize,
        expected: StackValueType,
        actual: StackValueType,
    },
    InvalidSuccessContract(OpcodeId),
    InvalidFailureContract(OpcodeId),
    AbstractStateLimitExceeded {
        maximum: u64,
    },
    InstructionLimitExceeded {
        maximum: u64,
    },
    StackLimitExceeded,
    AltstackLimitExceeded,
    ElementSizeExceeded,

    DuplicateCapabilityAssessment(RequiredCapability),
    CapabilityAssessmentCensusMismatch { ... },
    DuplicateEvidenceAssessment(ExternalEvidenceRole),
    EvidenceAssessmentCensusMismatch { ... },
}
```

Do not add operation-emission, constructor, linker, ABI, or relocation errors
yet.

## 19.3 Native conformance errors

```rust
pub enum NativeConformanceError {
    UnsupportedProtocolSchema(u32),
    ExecutorStartupFailed,
    ExecutorHandshakeFailed,
    ExecutorProtocolMismatch,
    ExecutorTimeout,
    ExecutorExited,
    DuplicateCaseResponse(NativeCaseId),
    MissingCaseResponse(NativeCaseId),
    UnexpectedCaseResponse(NativeCaseId),
    ResponseOrderViolation,
    MalformedResponse,
    InfrastructureFailure(NativeCaseId),

    TargetContractMismatch,
    DevelopmentBindingMismatch,
    FixtureCensusMismatch,
    ExpectedOutcomeMismatch(NativeCaseId),
    FinalStackMismatch(NativeCaseId),
    FinalAltstackMismatch(NativeCaseId),
    FailureClassMismatch(NativeCaseId),
    ResourceObservationMismatch(NativeCaseId),

    EvidenceCensusMismatch,
    RequiredEvidenceMissing(TargetEvidenceRequirementId),
    RequiredEvidenceFailed(TargetEvidenceRequirementId),
    RequiredEvidenceInfrastructureError(TargetEvidenceRequirementId),
    MockExecutorCannotSatisfyNativeGate,

    AliasedOutputs,
    PublicationFailure,
}
```

Do not include raw child stderr or argv in an error value.

---

# 20. Test strategy

## 20.1 `target-elements` tests

Required suites:

```text
reviewed trust-state tests
success-contract tests
cross-contract weld tests
capability status-closure tests
encoding interpretation tests
definition mutation tests
definition permutation tests
deployment binding tests
```

## 20.2 `tapscript` tests

Required suites:

```text
instruction census
opcode encoding
push encoding
program parsing
stack success alternatives
non-aborting failure stacks
aborting failure classes
resource projection
static capability assessment
external-evidence assessment
non-weakening
permutation equality
complexity-limit behavior
```

## 20.3 Conformance-package tests

Required suites:

```text
executor handshake
request/response round trip
unknown fields
wrong schema
duplicate result
missing result
unexpected result
timeout
child failure
raw stderr omission
mock gate refusal
report/stamp contract
mode-only publication repair
canonical report order
evidence-plan closure
```

## 20.4 Public API tests

### `target-elements`

Prove externally:

- generic definitions can be validated;
- generic validation does not yield reviewed trust state;
- exact reviewed target is constructible;
- reviewed wrapper fields remain private;
- production binding remains unavailable or rejected;
- no target digest exists.

### `tapscript`

Prove externally:

- typed instructions can be built safely;
- raw unchecked opcode/program constructors are unavailable;
- compiler requirements can be statically assessed;
- complete analyzed-program internals remain unavailable;
- no program identity exists.

### conformance package

Prove externally:

- the CLI follows ADR-010;
- executor protocol data is typed;
- no credential argument exists;
- direct and report/stamp modes behave correctly.

---

# 21. Independent oracles

## 21.1 Opcode-byte oracle

Maintain a test-only explicit table:

```text
OpcodeId → expected byte
```

Do not derive the expected byte from the production target registry.

Compare every `OpcodeId::ALL` member exactly.

## 21.2 Stack-contract oracle

Maintain an independently written expected contract for every opcode:

- operands;
- success cases;
- retention/consumption;
- failure effects;
- resources.

Do not call the production contract builder.

## 21.3 Encoding oracle

Use explicit known byte vectors and a small independent reference codec where
appropriate.

The production encoder must not generate its own expectations.

## 21.4 Abstract execution oracle

For bounded short instruction sequences, enumerate every contract-compatible
success/failure transition independently and compare complete state sets with
the production abstract validator.

## 21.5 Native result comparison

Native executor results are compared directly with the reviewed typed
contract.

A mismatch does not update the expected result automatically.

The mismatch is triaged as one of:

```text
static target transcription defect
executor/materialization defect
native target behavior disagreement
fixture defect
```

Expected values change only after source review identifies the correct owner.

## 21.6 Capability-adapter oracle

Retain an independently written mapping from:

```text
RequiredCapability
→ static target obligations
```

and add an independently written mapping from:

```text
ExternalEvidenceRole
→ target evidence obligations
```

Compare stable assessment projections exactly.

---

# 22. Target-native executor selection

## 22.1 Selection criteria

The selected executor must:

- use the actual Elements interpreter or node implementation;
- expose exact script verdicts;
- expose final stack where the interpreter permits;
- expose distinguishable failure information needed by the contract;
- construct generic public fixtures reproducibly;
- support the reviewed leaf version;
- support the reviewed custom opcodes;
- run in a development environment;
- require no credential through the first-party interface;
- provide public source/revision provenance;
- use a licence compatible with its deployment and evidence role.

## 22.2 Review provenance

Record in the human target reference:

- upstream repository;
- revision consulted;
- source locations;
- executor adapter source;
- build commands;
- node/interpreter version;
- network and genesis;
- activation configuration;
- upstream licence;
- exact target-native cases run;
- unresolved observations.

This is report provenance, not target identity.

## 22.3 Executor implementation location

The native interpreter adapter may live outside this repository if that is the
cleanest way to preserve the first-party public-data boundary.

If first-party source for the adapter enters this repository, it requires:

- explicit package ownership;
- Meson census membership;
- dependency and licence review;
- untrusted-execution review;
- no credential interface;
- no hidden production authority.

The conformance package remains the owner of the protocol and report regardless
of executor implementation location.

---

# 23. Meson and CI integration

## 23.1 Ordinary CI

Ordinary CI runs:

- target contract tests;
- instruction-core tests;
- executor-protocol tests with mocks;
- report/stamp tests;
- all existing workspace lanes.

It does not claim target-native evidence when no native executor is available.

A skipped native lane makes the Guide-9 evidence incomplete, even if ordinary
CI is green.

## 23.2 Explicit native target

Add a non-default Meson target or explicit command such as:

```text
target-elements-native-check
```

Configuration receives the executor path explicitly.

A suitable command shape is:

```sh
target/debug/check-target-elements-native \
  --executor /explicit/path/to/native-executor \
  --network-id <public-development-id> \
  --genesis-id <public-development-id> \
  --report build/target-elements-native.json \
  --stamp build/target-elements-native.ok
```

Do not put credentials in this command.

## 23.3 Mocked Meson contract

Extend `scripts/test-meson-mock.sh` only to test graph wiring:

- native checker binary edge;
- explicit executor argument;
- report before stamp;
- failure propagation;
- mode repair;
- no-op behavior.

The mock must not satisfy native evidence.

## 23.4 Guide-9 gate command

The final gate records a real invocation with the reviewed native executor.

The exact command and executor provenance belong in the backlog gate record.

---

# 24. Documentation requirements

## 24.1 `target-elements` README

Update the state to:

```text
implemented:
    reviewed-target trust-state separation
    complete success/failure stack contracts
    cross-contract welds
    transitive capability status closure
    independently typed encoding interpretation
    target-native primitive evidence plan

target-native evidence:
    exact Guide-9 results recorded separately

still unsupported:
    commitment equality
    authenticated opening
    any other claim not established by the native suite

not claimed:
    backend pattern correctness
    attestation-contract operation emission
    production deployment
```

## 24.2 `tapscript` README

Update to:

```text
implemented:
    package boundary
    static capability adapter
    external-evidence-role adapter
    typed instruction values
    canonical opcode/push encoding
    supported-subset parser
    abstract stack and failure-state validator

not implemented:
    backend proof patterns
    attestation-contract operation programs
    relation placement
    stack scheduler for protocol programs
    constructors
    relocatable bundle
```

## 24.3 Conformance-package README

State:

```text
purpose:
    secretless execution of generic target primitive fixtures

inputs:
    reviewed target
    development binding
    public fixtures
    explicit external executor capability

outputs:
    typed native-conformance report
    optional report/stamp assets

not claimed:
    executor authenticity
    implementation independence
    production activation
    backend correctness
    release evidence identity
```

## 24.4 Package contracts

Update:

```text
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/packages/README.md
```

Add a package contract for the conformance harness if the package is retained
after Guide 9.

## 24.5 Phase card

Update:

```text
plans/phases/03-target-foundation.md
```

Guide 9 may complete the target-native primitive and instruction-core portions
of Phase 3.

It does not complete Phase 3 until the STATE constructor, wide arithmetic, and
public-declassification research gates have their accepted results or explicit
target rejection, as required by the phase card.

## 24.6 Backlog

Add a compact Guide-9 gate record and update:

```text
T3-006  target-native primitive conformance
T3-007  typed tapscript instruction foundation
```

Mark them `DONE` only when the real native lane passes.

## 24.7 Guides index and Meson census

Add:

```text
plans/guides/guide_nine_concept.md
```

to:

```text
plans/guides/README.md
plans/guides/meson.build
```

---

# 25. Suggested implementation waves

## Wave 0 — Target-contract soundness preflight

Deliver:

- reviewed-target wrapper;
- complete success alternatives;
- cross-contract welds;
- transitive capability status closure;
- independent encoding interpretation;
- architecture identity validation boundary;
- mode-aware publication repair;
- digest policy ruling.

Required evidence:

- focused positive tests;
- every contradiction mutation;
- public-API trust-state tests;
- no generated architecture identity change unless explicitly migrated.

Suggested commit:

```text
target-elements: close the reviewed-contract boundary
```

## Wave 1 — Static versus deployment adapter boundary

Deliver:

- static capability assessment over reviewed target;
- deployment-aware API removed, renamed, or implemented honestly;
- exact external-evidence-role assessment;
- complete capability/evidence census validation;
- transitive prerequisite recheck.

Suggested commit:

```text
tapscript: make target assessment boundaries exact
```

## Wave 2 — Typed instruction and push encoding

Deliver:

- typed instruction;
- typed stack item;
- target-owned push contract;
- serializer;
- supported-subset parser;
- independent opcode-byte vectors;
- push minimality tests.

Suggested commit:

```text
tapscript: add the typed instruction core
```

## Wave 3 — Abstract stack and failure validator

Deliver:

- success alternatives;
- retained operands;
- non-aborting failure states;
- abort classes;
- stack/altstack/resource projection;
- explicit state limits;
- independent bounded oracle.

Suggested commit:

```text
tapscript: validate primitive stack relations
```

## Wave 4 — Native conformance package and protocol

Deliver:

- conformance package boundary;
- CLI under ADR-010;
- typed fixture and report schemas;
- executor handshake and request/response protocol;
- timeout and child-failure handling;
- mock executor tests;
- report/stamp publication.

Suggested commit:

```text
target-conformance: establish the native executor boundary
```

## Wave 5 — Encoding and interpreter primitives

Deliver native vectors for:

- domain and leaf version;
- opcode bytes;
- push encoding;
- streaming hash;
- input introspection;
- output introspection;
- transaction introspection;
- arithmetic;
- comparison;
- conversion;
- curve/tweak primitives.

Suggested commit:

```text
target-conformance: verify primitive execution and encodings
```

## Wave 6 — Signature, sighash, timelock, CT, and issuance

Deliver:

- signature success/failure matrix;
- sighash dimension mutation matrix;
- relative-timelock boundary matrix;
- confidential-value conservation fixtures;
- issuance/reissuance introspection fixtures;
- resource observations;
- static contract corrections where native evidence disagrees.

Suggested commit:

```text
target-conformance: verify authorization and consensus dimensions
```

## Wave 7 — Exact evidence closure and integration

Deliver:

- exact fixture census;
- exact evidence-plan census;
- required evidence all passed;
- unsupported/unresolved claims retained honestly;
- native/static report equality;
- deterministic report bytes;
- Meson native target;
- mocked wiring contract.

Suggested commit:

```text
target-conformance: close the primitive evidence census
```

## Wave 8 — Documentation and gate

Deliver:

- package READMEs;
- package contracts;
- Guide-9 gate record;
- Phase-3 card update;
- target reference provenance;
- identity/dependency impact;
- complete verification record.

Suggested commit:

```text
plans: record the Guide-9 target-native gate
```

Commit each green wave promptly.

---

# 26. Focused verification commands

## 26.1 Target static contract

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

## 26.2 Instruction core and adapter

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

## 26.3 Native conformance package

```sh
cargo test --locked -p tripod-target-elements-conformance
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements-conformance --no-deps
```

## 26.4 Compiler target boundary

```sh
cargo test --locked -p tripod-compiler target
cargo test --locked -p tripod-compiler public_api
```

Use actual test names and verify filters match nonzero tests.

## 26.5 Publication and architecture

```sh
cargo test --locked -p cli-common
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-artifacts
```

## 26.6 Native target

Use the reviewed explicit executor:

```sh
cargo run --locked \
  -p tripod-target-elements-conformance \
  --bin check-target-elements-native -- \
  --executor /explicit/path/to/reviewed-executor \
  --network-id <public-development-network-id> \
  --genesis-id <public-development-genesis-id> \
  --report build/target-elements-native.json \
  --stamp build/target-elements-native.ok
```

The exact argument names may differ after implementation.

The invocation must not carry credentials.

---

# 27. Working cadence

After each coherent wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Read `git status` after formatting.

Do not run the full native executor suite after every small edit. Run focused
static and protocol tests while working, then the complete native matrix once
the batch is coherent.

Every new tracked source enters its nearest Meson census in the same commit.

---

# 28. Dependency review

## 28.1 Expected dependency impact

`target-elements`:

```text
no dependencies
```

`tapscript`:

```text
no new dependency expected
```

`target-elements-conformance`:

```text
first-party:
    target-elements
    tapscript
    cli-common

workspace generic:
    clap
    serde
    serde_json
    thiserror
    tempfile for tests
```

No target transaction, RPC, cryptographic, or node library is added without a
present consumer and separate review.

## 28.2 Review requirements

For every newly activated dependency edge, record:

- exact workspace source;
- enabled features;
- transitive graph;
- licence;
- MSRV;
- unsafe boundary;
- determinism impact;
- parallelism impact;
- advisory status;
- lockfile impact.

Run:

```sh
cargo tree --locked -p tripod-target-elements -e features
cargo tree --locked -p tripod-tapscript -e features
cargo tree --locked -p tripod-target-elements-conformance -e features
cargo metadata --locked
```

Confirm:

```text
target-elements:
    still no dependency

tapscript:
    still only compiler and target-elements as first-party dependencies

target-elements-conformance:
    no architecture, realization, model, linker, transaction,
    vectors, release, artifacts, or labels dependency
```

---

# 29. Generated artifacts and identity impact

Expected impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged unless a separately reviewed identity migration is adopted

architecture semantic hash:
    unchanged unless a separately reviewed recipe migration is adopted

architecture behavioural hash:
    unchanged

architecture generated publications:
    unchanged

declassification publication:
    unchanged

model labels:
    may change only if deliberate participating Rust labels are added

realization identity:
    none minted

compiler identity:
    none minted

target definition digest:
    none minted

instruction/program digest:
    none minted

native report digest:
    none minted

deployment identity:
    none minted
```

Adding the conformance package updates the first-party package census in
`Cargo.lock`. Review that change explicitly.

No target-native report is committed as a canonical generated publication
unless a real persistent consumer and ADR-016 identity/report design are added
in the same implementation series.

Build-directory reports remain evidence assets for the gate, not semantic
inputs.

---

# 30. Full batch gate

After all waves:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

The mocked Meson contract is required because this guide adds package and
command wiring.

Run the real native target matrix separately with the reviewed executor.

Also run:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Run:

```sh
cargo audit
```

when installed.

If unavailable, record:

```text
cargo-audit:
    SKIPPED
```

not passed.

If no paper input changed, document byte reproducibility may be deferred for
this Guide-9 batch under repository policy. It must be recorded as deferred,
not passed.

Finally:

```sh
git status --porcelain=v1 --untracked-files=all
```

The final tree must be clean after commits.

---

# 31. Guide-9 exit criteria

Guide 9 is complete only when every required item below holds.

## Preflight target-contract soundness

- [ ] generic validated and reviewed Elements target states are distinct;
- [ ] only the exact first-party reviewed definition gains reviewed status;
- [ ] every primitive success form is representable;
- [ ] retained operands are represented explicitly;
- [ ] signature contracts are welded;
- [ ] timelock contracts are welded;
- [ ] issuance contracts are welded;
- [ ] confidential-value contracts are welded;
- [ ] resource contracts are welded;
- [ ] evidence contracts are welded;
- [ ] reviewed capability status is transitively closed;
- [ ] encoding interpretation is independent of byte-order presence;
- [ ] reviewed encoding semantics are exact;
- [ ] invalid architecture values cannot receive a public semantic identity;
- [ ] compare-if-changed publication repairs mode-only corruption;
- [ ] existing digest recipes are migrated or grandfathered explicitly, never
      silently changed.

## Static/deployment boundary

- [ ] static assessment consumes the reviewed static target;
- [ ] no API accepts an `ElementsTarget` while silently ignoring its deployment;
- [ ] deployment-aware assessment, if present, reads activation and policy;
- [ ] compiler capability census equals capability-assessment census;
- [ ] compiler external-evidence-role census equals evidence-assessment census;
- [ ] transitive target prerequisites remain visible;
- [ ] unsupported/incomplete primitives fail closed;
- [ ] no compiler capability disappears.

## Instruction core

- [ ] typed instruction enum exists;
- [ ] safe API exposes no unchecked raw opcode;
- [ ] typed stack-item construction is bounded;
- [ ] push encoding is canonical;
- [ ] every reviewed opcode serializes to the exact expected byte;
- [ ] supported bytes parse to the exact typed instruction;
- [ ] unknown and malformed instruction bytes fail;
- [ ] encode/decode round trips;
- [ ] no instruction or program digest is minted.

## Stack validation

- [ ] success alternatives are complete;
- [ ] operands consumed or retained exactly;
- [ ] aborting and non-aborting failures remain distinct;
- [ ] arithmetic failure retains operands and pushes false;
- [ ] empty-signature failure consumes operands and pushes false;
- [ ] invalid nonempty signature aborts;
- [ ] CSV success retains its operand;
- [ ] stack, altstack, element, instruction, and state limits fail closed;
- [ ] production stack validator agrees with independent bounded oracle.

## Native executor boundary

- [ ] conformance package exists;
- [ ] executor is selected explicitly;
- [ ] no credential argument exists;
- [ ] child argv and raw stderr are omitted from diagnostics;
- [ ] handshake is typed;
- [ ] request/response schemas reject unknown fields;
- [ ] duplicate, missing, unexpected, and malformed responses fail;
- [ ] timeouts and child failures are infrastructure errors;
- [ ] mock executor cannot satisfy native evidence;
- [ ] report publishes before success stamp;
- [ ] failed native check does not freshly date the stamp.

## Target-native primitive evidence

- [ ] execution-domain vectors pass;
- [ ] leaf-version vectors pass;
- [ ] every `OpcodeId::ALL` member has native positive coverage;
- [ ] every declared failure class has native focused coverage where
      constructible;
- [ ] every successful alternative has native coverage;
- [ ] input introspection vectors pass;
- [ ] output introspection vectors pass;
- [ ] transaction introspection vectors pass;
- [ ] arithmetic success/failure vectors pass;
- [ ] comparison vectors pass;
- [ ] conversion vectors pass;
- [ ] streaming hash vectors pass;
- [ ] elliptic-curve/tweak vectors pass;
- [ ] signature vectors pass;
- [ ] sighash dimensions are reviewed only when the mutation matrix proves them;
- [ ] relative-timelock vectors pass;
- [ ] issuance-present and issuance-absent vectors pass;
- [ ] CT conservation evidence passes where required;
- [ ] commitment equality remains unsupported unless a real mechanism is added;
- [ ] authenticated opening remains unsupported;
- [ ] primitive resource predictions agree with native observations.

## Evidence report

- [ ] exact fixture census appears once;
- [ ] exact evidence-plan census appears once;
- [ ] required Guide-9 evidence rows pass;
- [ ] failed or infrastructure-error rows block the stamp;
- [ ] unresolved-by-design rows are not reported as passed;
- [ ] report binds target contract version and development context;
- [ ] report records executor provenance without making it target identity;
- [ ] report bytes are deterministic;
- [ ] no report digest is minted;
- [ ] no target-native report is described as backend-pattern or release evidence.

## Security and identity

- [ ] all first-party interfaces remain public-data interfaces;
- [ ] only synthetic public test keys are used;
- [ ] no production private key or credential enters;
- [ ] executor runs in a secretless external environment;
- [ ] no target program, transaction ABI, linked bundle, or release identity is
      introduced;
- [ ] no attestation-contract operation is emitted;
- [ ] no target-specific type flows back into compiler core;
- [ ] no local handle or native fixture position enters semantic identity.

## Documentation and build

- [ ] package READMEs are current;
- [ ] package contracts are current;
- [ ] package index statuses agree;
- [ ] Phase-3 card is current;
- [ ] target reference carries exact executor provenance;
- [ ] backlog records Guide-9 evidence honestly;
- [ ] every new file is in the Meson census;
- [ ] focused package tests pass;
- [ ] Rustdoc passes with warnings denied;
- [ ] workspace formatting passes;
- [ ] workspace Clippy passes;
- [ ] debug tests pass;
- [ ] release tests pass through the full CI gate;
- [ ] mocked Meson contract passes;
- [ ] canonical Meson compile passes;
- [ ] canonical Meson tests pass;
- [ ] real native executor matrix passes;
- [ ] skipped/deferred lanes are reported honestly;
- [ ] final tree is clean.

---

# 32. Completion report template

```text
Guide 9 result
==============

Starting state:
    source revision:
    Guide-8 gate:
    second-review finding register:
    clean tree:

Target-contract preflight:
    generic/reviewed trust-state split:
    success alternatives:
    retained operands:
    signature weld:
    timelock weld:
    issuance weld:
    confidential-value weld:
    resource weld:
    evidence weld:
    transitive capability closure:
    encoding interpretation:
    architecture identity boundary:
    publication mode repair:
    digest-policy result:

Static/deployment assessment:
    static assessment input:
    deployment-aware assessment:
    compiler capability census:
    compiler external-evidence-role census:
    unsupported assessments:
    incomplete assessments:
    backend-pattern obligations:
    structural obligations:
    external-evidence obligations:

Instruction core:
    instruction types:
    stack-item types:
    push contract:
    opcode census:
    serializer:
    parser:
    encode/decode result:
    program identity:
        none

Abstract stack validation:
    success alternatives:
    retained operands:
    non-aborting failure states:
    abort classes:
    stack limits:
    state limits:
    independent oracle:

Native conformance package:
    package:
    dependencies:
    command:
    executor protocol schema:
    report schema:
    direct mode:
    report/stamp mode:
    mock behavior:
    credential fields:
        none

Native executor:
    implementation:
    upstream repository:
    upstream revision:
    licence:
    node/interpreter version:
    development network:
    genesis:
    activation:
    secretless environment:
    exact command:

Primitive evidence:
    execution domain:
    leaf version:
    opcode byte census:
    streaming hash:
    input introspection:
    output introspection:
    transaction introspection:
    arithmetic:
    comparison:
    conversion:
    elliptic curve:
    signature:
    sighash:
    relative timelock:
    confidential-value conservation:
    commitment equality:
    issuance/reissuance:
    resources:

Fixture census:
    total cases:
    passing:
    failed:
    infrastructure errors:
    unresolved by design:
    duplicate/missing/unexpected cases:

Evidence census:
    required Guide-9 rows:
    passed:
    failed:
    infrastructure errors:
    unresolved:
    report status:

Identity impact:
    Attestation version:
    realization version:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:
    target digest:
    instruction/program digest:
    native report digest:
    deployment identity:

Dependency impact:
    new package:
    new first-party edges:
    new third-party dependencies:
    Cargo.lock:
    licence:
    MSRV:
    unsafe boundary:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
    target-elements tests:
    tapscript tests:
    conformance-package tests:
    compiler boundary tests:
    Rustdoc:
    cargo tree:
    cargo metadata:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real native target command:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning result:
    T3-006:
    T3-007:
    Phase 3:
    next guide:

Residuals:
```

---

# 33. What follows Guide 9

After Guide 9 passes, the target primitive substrate is evidenced and the typed
instruction core exists.

The next implementation guide should be:

```text
Guide 10 — STATE Constructor and Exact Wide-Arithmetic Prototypes
```

Its scope should remain prototype-only and should include:

- metadata-dependent constructor continuity;
- deterministic internal-key policy;
- target tapleaf/tapbranch/tweak construction;
- predecessor and successor constructor verification;
- wrong-root, wrong-key, wrong-schema, and path-escape vectors;
- exact quotient/remainder wide floor proof;
- limb and carry bounds;
- arithmetic success-flag enforcement;
- standalone target-native vectors;
- resource measurements;
- accepted decisions or explicit target infeasibility;
- no attestation-contract operation release yet.

Guide 10 must not begin production `announce-maturity`, redemption, settlement,
or cycle emission until the relevant constructor and arithmetic prototypes have
passed their own target-native gates and have been adopted through the owning
research and decision process.
