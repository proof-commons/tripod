# Guide 8 — Typed Elements Target Contract and Capability-Adapter Foundation

## Mission

Begin Phase 3 by replacing informal Liquid/Elements assumptions with one
validated, target-owned typed compatibility contract.

Guide 8 should establish two package boundaries:

```text
tripod-target-elements
    owns typed Elements/Liquid target facts

tripod-tapscript
    owns the adapter from compiler-required abstract capabilities
    to target primitives, backend-pattern obligations, and external
    evidence requirements
```

The package dependency direction should be:

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
typed Elements target definition
    ↓
one of:
    ├── primitive prerequisites available; backend pattern still required
    ├── compiler/ABI structural obligation
    ├── external target evidence required
    └── unsupported or incomplete capability
```

Guide 8 must **not** emit an attestation-contract operation, link a bundle, define a
transaction ABI, or claim deployment evidence.

At completion, the project should possess an honest typed answer to:

> What exact target contract is being assumed, and what remains to be proved
> before any compiler requirement can be described as implemented on Elements?

---

# 1. Executive rulings

Guide 8 should adopt these rulings before implementation.

## 1.1 Target facts and protocol semantics remain separate

`target-elements` owns:

- execution-domain facts;
- leaf-version facts;
- reviewed opcode identities and semantics;
- operand and result encodings;
- success and failure stack behavior;
- asset/value/nonce encoding classes;
- sighash dimensions;
- relative-timelock dimensions;
- confidential-transaction capabilities;
- issuance and reissuance capabilities;
- target resource interfaces;
- target evidence requirements.

It does **not** own:

- attestation-contract operations;
- attestation-contract objects;
- compiler relations;
- protocol authorization policy;
- protocol batch bounds;
- transaction layouts;
- proof-plan selection;
- backend patterns.

## 1.2 The compiler does not depend on the target

The compiler remains target-independent.

It must not add a dependency on:

```text
target-elements
tapscript
```

The target adapter belongs downstream, in `tapscript`, because that package may
depend on both:

```text
compiler
target-elements
```

## 1.3 Expose only the smallest compiler target boundary

Guide 7 should have kept the complete scoped analyzed-program value
crate-private. Guide 8 introduces the first real downstream consumer, but that
does not justify exposing the whole compiler implementation.

Expose only the minimum read-only public vocabulary needed by the adapter, such
as:

- `RequiredCapability`;
- a stable compiler-owned capability census;
- a read-only target-requirement projection where necessary;
- typed evidence and structural disposition classes needed by the adapter.

Do not expose:

- compiler Petgraph values;
- proof-search state;
- placement-search internals;
- private analyzed-program constructors;
- local graph handles;
- the complete internal analyzed-program DTO merely for convenience.

## 1.4 No target hash yet

Guide 8 should not mint:

```text
TargetDefinitionId
TargetDefinitionHash
DeploymentInstanceHash
TapscriptConfigurationHash
```

unless a persistent cross-process, publication, cache, signature, or report
consumer is introduced in the same series.

For this guide, direct typed comparison is sufficient:

```text
validated typed target definition
+
validated typed development binding
```

ADR-016 rejects a digest when direct typed comparison already makes the
required decision.

A stable target contract version is permitted if a present adapter or validator
uses it to accept or reject supported contract revisions. It is not a digest.

## 1.5 Review provenance is not target identity

The following are review or test provenance:

- upstream repository;
- source revision consulted;
- source paths consulted;
- node version;
- node build;
- local tool version;
- date of review;
- development host;
- test runner.

They must not enter the canonical target-definition projection.

The target definition identifies the typed compatibility contract, not one
Elements implementation revision.

## 1.6 Capability support is not a boolean

Guide 8 must distinguish at least:

```text
unsupported

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
- constructor continuity.

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

A target-native runner requiring credentials needs its own later security and
execution design. Guide 8 should not smuggle one into a unit-test fixture or
environment variable.

---

# 2. Entry conditions

Guide 8 begins only after Guide 7 and the Phase-2 exit gate are complete.

Expected entry state:

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

If the Phase-2 exit record is incomplete or deferred under policy, Guide 8 may
remain a conceptual design but should not advance the active phase.

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
packages/compiler/src/lib.rs
packages/compiler/tests/public_api.rs
```

The human reference under:

```text
plans/reference/elements-tapscript.md
```

is review support only.

No package may parse it.

---

# 4. Scope and non-goals

## 4.1 In scope

Guide 8 should implement:

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
- concrete coordinator index;
- object constructors;
- STATE constructor;
- wide arithmetic;
- public-opening proof;
- transaction ABI;
- target vector execution;
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

A generic third-party dependency may be added only if it has a present consumer
and receives ADR-011 review. Prefer standard-library types initially.

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
linker
transaction
vectors
release
model
artifacts
```

No target program type needs to exist in Guide 8.

The tapscript package’s initial purpose is narrow:

> adapt compiler-owned abstract target requirements to target-owned primitive
> contracts, backend-pattern obligations, and external evidence requirements.

## 5.3 Workspace and Meson

Update:

```text
Cargo.toml
packages/meson.build
meson.build role groups as required
```

Every new source file must enter its package’s explicit `meson.build` census in
the same commit.

Add crate documentation to the DOC census through the package-level Meson
variables.

If integration-test Rust files remain outside the label graph, add explicit
same-typed exclusions as required by ADR-014.

---

# 6. Source review before typed declaration

## 6.1 Do not promote the surveyed reference directly

The current human reference contains provisional opcode numbers and behavioral
summaries.

Guide 8 should not copy those values into typed source solely because they are
already written in Markdown.

For every target fact admitted into `target-elements`, record the review
procedure:

1. identify the deployed capability being relied upon;
2. locate the relevant upstream definition;
3. inspect interpreter behavior;
4. inspect relevant upstream tests;
5. identify activation and execution domain;
6. identify operand/result encodings;
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

Guide 8 should review only primitives needed by the first backend foundation
and capability adapter.

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
- transaction version/locktime/count/weight inspection;
- exact fixed-width arithmetic used by the first target plans;
- exact comparison operations used by the first target plans;
- signature primitive and sighash dimensions;
- relative-timelock dimensions;
- CT value conservation;
- commitment equality;
- issuance/reissuance interfaces;
- consensus and policy resource dimensions.

A primitive may be represented as unavailable or unreviewed. Guide 8 must not
declare support merely to make the adapter succeed.

---

# 7. Typed target definition

## 7.1 Contract shape

A conceptual target definition might be:

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

Fields should remain private. Construction should pass through one validator or
one built-in reviewed declaration.

A public consumer should receive a validated value, not assemble a target
definition through a public struct literal.

## 7.2 Stable typed keys

Target-owned stable keys may include:

```rust
pub enum OpcodeId { ... }

pub enum ElementsCapability { ... }

pub enum TargetEvidenceRequirementId { ... }

pub enum ResourceDimension { ... }

pub enum EncodingClass { ... }

pub enum ExecutionDomain { Tapscript }

pub struct LeafVersion(u8);
```

These are stable typed keys, not digests.

Opcode byte values should be explicit data validated against the reviewed
contract. Avoid assuming enum declaration order equals target opcode code.

## 7.3 Target contract version

A small typed version may be introduced:

```rust
pub struct TargetContractVersion(u32);
```

or:

```rust
pub enum TargetContractVersion {
    V1,
}
```

The adapter should explicitly accept the supported version.

Changing semantic shape or interpretation requires a deliberate contract
version migration. Editorial source-review provenance does not.

Do not add a serialized schema unless external bytes are actually introduced.

## 7.4 Built-in reviewed definition

If the project provides one built-in Liquid/Elements definition, expose it
through a function such as:

```rust
pub fn reviewed_elements_tapscript() -> Result<TargetDefinition, TargetError>;
```

or a validated static value.

Avoid names such as:

```text
production_target
final_target
verified_target
```

The definition is a reviewed static contract. It is not deployment evidence.

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

## 8.3 Success and failure effects

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

This is especially important for fixed-width arithmetic, where failure may
leave values on the stack rather than abort.

## 8.4 Opcode validation

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

## 8.5 No backend instruction builder yet

Guide 8 defines what target primitives mean.

It does not yet define:

```rust
enum TapscriptInstruction
```

or serialize programs.

That belongs to the next backend-instruction guide.

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
null/absent forms
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

Do not automatically preserve an unknown prefix as an opaque valid protocol
value merely because a future target may define it.

Forward-compatibility behavior must be explicit per encoding boundary.

## 9.4 Closed asset policy remains downstream

`target-elements` may describe confidential asset encodings because they exist
on the target.

The initial attestation-contract policy that closed protocol assets must be
explicit is enforced by compiler/backend planning under D005.

The target package does not know which assets are attestation-contract closed
assets.

---

# 10. Authorization and timelock contracts

## 10.1 Sighash dimensions

Represent the target dimensions required by later backend policy:

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

The backend later selects a profile satisfying compiler and operation
requirements.

## 10.2 Signature primitive

Record:

- public-key encoding;
- signature encoding;
- empty or malformed behavior;
- successful and unsuccessful stack effects;
- output-commitment capability dimensions;
- crypto resource cost;
- evidence requirements.

Do not claim that target signature verification proves model authorization. It
is one later evidence layer.

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

## 10.4 Maturity remains out of target timelocks

The target contract may describe absolute and relative timelock facilities if
reviewed.

It must not infer that attestation-contract maturity uses them. The realization
keeps maturity in committed cycle arithmetic.

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

Do not mark `AuthenticatedOpening` complete merely because low-level EC or hash
primitives exist.

## 11.2 Consensus conservation

Whole-transaction value conservation is an external target/consensus claim.

The target contract may state:

- what target mechanism is relied upon;
- what explicit and confidential value classes participate;
- what evidence requirement must later verify it.

It must not mark the compiler relation complete.

## 11.3 Commitment equality

Describe the exact target mechanism only after review.

This capability is relevant to amount-blind relabel and future private
lifecycle paths, but Guide 8 does not implement those paths.

## 11.4 Issuance and reissuance

Describe target facts for:

- issuance field presence;
- null issuance;
- asset entropy;
- issued asset identity;
- issuance amount;
- reissuance/inflation authority;
- input issuance inspection;
- target transaction commitment.

Do not map these facts to `U`, `ENT`, or `DIST_CTL` inside `target-elements`.

That mapping belongs to the backend and transaction ABI.

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

Those remain architecture-declared calibration requirements until complete
transactions and final bundles exist.

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

A practical enum may need non-parameterized variants if stable keys must remain
simple. Equivalent typed factoring is acceptable.

## 13.2 Requirement contents

Each requirement may state:

- subject;
- claim class;
- required test environment;
- static or deployment-scoped status;
- expected evidence role;
- stale conditions.

It should not contain:

- mutable pass/fail result;
- report digest;
- ambient timestamp;
- RPC endpoint;
- credentials;
- node source revision as target identity.

## 13.3 Exact consumer mapping

Every target capability exposed to the adapter must map to one or more evidence
requirements.

A capability with no evidence requirement should fail validation unless its
claim is purely type-level and the contract explains why.

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

Guide 8 should initially construct only a development binding.

Do not provide an API that silently upgrades a development target into a
production target.

## 14.3 Validation

Reject:

- zero network ID;
- zero genesis ID;
- unsupported target contract version;
- target/deployment contract mismatch;
- missing activation declaration;
- incompatible resource declaration;
- production status without production evidence boundary;
- environment-dependent target-definition mutation.

## 14.4 Activation declaration versus evidence

A development binding may declare the expected activation state.

It must not describe that declaration as verified target evidence.

Use distinct terminology:

```text
activation requirement/declaration:
    typed input

activation report:
    future evidence
```

## 14.5 Validated combination

Expose a read-only validated value:

```rust
pub struct ElementsTarget {
    definition: TargetDefinition,
    deployment: DevelopmentDeploymentBinding,
}
```

The exact constructor validates compatibility.

Passing this constructor means only:

- typed static definition is internally valid;
- deployment declaration is internally valid;
- the two agree.

It does not prove the node or network actually satisfies the declaration.

---

# 15. Minimal compiler target boundary

## 15.1 First real downstream consumer

The tapscript adapter is the first real consumer of compiler-owned abstract
target capabilities.

Guide 8 may therefore expose the minimum stable public API required by that
consumer.

At minimum:

```rust
pub enum RequiredCapability { ... }
```

should become reachable through a reviewed public module or re-export.

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

or equivalent generated census.

The adapter must handle every variant exhaustively.

A new compiler capability should cause a compile failure or exact census-test
failure until the tapscript adapter states its disposition.

## 15.3 Optional requirement-set projection

If the adapter needs more than individual capability values, expose a
read-only, target-independent projection such as:

```rust
pub struct TargetRequirementSet {
    required_capabilities: BTreeSet<RequiredCapability>,
    external_evidence:
        BTreeSet<ExternalEvidenceRequirement>,
}
```

Do not expose placements, layout internals, or complete analyzed programs
unless the adapter genuinely consumes them in this guide.

## 15.4 No compiler-target dependency

The compiler public API must not name:

```text
ElementsCapability
OpcodeId
TargetDefinition
ElementsTarget
```

It remains abstract.

---

# 16. Tapscript capability adapter

## 16.1 Adapter purpose

The adapter translates compiler-required abstract capabilities into an
assessment against one validated target definition and development binding.

A suitable input is:

```rust
pub fn assess_capability(
    target: &target_elements::ElementsTarget,
    required: compiler::RequiredCapability,
) -> CapabilityAssessment;
```

The adapter should be pure and deterministic.

## 16.2 Assessment model

Use a typed multi-state result, for example:

```rust
pub enum CapabilityAssessment {
    Unsupported {
        required: compiler::RequiredCapability,
        reason: UnsupportedReason,
    },

    MissingTargetPrimitives {
        required: compiler::RequiredCapability,
        missing: BTreeSet<target_elements::ElementsCapability>,
    },

    BackendPatternRequired {
        required: compiler::RequiredCapability,
        primitives: BTreeSet<target_elements::ElementsCapability>,
        evidence:
            BTreeSet<target_elements::TargetEvidenceRequirementId>,
    },

    BackendStructural {
        required: compiler::RequiredCapability,
        requirements: BTreeSet<BackendFoundationRequirement>,
    },

    ExternalEvidenceRequired {
        required: compiler::RequiredCapability,
        evidence:
            BTreeSet<target_elements::TargetEvidenceRequirementId>,
    },
}
```

Exact names may differ.

Avoid a premature:

```rust
Supported(bool)
```

## 16.3 Initial compiler-capability mapping

The adapter should exhaustively classify current compiler capabilities.

Conceptually:

| Compiler capability | Guide-8 disposition |
|---|---|
| authenticated object recognition | backend pattern required; target inspection primitives required |
| authenticated family cardinality | backend/ABI structural proof required; count/index primitives may be required |
| authenticated canonical partition | backend pattern and complete transaction-layout proof required |
| authenticated open-flow partition | backend pattern and sponsor/protocol region proof required |
| authenticated root effects | backend constructor/root pattern required; unavailable for current operation emission |
| authenticated projection set | backend structural/event-shape pattern required |
| exact public amount arithmetic | target arithmetic and authenticated-value primitives required |
| confidential value conservation | target CT capability plus external target evidence required |
| owner authorization | signature primitive plus compatible sighash pattern required |
| operator authorization | signature primitive plus compatible sighash pattern required |
| refund authorization | signature primitive plus compatible sighash pattern required |
| public constructibility | compiler/ABI structural obligation; not one target opcode |
| whole-transaction value conservation | external target/consensus evidence required |

The exact table must match the actual `RequiredCapability` enum at implementation
time.

## 16.4 No semantic weakening

If a required primitive or target contract is missing, the adapter returns a
typed blocked or unsupported assessment.

It must not:

- drop the compiler capability;
- replace exact arithmetic with approximate arithmetic;
- treat public constructibility as signature availability;
- replace whole-transaction conservation with sponsor positivity;
- treat CT conservation as object-family closure;
- invent an authenticated opening pattern;
- claim a target relation is implemented.

## 16.5 No proof-pattern completion yet

Guide 8 should normally produce no:

```text
CompleteBackendPattern
```

assessment for attestation-contract relations.

The target primitives may be reviewed, but the first complete backend proof
patterns belong to later guides.

---

# 17. Validation rules

## 17.1 Target-definition validation

Require:

- supported target contract version;
- one execution domain;
- valid leaf version;
- opcode IDs and codes unique;
- every opcode fully specified;
- encoding prefixes unambiguous within their domains;
- field widths valid;
- failure semantics complete;
- resource dimensions valid;
- capability prerequisites resolve;
- evidence requirements resolve;
- capability dependency graph acyclic unless a reviewed strategy exists;
- no mutable evidence status in the definition;
- no review provenance in the semantic projection.

## 17.2 Deployment-binding validation

Require:

- nonzero network/genesis;
- explicit development environment;
- target contract version equality;
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
- no target-specific type flows back into compiler.

## 17.4 Cross-package closure

The target definition may expose more target capabilities than one pilot
requires.

The adapter must ensure:

```text
compiler required capability census
=
adapter assessment census
```

for any requirement set it receives.

No required capability may disappear from the assessment result.

---

# 18. Stable projections and determinism

## 18.1 Target-definition projection

Provide a stable typed projection for comparison and tests.

It may contain:

- target contract version;
- execution domain;
- leaf version;
- sorted opcode specs;
- sorted encoding specs;
- authorization/timelock contracts;
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
- target contract version;
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
- report digest.

## 18.4 Determinism tests

Test equality under:

- opcode declaration permutation;
- encoding declaration permutation;
- capability declaration permutation;
- evidence-requirement permutation;
- compiler capability requirement permutation;
- repeated construction;
- equivalent development-binding input ordering where set-like values exist.

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
        compiler::RequiredCapability,
    ),

    MissingTargetPrimitive {
        required: compiler::RequiredCapability,
        primitive: target_elements::ElementsCapability,
    },

    MissingTargetEvidenceRequirement {
        required: compiler::RequiredCapability,
        evidence:
            target_elements::TargetEvidenceRequirementId,
    },

    CapabilityAssessmentCensusMismatch {
        missing: Vec<compiler::RequiredCapability>,
        unexpected: Vec<compiler::RequiredCapability>,
    },

    UnsupportedTargetContractVersion,

    TargetDefinitionRejected,
}
```

Do not add emission, stack-scheduling, relocation, or target-program errors
until those paths exist.

---

# 20. Test plan

## 20.1 Target-definition positive tests

- reviewed built-in definition validates;
- supported target contract version;
- target definition projection is deterministic;
- each declared opcode has complete stack/failure/resource contracts;
- encoding classes remain distinct;
- capability prerequisites resolve;
- every capability has evidence requirements where required;
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
- source revision inserted into semantic projection, if the type design makes
  such a regression possible.

Each mutation must fail for a focused typed reason.

## 20.3 Development-binding tests

- nonzero synthetic development network/genesis accepted;
- zero network rejected;
- zero genesis rejected;
- target version mismatch rejected;
- missing activation declaration rejected;
- production binding rejected or unavailable;
- binding carries no credential field;
- repeated binding construction is equal.

## 20.4 Compiler public boundary tests

In the compiler’s external integration test:

- every required capability is publicly nameable;
- `RequiredCapability::ALL` is complete and duplicate-free;
- no target-specific type appears in the public compiler API;
- the complete internal analyzed program remains unavailable externally;
- no compiler-plan identity exists.

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
3. require blocked/incomplete result;
4. verify the original compiler requirement remains in the assessment.

Add focused regressions for:

- no whole-transaction conservation;
- no output-committing sighash capability;
- no input value inspection;
- no output program inspection;
- no transaction count inspection;
- no CT conservation;
- no relative timelock semantics;
- no issuance inspection.

Not every missing primitive must block both pilots today, but the adapter’s
general mapping must remain exact.

## 20.7 Sponsor-opacity tests

Assert the target adapter and target definition contain no attestation-contract
sponsor amount requirement.

Specifically, no adapter assessment should require:

- sponsor positivity;
- exact sponsor input amount;
- exact sponsor output amount;
- public sponsor aggregate;
- sponsor opening.

Whole-transaction conservation remains a target/consensus evidence requirement.

## 20.8 Permutation tests

Use deterministic or property-generated permutations for:

- opcode declarations;
- encoding declarations;
- capability declarations;
- evidence requirements;
- compiler capability input sets.

Stable projections must remain equal.

---

# 21. Independent cross-checks

## 21.1 Registry census oracle

Implement a direct test oracle that compares:

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

The expected bytes should not be generated by the same encoder under test.

Guide 8 need not implement complete target transaction parsing.

## 21.3 Stack-contract review fixtures

For each reviewed opcode:

- assert exact operand sequence;
- assert exact successful result sequence;
- assert exact failure effect;
- assert exact execution domain;
- assert exact resource cost.

These are typed contract tests, not target-native execution evidence.

## 21.4 Capability-adapter oracle

Maintain an independently written expected mapping table in tests for the
current compiler capability census.

The production adapter should use an exhaustive match. The oracle should
compare stable assessment projections without calling the production mapping
helper.

This duplication is appropriate because the mapping is the property being
verified.

---

# 22. Documentation requirements

## 22.1 Package READMEs

`packages/target-elements/README.md` should state:

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

`packages/tapscript/README.md` should state:

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
```

Mark only completed milestones.

Do not describe reviewed primitives as deployment-evidenced.

## 22.3 Phase card

Update:

```text
plans/phases/03-target-foundation.md
```

Guide 8 should make Phase 3 active but not complete.

A useful status is:

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

Likely new active tasks:

```text
T3-001  typed target package boundary
T3-002  reviewed initial target definition
T3-003  encoding and opcode contracts
T3-004  development deployment binding
T3-005  compiler capability adapter
T3-006  target-native primitive conformance
T3-007  typed tapscript instruction foundation
```

The exact numbering should follow the project’s current backlog convention.

Guide 8 should complete the first five and leave the last two for the next
guide.

---

# 23. Suggested implementation waves

## Wave 0 — Source review and dependency ruling

Deliver:

- reviewed initial target fact set;
- exact source/review provenance;
- licence review;
- decision whether Guide 8 needs any third-party dependency;
- explicit list of unresolved target-native claims.

Suggested commit:

```text
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

- target contract version;
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
- complete analyzed-program internals remain private.

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
- evidence/structural distinctions;
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
- active Phase-3 card;
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

## 24.4 Package boundary and dependency checks

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

and no accidental dependency on model, linker, transaction, vectors, release,
or artifacts.

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

Every new tracked source must join its nearest `meson.build` census in the same
commit.

Read `git status` after formatting.

---

# 26. Generated artifacts and identities

No architecture or model publication should change because Guide 8 adds
downstream packages only.

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

Adding workspace packages will normally update `Cargo.lock`’s first-party
package census. Review that change explicitly even if no new third-party
dependency is added.

---

# 27. Full batch gate

After all waves:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Because this guide adds packages and build census entries, the mocked Meson
contract and census audit are required.

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

Paper inputs should remain unchanged. Document byte reproducibility may be
deferred for this intermediate Phase-3 guide, but it must be recorded as
deferred rather than passed.

Finally:

```sh
git status --porcelain=v1 --untracked-files=all
```

The final tree must be clean after commits.

---

# 28. Guide-8 exit criteria

Guide 8 is complete only when all assertions below hold.

## Package boundaries

- [ ] `tripod-target-elements` exists;
- [ ] `target_elements` library exists;
- [ ] target-elements has no first-party semantic dependency;
- [ ] `tripod-tapscript` exists;
- [ ] tapscript depends directly on compiler and target-elements;
- [ ] compiler depends on neither target package;
- [ ] Meson census includes every new source.

## Target definition

- [ ] target contract version is explicit;
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
- [ ] review provenance remains outside semantic target projection;
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
- [ ] no compiler digest is added.

## Adapter

- [ ] every compiler capability is assessed exactly once;
- [ ] assessment is multi-state, not boolean;
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
- [ ] skipped/deferred lanes are reported honestly;
- [ ] final tree is clean.

---

# 29. Completion report template

```text
Guide 8 result
==============

Source review:
    upstream source:
    reviewed revision:
    reviewed locations:
    licence:
    accepted static claims:
    unresolved target-native claims:

Target-elements package:
    package boundary:
    first-party dependencies:
    target contract version:
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
    analyzed-program visibility:
    target-specific compiler types:

Tapscript adapter:
    compiler capability census:
    unsupported assessments:
    missing-primitive assessments:
    backend-pattern obligations:
    structural obligations:
    external-evidence obligations:
    sponsor opacity:

Identity impact:
    architecture semantic hash:
    architecture behavioural hash:
    generated architecture publications:
    realization identity:
    compiler identity:
    target definition identity:
    deployment identity:
    backend identity:

Dependency impact:
    new first-party packages:
    new third-party dependencies:
    Cargo.lock:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    cargo test --workspace --release --locked:
    target-elements tests:
    tapscript tests:
    compiler tests:
    Rustdoc:
    cargo tree:
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
    completed target milestones:
    completed backend milestones:
    next task:

Residuals:
```

---

# 30. Next guide after Guide 8

After Guide 8 passes, the next guide should be:

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

Guide 9 should not start STATE continuity, wide floor arithmetic, or compact-ASH
operation emission until the primitive target contracts used by those
constructions have target-native evidence.
