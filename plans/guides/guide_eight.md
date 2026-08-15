# Guide 8 — Typed Elements Target Contract and Capability-Adapter Foundation

> **Phase:** 3 — Elements target and foundational prototypes
> **Status:** Ready to execute
> **Entry gate:** Phase 2 complete
> **Primary packages:** `tripod-target-elements`, `tripod-tapscript`
> **Primary policies:** ADR-011, ADR-015, ADR-016, ADR-017
> **Primary decisions:** D001, D003, D004, D005, D006, D007, D008
> **Non-claim:** This guide does not establish a production backend, transaction ABI, deployment, or release.

---

## Mission

Begin Phase 3 by replacing informal Liquid/Elements assumptions with one validated, target-owned typed compatibility contract.

Guide 8 establishes two package boundaries:

```text
tripod-target-elements
    owns typed Elements/Liquid target facts

tripod-tapscript
    owns the adapter from compiler-required abstract capabilities
    to target primitives, backend-pattern obligations, structural
    obligations, and external evidence requirements
```

The package dependency direction is:

```text
compiler ────────────┐
                     ├──▶ tapscript
target-elements ─────┘
```

and:

```text
target-elements
    depends on no first-party semantic package
```

The intended analysis path is:

```text
compiler abstract target requirements
    ↓
tapscript capability adapter
    ↓
validated Elements target definition
    ↓
one of:
    ├── primitive prerequisites available; backend pattern still required
    ├── compiler/ABI structural obligation
    ├── external target evidence required
    └── unsupported or incomplete capability
```

Guide 8 must **not** emit an attestation-contract operation, link a bundle, define a transaction ABI, or claim deployment evidence.

At completion, the repository must have an honest typed answer to:

> What exact target contract is being assumed, and what remains to be proved before any compiler requirement can be described as implemented on Elements?

---

# 1. Executive rulings

Guide 8 adopts the following rulings before implementation begins.

## 1.1 Target facts and protocol semantics remain separate

`target-elements` owns:

- execution-domain facts;
- leaf-version facts;
- reviewed opcode identities and semantics;
- operand and result encodings;
- success and failure stack behavior;
- asset, value, nonce, issuance, and program encoding classes;
- sighash dimensions;
- relative-timelock dimensions;
- confidential-transaction capabilities;
- issuance and reissuance capabilities;
- target resource interfaces;
- target evidence requirements.

It does **not** own:

- attestation-contract operations;
- attestation-contract objects;
- realization relations;
- compiler relations;
- protocol authorization policy;
- protocol batch bounds;
- transaction layouts;
- proof-plan selection;
- backend proof patterns.

## 1.2 The compiler does not depend on the target

The compiler remains target-independent.

It must not add a dependency on:

```text
target-elements
tapscript
```

The target adapter belongs downstream, in `tapscript`, because that package may depend on both:

```text
compiler
target-elements
```

## 1.3 Expose only the smallest compiler target boundary

Guide 7 kept the complete scoped analyzed-program value crate-private. Guide 8 introduces the first real downstream consumer, but that does not justify exposing the entire compiler implementation.

Expose only the minimum read-only public vocabulary needed by the adapter, such as:

- `RequiredCapability`;
- a complete stable compiler-owned capability census;
- a read-only target-requirement projection where necessary;
- typed evidence and structural disposition classes needed by the adapter.

Do not expose:

- compiler Petgraph values;
- proof-search state;
- placement-search state;
- private analyzed-program constructors;
- local graph handles;
- coverage-graph internals;
- the complete internal analyzed-program DTO merely for convenience.

## 1.4 No target hash yet

Guide 8 must not mint:

```text
TargetDefinitionId
TargetDefinitionHash
DeploymentInstanceHash
TapscriptConfigurationHash
CapabilityAssessmentHash
```

unless a persistent cross-process, publication, cache, signature, or report consumer is introduced in the same implementation series.

For this guide, direct typed comparison is sufficient:

```text
validated typed target definition
+
validated typed development binding
+
typed capability assessments
```

ADR-016 rejects a digest when direct typed comparison already makes the required decision.

A stable target-contract version is permitted if a present adapter or validator uses it to accept or reject supported contract revisions. It is not a digest.

## 1.5 Review provenance is not target identity

The following are review or test provenance:

- upstream repository;
- source revision consulted;
- source paths consulted;
- upstream tests consulted;
- node version;
- node build;
- local tool version;
- date of review;
- development host;
- test runner.

They must not enter the canonical target-definition projection.

The target definition identifies the typed compatibility contract, not one Elements implementation revision.

## 1.6 Capability support is not a boolean

Guide 8 must distinguish at least:

```text
unsupported

primitive prerequisites missing

primitive prerequisites available,
but no approved complete backend proof pattern exists

compiler- or ABI-structural obligation

external target evidence required

complete backend pattern available
    — expected to remain unused in Guide 8
```

A low-level opcode existing does not establish a complete proof for:

- object recognition;
- family closure;
- canonical partitioning;
- permissionless constructibility;
- confidential value conservation;
- constructor continuity;
- event-projection correctness.

## 1.7 No secret-bearing target interface

Guide 8 remains within the current public-data contract.

It must not add command-line or library interfaces for:

- RPC credentials;
- wallet credentials;
- private keys;
- signing nonces;
- blinding factors;
- private openings;
- production deployment authority.

A future target-native runner requiring credentials needs a separate security and execution design. Guide 8 must not smuggle such an interface into a unit-test fixture, environment variable, URL, or command-line argument.

## 1.8 Sponsor erasure is role-based

`PLAIN_LBTC` is an architecture object family. “Sponsor” is a transaction-region role.

The adapter and any newly exposed compiler target boundary must not equate:

```text
PLAIN_LBTC
=
fee sponsor
```

Amounts are erased only for references belonging to the exact fee-sponsor region. Protocol-role L-BTC amounts—request funding, refund, admission reward, RESV carry, and redemption payout—remain available where their owning relation requires them.

Guide 8 must not freeze the current pilot-only coincidence into a public target API.

## 1.9 Validation precedes consumption

The first downstream target adapter must consume only a fully validated compiler requirement projection.

Before exposing the boundary:

- the complete analyzed-program validator must be on the construction path;
- exact projection censuses must reject duplicates;
- graph semantics used by the projection must be unambiguous;
- no partial analysis may be accepted as complete target requirements.

---

# 2. Entry conditions

Guide 8 begins after the Guide-7 and Phase-2 exit gates are complete.

Recorded entry state:

```text
Phase 2:
    complete

P2-012:
    DONE

P2-013:
    DONE

C1-014:
    DONE

complete scoped pilot analysis:
    implemented internally

factorized operation analysis:
    implemented

public target adapter:
    absent

target package:
    absent

backend package:
    absent
```

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

The tree must be clean.

Record the exact starting revision in the Guide-8 gate report.

The following review issues must be closed before the compiler boundary becomes public:

- role-sensitive sponsor-region erasure;
- complete analyzed-program validation on the construction path;
- duplicate-sensitive stable projection validation;
- one coherent dependency-graph orientation or an explicit graph split;
- typed overflow-safe exact-search counters;
- the realization’s zero-valued sponsor-output vector contradiction.

These may be implemented as a short Guide-8 preflight wave if they are not already repaired.

---

# 3. Required reading and authority

Read these policy and planning owners first:

```text
AGENTS.md

adr/011-toolchain-and-dependency-policy.md
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

plans/packages/compiler.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/phases/03-target-foundation.md
plans/reference/elements-tapscript.md

packages/compiler/src/capability.rs
packages/compiler/src/analyzed.rs
packages/compiler/src/analyzed_validate.rs
packages/compiler/src/lib.rs
packages/compiler/tests/public_api.rs
```

The human reference under:

```text
plans/reference/elements-tapscript.md
```

is review support only.

No package may parse it.

Authority remains:

```text
typed target source
    > generated target publication, if one is later introduced
    > package and phase plans
    > human target reference
```

---

# 4. Scope and non-goals

## 4.1 In scope

Guide 8 implements:

- `tripod-target-elements` package boundary;
- typed target-definition contract;
- target contract versioning;
- execution-domain and leaf-version types;
- reviewed initial opcode registry;
- typed stack operand/result contracts;
- typed success and failure behavior;
- field-specific encoding registry;
- explicit/confidential asset and value classes;
- sighash capability dimensions;
- relative-timelock capability dimensions;
- confidential-value and issuance capability descriptions;
- consensus and policy resource dimensions;
- target evidence-requirement registry;
- typed development deployment binding;
- validated target-definition/development-binding combination;
- the smallest compiler public target-requirement API;
- `tripod-tapscript` package boundary;
- compiler-to-target capability adapter;
- exact compiler-capability census matching;
- fail-closed incomplete-capability handling;
- stable typed projections;
- mutation and permutation tests;
- package, phase, and backlog documentation updates.

## 4.2 Out of scope

Guide 8 must not implement:

- compact-ASH target programs;
- live-transfer target programs;
- typed instruction emission;
- stack scheduling;
- peephole optimization;
- concrete relation placement;
- transaction family positions;
- concrete coordinator indexes;
- object constructors;
- STATE constructor;
- wide arithmetic;
- public-opening proof;
- transaction ABI;
- target transaction vectors;
- target RPC;
- linked bundle;
- taptree construction;
- resource calibration;
- production deployment binding;
- production activation claim;
- persistent target identity;
- evidence report digest;
- release integration.

---

# 5. Package boundaries

## 5.1 `tripod-target-elements`

Create:

```text
packages/target-elements/
    Cargo.toml
    README.md
    meson.build
    src/lib.rs
    src/definition.rs
    src/opcode.rs
    src/encoding.rs
    src/authorization.rs
    src/capability.rs
    src/resource.rs
    src/deployment.rs
    src/error.rs
    src/tests/...
    tests/public_api.rs
```

Exact module factoring may differ.

Cargo identities:

```toml
[package]
name = "tripod-target-elements"

[lib]
name = "target_elements"
```

Expected first-party dependencies:

```text
none
```

A generic third-party dependency may be added only if it has a present consumer and receives ADR-011 review. Prefer standard-library types initially.

Do not depend on:

```text
architecture
realization
model
compiler
tapscript
linker
transaction
vectors
release
artifacts
labels
```

## 5.2 `tripod-tapscript`

Create the backend package boundary:

```text
packages/tapscript/
    Cargo.toml
    README.md
    meson.build
    src/lib.rs
    src/capability.rs
    src/error.rs
    src/tests/...
    tests/public_api.rs
```

Expected direct first-party dependencies:

```text
compiler
target-elements
```

Do not yet add:

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

A direct architecture dependency is not justified merely because target assessments mention compiler capabilities originally derived from architecture-owned relations. The adapter consumes the compiler-owned public projection.

No target-program type needs to exist in Guide 8.

The tapscript package’s initial purpose is narrow:

> Adapt compiler-owned abstract target requirements to target-owned primitive contracts, backend-pattern obligations, structural obligations, and external evidence requirements.

## 5.3 Workspace and Meson

Update:

```text
Cargo.toml
packages/meson.build
meson.build role groups as required
```

Every new source file must enter its package’s explicit `meson.build` census in the same commit.

Add crate documentation to the DOC census through package-level Meson variables.

If integration-test Rust files remain outside the label graph, add explicit same-typed exclusions as required by ADR-014.

The paper subproject should not require an unused C toolchain. If no C target exists, remove the `'c'` project language rather than silently adding a compiler prerequisite.

---

# 6. Source review before typed declaration

## 6.1 Do not promote the surveyed reference directly

The current human reference contains provisional opcode numbers and behavioral summaries.

Guide 8 must not copy those values into typed source solely because they are already written in Markdown.

For every target fact admitted into `target-elements`, record the review procedure:

1. identify the deployed capability being relied upon;
2. locate the relevant upstream definition;
3. inspect interpreter behavior;
4. inspect relevant upstream tests;
5. identify activation and execution domain;
6. identify operand and result encodings;
7. identify success and failure effects;
8. identify resource accounting;
9. record unresolved deployment evidence;
10. transcribe the accepted fact into typed Rust;
11. add focused first-party tests.

## 6.2 Review provenance record

Update the human reference or add a focused research/evidence note containing:

- upstream repository;
- revision consulted;
- source locations;
- upstream tests consulted;
- upstream licence;
- exact claims accepted;
- unresolved claims;
- target-native tests still required.

The consulted revision remains review provenance only.

The typed target contract must not consume this Markdown.

## 6.3 Initial target subset

Guide 8 reviews only primitives needed by the first backend foundation and capability adapter.

Likely groups include:

- tapscript execution domain;
- leaf version;
- input count and output count inspection;
- current input index;
- input asset inspection;
- input value inspection;
- input program inspection;
- input sequence inspection where cadence is anticipated;
- input issuance inspection where future issuance is anticipated;
- output asset inspection;
- output value inspection;
- output nonce inspection where applicable;
- output program inspection;
- transaction version, locktime, count, and weight inspection;
- exact fixed-width arithmetic used by initial target plans;
- exact comparison operations used by initial target plans;
- signature primitive and sighash dimensions;
- relative-timelock dimensions;
- CT value conservation;
- commitment equality;
- issuance/reissuance interfaces;
- consensus and policy resource dimensions.

A primitive may be represented as unavailable or unreviewed. Guide 8 must not declare support merely to make the adapter succeed.

---

# 7. Typed target definition

## 7.1 Contract shape

A conceptual target definition is:

```rust
pub struct TargetDefinition {
    version: TargetContractVersion,
    execution_domain: ExecutionDomain,
    leaf_version: LeafVersion,

    opcodes: BTreeMap<OpcodeId, OpcodeSpec>,
    encodings: EncodingRegistry,
    authorization: AuthorizationContract,
    confidential_values: ConfidentialValueContract,
    issuance: IssuanceContract,
    resources: ResourceContract,
    capabilities: BTreeMap<ElementsCapability, CapabilityContract>,
    evidence_requirements:
        BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement>,
}
```

Fields remain private. Construction passes through one validator or one built-in reviewed declaration.

A public consumer receives a validated value. It must not assemble a target definition through a public struct literal.

## 7.2 Stable typed keys

Target-owned stable keys may include:

```rust
pub enum OpcodeId {
    // reviewed target primitives only
}

pub enum ElementsCapability {
    // target-owned primitive and substrate capabilities
}

pub enum TargetEvidenceRequirementId {
    // typed target-evidence roles
}

pub enum ResourceDimension {
    // distinct units
}

pub enum EncodingClass {
    // field-specific encoding families
}

pub enum ExecutionDomain {
    Tapscript,
}

pub struct LeafVersion(u8);
```

These are stable typed keys, not digests.

Opcode byte values are explicit data validated against the reviewed contract. Do not assume enum declaration order equals target opcode code.

## 7.3 Target contract version

Introduce a small typed version:

```rust
pub struct TargetContractVersion(u32);
```

or:

```rust
pub enum TargetContractVersion {
    V1,
}
```

The adapter explicitly accepts the supported version.

Changing semantic shape or interpretation requires a deliberate contract-version migration. Editorial review provenance does not.

Do not add a serialized schema unless external bytes are actually introduced.

## 7.4 Built-in reviewed definition

If the project provides one built-in Liquid/Elements definition, expose it through a function such as:

```rust
pub fn reviewed_elements_tapscript()
    -> Result<ValidatedTargetDefinition, TargetError>;
```

or a validated static value.

Avoid names such as:

```text
production_target
final_target
verified_target
```

The definition is a reviewed static contract. It is not deployment evidence.

## 7.5 Validated wrapper

Validation should produce an opaque wrapper:

```rust
pub struct ValidatedTargetDefinition {
    definition: TargetDefinition,
}
```

Only this wrapper may be consumed by a deployment binding or capability adapter.

Validation precedes identity and consumption, following ADR-016’s mechanism ordering.

---

# 8. Opcode contract

## 8.1 Each opcode needs a complete typed contract

A suitable specification is:

```rust
pub struct OpcodeSpec {
    id: OpcodeId,
    code: u8,
    execution_domains: BTreeSet<ExecutionDomain>,
    stack: StackContract,
    failure: FailureContract,
    resources: OpcodeResourceCost,
    evidence_requirements:
        BTreeSet<TargetEvidenceRequirementId>,
}
```

## 8.2 Stack value types

Define target-level stack representations without importing protocol objects:

```rust
pub enum StackValueType {
    Bytes {
        minimum: usize,
        maximum: usize,
    },

    Bool,
    ScriptNumber,

    SignedFixedWidth {
        bytes: NonZeroUsize,
        byte_order: ByteOrder,
    },

    AssetEncoding,
    ValueEncoding,
    ProgramEncoding,
    OutPointEncoding,
    SequenceEncoding,
    IssuanceEncoding,
}
```

Do not define:

```text
AshValue
ReceiptOwner
StateOmega
SettlementPrincipal
```

Those are protocol semantics and belong upstream.

## 8.3 Stack contracts

A stack contract should state:

- operand sequence;
- result sequence;
- main-stack effects;
- altstack effects where relevant;
- operand consumption on success;
- operand consumption on failure;
- canonical truth result if applicable;
- malformed-width behavior;
- execution-domain constraints.

A conceptual type is:

```rust
pub struct StackContract {
    operands: Vec<StackValueType>,
    success_results: Vec<StackValueType>,
    altstack_before: Vec<StackValueType>,
    altstack_after: Vec<StackValueType>,
}
```

Guide 8 need not model arbitrary program execution. It models reviewed primitive contracts.

## 8.4 Success and failure effects

Do not describe an opcode only by successful results.

A target operation may:

- abort script evaluation;
- push a false success flag;
- preserve operands and push false;
- consume operands before failure;
- reject malformed width;
- produce a target-specific null encoding.

Model this explicitly.

For example:

```rust
pub enum FailureMode {
    Abort,

    PushFalse {
        stack_after: Vec<StackValueType>,
    },

    Conditional {
        causes: Vec<FailureCause>,
    },
}
```

The exact form may differ.

This is especially important for fixed-width arithmetic, where failure may leave values on the stack rather than abort.

## 8.5 Opcode validation

Reject:

- duplicate opcode code;
- duplicate opcode ID;
- missing execution domain;
- missing stack contract;
- missing failure contract;
- malformed operand/result width;
- missing resource cost;
- unknown encoding dependency;
- capability naming an undeclared opcode;
- evidence requirement naming an unknown opcode or capability.

## 8.6 No backend instruction builder yet

Guide 8 defines what target primitives mean.

It does not yet define:

```rust
enum TapscriptInstruction
```

and does not serialize programs.

That belongs to Guide 9.

---

# 9. Encoding registry

## 9.1 Asset and value are independent axes

The target contract must distinguish:

```text
explicit asset
confidential asset
explicit value
confidential value
nonce encodings
null or absent forms
```

Do not infer asset confidentiality from value confidentiality.

## 9.2 Encoding specifications

A suitable typed shape is:

```rust
pub struct EncodingSpec {
    class: EncodingClass,
    prefixes: BTreeSet<u8>,
    payload_width: PayloadWidth,
    byte_order: ByteOrder,
    canonicality: CanonicalEncodingRule,
    unknown_prefix: UnknownPrefixRule,
}
```

Use field-specific byte order.

Do not create one global “Elements byte order” setting.

## 9.3 Unknown encodings

Unknown or unsupported encodings fail closed.

Do not automatically preserve an unknown prefix as an opaque valid protocol value merely because a future target may define it.

Forward-compatibility behavior must be explicit per encoding boundary.

## 9.4 Closed-asset policy remains downstream

`target-elements` may describe confidential asset encodings because they exist on the target.

The initial attestation-contract policy that closed protocol assets must be explicit is enforced by compiler/backend planning under D005.

The target package does not know which assets are attestation-contract closed assets.

## 9.5 Encoding validation

Reject:

- duplicate prefix within one encoding domain;
- overlapping encodings whose decoder cannot distinguish them;
- zero or impossible width;
- missing byte order for ordered fixed-width fields;
- contradictory canonicality rules;
- unknown null encoding;
- capability depending on an undeclared encoding class;
- one field reusing another field’s encoding without an explicit typed equivalence.

---

# 10. Authorization and timelock contracts

## 10.1 Sighash dimensions

Represent target dimensions required by later backend policy:

```rust
pub struct SighashCapability {
    commits_outputs: OutputCommitment,
    commits_inputs: InputCommitment,
    permits_input_extension: bool,
    commits_issuance: bool,
    commits_version: bool,
    commits_locktime: bool,
    script_path_semantics: ScriptPathCommitment,
}
```

Do not select the attestation-contract sighash profile in `target-elements`.

The backend later selects a profile satisfying compiler and operation requirements.

## 10.2 Signature primitive

Record:

- public-key encoding;
- signature encoding;
- empty or malformed behavior;
- successful stack effect;
- unsuccessful stack effect;
- output-commitment capability dimensions;
- input-commitment dimensions;
- crypto resource cost;
- evidence requirements.

Do not claim that target signature verification proves model authorization. It is one later evidence layer.

## 10.3 Relative timelock dimensions

Record:

- block/time mode;
- sequence interpretation;
- transaction-version prerequisites;
- disabled flags;
- minimum-age boundary behavior;
- failure behavior;
- evidence requirements.

Do not encode the protocol’s cadence band in the target package.

The compiler/backend later uses these primitives to implement:

```text
before MIN:
    invalid

MIN ≤ age < MAX:
    operator

age ≥ MAX:
    permissionless
```

## 10.4 Maturity remains outside target timelocks

The target contract may describe absolute and relative timelock facilities if reviewed.

It must not infer that attestation-contract maturity uses them. The realization keeps maturity in committed cycle arithmetic.

---

# 11. Confidential values and issuance contracts

## 11.1 Confidential-value capabilities

Model distinct target claims:

```rust
pub enum ConfidentialValueCapability {
    ConsensusValueConservation,
    CommitmentEquality,
    ExplicitValueInspection,
    ConfidentialValueInspection,
    AuthenticatedOpening,
}
```

A capability may be unsupported or incomplete.

Do not mark `AuthenticatedOpening` complete merely because low-level elliptic-curve or hash primitives exist.

## 11.2 Consensus conservation

Whole-transaction value conservation is an external target/consensus claim.

The target contract may state:

- what target mechanism is relied upon;
- what explicit and confidential value classes participate;
- what evidence requirement must later verify it.

It must not mark the compiler relation complete.

## 11.3 Commitment equality

Describe the exact target mechanism only after review.

This capability is relevant to amount-blind relabel and future private lifecycle paths, but Guide 8 does not implement those paths.

## 11.4 Issuance and reissuance

Describe target facts for:

- issuance-field presence;
- null issuance;
- asset entropy;
- issued-asset identity;
- issuance amount;
- reissuance/inflation authority;
- input issuance inspection;
- target transaction commitment.

Do not map these facts to `U`, `ENT`, or `DIST_CTL` inside `target-elements`.

That mapping belongs to the backend and transaction ABI.

## 11.5 Sponsor-region conservation

The target contract may describe whole-transaction explicit or confidential value conservation.

The adapter must not introduce:

- sponsor positivity;
- public sponsor subtotal;
- exact sponsor input amount;
- exact sponsor output amount;
- sponsor opening;
- sponsor value inspection.

The sponsor-region proof remains the residual of independently authenticated protocol flows plus target-wide conservation.

---

# 12. Resource interfaces

## 12.1 Typed dimensions

Use separate types or clearly distinct fields for:

```text
transaction weight
witness bytes
script bytes
initial stack items
peak stack items
altstack items
stack element bytes
crypto budget
target operation cost
control-path depth
policy package limits
```

Do not combine unlike units into one score.

## 12.2 Consensus and policy are separate

Represent:

```rust
pub struct ResourceContract {
    consensus: ConsensusResourceLimits,
    policy: PolicyResourceLimits,
}
```

A deployment may impose stricter policy than consensus.

Neither target package nor Guide 8 chooses attestation-contract batch bounds.

## 12.3 No calibration

Guide 8 must not derive:

```text
ASH_BATCH_MAX
TRANSFER_INPUT_MAX
FEE_SPONSOR_INPUT_MAX
```

from target limits.

Those remain architecture-declared calibration requirements until complete transactions and final bundles exist.

## 12.4 Resource validation

Reject:

- zero limits where the target requires positive capacity;
- policy limits exceeding incompatible consensus dimensions;
- missing unit distinctions;
- capability claiming an operation whose resource rule is absent;
- opcode with no resource cost;
- deployment override without an explicit policy relation;
- one dimension silently substituted for another.

---

# 13. Target evidence requirements

## 13.1 Requirements, not statuses

Define immutable requirement identities such as:

```rust
pub enum TargetEvidenceRequirementId {
    ExecutionDomain,
    OpcodeSemantics(OpcodeId),
    EncodingSemantics(EncodingClass),
    SighashSemantics,
    RelativeTimelockSemantics,
    ConfidentialValueConservation,
    CommitmentEquality,
    IssuanceIntrospection,
    ResourceLimits,
    Activation,
    PolicyBehavior,
}
```

A practical enum may use non-parameterized variants if stable keys must remain simple. Equivalent typed factoring is acceptable.

## 13.2 Requirement contents

Each requirement may state:

- subject;
- claim class;
- required test environment;
- static or deployment-scoped status;
- expected evidence role;
- stale conditions.

It must not contain:

- mutable pass/fail result;
- report digest;
- ambient timestamp;
- RPC endpoint;
- credentials;
- node source revision as target identity.

## 13.3 Exact consumer mapping

Every target capability exposed to the adapter must map to one or more evidence requirements where the claim is not wholly established by the static typed contract.

A capability with no evidence requirement should fail validation unless its claim is purely type-level and the contract explains why.

## 13.4 Guide-8 residual

Guide 8 may finish with every target-native evidence requirement unresolved.

That is acceptable if documented honestly:

```text
typed static target contract:
    implemented

target-native deployment evidence:
    not yet produced

production target support:
    not claimed
```

---

# 14. Development deployment binding

## 14.1 Keep static target and deployment instance separate

Define:

```rust
pub struct DevelopmentDeploymentBinding {
    target_version: TargetContractVersion,
    environment: DevelopmentEnvironment,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    activation: ActivationDeclaration,
    resource_overrides: Option<DevelopmentResourceOverrides>,
}
```

Exact fields may differ.

The deployment binding must not mutate the target definition.

## 14.2 Environment class

Use explicit classes:

```rust
pub enum DeploymentEnvironment {
    Development,
    Production,
}
```

Guide 8 initially constructs only a development binding.

Do not provide an API that silently upgrades a development target into a production target.

## 14.3 Validation

Reject:

- zero network ID;
- zero genesis ID;
- unsupported target contract version;
- target/deployment contract mismatch;
- missing activation declaration;
- incompatible resource declaration;
- production status without a production evidence boundary;
- environment-dependent target-definition mutation.

## 14.4 Activation declaration versus evidence

A development binding may declare the expected activation state.

It must not describe that declaration as verified target evidence.

Use distinct terminology:

```text
activation requirement or declaration:
    typed input

activation report:
    future evidence
```

## 14.5 Validated combination

Expose a read-only validated value:

```rust
pub struct ElementsTarget {
    definition: ValidatedTargetDefinition,
    deployment: ValidatedDevelopmentBinding,
}
```

The exact constructor validates compatibility.

Passing this constructor means only:

- the typed static definition is internally valid;
- the deployment declaration is internally valid;
- the two agree.

It does not prove that a node or network actually satisfies the declaration.

## 14.6 Public-data boundary

The deployment binding contains no:

- RPC URL;
- RPC username;
- RPC password;
- cookie path;
- bearer token;
- private key;
- wallet path;
- release authority.

A future execution adapter owns any such operational boundary under a separate security design.

---

# 15. Minimal compiler target boundary

## 15.1 First real downstream consumer

The tapscript adapter is the first real consumer of compiler-owned abstract target capabilities.

Guide 8 may therefore expose the minimum stable public API required by that consumer.

At minimum:

```rust
pub enum RequiredCapability {
    // existing abstract compiler capabilities
}
```

must become reachable through a reviewed public module or re-export.

Prefer:

```rust
pub mod target {
    pub use ...RequiredCapability;
}
```

or another intentionally named boundary.

Do not expose `capability.rs` wholesale merely because it already exists.

## 15.2 Complete capability census

Add:

```rust
impl RequiredCapability {
    pub const ALL: &'static [Self] = &[...];
}
```

or an equivalent generated census.

The adapter must handle every variant exhaustively.

A new compiler capability must cause a compile failure or exact census-test failure until the tapscript adapter states its disposition.

## 15.3 Optional requirement-set projection

If the adapter needs more than individual capability values, expose a read-only target-independent projection such as:

```rust
pub struct TargetRequirementSet {
    required_capabilities: BTreeSet<RequiredCapability>,
    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,
}
```

The type should have private fields and read-only iteration.

Do not expose:

- placements;
- layout internals;
- relation graph;
- coverage graph;
- proof-search reports;
- complete analyzed programs;

unless the adapter genuinely consumes those values in this guide.

## 15.4 No compiler-target dependency

The compiler public API must not name:

```text
ElementsCapability
OpcodeId
TargetDefinition
ElementsTarget
TargetEvidenceRequirementId
```

It remains abstract.

## 15.5 Validation precondition

The projection must only be constructible from a fully validated internal analyzed program.

The construction path must:

1. build the scoped analyzed program;
2. run complete fresh re-derivation validation;
3. reject duplicate or noncanonical stable projection entries;
4. derive the minimal target requirement set;
5. return an opaque read-only projection.

No public caller may construct `TargetRequirementSet` directly.

## 15.6 Sponsor-role boundary

The public compiler target boundary must not encode:

```text
PLAIN_LBTC means sponsor
```

If sponsor-specific requirements are exposed, they must derive from typed fee-sponsor-region semantics. Protocol-role L-BTC amounts remain distinct.

---

# 16. Tapscript capability adapter

## 16.1 Adapter purpose

The adapter translates compiler-required abstract capabilities into an assessment against one validated target definition and development binding.

A suitable input is:

```rust
pub fn assess_capability(
    target: &target_elements::ElementsTarget,
    required: compiler::target::RequiredCapability,
) -> CapabilityAssessment;
```

The adapter is pure and deterministic.

## 16.2 Assessment model

Use a typed multi-state result, for example:

```rust
pub enum CapabilityAssessment {
    Unsupported {
        required: compiler::target::RequiredCapability,
        reason: UnsupportedReason,
    },

    MissingTargetPrimitives {
        required: compiler::target::RequiredCapability,
        missing: BTreeSet<target_elements::ElementsCapability>,
    },

    BackendPatternRequired {
        required: compiler::target::RequiredCapability,
        primitives: BTreeSet<target_elements::ElementsCapability>,
        evidence:
            BTreeSet<target_elements::TargetEvidenceRequirementId>,
    },

    BackendStructural {
        required: compiler::target::RequiredCapability,
        requirements: BTreeSet<BackendFoundationRequirement>,
    },

    ExternalEvidenceRequired {
        required: compiler::target::RequiredCapability,
        evidence:
            BTreeSet<target_elements::TargetEvidenceRequirementId>,
    },

    CompleteBackendPattern {
        required: compiler::target::RequiredCapability,
        pattern: BackendPatternId,
    },
}
```

`CompleteBackendPattern` should normally remain unused in Guide 8.

Avoid:

```rust
Supported(bool)
```

## 16.3 Initial compiler-capability mapping

The adapter must exhaustively classify the current compiler capability census.

Conceptually:

| Compiler capability | Guide-8 disposition |
|---|---|
| authenticated object recognition | backend pattern required; target inspection primitives required |
| authenticated family cardinality | backend/ABI structural proof required; count/index primitives may be required |
| authenticated canonical partition | backend pattern and complete transaction-layout proof required |
| authenticated open-flow partition | backend pattern and protocol/sponsor-region proof required |
| authenticated root effects | backend constructor/root pattern required; unavailable for operation emission today |
| authenticated projection set | backend structural/event-shape pattern required |
| exact public amount arithmetic | target arithmetic and authenticated-value primitives required |
| confidential value conservation | target CT capability plus external target evidence required |
| owner authorization | signature primitive plus compatible sighash pattern required |
| operator authorization | signature primitive plus compatible sighash pattern required |
| refund authorization | signature primitive plus compatible sighash pattern required |
| public constructibility | compiler/ABI structural obligation; not one target opcode |
| whole-transaction value conservation | external target/consensus evidence required |

The exact table must match the actual `RequiredCapability` enum at implementation time.

## 16.4 No semantic weakening

If a required primitive or target contract is missing, the adapter returns a typed blocked or unsupported assessment.

It must not:

- drop the compiler capability;
- replace exact arithmetic with approximate arithmetic;
- treat public constructibility as signature availability;
- replace whole-transaction conservation with sponsor positivity;
- treat CT conservation as object-family closure;
- invent an authenticated-opening pattern;
- claim a target relation is implemented;
- collapse a structural obligation into external evidence;
- collapse external evidence into primitive availability.

## 16.5 No proof-pattern completion yet

Guide 8 should normally produce no:

```text
CompleteBackendPattern
```

assessment for attestation-contract relations.

The target primitives may be reviewed, but the first complete backend proof patterns belong to later guides.

## 16.6 Set assessment

Provide a set-level function:

```rust
pub fn assess_requirements(
    target: &target_elements::ElementsTarget,
    requirements: &compiler::target::TargetRequirementSet,
) -> Result<CapabilityAssessmentSet, TapscriptError>;
```

The result must satisfy:

```text
required capability census
=
assessment capability census
```

exactly.

No duplicate, missing, or unexpected assessment is accepted.

---

# 17. Validation rules

## 17.1 Target-definition validation

Require:

- supported target-contract version;
- exactly one intended execution domain for the built-in definition;
- valid leaf version;
- opcode IDs unique;
- opcode byte codes unique;
- every opcode fully specified;
- every stack contract valid;
- every failure contract complete;
- every resource cost present;
- encoding prefixes unambiguous within their domains;
- field widths valid;
- capability prerequisites resolve;
- evidence requirements resolve;
- capability dependency graph acyclic unless a reviewed strategy exists;
- no mutable evidence status in the definition;
- no review provenance in the semantic projection.

## 17.2 Deployment-binding validation

Require:

- nonzero network ID;
- nonzero genesis ID;
- explicit development environment;
- target-contract version equality;
- activation declaration present;
- no production status;
- resource declarations compatible;
- no credential-bearing field.

## 17.3 Adapter validation

Require:

- every compiler capability classified exactly once;
- no unknown compiler capability;
- every target primitive prerequisite exists;
- every evidence requirement exists;
- unsupported capability remains visible;
- no assessment claims evidence completion;
- no sponsor amount appears;
- no target-specific type flows back into compiler;
- no complete backend pattern appears without a typed implemented pattern.

## 17.4 Cross-package closure

The target definition may expose more target capabilities than one pilot requires.

The adapter must ensure:

```text
compiler required capability census
=
adapter assessment census
```

for any requirement set it receives.

No required capability may disappear from the assessment result.

## 17.5 Graph semantics

If target capabilities or evidence requirements form a dependency graph:

- use one documented edge orientation;
- validate source and target node classes per edge kind;
- reject duplicate nodes and edges;
- normalize SCC results by stable typed key;
- reject cycles unless a reviewed resolution strategy exists;
- exclude Petgraph indices from stable projections.

If ownership and prerequisite relationships require opposite orientations, use separate typed graphs rather than one ambiguous graph.

---

# 18. Stable projections and determinism

## 18.1 Target-definition projection

Provide a stable typed projection for comparison and tests.

It may contain:

- target-contract version;
- execution domain;
- leaf version;
- sorted opcode specs;
- sorted encoding specs;
- authorization and timelock contracts;
- confidential-value and issuance contracts;
- resource interfaces;
- sorted capabilities;
- sorted evidence requirements.

It must exclude:

- source revision;
- source paths;
- node version;
- review date;
- test host;
- environment variables;
- local graph handles;
- diagnostics;
- hashes.

## 18.2 Development-binding projection

It may contain:

- environment class;
- network ID;
- genesis ID;
- target-contract version;
- activation declaration;
- explicit development resource policy.

It must not claim actual activation evidence.

## 18.3 Adapter projection

A stable assessment projection may contain:

- compiler-required capability;
- disposition;
- required target primitives;
- required backend structural facts;
- required evidence roles.

It must exclude:

- target program;
- program position;
- transaction position;
- test result;
- report digest;
- source revision;
- target-execution status.

## 18.4 Determinism tests

Test equality under:

- opcode declaration permutation;
- encoding declaration permutation;
- capability declaration permutation;
- evidence-requirement permutation;
- compiler-capability requirement permutation;
- repeated construction;
- equivalent development-binding input ordering where set-like values exist.

## 18.5 Exact projection validation

A canonical vector projection must reject:

- duplicate entries;
- missing entries;
- unexpected entries;
- out-of-order entries where canonical ordering is part of the contract.

Do not validate vectors only by converting them to sets; that silently accepts duplicates.

---

# 19. Error vocabulary

## 19.1 Target errors

Add only variants reached by real validation.

Likely classes:

```rust
pub enum TargetError {
    UnsupportedTargetContractVersion,

    DuplicateOpcodeId(OpcodeId),
    DuplicateOpcodeCode(u8),
    MissingOpcodeContract(OpcodeId),
    InvalidOpcodeStackContract(OpcodeId),
    MissingOpcodeFailureContract(OpcodeId),
    MissingOpcodeResourceCost(OpcodeId),
    UnsupportedOpcodeExecutionDomain(OpcodeId),

    DuplicateEncodingPrefix {
        class: EncodingClass,
        prefix: u8,
    },
    InvalidEncodingWidth(EncodingClass),
    MissingByteOrder(EncodingClass),
    MissingCanonicalEncodingRule(EncodingClass),

    UnknownCapabilityPrerequisite(ElementsCapability),
    CapabilityDependencyCycle {
        components: Vec<Vec<ElementsCapability>>,
    },
    MissingCapabilityEvidence(ElementsCapability),
    UnknownEvidenceRequirement(TargetEvidenceRequirementId),

    ZeroNetworkId,
    ZeroGenesisId,
    TargetDeploymentVersionMismatch,
    MissingActivationDeclaration,
    ProductionBindingUnsupported,
    InvalidResourceContract(ResourceDimension),
}
```

Exact variants should follow actual branches.

## 19.2 Tapscript adapter errors

Possible classes:

```rust
pub enum TapscriptError {
    UnsupportedCompilerCapability(
        compiler::target::RequiredCapability,
    ),

    MissingTargetPrimitive {
        required: compiler::target::RequiredCapability,
        primitive: target_elements::ElementsCapability,
    },

    MissingTargetEvidenceRequirement {
        required: compiler::target::RequiredCapability,
        evidence:
            target_elements::TargetEvidenceRequirementId,
    },

    CapabilityAssessmentCensusMismatch {
        missing: Vec<compiler::target::RequiredCapability>,
        unexpected: Vec<compiler::target::RequiredCapability>,
    },

    DuplicateCapabilityAssessment(
        compiler::target::RequiredCapability,
    ),

    UnsupportedTargetContractVersion,

    TargetDefinitionRejected,
}
```

Do not add emission, stack-scheduling, relocation, or target-program errors until those paths exist.

---

# 20. Test plan

## 20.1 Target-definition positive tests

- reviewed built-in definition validates;
- supported target-contract version validates;
- target-definition projection is deterministic;
- each declared opcode has complete stack, failure, and resource contracts;
- encoding classes remain distinct;
- capability prerequisites resolve;
- every capability has evidence requirements where needed;
- consensus and policy limits remain separate;
- source-review provenance is absent from canonical projection.

## 20.2 Target-definition mutation tests

Mutate independently:

- duplicate opcode code;
- duplicate opcode ID;
- missing operand;
- missing result;
- missing failure behavior;
- wrong execution domain;
- malformed fixed width;
- conflicting encoding prefix;
- missing byte order;
- unknown capability prerequisite;
- missing evidence requirement;
- capability dependency cycle;
- zero resource limit where prohibited;
- source revision inserted into semantic projection, if the type design makes such a regression possible.

Each mutation must fail for a focused typed reason.

## 20.3 Development-binding tests

- nonzero synthetic development network/genesis accepted;
- zero network rejected;
- zero genesis rejected;
- target-version mismatch rejected;
- missing activation declaration rejected;
- production binding rejected or unavailable;
- binding carries no credential field;
- repeated binding construction is equal.

## 20.4 Compiler public-boundary tests

In the compiler’s external integration test:

- every required capability is publicly nameable;
- `RequiredCapability::ALL` is complete and duplicate-free;
- no target-specific type appears in the public compiler API;
- complete internal analyzed program remains unavailable externally;
- no compiler-plan identity exists;
- public target requirements can only come from a completely validated analysis;
- duplicate stable projection members reject.

## 20.5 Adapter census tests

For:

```rust
RequiredCapability::ALL
```

require exactly one assessment per capability.

Test:

- no capability omitted;
- no capability duplicated;
- requirement ordering does not matter;
- missing target primitive produces `MissingTargetPrimitives`;
- unsupported target contract produces a typed failure;
- whole-transaction conservation remains external evidence;
- public constructibility remains structural;
- owner authorization requires both signature and sighash obligations;
- confidential conservation does not imply object recognition;
- no assessment is falsely marked complete.

## 20.6 Adapter non-weakening tests

For each compiler capability:

1. remove one target prerequisite;
2. reassess;
3. require blocked or incomplete result;
4. verify the original compiler requirement remains in the assessment.

Add focused regressions for:

- no whole-transaction conservation;
- no output-committing sighash capability;
- no input-value inspection;
- no output-program inspection;
- no transaction-count inspection;
- no CT conservation;
- no relative-timelock semantics;
- no issuance inspection.

Not every missing primitive must block both pilots today, but the adapter’s general mapping must remain exact.

## 20.7 Sponsor-opacity tests

Assert the target adapter and target definition contain no attestation-contract sponsor-amount requirement.

No adapter assessment should require:

- sponsor positivity;
- exact sponsor input amount;
- exact sponsor output amount;
- public sponsor aggregate;
- sponsor opening.

Whole-transaction conservation remains a target/consensus evidence requirement.

Add role-sensitive tests:

- fee-sponsor `PLAIN_LBTC` amount remains opaque;
- protocol-role `PLAIN_LBTC` amount remains readable when required;
- one reference cannot belong to both protocol and sponsor regions;
- presence of protocol-role L-BTC does not activate the sponsor case;
- zero-valued ordinary sponsor member remains semantically acceptable under exact role structure;
- CPFP anchor remains distinct by family and ABI role, not by testing ordinary output value.

## 20.8 Permutation tests

Use deterministic or property-generated permutations for:

- opcode declarations;
- encoding declarations;
- capability declarations;
- evidence requirements;
- compiler capability input sets.

Stable projections must remain equal.

## 20.9 Complexity and counter tests

For compiler exact-search infrastructure exposed through the target boundary:

- state limit exactly met;
- state limit exceeded by one;
- candidate limit exactly met;
- candidate limit exceeded by one;
- counters never wrap;
- constructibility rejection is not counted as capability rejection;
- successful semantic projections remain independent of diagnostic counters.

---

# 21. Independent cross-checks

## 21.1 Registry census oracle

Implement a direct test oracle comparing:

```text
declared target opcode IDs
declared opcode codes
declared capability IDs
declared evidence requirement IDs
```

against their typed `ALL` censuses.

Do not derive the expected set from the registry being checked.

## 21.2 Encoding vectors

For fixed-width and prefix encodings, use explicit reviewed byte vectors.

Expected bytes must not be generated by the same encoder under test.

Guide 8 need not implement complete target transaction parsing.

## 21.3 Stack-contract review fixtures

For each reviewed opcode:

- assert exact operand sequence;
- assert exact successful result sequence;
- assert exact failure effect;
- assert exact execution domain;
- assert exact resource cost.

These are typed-contract tests, not target-native execution evidence.

## 21.4 Capability-adapter oracle

Maintain an independently written expected mapping table in tests for the current compiler capability census.

The production adapter should use an exhaustive match. The oracle compares stable assessment projections without calling the production mapping helper.

This duplication is appropriate because the mapping is the property being verified.

## 21.5 Graph oracle

If a capability dependency graph is introduced:

- compare cycle detection against an independent mutual-reachability oracle on small generated graphs;
- compare canonical SCC members by stable key;
- verify edge direction through explicit prerequisite-order fixtures;
- reject duplicate nodes and edges before graph construction.

---

# 22. Documentation requirements

## 22.1 Package READMEs

`packages/target-elements/README.md` must state:

```text
implemented:
    typed static target contract
    reviewed initial primitive registry
    development deployment declaration
    evidence requirement registry

not implemented:
    target-native deployment evidence
    production activation
    backend patterns
    protocol operation emission
    linked bundle
    ABI
```

`packages/tapscript/README.md` must state:

```text
implemented:
    package boundary
    compiler-to-target capability adapter

not implemented:
    target program type
    instruction builder
    stack scheduler
    backend proof patterns
    constructors
    relocatable bundle
```

## 22.2 Package contracts

Update:

```text
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/packages/compiler.md
plans/packages/README.md
```

Mark only completed milestones.

Do not describe reviewed primitives as deployment-evidenced.

Reconcile the compiler package index so it no longer says complete analyzed pilots are open.

Remove resolved factorization questions from the compiler package contract.

## 22.3 Phase card

Update:

```text
plans/phases/03-target-foundation.md
```

Guide 8 leaves Phase 3 active, not complete.

A suitable status is:

```text
typed target-contract and capability-adapter foundation implemented;
target-native primitive evidence and backend instruction core remain open
```

## 22.4 Human target reference

Update:

```text
plans/reference/elements-tapscript.md
```

Separate each accepted fact into:

```text
surveyed
reviewed
typed
pattern-complete
deployment-evidenced
```

Only claim levels actually achieved.

## 22.5 Backlog

Add a compact Guide-8 gate record.

Suggested Phase-3 tasks:

```text
T3-001  typed target package boundary
T3-002  reviewed initial target definition
T3-003  encoding and opcode contracts
T3-004  development deployment binding
T3-005  compiler capability adapter
T3-006  target-native primitive conformance
T3-007  typed tapscript instruction foundation
```

Guide 8 completes the first five and leaves the final two for Guide 9.

## 22.6 Realization correction

Correct the seeded-reduction contradiction:

```text
zero-valued ordinary sponsor member with exact role structure:
    semantic acceptance

known zero sponsor change in first-party builder:
    omitted by wallet construction policy

ordinary sponsor member substituted for CPFP anchor:
    rejected by family/constructor/ABI identity
```

Do not preserve a generic domain-failure vector for zero-valued ordinary sponsor output.

---

# 23. Suggested implementation waves

## Wave 0 — Preflight and source review

Deliver:

- close the compiler validation and projection prerequisites;
- close role-sensitive sponsor erasure;
- correct the zero-sponsor vector;
- choose coherent dependency-graph semantics;
- make exact-search counters overflow-safe;
- review initial target fact set;
- record exact source/review provenance;
- review upstream licence;
- decide whether Guide 8 needs any third-party dependency;
- list unresolved target-native claims.

Suggested commits:

```text
compiler: harden the target-requirement boundary
realization: make sponsor erasure flow-role specific
docs: correct the zero-valued sponsor conformance vector
plans: review the initial Elements target contract
```

## Wave 1 — `target-elements` crate boundary

Deliver:

- package and library;
- workspace membership;
- Meson census;
- README;
- typed error root;
- public API test;
- no first-party dependency.

Suggested commit:

```text
target-elements: establish the target contract boundary
```

## Wave 2 — Typed target definition

Deliver:

- target-contract version;
- execution domain;
- leaf version;
- opcode registry;
- stack and failure contracts;
- stable target projection;
- validation and mutation tests.

Suggested commit:

```text
target-elements: define reviewed tapscript primitives
```

## Wave 3 — Encodings, authorization, CT, issuance, and resources

Deliver:

- encoding registry;
- sighash dimensions;
- timelock dimensions;
- confidential-value contract;
- issuance contract;
- resource interfaces;
- evidence requirements;
- focused tests.

Suggested commit:

```text
target-elements: type encodings and target capability contracts
```

## Wave 4 — Development deployment binding

Deliver:

- development environment class;
- network/genesis binding;
- activation declaration;
- target/deployment validation;
- no production upgrade;
- no credential fields.

Suggested commit:

```text
target-elements: bind an explicit development instance
```

## Wave 5 — Minimal compiler target API

Deliver:

- public `RequiredCapability` boundary;
- complete `ALL` census;
- optional read-only requirement projection if genuinely consumed;
- external public API tests;
- complete analyzed-program internals remain private;
- complete validator required before projection.

Suggested commit:

```text
compiler: expose the abstract target requirement boundary
```

## Wave 6 — Tapscript capability adapter

Deliver:

- package boundary;
- direct dependencies on compiler and target-elements;
- exhaustive capability mapping;
- multi-state assessments;
- evidence and structural distinctions;
- no program emission;
- independent mapping oracle.

Suggested commit:

```text
tapscript: map compiler requirements to target obligations
```

## Wave 7 — Integration, documentation, and gate

Deliver:

- exact target/adapter census tests;
- permutation tests;
- package contracts;
- updated active Phase-3 card;
- backlog gate record;
- identity/dependency report;
- full repository verification.

Suggested commit:

```text
plans: record the Elements target foundation gate
```

Commit each green wave promptly.

---

# 24. Focused verification

## 24.1 Target package

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-target-elements --no-deps
```

## 24.2 Compiler boundary

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-compiler --no-deps
```

## 24.3 Tapscript adapter

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-tapscript --no-deps
```

## 24.4 Realization and model boundary

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-model realization_conformance
```

Verify role-specific L-BTC projection and sponsor opacity before accepting the compiler target boundary.

## 24.5 Package boundary and dependency checks

```sh
cargo tree --locked -p tripod-target-elements -e features
cargo tree --locked -p tripod-tapscript -e features
cargo metadata --locked
```

Verify:

```text
target-elements:
    no first-party semantic dependency

tapscript:
    compiler
    target-elements
```

and no accidental dependency on:

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

unless a separately reviewed ownership reason changes the contract.

---

# 25. Working cadence

After each coherent wave:

```sh
cargo fmt --all
git status --short
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Do not run the full Meson/document surface after every small edit.

Every new tracked source must join its nearest `meson.build` census in the same commit.

Read `git status` after formatting.

Documentation-only changes do not require the Rust lane, but the relevant label, plan, and census checks still apply.

---

# 26. Generated artifacts and identities

No architecture or model publication should change merely because Guide 8 adds downstream packages.

Run:

```sh
meson compile -C build lint
```

Expected identity impact:

```text
Attestation version:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

architecture schema:
    unchanged

realization identity:
    none exists

compiler identity:
    not minted

target definition digest:
    not minted

deployment binding digest:
    not minted

linked bundle identity:
    absent

transaction ABI identity:
    absent
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
    may change only if deliberate participating Rust labels are added

Specification and realization label registers:
    unchanged unless source labels change
```

Adding workspace packages normally updates `Cargo.lock`’s first-party package census. Review that change explicitly even if no new third-party dependency is added.

If the sponsor-role correction changes internal realization declarations but not architecture behavior:

```text
architecture hashes:
    unchanged

realization public identity:
    none exists

generated architecture publications:
    unchanged
```

Any generated label publication change must derive only from deliberate participating source labels.

---

# 27. Full batch gate

After all waves:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Because this guide adds packages and build-census entries, the mocked Meson contract and census audit are required.

Run:

```sh
CI_REQUIRE_MESON=1 scripts/ci.sh
```

in at least one required gate environment.

Run:

```sh
scripts/check-plans.sh
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

Paper inputs should normally remain unchanged. Document byte reproducibility may be deferred for this intermediate Phase-3 guide if repository policy permits, but it must be recorded as deferred rather than passed.

If the realization document is changed to correct the sponsor vector, run the relevant documentation and reproducibility surface required by the owning policy.

Finally:

```sh
git status --porcelain=v1 --untracked-files=all
```

The final tree must be clean after commits.

---

# 28. Guide-8 exit criteria

Guide 8 is complete only when every applicable assertion below holds.

## Package boundaries

- [ ] `tripod-target-elements` exists;
- [ ] `target_elements` library exists;
- [ ] target-elements has no first-party semantic dependency;
- [ ] `tripod-tapscript` exists;
- [ ] tapscript depends directly on compiler and target-elements;
- [ ] compiler depends on neither target package;
- [ ] Meson census includes every new source;
- [ ] no unused C toolchain is required by the documentation-only subproject.

## Preflight boundary repairs

- [ ] sponsor erasure is keyed to exact fee-sponsor-region membership;
- [ ] protocol-role `PLAIN_LBTC` values remain readable where required;
- [ ] complete analyzed-program validation runs before public projection;
- [ ] duplicate graph/projection members reject;
- [ ] dependency graph orientation is coherent or graph roles are separated;
- [ ] exact-search counters cannot wrap;
- [ ] zero-valued ordinary sponsor-member semantics are consistent across prose, model, compiler, and vectors.

## Target definition

- [ ] target-contract version is explicit;
- [ ] execution domain is typed;
- [ ] leaf version is typed;
- [ ] reviewed opcode identities are typed;
- [ ] every opcode has operand, result, failure, and resource contracts;
- [ ] encoding classes are separate and canonical;
- [ ] asset and value representation are independent;
- [ ] unknown encoding behavior is fail-closed;
- [ ] target definition validates deterministically.

## Authorization, CT, issuance, and resources

- [ ] sighash dimensions are typed without selecting protocol policy;
- [ ] relative-timelock dimensions are typed;
- [ ] CT conservation and commitment equality remain distinct;
- [ ] authenticated opening is not overclaimed;
- [ ] issuance and reissuance interfaces are typed;
- [ ] consensus and policy limits remain separate;
- [ ] protocol batch bounds remain outside target-elements.

## Evidence

- [ ] every target capability has typed evidence requirements where needed;
- [ ] evidence requirements contain no mutable result;
- [ ] review provenance remains outside the semantic target projection;
- [ ] node revision remains provenance only;
- [ ] no target-native report is claimed unless actually produced;
- [ ] production activation is not claimed.

## Deployment binding

- [ ] development environment is explicit;
- [ ] network ID is nonzero;
- [ ] genesis ID is nonzero;
- [ ] activation is a declaration, not a verified report;
- [ ] production binding is unavailable or rejected;
- [ ] no credential-bearing field exists;
- [ ] target/deployment mismatch fails.

## Compiler boundary

- [ ] `RequiredCapability` is available through a reviewed public boundary;
- [ ] capability census is complete;
- [ ] complete analyzed-program internals remain private;
- [ ] no target-specific type enters compiler API;
- [ ] no compiler digest is added;
- [ ] public target requirements are obtainable only from a fully validated internal analysis.

## Adapter

- [ ] every compiler capability is assessed exactly once;
- [ ] assessment is multi-state, not Boolean;
- [ ] missing primitives fail closed;
- [ ] backend-pattern requirements remain unresolved;
- [ ] compiler/ABI structural obligations remain distinct;
- [ ] external evidence remains distinct;
- [ ] no compiler capability disappears;
- [ ] no unsupported capability is weakened;
- [ ] no sponsor amount is required;
- [ ] no attestation-contract operation is emitted.

## Determinism and identity

- [ ] target projections are insertion-order independent;
- [ ] adapter projections are requirement-order independent;
- [ ] duplicate projection entries reject;
- [ ] no local graph handle enters a stable projection;
- [ ] no target hash is minted;
- [ ] no deployment hash is minted;
- [ ] no report digest is minted;
- [ ] no generated target publication is added without a consumer.

## Verification

- [ ] focused package tests pass;
- [ ] Rustdoc passes with warnings denied;
- [ ] workspace formatting passes;
- [ ] workspace Clippy passes;
- [ ] workspace debug tests pass;
- [ ] workspace release tests pass through CI;
- [ ] dependency graph is reviewed;
- [ ] labels and generated checks pass;
- [ ] Meson compile passes;
- [ ] Meson tests pass;
- [ ] mocked census-audit edge passes;
- [ ] skipped or deferred lanes are reported honestly;
- [ ] final tree is clean.

---

# 29. Completion report template

```text
Guide 8 result
==============

Entry:
    starting revision:
    Phase-2 gate:
    preflight tree status:

Preflight repairs:
    sponsor erasure:
    protocol-role L-BTC:
    analyzed-program validation:
    duplicate projection rejection:
    graph orientation:
    exact-search counters:
    zero-valued sponsor vector:

Source review:
    upstream source:
    reviewed revision:
    reviewed locations:
    upstream tests:
    licence:
    accepted static claims:
    unresolved target-native claims:

Target-elements package:
    package boundary:
    first-party dependencies:
    target-contract version:
    execution domain:
    leaf version:
    opcode census:
    encoding census:
    authorization contract:
    CT contract:
    issuance contract:
    resource contract:
    evidence registry:

Development binding:
    environment:
    network/genesis:
    activation declaration:
    production status:

Compiler target boundary:
    exposed types:
    capability census:
    requirement-set projection:
    validation precondition:
    analyzed-program visibility:
    target-specific compiler types:

Tapscript adapter:
    compiler capability census:
    unsupported assessments:
    missing-primitive assessments:
    backend-pattern obligations:
    structural obligations:
    external-evidence obligations:
    complete backend patterns:
    sponsor opacity:

Determinism:
    target-definition permutations:
    encoding permutations:
    capability permutations:
    evidence permutations:
    adapter input permutations:
    duplicate projection mutations:

Identity impact:
    Attestation version:
    realization letter:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:
    target-definition identity:
    deployment identity:
    backend identity:

Dependency impact:
    new first-party packages:
    new third-party dependencies:
    Cargo.toml:
    Cargo.lock:
    licence:
    MSRV:
    unsafe boundary:
    advisory status:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
    target-elements tests:
    tapscript tests:
    compiler tests:
    realization tests:
    model conformance tests:
    Rustdoc:
    cargo tree:
    cargo metadata:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    CI_REQUIRE_MESON=1 scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning result:
    Phase 3:
    completed target milestones:
    completed backend milestones:
    next task:

Residuals:
```

---

# 30. Next guide after Guide 8

After Guide 8 passes, the next guide is:

```text
Guide 9 — Target-Native Primitive Conformance and Typed Tapscript Instruction Core
```

Its scope should include:

- secretless development target-execution boundary;
- explicit test-environment configuration;
- exact opcode byte encoding;
- typed tapscript instruction values;
- stack and altstack validation;
- success/failure stack conformance;
- target-native positive and malformed vectors for the reviewed primitive set;
- resource observation for individual primitives;
- evidence reports that remain distinct from backend-pattern evidence;
- no attestation-contract operation emission yet.

Guide 9 must not start STATE continuity, wide floor arithmetic, or compact-ASH operation emission until the primitive target contracts used by those constructions have target-native evidence.

The intended continuation is:

```text
Guide 8
    typed static target contract
    +
    compiler capability adapter

Guide 9
    target-native primitive evidence
    +
    typed instruction core

later guides
    backend proof patterns
    →
    relocatable operation programs
    →
    linking
    →
    transaction ABI
    →
    bundle-specific evidence
```

The governing rule survives unchanged:

> Primitive availability is not a backend proof; a backend proof is not linked-bundle evidence; linked-bundle evidence is not deployment release.

# Guide 8 — Remainder and implementation appendices

The main guide ends at §30. The following appendices continue it with the concrete implementation inventory, provisional typed interfaces, exhaustive capability mapping, validation matrices, review checklist, and gate-record format needed to execute the guide without inventing policy during implementation.

---

# Appendix A — Exact implementation inventory

## A.1 Expected new package tree

Guide 8 should create, at minimum:

```text
packages/
├── target-elements/
│   ├── Cargo.toml
│   ├── README.md
│   ├── meson.build
│   ├── src/
│   │   ├── authorization.rs
│   │   ├── capability.rs
│   │   ├── definition.rs
│   │   ├── deployment.rs
│   │   ├── encoding.rs
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   ├── opcode.rs
│   │   ├── resource.rs
│   │   └── tests/
│   │       ├── authorization_tests.rs
│   │       ├── capability_tests.rs
│   │       ├── definition_tests.rs
│   │       ├── deployment_tests.rs
│   │       ├── encoding_tests.rs
│   │       ├── mutation_tests.rs
│   │       ├── opcode_tests.rs
│   │       ├── permutation_tests.rs
│   │       ├── resource_tests.rs
│   │       └── mod.rs
│   └── tests/
│       └── public_api.rs
└── tapscript/
    ├── Cargo.toml
    ├── README.md
    ├── meson.build
    ├── src/
    │   ├── capability.rs
    │   ├── error.rs
    │   ├── lib.rs
    │   └── tests/
    │       ├── capability_tests.rs
    │       ├── census_tests.rs
    │       ├── mutation_tests.rs
    │       ├── oracle_tests.rs
    │       ├── permutation_tests.rs
    │       └── mod.rs
    └── tests/
        └── public_api.rs
```

This is a recommended factoring, not a requirement to create empty modules. A source file should exist only when it owns implemented behavior.

Do not add placeholder modules for:

```text
instruction
program
stack_scheduler
constructor
relocation
bundle
abi
transaction
executor
rpc
report
release
```

Those modules belong to later guides.

## A.2 Expected modified files

At minimum:

```text
Cargo.toml
Cargo.lock
packages/meson.build
packages/compiler/Cargo.toml
packages/compiler/README.md
packages/compiler/meson.build
packages/compiler/src/capability.rs
packages/compiler/src/lib.rs
packages/compiler/tests/public_api.rs

plans/backlog.md
plans/packages/README.md
plans/packages/compiler.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/phases/03-target-foundation.md
plans/reference/elements-tapscript.md
```

If preflight repairs remain open, also expect changes in:

```text
docs/attestation/realization.md

packages/compiler/src/analyzed.rs
packages/compiler/src/analyzed_validate.rs
packages/compiler/src/coverage_graph.rs
packages/compiler/src/proof.rs
packages/compiler/src/placement.rs

packages/model/src/conformance.rs

packages/realization/src/evaluate.rs
packages/realization/src/observation.rs
packages/realization/src/validate.rs
```

Generated files change only when their typed source actually changes.

## A.3 Workspace dependency declarations

Recommended workspace entries:

```toml
[workspace]
members = [
    # existing members
    "packages/target-elements",
    "packages/tapscript",
]

[workspace.dependencies]
target-elements = {
    package = "tripod-target-elements",
    path = "packages/target-elements",
}
tapscript = {
    package = "tripod-tapscript",
    path = "packages/tapscript",
}
```

Package dependencies:

```toml
# packages/target-elements/Cargo.toml
[dependencies]
# Prefer none initially.

[lints]
workspace = true
```


```toml
# packages/tapscript/Cargo.toml
[dependencies]
compiler = { workspace = true }
target-elements = { workspace = true }

[lints]
workspace = true
```

Do not add Serde merely because a future publication may exist. No target publication exists in Guide 8.

Do not add hashing merely because a future target identity may exist. No target digest is admitted in Guide 8.

Do not add an Elements transaction library merely to define static target facts if standard-library typed declarations are sufficient.

## A.4 Meson census ownership

Each package’s `meson.build` must list every authored Rust source explicitly.

Conceptually:

```meson
target_elements_source_files = files(
  'src/authorization.rs',
  'src/capability.rs',
  'src/definition.rs',
  'src/deployment.rs',
  'src/encoding.rs',
  'src/error.rs',
  'src/lib.rs',
  'src/opcode.rs',
  'src/resource.rs',
  'src/tests/authorization_tests.rs',
  'src/tests/capability_tests.rs',
  'src/tests/definition_tests.rs',
  'src/tests/deployment_tests.rs',
  'src/tests/encoding_tests.rs',
  'src/tests/mod.rs',
  'src/tests/mutation_tests.rs',
  'src/tests/opcode_tests.rs',
  'src/tests/permutation_tests.rs',
  'src/tests/resource_tests.rs',
)

target_elements_doc_files = files('README.md')

target_elements_excluded_files = files(
  'tests/public_api.rs',
)
```

Equivalent rules apply to tapscript.

Update `packages/meson.build` to compose:

```text
crate_source_files
packages_doc_files
packages_excluded_files
```

with both new packages.

A newly created tracked file and its census entry belong in the same commit.

---

# Appendix B — Provisional `target-elements` type model

The following sketches define the intended ownership and validation shape. Exact names are not frozen.

## B.1 Target contract version

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetContractVersion(u32);

impl TargetContractVersion {
    pub const V1: Self = Self(1);

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
```

A version is accepted because the adapter makes an explicit compatibility decision from it. It is not an identity digest.

## B.2 Execution domain

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExecutionDomain {
    Tapscript,
}
```

Do not add:

```text
LegacyScript
SegwitV0
Simplicity
```

unless the target package actually reviews and exposes those domains.

## B.3 Leaf version

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeafVersion(u8);

impl LeafVersion {
    pub fn new(value: u8) -> Result<Self, TargetError> {
        // Validate against the reviewed target contract.
        todo!()
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}
```

Do not make every byte representable merely because the target field is one byte.

## B.4 Byte order and width

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}
```


```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PayloadWidth {
    Exact(std::num::NonZeroUsize),
    Bounded {
        minimum: usize,
        maximum: std::num::NonZeroUsize,
    },
}
```

Validation must reject:

- minimum greater than maximum;
- zero exact width;
- a field requiring byte order but carrying none;
- a variable-width representation described as fixed width;
- two canonical encodings for one logical value without a disambiguating rule.

## B.5 Encoding classes

A possible initial vocabulary:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EncodingClass {
    ExplicitAsset,
    ConfidentialAsset,
    ExplicitValue,
    ConfidentialValue,
    NullValue,
    NullNonce,
    ConfidentialNonce,
    ScriptProgram,
    OutPoint,
    Sequence,
    Issuance,
    SignedFixedWidth64,
    ScriptNumber,
    XOnlyPublicKey,
    Signature,
}
```

Do not force all target fields into one encoding class.

## B.6 Canonical encoding behavior

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnknownPrefixRule {
    Reject,
}
```


```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalEncodingRule {
    Unique,
    Minimal,
    FixedWidth,
    PrefixDiscriminated,
}
```

Guide 8 should not add a permissive unknown-prefix mode unless a reviewed target rule requires it.

## B.7 Opcode identity

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpcodeId {
    Sha256Initialize,
    Sha256Update,
    Sha256Finalize,

    InspectInputOutpoint,
    InspectInputAsset,
    InspectInputValue,
    InspectInputProgram,
    InspectInputSequence,
    InspectInputIssuance,

    PushCurrentInputIndex,

    InspectOutputAsset,
    InspectOutputValue,
    InspectOutputNonce,
    InspectOutputProgram,

    InspectVersion,
    InspectLockTime,
    InspectInputCount,
    InspectOutputCount,
    InspectTransactionWeight,

    Add64,
    Sub64,
    Mul64,
    Div64,
    Neg64,

    Equal64,
    LessThan64,
    LessThanOrEqual64,
    GreaterThan64,
    GreaterThanOrEqual64,

    ScriptNumberTo64,
    SixtyFourToScriptNumber,

    CheckSignature,
    CheckSequenceVerify,

    EcMulScalarVerify,
    TweakVerify,
}
```

This list is illustrative. Admit only reviewed primitives.

Each variant must have a stable typed census:

```rust
impl OpcodeId {
    pub const ALL: &'static [Self] = &[
        // exact complete census
    ];
}
```

## B.8 Stack contract

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackContract {
    operands: Vec<StackValueType>,
    success_results: Vec<StackValueType>,
    failure: FailureContract,
}
```

Read-only accessors should expose slices. Fields stay private.

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackValueType {
    Bool,
    ScriptNumber,

    Bytes {
        minimum: usize,
        maximum: usize,
    },

    SignedFixedWidth {
        bytes: std::num::NonZeroUsize,
        byte_order: ByteOrder,
    },

    Encoded(EncodingClass),
}
```

## B.9 Failure contract

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureContract {
    Abort,

    PushFalse {
        retained_operands: Vec<StackValueType>,
        additional_results: Vec<StackValueType>,
    },

    Conditional {
        causes: Vec<FailureCause>,
    },
}
```


```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureCause {
    StackUnderflow,
    InvalidWidth,
    InvalidEncoding,
    IndexOutOfRange,
    ArithmeticOverflow,
    DivisionByZero,
    InvalidSignature,
    InvalidTimelock,
    InvalidPoint,
    InvalidTweak,
    ResourceLimit,
}
```

The typed contract must distinguish:

```text
abort
≠
push false
≠
preserve operands and push false
```

## B.10 Opcode resource cost

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpcodeResourceCost {
    script_bytes: u64,
    operation_cost: u64,
    crypto_budget: u64,
    maximum_stack_growth: i64,
    maximum_altstack_growth: i64,
}
```

Units remain separate.

Do not define a single weighted “cost” score.

## B.11 Opcode specification

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpcodeSpec {
    id: OpcodeId,
    code: u8,
    domains: BTreeSet<ExecutionDomain>,
    stack: StackContract,
    resources: OpcodeResourceCost,
    evidence:
        BTreeSet<TargetEvidenceRequirementId>,
}
```

Validation must ensure `id` agrees with the registry key. A map entry must not claim one ID while containing another.

## B.12 Target capabilities

A possible target-owned vocabulary:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElementsCapability {
    TapscriptExecution,
    RequiredLeafVersion,

    InputCountInspection,
    OutputCountInspection,
    CurrentInputIndexInspection,

    InputOutpointInspection,
    InputAssetInspection,
    InputValueInspection,
    InputProgramInspection,
    InputSequenceInspection,
    InputIssuanceInspection,

    OutputAssetInspection,
    OutputValueInspection,
    OutputNonceInspection,
    OutputProgramInspection,

    TransactionVersionInspection,
    TransactionLockTimeInspection,
    TransactionWeightInspection,

    SignedFixedWidthArithmetic,
    SignedFixedWidthComparison,
    ScriptNumberConversion,

    SignatureVerification,
    OutputCommittingSighash,
    InputCommitmentControl,
    RelativeTimelock,

    StreamingSha256,
    EcScalarVerification,
    TweakVerification,

    ConfidentialValueConservation,
    CommitmentEquality,
    ExplicitValueInspection,
    IssuanceIntrospection,
    ReissuanceIntrospection,

    ConsensusResourceLimits,
    PolicyResourceLimits,
}
```

This vocabulary names target facts. It does not name protocol proof completion.

## B.13 Capability contracts

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityContract {
    capability: ElementsCapability,
    prerequisites: BTreeSet<ElementsCapability>,
    opcodes: BTreeSet<OpcodeId>,
    encodings: BTreeSet<EncodingClass>,
    evidence:
        BTreeSet<TargetEvidenceRequirementId>,
    status: StaticCapabilityStatus,
}
```


```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticCapabilityStatus {
    Reviewed,
    Incomplete,
    Unsupported,
}
```

`Reviewed` means the typed static contract has been reviewed. It does not mean deployment-evidenced.

## B.14 Target evidence requirements

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetEvidenceRequirementId {
    TapscriptExecutionDomain,
    LeafVersionActivation,
    OpcodeSemantics,
    InputIntrospectionSemantics,
    OutputIntrospectionSemantics,
    TransactionIntrospectionSemantics,
    ArithmeticSemantics,
    SignatureSemantics,
    SighashSemantics,
    RelativeTimelockSemantics,
    ConfidentialValueConservation,
    CommitmentEquality,
    IssuanceIntrospection,
    ConsensusResourceLimits,
    PolicyResourceLimits,
}
```

If one requirement needs a narrower subject, store it as typed data:

```rust
pub struct TargetEvidenceRequirement {
    id: TargetEvidenceRequirementId,
    subject: TargetEvidenceSubject,
    environment: RequiredEvidenceEnvironment,
    stale_on: BTreeSet<EvidenceStaleCondition>,
}
```

No pass/fail field belongs here.

## B.15 Target definition and wrapper

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetDefinition {
    version: TargetContractVersion,
    execution_domain: ExecutionDomain,
    leaf_version: LeafVersion,
    opcodes: BTreeMap<OpcodeId, OpcodeSpec>,
    encodings: BTreeMap<EncodingClass, EncodingSpec>,
    authorization: AuthorizationContract,
    confidential_values: ConfidentialValueContract,
    issuance: IssuanceContract,
    resources: ResourceContract,
    capabilities:
        BTreeMap<ElementsCapability, CapabilityContract>,
    evidence_requirements:
        BTreeMap<TargetEvidenceRequirementId, TargetEvidenceRequirement>,
}
```


```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedTargetDefinition {
    definition: TargetDefinition,
}
```

Only:

```rust
pub fn validate_target_definition(
    definition: TargetDefinition,
) -> Result<ValidatedTargetDefinition, Vec<TargetError>>;
```

or a built-in validated constructor may produce the wrapper.

---

# Appendix C — Development deployment model

## C.1 Environment class

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeploymentEnvironment {
    Development,
    Production,
}
```

Guide 8 must expose no successful production constructor.

## C.2 Activation declaration

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationDeclaration {
    tapscript_expected_active: bool,
    required_leaf_version: LeafVersion,
    required_capabilities: BTreeSet<ElementsCapability>,
}
```

This is a declaration of the environment the caller intends to test.

It is not an observation or report.

## C.3 Development binding

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentDeploymentBinding {
    target_version: TargetContractVersion,
    environment: DeploymentEnvironment,
    network_id: [u8; 32],
    genesis_id: [u8; 32],
    activation: ActivationDeclaration,
    resource_overrides:
        Option<DevelopmentResourceOverrides>,
}
```

## C.4 Validation wrapper

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedDevelopmentBinding {
    binding: DevelopmentDeploymentBinding,
}
```

Validation rejects:

```text
environment = Production
network_id = 0
genesis_id = 0
target version unsupported
activation leaf version inconsistent
capability not declared by target
resource override incompatible with target
```

## C.5 Combined target

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementsTarget {
    definition: ValidatedTargetDefinition,
    deployment: ValidatedDevelopmentBinding,
}
```

Construction:

```rust
pub fn bind_development_target(
    definition: ValidatedTargetDefinition,
    deployment: ValidatedDevelopmentBinding,
) -> Result<ElementsTarget, TargetError>;
```

The result proves only typed self-consistency.

It does not prove:

- actual activation;
- actual network behavior;
- target implementation correctness;
- production equivalence;
- deployment readiness.

---

# Appendix D — Minimal compiler public boundary

## D.1 Public module

A narrow public module is preferable:

```rust
pub mod target {
    pub use crate::capability::RequiredCapability;

    pub struct TargetRequirementSet {
        // private fields
    }
}
```

Do not make the complete compiler module tree public.

## D.2 Capability census

The current compiler capability set is:

```rust
pub enum RequiredCapability {
    AuthenticatedObjectRecognition,
    AuthenticatedFamilyCardinality,
    AuthenticatedCanonicalPartition,
    AuthenticatedOpenFlowPartition,
    AuthenticatedRootEffects,
    AuthenticatedProjectionSet,

    ExactPublicAmountArithmetic,
    ConfidentialValueConservation,

    OwnerAuthorization,
    OperatorAuthorization,
    RefundAuthorization,
    PublicConstructibility,

    WholeTransactionValueConservation,
}
```

Add:

```rust
impl RequiredCapability {
    pub const ALL: &'static [Self] = &[
        Self::AuthenticatedObjectRecognition,
        Self::AuthenticatedFamilyCardinality,
        Self::AuthenticatedCanonicalPartition,
        Self::AuthenticatedOpenFlowPartition,
        Self::AuthenticatedRootEffects,
        Self::AuthenticatedProjectionSet,
        Self::ExactPublicAmountArithmetic,
        Self::ConfidentialValueConservation,
        Self::OwnerAuthorization,
        Self::OperatorAuthorization,
        Self::RefundAuthorization,
        Self::PublicConstructibility,
        Self::WholeTransactionValueConservation,
    ];
}
```

The order is a stable typed census order, not semantic priority.

## D.3 Read-only target requirement set

A provisional shape:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRequirementSet {
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence:
        BTreeSet<realization::ExternalEvidenceRequirement>,
}
```

Public read-only methods:

```rust
impl TargetRequirementSet {
    pub fn capabilities(
        &self,
    ) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    pub fn external_evidence(
        &self,
    ) -> impl Iterator<
        Item = &realization::ExternalEvidenceRequirement,
    > {
        self.external_evidence.iter()
    }
}
```

If exposing `realization::ExternalEvidenceRequirement` would force tapscript to depend directly on realization through the public type, prefer a compiler-owned projection of the evidence role.

The package boundary must reflect actual public type ownership rather than hide it through accidental transitive re-exports.

## D.4 Construction rule

Only the compiler may construct the set:

```rust
pub(crate) fn project_target_requirements(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<TargetRequirementSet, CompileError>;
```

Before projection:

```rust
validate_scoped_analyzed_program(
    input,
    placement_limits,
    analyzed,
)?;
```

must have passed.

The public set must not expose a constructor from arbitrary capabilities unless the type is explicitly named as an unvalidated request rather than compiler analysis.

## D.5 Complete validator integration

The canonical analysis path should become conceptually:

```rust
let program = assemble(...)?;
let expectations = derive_expectations(...)?;
validate_against_expectations(
    input,
    &expectations,
    &program,
)?;
Ok(program)
```

The narrow `validate_assembly_closure` remains useful internally but does not replace complete validation.

## D.6 Exact graph projection validation

Coverage graph validation must reject:

- duplicate node;
- duplicate edge;
- missing node;
- unexpected node;
- missing edge;
- unexpected edge;
- noncanonical order where order is part of the projection contract.

Do not compare only:

```rust
BTreeSet<&CoverageNode>
```

because that erases duplicates.

## D.7 Dependency-graph semantics

Before exposing a projection that carries dependency edges, settle whether the graph means:

```text
prerequisite → dependent
```

or:

```text
owner → owned requirement
```

Do not mix both in one graph while documenting one global orientation.

A recommended split is:

```text
CoveragePrerequisiteGraph
    relation prerequisites and collateral closure

CoverageOwnershipProjection
    relation → carrier/layout/evidence presentation
```

Only the prerequisite graph receives SCC and dependency-closure semantics.

## D.8 Exact-search reports

Search counters must not panic or wrap.

The implementation may use:

```rust
fn increment_states(
    current: &mut u64,
    limit: NonZeroU64,
) -> Result<(), CompileError>
```

with a precisely tested inclusive/exclusive rule.

Diagnostic rejection reasons should be typed:

```rust
enum LocalProofRejection {
    MissingCapability,
    SourceDerivation,
    Constructibility,
    Representation,
    Disclosure,
    Lifecycle,
}
```

Do not count every local failure as capability rejection.

---

# Appendix E — Exhaustive tapscript adapter mapping

## E.1 Assessment vocabulary

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityAssessment {
    Unsupported {
        required: RequiredCapability,
        reason: UnsupportedReason,
    },

    MissingTargetPrimitives {
        required: RequiredCapability,
        missing: BTreeSet<ElementsCapability>,
    },

    BackendPatternRequired {
        required: RequiredCapability,
        primitives: BTreeSet<ElementsCapability>,
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },

    BackendStructural {
        required: RequiredCapability,
        requirements: BTreeSet<BackendFoundationRequirement>,
    },

    ExternalEvidenceRequired {
        required: RequiredCapability,
        evidence: BTreeSet<TargetEvidenceRequirementId>,
    },

    CompleteBackendPattern {
        required: RequiredCapability,
        pattern: BackendPatternId,
    },
}
```

## E.2 Backend foundation requirements

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendFoundationRequirement {
    CanonicalFamilyLayout,
    CompleteFamilyCensus,
    CompleteAndDisjointProtocolFamilies,
    ProtocolSponsorRegionSeparation,
    CanonicalCoordinator,
    RootConstructorContinuity,
    ProjectionShape,
    PublicConstructionData,
    SecretFreePermissionlessPath,
}
```

These requirements are not completed target programs.

## E.3 Production mapping

The production adapter should use an exhaustive match with no wildcard arm.

### Authenticated object recognition

Required target primitives may include:

```text
InputAssetInspection
InputProgramInspection
OutputAssetInspection
OutputProgramInspection
Explicit encoding recognition
```

Assessment:

```text
BackendPatternRequired
```

Reason:

- primitives can inspect facts;
- no complete object-recognition backend pattern exists yet;
- constructor and object-family policy remain protocol/backend-owned.

### Authenticated family cardinality

Required target primitives may include:

```text
InputCountInspection
OutputCountInspection
CurrentInputIndexInspection
```

Structural requirements:

```text
CanonicalFamilyLayout
CompleteFamilyCensus
CanonicalCoordinator
```

Assessment:

```text
BackendStructural
```

or `BackendPatternRequired` plus structural requirements, depending on the final assessment factoring.

A count opcode alone does not prove family membership.

### Authenticated canonical partition

Required target primitives may include:

```text
input/output count inspection
asset inspection
value inspection or commitment relation
program inspection
```

Structural requirements:

```text
CanonicalFamilyLayout
CompleteAndDisjointProtocolFamilies
CanonicalCoordinator
```

Assessment:

```text
BackendPatternRequired
```

No complete proof pattern exists in Guide 8.

### Authenticated open-flow partition

Required target primitives may include:

```text
L-BTC asset inspection
program inspection
count/index inspection
whole-transaction conservation evidence
```

Structural requirements:

```text
ProtocolSponsorRegionSeparation
CanonicalFamilyLayout
CompleteFamilyCensus
```

Assessment:

```text
BackendPatternRequired
```

Important:

```text
no sponsor amount inspection
no sponsor positivity
no public sponsor subtotal
```

### Authenticated root effects

Required target primitives may include:

```text
input asset and program inspection
output asset and program inspection
root predecessor/successor binding
target hashing/tweak primitives
```

Structural requirement:

```text
RootConstructorContinuity
```

Assessment:

```text
BackendPatternRequired
```

or `Unsupported` if the reviewed target contract lacks a required primitive.

Guide 8 does not claim STATE or RESV constructor continuity.

### Authenticated projection set

Required target primitives may include:

```text
output count and program inspection
data-output encoding rules
```

Structural requirement:

```text
ProjectionShape
```

Assessment:

```text
BackendPatternRequired
```

### Exact public amount arithmetic

Required target primitives may include:

```text
ExplicitValueInspection
SignedFixedWidthArithmetic
SignedFixedWidthComparison
ScriptNumberConversion
```

Evidence:

```text
ArithmeticSemantics
```

Assessment:

```text
BackendPatternRequired
```

The presence of 64-bit arithmetic does not complete wide floor arithmetic.

### Confidential value conservation

Target capability:

```text
ConfidentialValueConservation
```

Evidence:

```text
ConfidentialValueConservation
```

Assessment:

```text
ExternalEvidenceRequired
```

Object and recipient closure remain separate backend obligations.

### Owner authorization

Target capabilities may include:

```text
SignatureVerification
OutputCommittingSighash
InputCommitmentControl
```

Evidence:

```text
SignatureSemantics
SighashSemantics
```

Assessment:

```text
BackendPatternRequired
```

No operation-specific owner pattern exists yet.

### Operator authorization

Same target primitives as owner authorization.

Assessment:

```text
BackendPatternRequired
```

The adapter does not select which operation uses an operator.

### Refund authorization

Same target primitives as owner authorization.

Assessment:

```text
BackendPatternRequired
```

The adapter does not know request semantics.

### Public constructibility

Structural requirements:

```text
PublicConstructionData
SecretFreePermissionlessPath
CanonicalFamilyLayout
```

Assessment:

```text
BackendStructural
```

This is not reducible to one opcode.

### Whole-transaction value conservation

Target capability:

```text
ConfidentialValueConservation
```

or the target’s exact explicit/confidential whole-transaction conservation contract.

Evidence:

```text
ConfidentialValueConservation
```

Assessment:

```text
ExternalEvidenceRequired
```

This capability must not be replaced by sponsor positivity or a protocol-local subtotal.

## E.4 Set-level result

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityAssessmentSet {
    assessments:
        BTreeMap<RequiredCapability, CapabilityAssessment>,
}
```

Construction must verify:

```text
keys(assessments)
=
keys(requirements)
```

exactly.

The set should expose read-only iteration in stable compiler capability order.

---

# Appendix F — Sponsor-region correction

## F.1 Required semantic distinction

The system needs at least three concepts:

```text
object family:
    PLAIN_LBTC

protocol open-flow roles:
    request-creation
    request-refund
    deposit-admission
    reserve-carry
    redemption

sponsor open-flow role:
    fee-sponsor
```

Object family does not determine role.

## F.2 Model projection

The model conformance adapter should derive the region from exact open-flow membership before projecting values.

Conceptually:

```rust
let sponsor_refs = certificate
    .open_flows
    .iter()
    .filter(|flow| flow.kind == OpenFlowKind::FeeSponsor)
    .flat_map(|flow| {
        flow.source_inputs
            .iter()
            .chain(&flow.destination_outputs)
    })
    .copied()
    .collect::<BTreeSet<_>>();
```

Then:

```rust
let value = if sponsor_refs.contains(&outpoint) {
    ObservedValue::SponsorOpaque
} else {
    ObservedValue::Protocol(observed_amount(utxo.value)?)
};
```

The projection must reject overlap rather than rely solely on prior construction.

## F.3 Realization observation

`ObservedValue::SponsorOpaque` should be legal only for references in the typed fee-sponsor region.

A `PLAIN_LBTC` object in a protocol flow may carry:

```rust
ObservedValue::Protocol(amount)
```

A `PLAIN_LBTC` object in the fee-sponsor flow must carry:

```rust
ObservedValue::SponsorOpaque
```

## F.4 Compiler activation

Replace object-based sponsor activation:

```text
WhenSponsorPresent because PLAIN_LBTC exists
```

with flow-role activation:

```text
WhenOpenFlowPresent(FeeSponsor)
```

or an equivalent typed case dimension.

Request creation’s mandatory `PLAIN_LBTC` input must not make the operation a sponsored case.

## F.5 Compiler operand identity

A protocol L-BTC amount and sponsor amount need distinct operands.

Conceptually:

```rust
pub enum OperandRole {
    ProtocolOpenFlowAmount {
        flow: OpenFlowKind,
        side: TransactionSide,
    },

    SponsorRegion,

    // no SponsorAmount operand
}
```

Sponsor amount remains structurally absent.

## F.6 Required regressions

For redemption:

\[
p=\left\lfloor\frac{x\Omega}{Y}\right\rfloor.
\]

Construct two observations:

```text
good:
    payout = p
    sponsor change = c

bad:
    payout = p - 1
    sponsor change = c + 1
```

Both preserve total L-BTC conservation.

Expected:

```text
good:
    protocol payout relation passes

bad:
    protocol payout relation fails

sponsor positivity:
    irrelevant in both
```

For zero-valued sponsor output:

```text
exact fee-sponsor membership:
    accepted semantically

first-party builder:
    omits known zero change

anchor substitution:
    rejected by family or ABI role
```

---

# Appendix G — Validation and mutation matrix

## G.1 Target-definition validation matrix

| Mutation | Expected error |
|---|---|
| duplicate opcode ID | `DuplicateOpcodeId` |
| duplicate opcode byte | `DuplicateOpcodeCode` |
| missing stack contract | `MissingOpcodeContract` or focused equivalent |
| missing failure contract | `MissingOpcodeFailureContract` |
| missing resource cost | `MissingOpcodeResourceCost` |
| unsupported domain | `UnsupportedOpcodeExecutionDomain` |
| zero fixed width | `InvalidEncodingWidth` |
| conflicting prefix | `DuplicateEncodingPrefix` |
| missing byte order | `MissingByteOrder` |
| unknown capability prerequisite | `UnknownCapabilityPrerequisite` |
| capability cycle | `CapabilityDependencyCycle` |
| missing evidence role | `MissingCapabilityEvidence` |
| zero network ID | `ZeroNetworkId` |
| zero genesis ID | `ZeroGenesisId` |
| production binding | `ProductionBindingUnsupported` |
| target/deployment version mismatch | `TargetDeploymentVersionMismatch` |

## G.2 Compiler boundary mutation matrix

| Mutation | Expected result |
|---|---|
| omit required capability | reject |
| add unowned capability | reject |
| duplicate capability projection | reject |
| omit external evidence | reject |
| add unowned external evidence | reject |
| project from unvalidated analysis | unconstructible or reject |
| inject target type into compiler public API | compile-time public API test fails |
| expose full analyzed program | compile-time public API test fails |
| add compiler digest | compile-time/public API policy test fails |
| duplicate coverage graph node | reject |
| duplicate coverage graph edge | reject |
| reverse prerequisite edge | graph-orientation regression fails |

## G.3 Adapter mutation matrix

| Mutation | Expected result |
|---|---|
| omit one compiler capability assessment | census mismatch |
| assess one capability twice | duplicate assessment |
| add assessment for unrequested capability | census mismatch |
| remove one required target primitive | missing-primitives disposition |
| remove required evidence role | typed adapter failure |
| mark whole-transaction conservation complete | oracle mismatch |
| mark public constructibility as opcode-complete | oracle mismatch |
| infer object recognition from CT conservation | oracle mismatch |
| infer owner authorization without sighash | oracle mismatch |
| add sponsor positivity | sponsor-opacity regression |
| add exact sponsor amount | sponsor-opacity regression |
| return Boolean support only | public API/type test fails |

## G.4 Stable projection mutation matrix

| Mutation | Expected result |
|---|---|
| reorder set-like declarations | equal projection |
| duplicate declaration | reject |
| reorder map insertion | equal projection |
| change opcode code | unequal projection and validation as applicable |
| change source-review revision only | target projection unchanged |
| change development network ID | deployment projection changes |
| change target contract version | compatibility validation reruns |
| add diagnostic text | semantic projection unchanged |
| add local graph handle | type/projection review failure |
| add digest field | ADR-016 admission failure |

---

# Appendix H — Commit and review discipline

## H.1 Recommended commit series

A clean series is:

```text
compiler: harden the target requirement boundary
realization: make sponsor erasure flow-role specific
docs: correct zero-valued sponsor conformance
plans: review the initial Elements target contract

target-elements: establish the target contract boundary
target-elements: define reviewed tapscript primitives
target-elements: type encodings and capability contracts
target-elements: bind an explicit development instance

compiler: expose the abstract target requirement boundary
tapscript: map compiler requirements to target obligations

plans: record the Elements target foundation gate
```

A wave may need several commits, but each commit should remain coherent and green under its focused checks.

## H.2 Review questions for `target-elements`

A reviewer should answer:

1. Does every typed fact have reviewed upstream support?
2. Is any implementation revision being mistaken for target identity?
3. Does every opcode specify both success and failure behavior?
4. Are operand order and result order explicit?
5. Are byte orders field-specific?
6. Are asset and value representations independent?
7. Are unknown encodings rejected?
8. Are consensus and policy limits separate?
9. Does every non-type-level capability map to evidence requirements?
10. Does the package contain any attestation-contract object, operation, or relation?
11. Does it contain any secret-bearing field?
12. Does it claim production activation?
13. Has any speculative digest been introduced?

## H.3 Review questions for the compiler boundary

1. Is the public surface smaller than the complete analyzed program?
2. Can a caller construct a target requirement set directly?
3. Did the complete analyzed-program validator run first?
4. Are capability and evidence censuses exact in both directions?
5. Do duplicate projection entries fail?
6. Are local graph handles absent?
7. Are target-specific types absent?
8. Is sponsor erasure role-based rather than object-based?
9. Does a target requirement retain every unsupported capability visibly?
10. Was a compiler identity avoided?

## H.4 Review questions for tapscript

1. Does every compiler capability have exactly one assessment?
2. Is the mapping exhaustive with no wildcard arm?
3. Does the oracle independently restate the mapping?
4. Are primitive availability and backend-pattern completion separate?
5. Are structural and external-evidence obligations separate?
6. Is whole-transaction conservation still external evidence?
7. Is public constructibility structural?
8. Is owner authorization incomplete without a compatible sighash contract?
9. Does any assessment require a sponsor amount?
10. Does any type imply that an operation has been emitted?
11. Does any assessment claim target-native evidence completion?
12. Is there any target-program or transaction-layout type that belongs to a later guide?

---

# Appendix I — Full verification sequence

## I.1 Focused working checks

After compiler preflight:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --locked -p tripod-compiler
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-model realization_conformance
```

After target-elements:

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

After tapscript:

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

After package integration:

```sh
cargo tree --locked -p tripod-target-elements -e features
cargo tree --locked -p tripod-tapscript -e features
cargo metadata --locked
```

## I.2 Documentation checks

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

## I.3 Full Rust gate

```sh
scripts/ci.sh
```

At least one required environment must run:

```sh
CI_REQUIRE_MESON=1 scripts/ci.sh
```

## I.4 Canonical Meson gate

```sh
meson compile -C build
meson test -C build --print-errorlogs
```

Expected Meson test count may increase if new package-specific tests are added. Record the actual count rather than preserving `10/10` as a hardcoded expectation.

## I.5 Advisory lane

```sh
cargo audit
```

If unavailable:

```text
SKIPPED — cargo-audit unavailable
```

Do not report the complete gate as fully green in the advisory dimension.

## I.6 Document reproducibility

If paper or realization publication inputs changed:

```sh
scripts/check-document-reproducibility.sh
```

Exit success must mean every advertised subcheck ran and passed. A dirty-tree partial skip must not be reported as complete success.

## I.7 Final clean tree

```sh
git status --porcelain=v1 --untracked-files=all
```

Expected output:

```text
<empty>
```

---

# Appendix J — Gate record

After completion, add a compact durable gate record rather than copying this guide into the backlog.

Suggested form:

```text
Guide-8 target foundation gate
==============================

Starting revision:
    <commit>

Preflight:
    sponsor erasure:
        role-based and tested
    protocol-role L-BTC:
        readable where required
    analyzed validation:
        complete validator on construction path
    duplicate projections:
        rejected
    graph orientation:
        <selected rule>
    exact-search counters:
        checked and typed
    zero sponsor member:
        semantic acceptance; builder omission only

Source review:
    upstream:
    reviewed revision:
    source locations:
    upstream tests:
    licence:
    unresolved target-native claims:

Target-elements:
    package:
    first-party dependencies:
    contract version:
    execution domain:
    leaf version:
    opcode count:
    encoding count:
    capability count:
    evidence requirement count:
    production claim:
        none

Development binding:
    environment:
        development
    network/genesis:
        nonzero synthetic values
    activation:
        declaration only
    production constructor:
        absent

Compiler boundary:
    public types:
    capability census:
    target requirement projection:
    complete analyzed program:
        private
    target-specific compiler types:
        none
    compiler identity:
        not minted

Tapscript:
    package:
    dependencies:
        compiler
        target-elements
    assessment census:
    backend pattern completion:
        none
    external evidence retained:
    target programs emitted:
        none
    sponsor amount requirements:
        none

Identity impact:
    Attestation version:
    realization letter:
    architecture semantic hash:
    architecture behavioural hash:
    compiler identity:
        none
    target digest:
        none
    deployment digest:
        none

Dependency impact:
    new first-party crates:
    new third-party crates:
    Cargo.lock:
    licence:
    MSRV:
    unsafe boundary:
    advisories:

Verification:
    MSRV:
    stable:
    cargo fmt:
    clippy:
    debug tests:
    release tests:
    target-elements tests:
    tapscript tests:
    compiler tests:
    realization/model tests:
    Rustdoc:
    plans:
    labels/generated:
    mocked Meson:
    real Meson:
    cargo audit:
    document reproducibility:
    clean tree:

Phase result:
    Phase 3:
        remains active
    completed:
        typed target contract
        development binding
        capability adapter
    next:
        Guide 9 — target-native primitive conformance and typed
        tapscript instruction core

Residuals:
    no target-native primitive report
    no backend proof pattern
    no target program
    no linked bundle
    no ABI
    no deployment readiness
```

---

# Appendix K — Final one-line acceptance statement

When Guide 8 is complete, the strongest permitted claim is:

> The repository contains a validated typed description of the reviewed Elements tapscript capability subset, an explicit development deployment declaration, and an exhaustive target-independent adapter that preserves every compiler-required capability as an unsupported, primitive-dependent, structural, backend-pattern, or external-evidence obligation; it emits no operation and establishes no target-native or production deployment evidence.

The following stronger claims remain prohibited:

```text
Elements backend implemented
compact ASH implemented on target
target semantics verified
production activation verified
compiler translation verified
transaction ABI available
deployment ready
release ready
```

Guide 8 is complete when its typed boundary is honest precisely about those absences.
