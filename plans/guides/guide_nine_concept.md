# Guide 9 — Target-Native Primitive Conformance and Typed Tapscript Instruction Core

> **Status:** Concept — begins only after the Guide-8 exit gate passes
> **Phase:** 3 — Elements target and foundational prototypes
> **Primary packages:** `tripod-target-elements`, `tripod-tapscript`
> **Primary policies:** ADR-011, ADR-015, ADR-016, ADR-017
> **Primary decisions:** D001, D003, D004, D005, D006, D007, D008
> **Required environment:** Externally established secretless development environment with an explicitly selected Elements target
> **Non-claim:** This guide does not implement an attestation-contract operation, linked bundle, transaction ABI, deployment profile, or production release.

---

## Mission

Guide 8 establishes a typed static Elements target contract and an exhaustive adapter from compiler-required abstract capabilities to target obligations.

Guide 9 tests the next boundary:

```text
reviewed typed target claim
    ↓
typed tapscript instruction
    ↓
canonical target bytes
    ↓
synthetic target transaction context
    ↓
actual selected Elements execution
    ↓
typed verdict and resource observation
```

The guide has two deliverables:

1. **Target-native primitive conformance**
   - execute the reviewed primitive subset against an exact development target;
   - compare observed behavior with the typed target contract;
   - keep activation, opcode semantics, encodings, stack effects, failure effects, and resources as distinct claims.

2. **Typed tapscript instruction core**
   - represent backend instructions as typed values;
   - serialize them canonically;
   - validate stack and altstack behavior against target-owned contracts;
   - reject unsupported, malformed, ambiguous, and unreviewed instruction forms.

Guide 9 must not yet lower a realization relation into a complete target program.

At completion, the repository should possess an honest typed answer to:

> Do the target primitives described by `target-elements` encode and behave as declared on the selected development target, and can the tapscript package construct and statically validate those primitive instructions without introducing protocol semantics or target-position assumptions?

---

# 1. Executive rulings

## 1.1 Typed source remains authoritative

The authoritative direction is:

```text
reviewed target facts
    ↓
typed target definition
    ↓
typed tapscript instructions
    ↓
target-native conformance fixtures
```

Target-native results check the typed contract. They do not become semantic source.

The implementation must not:

- scrape node source at runtime;
- parse planning Markdown;
- infer an opcode contract from test output;
- rewrite typed expected behavior to match an unexpected target result;
- treat a development node as protocol authority.

If the selected target disagrees with the typed contract, the lane fails. The discrepancy is reviewed and resolved deliberately.

## 1.2 Primitive conformance is not backend-pattern conformance

A primitive behaving as declared does not prove that a composition of primitives enforces a compiler relation.

For example:

```text
input asset inspection works
+
output program inspection works
+
count inspection works
≠
authenticated object recognition is implemented
```

Likewise:

```text
fixed-width multiplication works
≠
wide floor arithmetic is implemented
```

Guide 9 may upgrade a target primitive claim from “typed” to “development-target tested.” It does not mark an attestation-contract backend pattern complete.

## 1.3 Development evidence is not production evidence

The Guide-9 target-native lane runs against an explicitly bound development instance.

It may establish:

- behavior on the exact selected development target;
- agreement between typed contract and observed development execution;
- test-environment resource observations.

It does not establish:

- production activation;
- production network policy;
- production node equivalence;
- production package relay;
- production resource limits;
- production deployment readiness.

## 1.4 No production secret enters the lane

The lane accepts no production:

- RPC credential;
- wallet credential;
- private key;
- signing nonce;
- blinding factor;
- private opening;
- release authority;
- deployment authority.

Public deterministic test vectors may contain values that are cryptographically private in form—such as a fixed test signing scalar—only when they are explicitly nonsecret, committed test fixtures with no production authority.

No public library or CLI introduces a legitimate secret-valued input.

## 1.5 The execution environment owns isolation

The repository does not sandbox the selected target.

The external environment owns:

- credential removal;
- loopback/network policy;
- process isolation;
- disposable data directories;
- temporary-file policy;
- agent-socket exclusion;
- worker destruction;
- crash-artifact policy;
- target executable provisioning.

The target executable path is an execution capability.

## 1.6 Exact selected bytes matter

Every target-native primitive fixture must execute the exact bytes emitted by the typed instruction serializer.

The lane must not:

- test hand-written bytes while production serialization remains untested;
- test a local mock instruction while claiming target execution;
- normalize unexpected bytes after serialization;
- accept a semantically similar opcode alias;
- run another target program than the one recorded by the fixture.

No persistent program digest is needed yet. The fixture may carry and compare exact bytes directly.

## 1.7 Success and failure stack effects are equally authoritative

An opcode contract includes:

- successful operand consumption;
- successful result stack;
- failure kind;
- failure stack where observable;
- altstack effect;
- malformed-input behavior;
- target verdict;
- resource effect.

Testing only valid execution is insufficient.

## 1.8 Canonical serialization is one-way

The backend constructs typed instructions and serializes them.

A parser may exist for:

- round-trip tests;
- independent byte inspection;
- target-result diagnostics.

Parsed arbitrary bytes are not automatically accepted as trusted backend programs.

## 1.9 No operation emission

Guide 9 may construct synthetic primitive programs such as:

```text
push operands
execute one reviewed opcode
normalize final truth
```

It must not construct:

- compact-ASH programs;
- live-transfer programs;
- STATE constructors;
- redemption programs;
- cycle programs;
- settlement programs;
- attestation-contract transaction layouts.

Synthetic fixtures test primitives, not protocol operations.

## 1.10 No new identity without admission

Guide 9 must not mint:

```text
InstructionId
ProgramId
PrimitiveVectorSetId
PrimitiveReportId
TargetExecutionReportHash
TargetProgramHash
```

unless a real persistent consumer requiring that identity is introduced in the same implementation series and satisfies ADR-016.

Typed values and exact-byte comparison remain sufficient.

---

# 2. Entry conditions

Guide 9 begins only after Guide 8 passes.

Expected entry state:

```text
tripod-target-elements:
    exists

typed target definition:
    implemented and validated

development deployment binding:
    implemented

target evidence-requirement registry:
    implemented

tripod-tapscript:
    exists

compiler capability adapter:
    exhaustive and validated

typed target programs:
    absent

target-native primitive evidence:
    absent

backend proof patterns:
    absent

transaction ABI:
    absent
```

Required Guide-8 properties:

- compiler target requirements come only from fully validated internal analysis;
- target-elements has no first-party semantic dependency;
- tapscript depends on compiler and target-elements;
- compiler depends on neither target package;
- target and development bindings remain distinct;
- production binding is unavailable;
- sponsor erasure remains role-based;
- no target, deployment, compiler, or evidence digest was minted.

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean.

Record:

```text
starting revision
Guide-8 gate revision
selected target contract version
development binding
```

before substantive implementation.

---

# 3. Required reading and authority

Read:

```text
AGENTS.md

adr/010-command-line-output-contract.md
adr/011-toolchain-and-dependency-policy.md
adr/014-meson-lint-census-and-stamps.md
adr/015-public-data-and-execution-trust.md
adr/016-semantic-identities-and-evidence-binding.md
adr/017-path-scope-and-host-filesystem-trust.md

plans/decisions/001-typed-rust-source.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/007-petgraph-graph-substrate.md
plans/decisions/008-exact-certified-mathematics.md

plans/packages/target-elements.md
plans/packages/tapscript.md
plans/packages/vectors.md
plans/phases/03-target-foundation.md
plans/reference/elements-tapscript.md
plans/guides/guide_eight_concept.md
```

Read the actual Guide-8 implementation:

```text
packages/target-elements/src/
packages/tapscript/src/
packages/compiler public target-requirement boundary
```

The human target reference remains non-authoritative.

The typed target contract owns expected primitive behavior.

The selected development target owns observed behavior for the exact run.

Neither silently overrides the other.

---

# 4. Scope and non-goals

## 4.1 In scope

Guide 9 implements:

- typed tapscript instruction values;
- typed push-data values;
- canonical instruction serialization;
- optional test-only decoding or independent byte inspection;
- typed program fragments for primitive tests;
- execution-domain validation;
- leaf-version validation;
- opcode availability checks;
- static main-stack analysis;
- static altstack analysis where relevant;
- conditional branch stack-join validation for the primitive fixture language;
- final truth normalization for test fragments;
- target-native fixture definitions;
- secretless development target-execution orchestration;
- exact selected-target provenance;
- positive and malformed primitive vectors;
- typed verdict classification;
- expected/actual stack-effect comparison where observable;
- resource observations for primitive fixtures;
- target evidence-requirement result classification;
- development-only primitive conformance summary;
- deterministic and permutation tests;
- package, reference, phase, and backlog updates.

## 4.2 Out of scope

Guide 9 must not implement:

- target-independent relation lowering;
- complete backend proof patterns;
- proof-pattern selection;
- concrete compiler relation placement;
- protocol coordinators;
- transaction family ranges;
- object constructors;
- STATE continuity;
- metadata commitments;
- wide floor arithmetic;
- confidential-to-public opening;
- compact-ASH operation emission;
- live-transfer operation emission;
- linker symbols or relocations;
- taptrees or control paths;
- transaction ABI;
- generalized wallet/signing interface;
- production RPC interface;
- batch calibration;
- production activation report;
- persistent target execution report identity;
- release integration.

---

# 5. Package and source organization

## 5.1 `target-elements`

Guide 9 may extend target-elements only where target-native evidence exposes a missing typed fact.

Likely additions:

```text
packages/target-elements/src/
    conformance.rs
```

A `conformance.rs` module should define immutable expected-evidence roles or development-result vocabulary only if target-elements is the correct owner.

It must not:

- invoke a node;
- construct attestation-contract programs;
- depend on tapscript;
- contain compiler capabilities;
- contain mutable global evidence status.

Prefer keeping target execution in tapscript tests or an explicitly test-support boundary so the target contract remains pure.

## 5.2 `tapscript`

Recommended additions:

```text
packages/tapscript/src/
    instruction.rs
    program.rs
    stack.rs
    encode.rs
    primitive.rs
```

Possible tests:

```text
packages/tapscript/src/tests/
    instruction_tests.rs
    encoding_tests.rs
    stack_tests.rs
    branch_tests.rs
    primitive_contract_tests.rs
    primitive_oracle_tests.rs
    mutation_tests.rs
    permutation_tests.rs
```

Integration support may live under:

```text
packages/tapscript/tests/
```

only if the ordinary Cargo lane can run hermetically without an Elements target.

Target-native execution requiring external software belongs to an explicit separate lane, not an ordinary always-required Cargo unit test.

## 5.3 Native target lane

Add a focused runner such as:

```text
scripts/test-elements-primitives.sh
```

or an equivalent explicitly owned integration target.

The runner:

- requires an explicit development target;
- uses a disposable target data directory;
- runs loopback-only where applicable;
- receives no production credentials;
- publishes no canonical release evidence;
- clearly separates infrastructure failure from target verdict;
- cleans up its child processes and temporary state;
- returns nonzero unless every required primitive fixture executes and matches.

If a Rust helper executable is added, it must follow ADR-010 and must receive every path by argument. Do not compile repository paths into it.

## 5.4 Meson integration

Do not make ordinary Cargo tests depend on an Elements installation.

Possible shape:

```text
ordinary CI:
    typed instruction and static contract tests

explicit target-native lane:
    selected Elements target required
    Guide-9 gate requires this lane
```

A conditional Meson test is acceptable only if:

- absence is reported as skipped;
- a required environment can make the skip fatal;
- a skip is never reported as passed;
- the target executable and development binding are explicit.

Suggested environment rule:

```text
CI_REQUIRE_ELEMENTS_PRIMITIVES=1
```

or an explicit script argument.

---

# 6. Target execution boundary

## 6.1 Preferred execution architecture

The preferred architecture is:

```text
typed primitive fixture
    ↓
tapscript serializer
    ↓
public synthetic transaction fixture
    ↓
externally provisioned development target
    ↓
typed raw execution result
    ↓
comparison with target-elements contract
```

The selected mechanism may use:

- an Elements daemon in disposable regtest mode;
- a reviewed upstream functional-test interface;
- another actual Elements script-execution path.

A local first-party interpreter may be useful as an additional oracle. It is not target-native evidence.

## 6.2 Credential-free first-party interface

If a daemon requires an ephemeral cookie or internal RPC credential:

- the external harness owns creation and storage;
- no first-party library API accepts the credential value;
- no command-line argument carries the credential value;
- no diagnostic emits it;
- the disposable directory is destroyed after the run;
- the run remains development-only.

A first-party script may invoke an externally provisioned CLI with a disposable data-directory path, allowing the child tools to resolve their own ephemeral authentication internally. The first-party interface must not read or publish the credential bytes.

If this cannot be achieved honestly, stop and design a reviewed secret-bearing execution boundary under ADR-015 before proceeding.

## 6.3 Target executable identity

Record as test provenance:

- executable name or safe typed role;
- reported node version;
- build information where available;
- network flavor;
- genesis;
- activation configuration;
- target contract version;
- test harness version or source revision.

Do not make the node version or source revision target semantic identity.

## 6.4 External program diagnostics

Raw stderr from an argument-selected target program is arbitrary child output.

A first-party CLI must either:

- deliberately relay it as documented child result data; or
- omit it and report typed status metadata.

It must not parse arbitrary stderr into a trusted semantic verdict unless the selected target interface explicitly defines that output as a typed protocol.

## 6.5 Infrastructure failures

Classify separately:

```text
target executable unavailable
target startup failure
target readiness timeout
target shutdown failure
RPC or transport unavailable
fixture construction failure
target execution infrastructure failure
policy rejection
consensus/script rejection
target acceptance
comparison mismatch
```

Infrastructure failure never counts as an expected target rejection.

---

# 7. Typed instruction core

## 7.1 Instruction identity

An instruction is a typed value, not a byte offset or target-program position.

A provisional model:

```rust
pub enum Instruction {
    Opcode(target_elements::OpcodeId),
    Push(PushData),
}
```

Do not include arbitrary raw opcodes:

```rust
Instruction::RawOpcode(u8)
```

in the validated program path.

If a test requires unknown-opcode mutation, keep it in a deliberately unvalidated malformed-byte fixture type.

## 7.2 Push data

```rust
pub struct PushData {
    bytes: Vec<u8>,
}
```

Construction validates the target’s element-size and canonical-push rules.

Typed convenience constructors may exist:

```rust
PushData::boolean(...)
PushData::script_number(...)
PushData::signed_fixed_width(...)
PushData::encoded(...)
```

They must use target-owned encoding contracts.

Do not add protocol-specific constructors such as:

```text
PushData::ash_value
PushData::state_omega
PushData::receipt_owner
```

## 7.3 Program fragment

A primitive test program is:

```rust
pub struct ProgramFragment {
    instructions: Vec<Instruction>,
}
```

Fields remain private.

Construction validates:

- every opcode exists in the target definition;
- every opcode is admitted in the selected execution domain;
- every push is canonical;
- each element respects target limits;
- the fragment contains no unsupported raw byte;
- serialization is deterministic.

## 7.4 Validated fragment

Use an opaque validated wrapper:

```rust
pub struct ValidatedProgramFragment {
    fragment: ProgramFragment,
    analysis: StackAnalysis,
}
```

Only validated fragments may be serialized for the target-native lane.

A malformed-byte fixture is another type and cannot be converted into a validated fragment.

## 7.5 No protocol program identity

A fragment may be compared by typed value and exact bytes.

Do not add a fragment digest or persistent program ID.

---

# 8. Canonical serialization

## 8.1 Serializer ownership

`tapscript` owns instruction serialization.

`target-elements` owns:

- opcode byte values;
- push and encoding rules;
- execution-domain legality.

## 8.2 Serialization function

Conceptually:

```rust
pub fn serialize_fragment(
    target: &target_elements::ElementsTarget,
    fragment: &ValidatedProgramFragment,
) -> Result<Vec<u8>, TapscriptError>;
```

The serializer must be pure and deterministic.

## 8.3 Canonical pushes

Use one target-owned canonical push rule.

Reject:

- nonminimal push opcode;
- ambiguous empty/false encoding;
- oversized element;
- malformed fixed-width field;
- unknown encoding prefix;
- unsupported target field form.

Do not assume that every byte vector has one protocol-valid semantic interpretation.

## 8.4 Independent byte vectors

For every reviewed opcode:

- store its expected byte explicitly in tests;
- compare serializer output with that byte;
- do not derive expected bytes through the production opcode lookup.

For push boundaries, use explicit vectors for:

- empty;
- one byte;
- small direct-push boundaries;
- larger length-prefix boundaries used by the target;
- maximum accepted element;
- one above maximum;
- nonminimal alternatives.

## 8.5 Decoder or inspector

A test-only decoder may validate:

```text
typed instruction
→ bytes
→ independently inspected opcode/push sequence
```

The decoder must not become semantic source.

Unknown or malformed bytes return a typed inspection failure.

## 8.6 No post-serialization mutation

Validated program bytes are immutable after serialization.

A later stage needing relocations must operate on a typed relocatable representation, not patch bytes here.

---

# 9. Static stack and altstack validation

## 9.1 Purpose

Static validation proves that a typed primitive fragment is internally consistent with the target-owned stack contracts.

It does not prove target execution.

## 9.2 Stack state

A conceptual state:

```rust
pub struct StackState {
    main: Vec<StackValueType>,
    alt: Vec<StackValueType>,
}
```

Local positions are analysis handles only.

They do not enter semantic identity or public stable keys.

## 9.3 Instruction transfer

For each instruction:

1. validate operand availability;
2. validate operand types and widths;
3. apply successful stack contract;
4. record possible failure behavior;
5. validate altstack effects;
6. validate stack-item and element-size limits;
7. update peak-resource observations.

## 9.4 Branches

If the primitive fixture language admits conditional branches, every branch join must agree on:

- main-stack height;
- main-stack types;
- altstack height;
- altstack types;
- final truth discipline;
- resource bounds.

A branch mismatch is a typed construction failure.

Guide 9 does not need a general control-flow optimizer.

## 9.5 Final truth

A target-native positive fixture should terminate with one canonical truth result under the selected target rules.

A target-negative fixture may fail by:

- script abort;
- false final result;
- target transaction rejection.

The expected failure class must be stated per fixture.

## 9.6 Failure-path stack effects

Where the target exposes failure stack behavior rather than immediately aborting, static validation must retain both paths.

For example, an arithmetic instruction that preserves operands and pushes false cannot be modeled as:

```text
on failure:
    abort
```

The typed target contract, static analyzer, and target-native fixture must agree.

---

# 10. Primitive fixture model

## 10.1 Fixture identity

A fixture is a complete typed value:

```rust
pub struct PrimitiveFixture {
    name: PrimitiveFixtureName,
    target_contract_version: TargetContractVersion,
    execution_domain: ExecutionDomain,
    leaf_version: LeafVersion,
    initial_stack: Vec<FixtureStackValue>,
    fragment: ValidatedProgramFragment,
    context: PrimitiveExecutionContext,
    expected: ExpectedPrimitiveOutcome,
}
```

`PrimitiveFixtureName` is a stable test key, not a digest.

A test key may be an enum.

## 10.2 Execution context

Primitive contexts may include:

```rust
pub enum PrimitiveExecutionContext {
    PureStack,
    InputIntrospection(SyntheticInputContext),
    OutputIntrospection(SyntheticOutputContext),
    TransactionIntrospection(SyntheticTransactionContext),
    Signature(SyntheticSignatureContext),
    RelativeTimelock(SyntheticTimelockContext),
    ConfidentialValue(SyntheticConfidentialContext),
    Issuance(SyntheticIssuanceContext),
}
```

These are target test contexts, not attestation-contract operation layouts.

## 10.3 Expected outcome

```rust
pub enum ExpectedPrimitiveOutcome {
    Accept {
        final_stack: Vec<ExpectedStackValue>,
        final_altstack: Vec<ExpectedStackValue>,
    },

    Reject {
        class: ExpectedTargetRejection,
    },
}
```

Do not use one Boolean for all outcomes.

## 10.4 Raw observed outcome

```rust
pub enum ObservedPrimitiveOutcome {
    Accepted {
        final_stack: Option<Vec<Vec<u8>>>,
        resources: ObservedResources,
    },

    ScriptRejected {
        reason: TargetRejectionReason,
        resources: Option<ObservedResources>,
    },

    PolicyRejected {
        reason: TargetPolicyReason,
    },

    InfrastructureFailed {
        class: InfrastructureFailure,
    },
}
```

The actual target interface may not expose the final stack. If not, report it as unavailable rather than synthesizing it.

## 10.5 Comparison

A typed comparison establishes:

- target context matches;
- exact fragment bytes match the fixture;
- expected accept/reject class matches;
- expected observable stack result matches where available;
- expected resource dimensions match or remain explicitly unmeasured;
- no infrastructure failure is misclassified.

---

# 11. Vector classes

## 11.1 Positive primitive vectors

For each primitive:

- minimum valid operands;
- representative operands;
- boundary valid operands;
- exact target bytes;
- expected stack result;
- expected target acceptance;
- expected resources where declared.

## 11.2 Malformed encoding vectors

Test:

- wrong width;
- noncanonical encoding;
- unknown prefix;
- truncated payload;
- extra payload;
- wrong byte order;
- invalid signed form;
- invalid script-number form;
- oversized element.

## 11.3 Stack vectors

Test:

- missing operand;
- wrong operand order;
- wrong operand type;
- extra irrelevant stack item where behavior matters;
- altstack mismatch;
- branch-join mismatch;
- noncanonical final truth.

## 11.4 Domain vectors

Test:

- opcode in admitted tapscript domain;
- opcode outside admitted domain;
- wrong leaf version;
- relevant OP_SUCCESS treatment;
- disabled or unreviewed opcode byte;
- unknown opcode.

## 11.5 Resource vectors

Test:

- script byte count;
- witness byte count;
- initial stack count;
- peak main-stack count;
- peak altstack count;
- maximum element size;
- crypto budget;
- target operation cost;
- transaction weight where the target fixture exposes it.

These are primitive observations, not protocol calibration.

---

# 12. Introspection primitive plan

## 12.1 Input introspection

Review and test the admitted subset of:

```text
input outpoint
input asset
input value
input program
input sequence
input issuance
current input index
```

For each:

- exact opcode byte;
- input-index operand contract;
- valid index boundaries;
- out-of-range behavior;
- explicit/confidential result form;
- result width;
- result byte order;
- malformed context behavior;
- resource cost.

## 12.2 Output introspection

Review and test:

```text
output asset
output value
output nonce
output program
```

Required cases:

- first and last valid output index;
- one above output count;
- explicit asset;
- confidential asset;
- explicit value;
- confidential value;
- null nonce;
- confidential nonce;
- native program;
- non-native program where target behavior differs.

No attestation-contract closed-asset policy is applied in target-elements. That policy remains downstream.

## 12.3 Transaction introspection

Review and test:

```text
version
locktime
input count
output count
transaction weight
```

The typed target contract must state exact result encodings and failure behavior.

## 12.4 Count is not family authentication

Guide 9 should include a permanent explanatory regression:

```text
transaction input count inspection works
≠
a protocol family census is authenticated
```

The tapscript adapter must remain at `BackendStructural` or `BackendPatternRequired` for compiler family-cardinality capabilities.

---

# 13. Arithmetic primitive plan

## 13.1 Scope

Guide 9 tests the reviewed narrow arithmetic primitives only.

It does not implement:

\[
q=\left\lfloor \frac{ab}{d}\right\rfloor.
\]

That remains wide-arithmetic research.

## 13.2 Required arithmetic facts

For every selected fixed-width operation, test:

- exact operand width;
- signed interpretation;
- byte order;
- operand stack order;
- result stack order;
- successful result;
- overflow behavior;
- division-by-zero behavior;
- failure flag behavior;
- retained operands after failure;
- malformed-width behavior;
- resource cost.

## 13.3 Boundary values

For signed 64-bit operations, include as applicable:

```text
0
1
-1
minimum
maximum
minimum + 1
maximum - 1
```

For protocol-domain positive values, include:

```text
2^51 - 1
2^51
```

The target may compute both; the protocol domain remains a separate compiler/backend check.

## 13.4 Success-flag discipline

If an arithmetic primitive pushes a success flag, primitive conformance must prove its exact stack position and value.

Guide 9 should add a typed fragment showing:

```text
arithmetic operation
success flag consumed and required true
```

This is a primitive-control fixture, not a wide arithmetic proof.

## 13.5 Independent oracle

Expected arithmetic results use host checked arithmetic with a separately written reference.

Do not use the target serializer or target opcode implementation to compute expected results.

---

# 14. Hash and byte-operation plan

## 14.1 Streaming SHA-256

If admitted, test:

- initialize;
- zero updates where target contract permits;
- one update;
- several updates;
- finalize;
- chunk boundaries;
- empty message;
- known standard vectors;
- malformed context;
- wrong state width;
- resource costs.

Compare against a reviewed independent SHA-256 implementation or fixed known vectors.

## 14.2 Byte concatenation and slicing

If used by the reviewed target subset, record and test:

- operand order;
- exact length behavior;
- maximum element;
- overflow or size rejection;
- empty operand behavior;
- resulting stack shape.

Do not add generic byte operations merely because later constructors may need them.

---

# 15. Elliptic-curve and tweak primitive plan

## 15.1 Scope

Guide 9 may test low-level EC or tweak primitives that Guide 8 admitted.

It must not claim a STATE constructor.

## 15.2 Public deterministic fixtures

Use fixed, published, explicitly nonsecret test values.

The fixture must not expose an API for arbitrary private-key input.

## 15.3 Required vectors

As applicable:

- valid point/scalar relation;
- wrong scalar;
- wrong point;
- malformed point;
- invalid scalar encoding;
- boundary scalar;
- wrong parity;
- wrong tweak;
- tweak result mismatch;
- target failure behavior;
- crypto budget.

## 15.4 Non-claim

Passing tweak verification means:

```text
the primitive behaves as typed
```

It does not mean:

```text
metadata-dependent constructor continuity is implemented
```

That remains `state-constructor.md`.

---

# 16. Signature primitive plan

## 16.1 Scope

Guide 9 tests:

- signature opcode encoding;
- stack contract;
- valid known signature fixture;
- invalid signature fixture;
- malformed public key;
- malformed signature;
- empty signature behavior;
- selected sighash dimensions where the target test context exposes them;
- resource and crypto-budget behavior.

It does not implement a wallet or signer.

## 16.2 Test signing material

Any signing scalar used to construct a fixed test transaction must be:

- synthetic;
- publicly committed in the test fixture;
- explicitly nonsecret;
- incapable of authorizing production funds;
- absent from public production APIs;
- excluded from diagnostics where reproducing it serves no purpose.

A precomputed public transaction/signature vector is preferred where practical.

## 16.3 Output-commitment mutation

For a selected test sighash profile:

1. build a valid synthetic signed transaction;
2. execute successfully;
3. mutate one protected output;
4. retain the original signature;
5. require target rejection.

This proves development-target behavior for that exact fixture and profile.

It does not select the attestation-contract production sighash profile.

## 16.4 Input-extension behavior

If the reviewed profile permits or forbids input extension, test that dimension separately from output commitment.

Do not summarize both as “signature works.”

---

# 17. Relative-timelock primitive plan

## 17.1 Scope

Guide 9 tests target relative-timelock semantics needed by the future cadence backend.

It does not implement the cadence band.

## 17.2 Required dimensions

Test:

- transaction-version prerequisite;
- sequence disabled flag;
- block-based mode;
- time-based mode if admitted;
- one below minimum age;
- exactly minimum age;
- one above minimum age;
- wrong input sequence;
- wrong transaction version;
- target failure behavior;
- policy versus consensus result where distinguishable.

## 17.3 Development chain control

The external target harness owns block generation and target time advancement.

The repository must not use ambient wall-clock time as a canonical input.

Use explicit deterministic development-chain actions.

## 17.4 Non-claim

Passing relative-timelock vectors establishes the primitive boundary only.

The three-regime cadence relation remains a future backend pattern:

```text
before MIN:
    invalid

MIN ≤ age < MAX:
    operator

age ≥ MAX:
    permissionless
```

---

# 18. Confidential-value and issuance primitive plan

## 18.1 Confidential value conservation

If Guide 8 admitted the target capability, Guide 9 should execute synthetic target transactions covering:

- explicit input/output conservation;
- confidential input/output conservation;
- explicit/confidential mixed forms where target-valid;
- one-unit imbalance;
- malformed proof;
- incorrect blinding balance;
- wrong asset generator;
- policy and consensus verdicts.

The target contract remains asset/value-axis aware.

## 18.2 Commitment equality

Test exact commitment equality behavior if a reviewed target primitive exists.

Do not infer authenticated opening from commitment equality.

## 18.3 Sponsor opacity

No test should require the protocol to decode sponsor values.

Synthetic transactions may use target-known values to construct valid confidential transactions, but target test construction knowledge is not projected as a protocol fact.

## 18.4 Issuance and reissuance

Test reviewed target behavior for:

- no issuance;
- issuance present;
- issuance amount forms;
- asset entropy or ID derivation where in scope;
- reissuance field;
- input issuance introspection;
- malformed issuance;
- transaction commitment.

Do not map target issuance directly to `U`, `ENT`, or `DIST_CTL`.

## 18.5 Non-claim

Target issuance primitives behaving as typed do not establish attestation-contract issuance discipline.

The future backend must still prove:

- correct authority;
- correct operation;
- correct amount;
- complete destination exhaustion;
- no extra issuance.

---

# 19. Resource observation

## 19.1 Primitive-level only

Guide 9 records resources for synthetic primitive fixtures.

It must not derive final values for:

```text
ADMISSION_BATCH_MAX
SETTLEMENT_BATCH_MAX
RELABEL_BATCH_MAX
ASH_BATCH_MAX
BURN_INPUT_MAX
BURN_CHANGE_MAX
BURN_RECORD_MAX
TRANSFER_INPUT_MAX
TRANSFER_OUTPUT_MAX
FEE_SPONSOR_INPUT_MAX
```

## 19.2 Predicted and observed resources

For each fixture, compare applicable:

```text
serialized script bytes
witness bytes
initial stack items
peak stack items
peak altstack items
maximum element bytes
crypto budget
target operation cost
transaction weight
policy verdict
```

## 19.3 Exact units

Use checked integers.

Do not combine unlike dimensions into a floating or weighted total.

## 19.4 Resource mismatches

A mismatch may mean:

- typed opcode resource contract wrong;
- serializer wrong;
- static stack analyzer wrong;
- target environment differs;
- execution fixture contains unaccounted overhead;
- target policy claim is incomplete.

Do not “fix” a mismatch by weakening the observed requirement without identifying the cause.

## 19.5 No target resource identity

Resource observations are typed test results.

No resource report digest is minted in Guide 9.

---

# 20. Evidence and report separation

## 20.1 Evidence classes

Keep separate:

```text
typed static target contract
instruction serialization tests
static stack-contract tests
development target-native opcode tests
development activation tests
development signature/sighash tests
development relative-timelock tests
development CT tests
primitive resource observations
backend-pattern evidence
operation evidence
production deployment evidence
```

Guide 9 implements only the first eight where in scope.

## 20.2 Target evidence requirement status

A development run may report:

```rust
pub enum DevelopmentEvidenceStatus {
    Passed,
    Failed,
    Unsupported,
    InfrastructureFailure,
    NotRun,
}
```

This status lives in the run result, never in `TargetDefinition`.

## 20.3 Typed run result

A conceptual internal result:

```rust
pub struct PrimitiveConformanceResult {
    fixture: PrimitiveFixtureName,
    target_contract_version: TargetContractVersion,
    expected: ExpectedPrimitiveOutcome,
    observed: ObservedPrimitiveOutcome,
    comparison: PrimitiveComparison,
}
```

## 20.4 No persistent canonical report required

Guide 9 may keep conformance results inside the test process and record the gate command and aggregate outcome in planning/Git history.

Do not add a generated JSON report merely because results exist.

A persistent report schema and identity belong to the future vectors/evidence package when a real release consumer exists.

## 20.5 Failure behavior

The native lane fails if any required fixture is:

- not run;
- unsupported unexpectedly;
- infrastructure-failed;
- accepted when expected rejected;
- rejected when expected accepted;
- executed under a mismatched target context;
- resource-mismatched where exact comparison is required.

---

# 21. Capability-adapter update

## 21.1 Static assessment remains static

Guide 9 must not mutate the Guide-8 adapter’s static assessment because a development test passed.

For example:

```text
BackendPatternRequired
```

does not become:

```text
CompleteBackendPattern
```

merely because all primitive prerequisites passed individually.

## 21.2 Development evidence overlay

If useful, add a separate typed overlay:

```rust
pub struct DevelopmentCapabilityEvidence {
    capability: target_elements::ElementsCapability,
    requirements:
        BTreeMap<TargetEvidenceRequirementId, DevelopmentEvidenceStatus>,
}
```

This overlay answers:

```text
Were this target capability’s primitive claims tested on the selected
development target?
```

It does not answer:

```text
Does a complete backend proof pattern implement the compiler capability?
```

## 21.3 Exact census

Require:

```text
tested target capability/evidence census
⊆
typed target capability/evidence census
```

and, for Guide-9-required capabilities:

```text
required development evidence census
=
executed development evidence census
```

No unexpected evidence role is accepted.

## 21.4 Unsupported primitives

If a reviewed primitive is unavailable or behaves differently:

- return a typed unsupported or mismatch result;
- preserve every compiler requirement;
- update target capability status only after review;
- do not invent a weaker proof.

---

# 22. Independent oracles

## 22.1 Instruction-byte oracle

Maintain an independent expected byte table for every opcode tested.

The production serializer must not provide its own expected bytes.

## 22.2 Push-encoding oracle

Use explicit fixed vectors and, where practical, a separately implemented minimal encoder in tests.

## 22.3 Stack oracle

For bounded generated primitive fragments, compare the production stack analyzer with a simpler reference transfer function.

The reference must not call the production stack-step helper.

## 22.4 Arithmetic oracle

Use checked host arithmetic or arbitrary-precision integers for expected fixed-width results and overflow classification.

## 22.5 Hash oracle

Use standard known vectors or a separately reviewed implementation.

## 22.6 Target comparison

The actual target remains the independent execution boundary for the selected development environment.

A second local simulator invocation is not independent target-native evidence.

## 22.7 Resource oracle

For serializer-derived dimensions such as script bytes, compare:

- direct byte length;
- static predicted length.

For target-reported dimensions, compare with independently parsed target output where the interface permits.

---

# 23. Determinism and reproducibility

## 23.1 Typed construction

Equal explicit inputs produce equal:

- instructions;
- program fragments;
- serialized bytes;
- static stack analyses;
- primitive fixture definitions;
- expected outcomes.

## 23.2 Native execution context

Target-native results are context-bound to:

- target contract version;
- development environment;
- network ID;
- genesis ID;
- activation declaration;
- selected target implementation provenance;
- explicit chain fixture state;
- exact fragment bytes.

## 23.3 Ambient exclusions

Canonical fixture definitions must not depend on:

- wall-clock time;
- hostname;
- username;
- process ID;
- random temporary path;
- filesystem enumeration;
- hash-map order;
- locale;
- thread schedule.

## 23.4 Explicit randomness

If confidential transaction construction requires randomness:

- inject it explicitly;
- use fixed clearly test-only randomness in canonical fixtures;
- do not reuse deterministic test randomness in production;
- include the explicit test randomness in fixture input, not in semantic identity.

## 23.5 Repeated execution

Run selected primitive fixtures more than once against reset equivalent development contexts.

Unexpected byte or verdict differences fail the lane.

---

# 24. Error vocabulary

## 24.1 Tapscript instruction errors

Likely additions:

```rust
pub enum TapscriptError {
    UnsupportedTargetContractVersion,

    UnknownOpcode(target_elements::OpcodeId),
    OpcodeUnavailableInDomain {
        opcode: target_elements::OpcodeId,
        domain: target_elements::ExecutionDomain,
    },

    InvalidPushEncoding,
    NonCanonicalPush,
    StackElementTooLarge,
    ScriptTooLarge,

    StackUnderflow {
        instruction: usize,
    },
    StackTypeMismatch {
        instruction: usize,
    },
    AltstackUnderflow {
        instruction: usize,
    },
    AltstackTypeMismatch {
        instruction: usize,
    },
    BranchStackMismatch,
    NonCanonicalFinalTruth,

    MissingTargetPrimitive(
        target_elements::ElementsCapability,
    ),

    PrimitiveFixtureInvalid(PrimitiveFixtureName),
    PrimitiveExpectedActualMismatch(PrimitiveFixtureName),
    PrimitiveResourceMismatch(PrimitiveFixtureName),
}
```

Do not expose instruction vector indexes as stable semantic IDs. A local instruction position may appear in a diagnostic as ephemeral context, not as public identity.

## 24.2 Native-lane errors

If a Rust runner owns them:

```rust
pub enum PrimitiveRunnerError {
    TargetExecutableUnavailable,
    TargetStartupFailed,
    TargetReadinessTimedOut,
    TargetContextMismatch,
    FixtureConstructionFailed(PrimitiveFixtureName),
    TargetInfrastructureFailed(PrimitiveFixtureName),
    ExpectedAcceptActualReject(PrimitiveFixtureName),
    ExpectedRejectActualAccept(PrimitiveFixtureName),
    TargetResultMalformed(PrimitiveFixtureName),
    ResourceObservationUnavailable(PrimitiveFixtureName),
    ResourceObservationMismatch(PrimitiveFixtureName),
    TargetShutdownFailed,
}
```

A shell lane may use equivalent explicit branches and nonzero status.

## 24.3 No catch-all semantic fallback

Avoid public:

```rust
Other(String)
```

for target semantic failures.

External I/O may retain a source error internally, but release-relevant classification remains typed.

---

# 25. Source-review update

## 25.1 Upgrade evidence levels honestly

For each primitive tested, update the reference table from:

```text
surveyed
```

to applicable levels:

```text
reviewed
typed
development-target tested
```

Do not write:

```text
pattern-complete
deployment-evidenced
```

unless those stronger boundaries actually exist.

## 25.2 Review provenance fields

Record:

- upstream repository;
- revision consulted;
- source locations;
- test locations;
- selected node/library version;
- development network;
- activation configuration;
- exact claims tested;
- exact claims not tested.

## 25.3 Typed target handoff

For each target fact, identify:

```text
typed owner
target-native fixture
expected success behavior
expected failure behavior
resource rule
evidence requirement
remaining backend obligation
```

---

# 26. Suggested implementation waves

## Wave 0 — Execution-boundary design and dependency review

Deliver:

- selected target-native execution mechanism;
- exact target tool/library review;
- license and MSRV review;
- secretless execution design;
- development network/genesis binding;
- activation setup;
- infrastructure-failure taxonomy;
- explicit list of primitive groups included;
- explicit list deferred.

Do not write the native runner before the security and execution boundary is clear.

Suggested commit:

```text
plans: define the primitive conformance execution boundary
```

## Wave 1 — Typed instruction and push model

Deliver:

- `Instruction`;
- `PushData`;
- private validated constructors;
- opcode/domain checks;
- canonical push policy;
- explicit byte-vector tests;
- no arbitrary raw opcode in the validated path.

Suggested commit:

```text
tapscript: add typed primitive instructions
```

## Wave 2 — Canonical serializer

Deliver:

- exact opcode serialization;
- canonical push serialization;
- independent expected byte vectors;
- malformed-byte fixture type;
- deterministic round-trip or inspection tests.

Suggested commit:

```text
tapscript: serialize instructions canonically
```

## Wave 3 — Static stack validator

Deliver:

- main-stack model;
- altstack model where required;
- per-instruction transfer;
- success/failure path distinction;
- branch joins;
- final truth discipline;
- independent stack oracle.

Suggested commit:

```text
tapscript: validate primitive stack contracts
```

## Wave 4 — Primitive fixture registry

Deliver:

- typed fixture names;
- complete contexts;
- expected outcomes;
- target contract bindings;
- exact fragment bytes;
- fixture census and duplicate rejection;
- deterministic projections.

Suggested commit:

```text
tapscript: define target primitive fixtures
```

## Wave 5 — Secretless target-native runner

Deliver:

- explicit development-target invocation;
- disposable data directory;
- no production credentials;
- target readiness and cleanup;
- typed result classification;
- infrastructure failures separate from target rejections;
- exact context validation.

Suggested commit:

```text
scripts: add the Elements primitive conformance lane
```

If a first-party executable is added:

```text
tapscript: add the primitive conformance runner
```

with ADR-010 compliance.

## Wave 6 — Introspection, arithmetic, and hashing vectors

Deliver:

- input/output/transaction introspection fixtures;
- narrow arithmetic and comparison fixtures;
- success-flag behavior;
- streaming hash fixtures where admitted;
- malformed and boundary vectors;
- resource observations.

Suggested commit:

```text
tapscript: verify introspection and arithmetic primitives
```

## Wave 7 — Signature, timelock, CT, and issuance vectors

Deliver as far as the reviewed target supports:

- signature and output-mutation fixtures;
- relative-timelock boundaries;
- CT conservation fixtures;
- commitment-equality fixtures;
- issuance and reissuance introspection fixtures;
- explicit unsupported results for unimplemented claims.

Suggested commit:

```text
tapscript: verify authorization and substrate primitives
```

## Wave 8 — Adapter overlay and documentation

Deliver:

- development evidence overlay, if needed;
- exact evidence census;
- no backend-pattern completion;
- updated target reference;
- updated package contracts;
- updated Phase-3 card;
- Guide-9 gate record.

Suggested commit:

```text
plans: record the target primitive conformance gate
```

---

# 27. Focused tests

## 27.1 Target-elements

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

## 27.2 Tapscript typed core

```sh
cargo test --locked -p tripod-tapscript instruction
cargo test --locked -p tripod-tapscript encoding
cargo test --locked -p tripod-tapscript stack
cargo test --locked -p tripod-tapscript primitive
cargo test --locked -p tripod-tapscript oracle
cargo test --locked -p tripod-tapscript
```

Verify each filter matches at least one test.

## 27.3 Public API

```sh
cargo test --locked -p tripod-tapscript --test public_api
cargo test --locked -p tripod-target-elements --test public_api
```

Required public-boundary checks:

- no arbitrary validated raw-byte constructor;
- no target program identity;
- no persistent report identity;
- no production deployment constructor;
- no secret-bearing input;
- no attestation-contract operation type in target-elements;
- no compiler dependency in target-elements.

## 27.4 Target-native lane

Conceptually:

```sh
scripts/test-elements-primitives.sh \
  --target <externally-provisioned-elements-program> \
  --target-cli <externally-provisioned-cli-program> \
  --development-root <disposable-root>
```

The actual command must follow the selected execution design.

Guide-9 exit requires the native lane to run, not skip.

---

# 28. Full gate

After all waves:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --release --locked
```

Run the target-native primitive lane in a secretless isolated environment:

```sh
scripts/test-elements-primitives.sh ...
```

Run repository gates:

```sh
scripts/check-plans.sh
meson compile -C build lint
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Because Guide 9 adds source files and a script or explicit test lane, the Meson census audit and mocked graph contract remain required.

Run:

```sh
cargo audit
```

when installed.

If unavailable, record:

```text
SKIPPED
```

not passed.

Run:

```sh
git diff --check
git diff --cached --check
git status --porcelain=v1 --untracked-files=all
```

The final tree must be clean.

Document reproducibility may be deferred if no paper/publication inputs changed, but the gate record must say `DEFERRED`, not passed.

---

# 29. Identity and generated-publication impact

Expected:

```text
Attestation version:
    unchanged

realization version:
    unchanged unless normative realization prose is deliberately corrected

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

realization identity:
    none exists

compiler identity:
    none minted

target-definition digest:
    none minted

program digest:
    none minted

primitive-vector digest:
    none minted

primitive-report digest:
    none minted

linked bundle:
    absent

transaction ABI:
    absent
```

Generated architecture publications should remain byte-identical.

`model_labels.json` may change only if deliberate participating model Rust labels change. Tapscript and target-elements labels belong to their package owners and should not alter the model-label publication.

Do not add a generated target-definition JSON merely to inspect the type.

---

# 30. Guide-9 exit criteria

Guide 9 is complete only when all applicable assertions hold.

## Execution boundary

- [ ] exact development target is selected;
- [ ] target source/version provenance is recorded;
- [ ] external execution environment is secretless;
- [ ] disposable target state is used;
- [ ] no production credential enters first-party input;
- [ ] infrastructure failure is distinct from target rejection;
- [ ] target processes are cleaned up;
- [ ] target context matches the typed development binding.

## Typed instructions

- [ ] instruction values are typed;
- [ ] push data is canonical;
- [ ] arbitrary raw opcodes cannot enter validated fragments;
- [ ] unsupported opcode/domain combinations fail;
- [ ] validated fragment construction is private;
- [ ] no operation semantics appear in instruction types;
- [ ] no instruction or program digest is minted.

## Serialization

- [ ] every tested opcode serializes to independently specified bytes;
- [ ] push boundaries have fixed expected vectors;
- [ ] nonminimal pushes reject;
- [ ] unknown opcodes reject;
- [ ] serialization is deterministic;
- [ ] no post-serialization patching occurs.

## Stack validation

- [ ] main-stack effects are typed;
- [ ] altstack effects are typed where relevant;
- [ ] underflow fails;
- [ ] operand type mismatch fails;
- [ ] success and failure paths remain distinct;
- [ ] branch joins require equal typed stacks;
- [ ] final truth discipline is explicit;
- [ ] production and oracle stack analyses agree.

## Primitive fixtures

- [ ] fixture census is complete for the selected primitive subset;
- [ ] fixture names are unique;
- [ ] each fixture binds the target contract and context;
- [ ] exact serialized bytes are retained;
- [ ] expected outcomes are typed;
- [ ] malformed vectors cannot masquerade as validated programs;
- [ ] fixture ordering does not affect results.

## Native conformance

- [ ] execution-domain fixtures pass;
- [ ] leaf-version fixtures pass;
- [ ] opcode-byte fixtures pass;
- [ ] success-stack behavior matches where observable;
- [ ] failure behavior matches;
- [ ] malformed-width behavior matches;
- [ ] index-boundary behavior matches;
- [ ] target acceptance/rejection matches every required fixture;
- [ ] no infrastructure failure is counted as a semantic pass.

## Primitive groups

- [ ] selected input introspection primitives pass;
- [ ] selected output introspection primitives pass;
- [ ] selected transaction introspection primitives pass;
- [ ] selected arithmetic/comparison primitives pass;
- [ ] selected hash primitives pass;
- [ ] selected signature/sighash fixtures pass or remain explicitly unsupported;
- [ ] selected timelock fixtures pass or remain explicitly unsupported;
- [ ] selected CT fixtures pass or remain explicitly unsupported;
- [ ] selected issuance fixtures pass or remain explicitly unsupported.

## Resource observations

- [ ] script bytes agree;
- [ ] witness bytes agree where measured;
- [ ] stack/altstack maxima agree;
- [ ] element-size rules agree;
- [ ] crypto or target-operation costs agree where the interface exposes them;
- [ ] consensus and policy verdicts remain separate;
- [ ] no protocol batch bound is calibrated.

## Adapter boundary

- [ ] static Guide-8 capability assessments remain unchanged in meaning;
- [ ] development evidence is a separate overlay;
- [ ] evidence census is exact;
- [ ] no primitive pass is described as a complete backend pattern;
- [ ] no compiler capability disappears;
- [ ] no sponsor amount requirement appears;
- [ ] no production target support is claimed.

## Identity and publication

- [ ] no target digest is minted;
- [ ] no program digest is minted;
- [ ] no report digest is minted;
- [ ] no canonical target report publication is added without a consumer;
- [ ] review provenance stays outside semantic projection;
- [ ] local stack positions and graph handles stay outside stable projections.

## Repository gate

- [ ] focused package tests pass;
- [ ] Rustdoc passes with warnings denied;
- [ ] workspace formatting passes;
- [ ] workspace Clippy passes;
- [ ] debug workspace tests pass;
- [ ] release workspace tests pass;
- [ ] target-native primitive lane runs and passes;
- [ ] labels and generated checks pass;
- [ ] mocked Meson contract passes;
- [ ] real Meson compile passes;
- [ ] real Meson tests pass;
- [ ] dependency and licence review is recorded;
- [ ] advisory lane passes or is loudly skipped;
- [ ] deferred lanes are reported honestly;
- [ ] final tree is clean.

---

# 31. Completion report template

```text
Guide 9 result
==============

Entry:
    starting revision:
    Guide-8 gate:
    target contract version:
    development binding:

Execution boundary:
    selected target:
    target version/build:
    reviewed source revision:
    development network:
    genesis:
    activation:
    target invocation:
    credential handling:
    isolation:
    disposable state:
    cleanup:

Dependency impact:
    new first-party packages:
    new third-party dependencies:
    selected versions:
    features:
    licence:
    MSRV:
    unsafe boundary:
    native dependencies:
    Cargo.lock:
    advisories:

Typed instruction core:
    instruction type:
    validated fragment:
    push policy:
    arbitrary raw opcode:
    canonical serialization:
    decoder/inspector:
    program identity:
        none

Stack validation:
    main stack:
    altstack:
    branch joins:
    failure paths:
    final truth:
    independent oracle:

Primitive fixture census:
    execution-domain fixtures:
    introspection fixtures:
    arithmetic fixtures:
    hash fixtures:
    EC/tweak fixtures:
    signature fixtures:
    timelock fixtures:
    CT fixtures:
    issuance fixtures:
    total positive:
    total negative:

Target-native results:
    fixtures passed:
    fixtures rejected as expected:
    unsupported:
    mismatches:
    infrastructure failures:
    repeated-run equality:

Resource observations:
    script bytes:
    witness bytes:
    stack:
    altstack:
    element size:
    crypto budget:
    operation cost:
    transaction weight:
    consensus/policy distinction:
    calibrated protocol bounds:
        none

Capability-adapter result:
    static assessments changed:
        no, except reviewed target-contract corrections
    development evidence overlay:
    complete backend patterns:
        none
    external evidence remaining:
    sponsor amount requirements:
        none

Identity impact:
    Attestation:
    realization:
    architecture semantic hash:
    architecture behavioural hash:
    compiler identity:
        none
    target digest:
        none
    program digest:
        none
    vector/report digest:
        none
    bundle/ABI identity:
        absent

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
    target-elements tests:
    tapscript tests:
    public API tests:
    Rustdoc:
    target-native primitive lane:
    cargo tree:
    cargo metadata:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning result:
    Phase 3:
        remains active
    completed:
        typed primitive instructions
        target-native primitive conformance
        development resource observations
    next:
        STATE, arithmetic, and public-declassification prototype work
        or the first backend-pattern guide, according to accepted
        research results

Residuals:
    no attestation-contract target program
    no complete backend proof pattern
    no constructor
    no wide floor arithmetic
    no linked bundle
    no transaction ABI
    no production target evidence
```

---

# 32. Next work after Guide 9

Guide 9 completes the primitive target foundation. The next work should split along the unresolved Phase-3 research boundaries rather than immediately attempting a full operation.

Recommended order:

```text
Guide 10A — Metadata-Dependent STATE Constructor Prototype
Guide 10B — Exact Wide Floor Arithmetic Prototype
Guide 10C — Confidential-to-Public Synchronization Prototype
```

These may proceed in parallel when their package boundaries remain isolated.

After the required prototype decisions land, begin:

```text
Guide 11 — Compact-ASH Backend Pattern and Relocatable Program
```

That guide may finally lower compiler requirements for `compact-ash` into target programs, but still should not link a final bundle or define the complete transaction ABI unless those boundaries are explicitly included and reviewed.

The dependency sequence is:

```text
Guide 8
    typed target contract
    +
    compiler capability adapter

Guide 9
    typed instruction core
    +
    development target-native primitive evidence

Phase-3 prototypes
    STATE constructor
    wide arithmetic
    public declassification

first operation guide
    compiler relation requirements
    ↓
    complete backend patterns
    ↓
    relocatable target programs

later
    linker
    ↓
    transaction ABI
    ↓
    bundle-specific vectors
    ↓
    deployment evidence
    ↓
    release
```

---

# 33. Final acceptance statement

When Guide 9 is complete, the strongest permitted claim is:

> The repository can construct and canonically serialize a reviewed subset of typed Elements tapscript instructions, statically validate their stack contracts, and demonstrate on one explicitly bound secretless development target that the selected primitive instructions, encodings, success/failure behavior, and measured primitive resources agree with the typed target contract.

The following claims remain prohibited:

```text
attestation-contract tapscript backend implemented
compact ASH implemented on Elements
live transfer implemented on Elements
compiler translation validated
STATE continuity established
wide floor arithmetic established
transaction ABI available
production activation verified
deployment ready
release ready
```

Primitive correctness is a prerequisite for backend correctness. It is not a substitute for it.
